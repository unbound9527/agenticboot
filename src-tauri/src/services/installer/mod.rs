//! 安装引擎服务
//!
//! AgenticBoot 的核心安装引擎，封装网络检测、安装计划执行和工具卸载。
//! 通过 Tauri events 向推送安装进度。

pub mod dependency_resolver;
pub mod logging;
pub mod path_manager;
pub mod windows;

use crate::database::Database;
use crate::database::InstalledToolRecord;
use crate::plugin::{get_plugin_by_id, NpmRegistrySource, ToolInstallContext};
use crate::services::installer::logging::InstallLogEmitter;
use crate::services::installer::windows::find_managed_executable;
use crate::tool_types::{
    InstallPlan, InstallProgress, InstallStep, InstallStrategy, ToolUpdateInfo,
};
use path_manager::PathManager;
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

fn normalized_version_for_update(value: &str) -> Option<Vec<u64>> {
    static VERSION_RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    let regex = VERSION_RE.get_or_init(|| {
        Regex::new(r"(?i)\bv?(\d+\.\d+\.\d+(?:\.\d+)*)\b").expect("valid version regex")
    });
    let captures = regex.captures(value.trim())?;
    captures
        .get(1)?
        .as_str()
        .split('.')
        .map(|part| part.parse::<u64>().ok())
        .collect()
}

fn is_update_available(current_version: &str, latest_version: &str) -> bool {
    match (
        normalized_version_for_update(current_version),
        normalized_version_for_update(latest_version),
    ) {
        (Some(current), Some(latest)) => {
            let max_len = current.len().max(latest.len());
            for index in 0..max_len {
                let current_part = current.get(index).copied().unwrap_or(0);
                let latest_part = latest.get(index).copied().unwrap_or(0);
                match latest_part.cmp(&current_part) {
                    std::cmp::Ordering::Greater => return true,
                    std::cmp::Ordering::Less => return false,
                    std::cmp::Ordering::Equal => {}
                }
            }
            false
        }
        _ => latest_version.trim() != current_version.trim(),
    }
}

/// npm 包名映射表（tool_id → npm package name）
fn install_failure_summary(install_errors: &[String]) -> Result<(), String> {
    if install_errors.is_empty() {
        return Ok(());
    }

    let preview = install_errors
        .iter()
        .take(3)
        .cloned()
        .collect::<Vec<_>>()
        .join("; ");
    let suffix = if install_errors.len() > 3 {
        format!("; and {} more", install_errors.len() - 3)
    } else {
        String::new()
    };

    Err(format!(
        "{} tool install(s) failed: {}{}",
        install_errors.len(),
        preview,
        suffix
    ))
}

fn should_delete_install_dir(strategy: InstallStrategy, owned_by_root: bool) -> bool {
    owned_by_root
        && matches!(
            strategy,
            InstallStrategy::ManagedPrefix | InstallStrategy::GlobalNpm
        )
}

fn should_remove_shim(strategy: InstallStrategy, owned_by_root: bool) -> bool {
    let _ = owned_by_root;
    matches!(
        strategy,
        InstallStrategy::ManagedPrefix
            | InstallStrategy::GlobalNpm
            | InstallStrategy::PythonPackage
    )
}

fn should_ignore_uninstall_error(strategy: InstallStrategy, owned_by_root: bool) -> bool {
    owned_by_root && matches!(strategy, InstallStrategy::ManagedPrefix)
}

fn allows_uninstall_outside_managed_root(_strategy: InstallStrategy) -> bool {
    true
}

fn is_user_facing_tool_category(category: &str) -> bool {
    category == "tool" || category == "ai-cli"
}

fn normalize_for_ownership_check(path: &Path) -> Option<PathBuf> {
    if path.exists() {
        std::fs::canonicalize(path).ok()
    } else {
        let mut missing_components = Vec::new();
        let mut ancestor = path;

        while !ancestor.exists() {
            missing_components.push(ancestor.file_name()?.to_os_string());
            ancestor = ancestor.parent()?;
        }

        let mut normalized = std::fs::canonicalize(ancestor).ok()?;
        for component in missing_components.into_iter().rev() {
            normalized.push(component);
        }
        Some(normalized)
    }
}

pub(crate) fn is_install_owned_by_root(install_root: &Path, install_path: &Path) -> bool {
    let Some(normalized_root) = normalize_for_ownership_check(install_root) else {
        return false;
    };
    let Some(normalized_path) = normalize_for_ownership_check(install_path) else {
        return false;
    };

    normalized_path.starts_with(normalized_root)
}

fn detect_successful_install(
    tool_id: &str,
    target_dir: &Path,
    detect: crate::tool_types::DetectResult,
) -> Result<(Option<String>, String), String> {
    if !detect.installed {
        return Err(format!(
            "{} install finished but the tool could not be detected afterward",
            tool_id
        ));
    }

    let install_path = detect
        .install_path
        .unwrap_or_else(|| target_dir.to_string_lossy().to_string());
    Ok((detect.version, install_path))
}

fn should_publish_managed_shims(
    strategy: InstallStrategy,
    install_root: &Path,
    detect: &crate::tool_types::DetectResult,
) -> bool {
    matches!(
        strategy,
        InstallStrategy::ManagedPrefix
            | InstallStrategy::GlobalNpm
            | InstallStrategy::PythonPackage
    ) && detect
        .install_path
        .as_deref()
        .map(Path::new)
        .is_some_and(|path| is_install_owned_by_root(install_root, path))
}

async fn forward_install_progress_with<F>(mut rx: mpsc::Receiver<InstallProgress>, mut emit: F)
where
    F: FnMut(InstallProgress) + Send + 'static,
{
    while let Some(progress) = rx.recv().await {
        emit(progress);
    }
}

async fn fetch_latest_npm_version(client: &reqwest::Client, package_name: &str) -> Option<String> {
    let url = format!("https://registry.npmjs.org/{package_name}");
    let response = client.get(url).send().await.ok()?;
    let json = response.json::<serde_json::Value>().await.ok()?;
    json.get("dist-tags")
        .and_then(|tags| tags.get("latest"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

async fn fetch_latest_github_release_version(
    client: &reqwest::Client,
    repo: &str,
) -> Option<String> {
    let url = format!("https://api.github.com/repos/{repo}/releases/latest");
    let response = client
        .get(url)
        .header("User-Agent", "AgenticBoot")
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .ok()?;
    let json = response.json::<serde_json::Value>().await.ok()?;
    json.get("tag_name")
        .and_then(|value| value.as_str())
        .map(|value| value.trim_start_matches('v').to_string())
}

async fn fetch_latest_tool_version(client: &reqwest::Client, tool_id: &str) -> Option<String> {
    let plugin = get_plugin_by_id(tool_id)?;
    let source = plugin.update_source()?;
    match source.kind.as_str() {
        "npm" => fetch_latest_npm_version(client, &source.id).await,
        "github" => fetch_latest_github_release_version(client, &source.id).await,
        _ => None,
    }
}

/// 安装引擎
pub struct InstallerService {
    root_path: std::path::PathBuf,
    path_manager: PathManager,
}

impl InstallerService {
    /// 从根路径创建安装引擎
    pub fn new(root_path: &Path) -> Self {
        Self {
            root_path: root_path.to_path_buf(),
            path_manager: PathManager::new(root_path),
        }
    }

    /// 执行安装计划
    pub async fn execute_install_plan(
        &self,
        plan: &InstallPlan,
        app_handle: &AppHandle,
        db: &Arc<Database>,
    ) -> Result<(), String> {
        log::info!(
            "[InstallerService] execute_install_plan 开始, 共 {} 个步骤",
            plan.steps.len()
        );
        let npm_registry_source = NpmRegistrySource::Mirror;
        // 确保 bin 目录和 PATH 已就绪
        self.path_manager.ensure_bin_dir()?;
        log::info!(
            "[InstallerService] bin 目录已就绪: {}",
            self.path_manager.get_tool_install_dir("").display()
        );
        self.path_manager.register_in_path()?;
        log::info!("[InstallerService] PATH 注册完成");

        // ── Phase 1: 处理已安装的步骤（顺序执行，立即完成）──
        for step in &plan.steps {
            if !step.is_installed {
                continue;
            }
            log::info!(
                "[InstallerService] 已安装: tool_id={}, tool_name={}",
                step.tool_id,
                step.tool_name,
            );
            let plugin = get_plugin_by_id(&step.tool_id)
                .ok_or_else(|| format!("未知工具: {}", step.tool_id))?;
            if matches!(plugin.install_strategy(), InstallStrategy::GlobalNpm) {
                if let Some(shim_name) = plugin.managed_shim_name() {
                    self.path_manager.remove_windows_cli_shims(shim_name)?;
                }
            }
            let detect = plugin.detect(Some(&self.root_path));
            if should_publish_managed_shims(plugin.install_strategy(), &self.root_path, &detect) {
                let exe_name = plugin
                    .managed_shim_name()
                    .ok_or_else(|| format!("{} does not declare a managed shim", step.tool_id))?;
                let candidates = plugin.managed_executable_candidates();
                let candidate_refs = candidates.iter().map(String::as_str).collect::<Vec<_>>();
                let exe_path =
                    find_managed_executable(&self.root_path, &step.tool_id, &candidate_refs)
                        .ok_or_else(|| {
                            format!("已安装的 {} 缺少可执行文件，无法修复 shim", step.tool_id)
                        })?;
                self.path_manager
                    .create_windows_cli_shims(&exe_name, &exe_path)?;
            }
            let _ = app_handle.emit(
                "install-progress",
                InstallProgress {
                    tool_id: step.tool_id.clone(),
                    tool_name: step.tool_name.clone(),
                    phase: "skipped".to_string(),
                    percent: 100,
                    message: "已安装，跳过".to_string(),
                },
            );
        }

        // ── Phase 2: 构建依赖图（仅未安装的步骤）──
        let non_installed: Vec<&InstallStep> =
            plan.steps.iter().filter(|s| !s.is_installed).collect();

        if non_installed.is_empty() {
            return Ok(());
        }

        let step_ids: std::collections::HashSet<&str> =
            non_installed.iter().map(|s| s.tool_id.as_str()).collect();
        let mut deps_map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for step in &non_installed {
            if let Some(plugin) = get_plugin_by_id(&step.tool_id) {
                let deps: Vec<String> = plugin
                    .dependencies()
                    .iter()
                    .filter(|d| step_ids.contains(d.tool_id.as_str()))
                    .map(|d| d.tool_id.clone())
                    .collect();
                deps_map.insert(step.tool_id.clone(), deps);
            }
        }

        // ── Phase 3: 按依赖层级并行安装 ──
        let mut completed: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut pending: Vec<&&InstallStep> = non_installed.iter().collect();
        let mut install_errors: Vec<String> = Vec::new();

        while !pending.is_empty() {
            // 分离出当前层级：所有依赖项已完成的步骤
            let (ready, waiting): (Vec<_>, Vec<_>) = pending.into_iter().partition(|step| {
                deps_map
                    .get(&step.tool_id)
                    .map_or(true, |deps| deps.iter().all(|d| completed.contains(d)))
            });

            if ready.is_empty() {
                log::error!(
                    "[InstallerService] 安装计划死锁 — 缺少依赖: {:?}",
                    waiting
                        .iter()
                        .map(|s: &&&InstallStep| &s.tool_id)
                        .collect::<Vec<_>>()
                );
                for step in &waiting {
                    install_errors
                        .push(format!("{}: dependency installation failed", step.tool_id));
                    let _ = app_handle.emit(
                        "install-progress",
                        InstallProgress {
                            tool_id: step.tool_id.clone(),
                            tool_name: step.tool_name.clone(),
                            phase: "error".to_string(),
                            percent: 0,
                            message: "依赖项安装失败，无法继续".to_string(),
                        },
                    );
                    let _ = app_handle.emit(
                        "install-error",
                        serde_json::json!({
                            "toolId": step.tool_id,
                            "error": "依赖安装失败"
                        }),
                    );
                    completed.insert(step.tool_id.clone());
                }
                break;
            }

            log::info!(
                "[InstallerService] 并行安装 {} 个工具: {:?}",
                ready.len(),
                ready.iter().map(|s| &s.tool_id).collect::<Vec<_>>()
            );

            // 同层级并行启动
            let handles: Vec<_> = ready
                .into_iter()
                .map(|step| {
                    let tool_id = step.tool_id.clone();
                    let tool_name = step.tool_name.clone();
                    let category = step.category.clone();
                    let app_handle = app_handle.clone();
                    let db = Arc::clone(db);
                    let target_dir = self.path_manager.get_tool_install_dir(&step.tool_id);
                    let root_path = self.root_path.clone();
                    let npm_source = npm_registry_source;

                    tokio::spawn(async move {
                        let result = Self::install_single_tool(
                            tool_id.clone(),
                            tool_name,
                            category,
                            app_handle,
                            db,
                            target_dir,
                            root_path,
                            npm_source,
                        )
                        .await;
                        (tool_id, result)
                    })
                })
                .collect();

            // 等待当前层级全部完成
            for handle in handles {
                match handle.await {
                    Ok((tool_id, Ok(()))) => {
                        log::info!("[InstallerService] {} 安装成功", tool_id);
                        completed.insert(tool_id);
                    }
                    Ok((tool_id, Err(e))) => {
                        log::error!("[InstallerService] {} 安装失败: {}", tool_id, e);
                        install_errors.push(format!("{tool_id}: {e}"));
                        completed.insert(tool_id);
                    }
                    Err(join_err) => {
                        log::error!("[InstallerService] 安装任务 panic: {join_err}");
                    }
                }
            }

            pending = waiting;
        }

        if !install_errors.is_empty() {
            log::warn!(
                "[InstallerService] {} 个工具安装失败: {:?}",
                install_errors.len(),
                install_errors
            );
        }

        install_failure_summary(&install_errors)
    }

    /// 安装单个工具（可并行化）
    async fn install_single_tool(
        tool_id: String,
        tool_name: String,
        category: String,
        app_handle: AppHandle,
        db: Arc<Database>,
        target_dir: PathBuf,
        root_path: PathBuf,
        npm_registry_source: NpmRegistrySource,
    ) -> Result<(), String> {
        let install_log =
            InstallLogEmitter::new(app_handle.clone(), tool_id.clone(), tool_name.clone());
        install_log.emit_session_started();
        install_log.emit_phase("starting", "Preparing install");
        let _ = app_handle.emit(
            "install-progress",
            InstallProgress {
                tool_id: tool_id.clone(),
                tool_name: tool_name.clone(),
                phase: "starting".to_string(),
                percent: 0,
                message: "准备安装...".to_string(),
            },
        );

        let plugin = get_plugin_by_id(&tool_id).ok_or_else(|| format!("未知工具: {tool_id}"))?;
        log::info!(
            "[InstallerService] 找到插件: {} -> {}",
            tool_id,
            plugin.metadata().name
        );
        log::info!("[InstallerService] 目标安装目录: {}", target_dir.display());

        let (tx, rx) = mpsc::channel::<InstallProgress>(32);

        let install_target = target_dir.clone();
        let install_tool_id = tool_id.clone();
        let install_root = root_path.clone();

        log::info!(
            "[InstallerService] 开始调用 plugin.install_with_context for {}",
            install_tool_id
        );
        let progress_forwarder = tokio::spawn({
            let progress_handle = app_handle.clone();
            async move {
                forward_install_progress_with(rx, move |progress| {
                    let _ = progress_handle.emit("install-progress", progress);
                })
                .await;
            }
        });

        let install_log_for_plugin = install_log.clone();
        let install_result = tokio::task::spawn_blocking(move || {
            plugin.install_with_context(
                &install_target,
                &install_root,
                tx,
                ToolInstallContext::new(install_log_for_plugin, npm_registry_source),
            )
        })
        .await
        .map_err(|e| format!("安装线程错误: {e}"))?;

        log::info!(
            "[InstallerService] plugin.install_with_context 返回: {:?}",
            install_result
        );
        progress_forwarder
            .await
            .map_err(|e| format!("安装进度转发线程错误: {e}"))?;

        let post_plugin =
            get_plugin_by_id(&tool_id).ok_or_else(|| format!("未知工具: {tool_id}"))?;

        match install_result {
            Ok(()) => {
                let path_manager = PathManager::new(&root_path);
                if matches!(post_plugin.install_strategy(), InstallStrategy::GlobalNpm) {
                    if let Some(shim_name) = post_plugin.managed_shim_name() {
                        path_manager.remove_windows_cli_shims(shim_name)?;
                    }
                }
                let detect = post_plugin.detect(Some(&root_path));
                let publish_managed_shims = should_publish_managed_shims(
                    post_plugin.install_strategy(),
                    &root_path,
                    &detect,
                );
                let (version, install_path) =
                    detect_successful_install(&tool_id, &target_dir, detect)?;

                if publish_managed_shims {
                    let exe_name = post_plugin
                        .managed_shim_name()
                        .ok_or_else(|| format!("{tool_id} does not declare a managed shim"))?;
                    let candidates = post_plugin.managed_executable_candidates();
                    let candidate_refs = candidates.iter().map(String::as_str).collect::<Vec<_>>();
                    let exe_path =
                        find_managed_executable(&root_path, &tool_id, &candidate_refs)
                            .ok_or_else(|| format!("安装完成后未找到 {} 的可执行文件", tool_id))?;
                    path_manager.create_windows_cli_shims(&exe_name, &exe_path)?;
                }

                let now = chrono::Utc::now().timestamp();
                db.upsert_installed_tool(&InstalledToolRecord {
                    id: tool_id.clone(),
                    name: tool_name.clone(),
                    version,
                    install_path,
                    install_root: root_path.to_string_lossy().to_string(),
                    category,
                    status: "installed".to_string(),
                    state_source: "managed".to_string(),
                    installed_at: Some(now),
                    last_seen_at: Some(now),
                    updated_at: Some(now),
                })
                .map_err(|e| format!("保存安装记录失败: {e}"))?;

                install_log.emit_result("complete", "Install completed", Some(0), true);
                let _ = app_handle.emit(
                    "install-progress",
                    InstallProgress {
                        tool_id: tool_id.clone(),
                        tool_name: tool_name.clone(),
                        phase: "complete".to_string(),
                        percent: 100,
                        message: "安装完成".to_string(),
                    },
                );
                let _ = app_handle.emit("install-complete", &tool_id);
                Ok(())
            }
            Err(e) => {
                let now = chrono::Utc::now().timestamp();
                db.upsert_installed_tool(&InstalledToolRecord {
                    id: tool_id.clone(),
                    name: tool_name.clone(),
                    version: None,
                    install_path: target_dir.to_string_lossy().to_string(),
                    install_root: root_path.to_string_lossy().to_string(),
                    category,
                    status: "error".to_string(),
                    state_source: "managed".to_string(),
                    installed_at: None,
                    last_seen_at: None,
                    updated_at: Some(now),
                })
                .ok();

                install_log.emit_result("error", e.clone(), None, false);
                let _ = app_handle.emit(
                    "install-progress",
                    InstallProgress {
                        tool_id: tool_id.clone(),
                        tool_name: tool_name.clone(),
                        phase: "error".to_string(),
                        percent: 0,
                        message: e.clone(),
                    },
                );
                let _ = app_handle.emit(
                    "install-error",
                    serde_json::json!({
                        "toolId": tool_id,
                        "error": e
                    }),
                );

                Err(format!("安装 {} 失败", tool_name))
            }
        }
    }

    /// 卸载工具
    pub fn uninstall_tool(&self, tool_id: &str, db: &Arc<Database>) -> Result<(), String> {
        let plugin = get_plugin_by_id(tool_id).ok_or_else(|| format!("unknown tool: {tool_id}"))?;
        let strategy = plugin.install_strategy();
        let record = db
            .get_installed_tool(tool_id)
            .map_err(|e| format!("failed to load tool record: {e}"))?;

        let target_dir = record
            .as_ref()
            .map(|record| PathBuf::from(&record.install_path))
            .unwrap_or_else(|| self.root_path.clone());
        let owned_by_root = record.as_ref().is_some_and(|record| {
            is_install_owned_by_root(
                Path::new(&record.install_root),
                Path::new(&record.install_path),
            )
        });

        if !allows_uninstall_outside_managed_root(strategy) && !owned_by_root {
            return Err(
                "user-installed tools outside the managed root must be removed by their original uninstaller"
                    .to_string(),
            );
        }

        if let Err(err) = plugin.uninstall(&target_dir) {
            if !should_ignore_uninstall_error(strategy, owned_by_root) {
                return Err(err);
            }
        }

        if should_remove_shim(strategy, owned_by_root) {
            if let Some(shim_name) = plugin.managed_shim_name() {
                self.path_manager.remove_windows_cli_shims(shim_name)?;
            }
        }

        if should_delete_install_dir(strategy, owned_by_root) && target_dir.exists() {
            std::fs::remove_dir_all(&target_dir)
                .map_err(|e| format!("failed to remove install directory: {e}"))?;
        }

        db.delete_installed_tool(tool_id)
            .map_err(|e| format!("failed to delete tool record: {e}"))?;

        Ok(())
    }

    pub fn uninstall_tool_legacy(&self, tool_id: &str, db: &Arc<Database>) -> Result<(), String> {
        let plugin = get_plugin_by_id(tool_id).ok_or_else(|| format!("unknown tool: {tool_id}"))?;
        let strategy = plugin.install_strategy();
        let record = db
            .get_installed_tool(tool_id)
            .map_err(|e| format!("failed to load tool record: {e}"))?
            .ok_or_else(|| format!("tool not found: {tool_id}"))?;

        let target_dir = Path::new(&record.install_path);
        let owned_by_root = is_install_owned_by_root(Path::new(&record.install_root), target_dir);

        // 不允许卸载用户自行安装的工具（不在 AgenticBoot 管理目录下）
        if !allows_uninstall_outside_managed_root(strategy) && !owned_by_root {
            return Err(
                "用户自行安装的工具不允许卸载，请通过系统设置或相应工具的卸载程序移除".to_string(),
            );
        }

        let uninstall_result = plugin.uninstall(target_dir);

        if let Err(err) = uninstall_result {
            if !should_ignore_uninstall_error(strategy, owned_by_root) {
                return Err(err);
            }
        }

        if should_remove_shim(strategy, owned_by_root) {
            if let Some(shim_name) = plugin.managed_shim_name() {
                self.path_manager.remove_windows_cli_shims(shim_name)?;
            }
        }

        if should_delete_install_dir(strategy, owned_by_root) && target_dir.exists() {
            std::fs::remove_dir_all(target_dir).map_err(|e| format!("删除安装目录失败: {e}"))?;
        }

        db.delete_installed_tool(tool_id)
            .map_err(|e| format!("删除工具记录失败: {e}"))?;

        Ok(())
    }

    /// 检查工具更新
    pub async fn check_tool_updates(db: &Arc<Database>) -> Result<Vec<ToolUpdateInfo>, String> {
        let tools = db
            .get_installed_tools()
            .map_err(|e| format!("查询已安装工具失败: {e}"))?;

        let mut updates = Vec::new();
        let client = reqwest::Client::builder()
            .user_agent("AgenticBoot/0.1")
            .build()
            .map_err(|e| format!("failed to create update-check client: {e}"))?;

        for tool in tools {
            // 只检查用户工具（非依赖项）
            if !is_user_facing_tool_category(&tool.category) {
                continue;
            }

            let Some(current_version) = tool.version.as_ref() else {
                continue;
            };

            let Some(latest_version) = fetch_latest_tool_version(&client, &tool.id).await else {
                continue;
            };

            if is_update_available(current_version, &latest_version) {
                updates.push(ToolUpdateInfo {
                    tool_id: tool.id,
                    current_version: current_version.clone(),
                    latest_version,
                });
            }
        }

        Ok(updates)
    }
}

/// 根据工具 ID 推断可执行文件名
#[cfg(test)]
mod tests {
    use super::{
        allows_uninstall_outside_managed_root, detect_successful_install,
        forward_install_progress_with, is_install_owned_by_root, is_update_available,
        is_user_facing_tool_category, normalized_version_for_update, should_delete_install_dir,
        should_ignore_uninstall_error, should_publish_managed_shims, should_remove_shim,
    };
    use crate::plugin::get_plugin_by_id;
    use crate::tool_types::InstallProgress;
    use crate::tool_types::{DetectResult, InstallStrategy};
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;
    use tokio::sync::mpsc;

    #[test]
    fn install_failure_summary_returns_ok_when_no_errors_exist() {
        assert!(super::install_failure_summary(&[]).is_ok());
    }

    #[test]
    fn install_failure_summary_includes_count_and_first_failure() {
        let err = super::install_failure_summary(&[
            "codex-cli: install failed".to_string(),
            "nodejs: dependency failed".to_string(),
        ])
        .expect_err("summary should return an error");

        assert!(err.contains("2"));
        assert!(err.contains("codex-cli"));
    }

    #[test]
    fn install_strategy_uninstall_policy_only_deletes_managed_prefix_directories() {
        assert!(should_delete_install_dir(
            InstallStrategy::ManagedPrefix,
            true
        ));
        assert!(should_delete_install_dir(InstallStrategy::GlobalNpm, true));
        assert!(!should_delete_install_dir(
            InstallStrategy::DesktopInstaller,
            false
        ));
        assert!(!should_delete_install_dir(
            InstallStrategy::PythonPackage,
            false
        ));
    }

    #[test]
    fn install_strategy_uninstall_policy_does_not_delete_external_managed_prefix_directory() {
        let tmp = TempDir::new().unwrap();
        let managed_root = tmp.path().join("managed-root");
        let external_install = tmp.path().join("external-tool");

        std::fs::create_dir_all(&managed_root).unwrap();
        std::fs::create_dir_all(&external_install).unwrap();

        let owned_by_root = is_install_owned_by_root(&managed_root, &external_install);
        assert!(!owned_by_root);
        assert!(!should_delete_install_dir(
            InstallStrategy::ManagedPrefix,
            owned_by_root
        ));
    }

    #[test]
    fn install_strategy_uninstall_policy_only_removes_shims_for_owned_installs() {
        assert!(should_remove_shim(InstallStrategy::ManagedPrefix, true));
        assert!(should_remove_shim(InstallStrategy::PythonPackage, true));
        assert!(should_remove_shim(InstallStrategy::GlobalNpm, true));
        assert!(should_remove_shim(InstallStrategy::GlobalNpm, false));
        assert!(should_remove_shim(InstallStrategy::ManagedPrefix, false));
        assert!(should_remove_shim(InstallStrategy::PythonPackage, false));
        assert!(!should_remove_shim(InstallStrategy::DesktopInstaller, true));
    }

    #[test]
    fn install_strategy_uninstall_policy_only_ignores_errors_for_owned_managed_prefix_tools() {
        assert!(should_ignore_uninstall_error(
            InstallStrategy::ManagedPrefix,
            true
        ));
        assert!(!should_ignore_uninstall_error(
            InstallStrategy::ManagedPrefix,
            false
        ));
        assert!(!should_ignore_uninstall_error(
            InstallStrategy::DesktopInstaller,
            true
        ));
        assert!(!should_ignore_uninstall_error(
            InstallStrategy::GlobalNpm,
            false
        ));
    }

    #[test]
    fn install_strategy_uninstall_policy_allows_external_installs_for_every_strategy() {
        assert!(allows_uninstall_outside_managed_root(
            InstallStrategy::ManagedPrefix
        ));
        assert!(allows_uninstall_outside_managed_root(
            InstallStrategy::GlobalNpm
        ));
        assert!(allows_uninstall_outside_managed_root(
            InstallStrategy::OfficialScript
        ));
        assert!(allows_uninstall_outside_managed_root(
            InstallStrategy::PythonPackage
        ));
        assert!(allows_uninstall_outside_managed_root(
            InstallStrategy::DesktopInstaller
        ));
    }

    #[test]
    fn install_requires_post_install_detection_to_succeed() {
        let tmp = TempDir::new().unwrap();
        let err = detect_successful_install("openclaw", tmp.path(), DetectResult::not_installed())
            .unwrap_err();

        assert!(err.contains("openclaw"));
        assert!(err.contains("could not be detected"));
    }

    #[test]
    fn install_uses_detected_install_path_after_successful_redetect() {
        let tmp = TempDir::new().unwrap();
        let detect = DetectResult {
            installed: true,
            version: Some("1.2.3".to_string()),
            install_path: Some("C:\\Users\\me\\AppData\\Roaming\\npm".to_string()),
        };

        let (version, install_path) =
            detect_successful_install("openclaw", tmp.path(), detect).expect("success");

        assert_eq!(version.as_deref(), Some("1.2.3"));
        assert_eq!(install_path, "C:\\Users\\me\\AppData\\Roaming\\npm");
    }

    #[test]
    fn shim_publish_policy_repairs_owned_managed_installs() {
        let tmp = TempDir::new().unwrap();
        let managed_install = tmp.path().join("gemini-cli");
        std::fs::create_dir_all(&managed_install).unwrap();
        let detect = DetectResult {
            installed: true,
            version: Some("0.41.2".to_string()),
            install_path: Some(managed_install.to_string_lossy().to_string()),
        };

        assert!(should_publish_managed_shims(
            InstallStrategy::ManagedPrefix,
            tmp.path(),
            &detect
        ));
        assert!(should_publish_managed_shims(
            InstallStrategy::GlobalNpm,
            tmp.path(),
            &detect
        ));
    }

    #[test]
    fn shim_publish_policy_skips_external_or_non_managed_installs() {
        let tmp = TempDir::new().unwrap();
        let external_install = tmp.path().join("external").join("gemini-cli");
        let external_detect = DetectResult {
            installed: true,
            version: Some("0.41.2".to_string()),
            install_path: Some(external_install.to_string_lossy().to_string()),
        };

        assert!(!should_publish_managed_shims(
            InstallStrategy::ManagedPrefix,
            tmp.path().join("managed-root").as_path(),
            &external_detect
        ));
        assert!(!should_publish_managed_shims(
            InstallStrategy::DesktopInstaller,
            tmp.path(),
            &external_detect
        ));
    }

    #[test]
    fn nodejs_shim_uses_node_command_name() {
        let plugin = get_plugin_by_id("nodejs").expect("node plugin");
        assert_eq!(plugin.managed_shim_name(), Some("node"));
    }

    #[test]
    fn update_check_treats_ai_cli_records_as_user_tools() {
        assert!(is_user_facing_tool_category("ai-cli"));
        assert!(is_user_facing_tool_category("tool"));
        assert!(!is_user_facing_tool_category("dependency"));
    }

    #[test]
    fn update_check_uses_real_upstream_sources_for_supported_tools() {
        let codex = get_plugin_by_id("codex-cli")
            .and_then(|plugin| plugin.update_source())
            .expect("codex update source");
        assert_eq!(codex.kind, "npm");
        assert_eq!(codex.id, "@openai/codex");

        let openclaw = get_plugin_by_id("openclaw")
            .and_then(|plugin| plugin.update_source())
            .expect("openclaw update source");
        assert_eq!(openclaw.kind, "npm");
        assert_eq!(openclaw.id, "openclaw");

        let opencode_cli = get_plugin_by_id("opencode-cli")
            .and_then(|plugin| plugin.update_source())
            .expect("opencode cli update source");
        assert_eq!(opencode_cli.kind, "npm");
        assert_eq!(opencode_cli.id, "opencode-ai");

        let opencode_desktop = get_plugin_by_id("opencode-desktop")
            .and_then(|plugin| plugin.update_source())
            .expect("opencode desktop update source");
        assert_eq!(opencode_desktop.kind, "github");
        assert_eq!(opencode_desktop.id, "opencode-ai/opencode");

        assert!(get_plugin_by_id("hermes")
            .and_then(|plugin| plugin.update_source())
            .is_none());
    }

    #[test]
    fn update_check_normalizes_verbose_cli_version_output() {
        assert_eq!(
            normalized_version_for_update("codex 0.24.0"),
            Some(vec![0, 24, 0])
        );
        assert!(!is_update_available("codex 0.24.0", "0.24.0"));
        assert!(is_update_available("codex 0.24.0", "0.24.1"));
    }

    #[tokio::test]
    async fn install_progress_forwarder_drains_buffered_events_before_returning() {
        let (tx, rx) = mpsc::channel::<InstallProgress>(4);
        let collected = Arc::new(Mutex::new(Vec::new()));

        tx.send(InstallProgress {
            tool_id: "gemini-cli".into(),
            tool_name: "Gemini CLI".into(),
            phase: "downloading".into(),
            percent: 40,
            message: "Downloading".into(),
        })
        .await
        .unwrap();
        tx.send(InstallProgress {
            tool_id: "gemini-cli".into(),
            tool_name: "Gemini CLI".into(),
            phase: "installing".into(),
            percent: 80,
            message: "Installing".into(),
        })
        .await
        .unwrap();
        drop(tx);

        let sink = Arc::clone(&collected);
        forward_install_progress_with(rx, move |progress| {
            sink.lock().unwrap().push(progress);
        })
        .await;

        let collected = collected.lock().unwrap();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0].percent, 40);
        assert_eq!(collected[1].percent, 80);
    }
}
