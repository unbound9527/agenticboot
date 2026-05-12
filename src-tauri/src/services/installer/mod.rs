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
use crate::services::installer::windows::{find_managed_executable, npm_prefix_candidates};
use crate::tool_types::{
    InstallPlan, InstallProgress, InstallStrategy, NetworkStatus, ToolUpdateInfo,
};
use path_manager::PathManager;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

/// npm 包名映射表（tool_id → npm package name）
fn should_delete_install_dir(strategy: InstallStrategy, owned_by_root: bool) -> bool {
    owned_by_root
        && matches!(
            strategy,
            InstallStrategy::ManagedPrefix | InstallStrategy::GlobalNpm
        )
}

fn should_remove_shim(strategy: InstallStrategy, owned_by_root: bool) -> bool {
    matches!(strategy, InstallStrategy::GlobalNpm)
        || (owned_by_root
            && matches!(
                strategy,
                InstallStrategy::ManagedPrefix | InstallStrategy::PythonPackage
            ))
}

fn should_ignore_uninstall_error(strategy: InstallStrategy, owned_by_root: bool) -> bool {
    owned_by_root && matches!(strategy, InstallStrategy::ManagedPrefix)
}

fn allows_uninstall_outside_managed_root(_strategy: InstallStrategy) -> bool {
    true
}

fn normalize_for_ownership_check(path: &Path) -> Option<PathBuf> {
    if path.exists() {
        std::fs::canonicalize(path).ok()
    } else {
        Some(path.to_path_buf())
    }
}

fn is_install_owned_by_root(install_root: &Path, install_path: &Path) -> bool {
    let Some(normalized_root) = normalize_for_ownership_check(install_root) else {
        return false;
    };
    let Some(normalized_path) = normalize_for_ownership_check(install_path) else {
        return false;
    };

    normalized_path.starts_with(normalized_root)
}

fn managed_executable_candidates(tool_id: &str) -> Vec<String> {
    match tool_id {
        "nodejs" => vec!["node.exe".to_string(), "bin\\node.exe".to_string()],
        "git" => vec!["cmd\\git.exe".to_string(), "bin\\git.exe".to_string()],
        "claude-code-cli" => npm_prefix_candidates("claude"),
        "codex-cli" => npm_prefix_candidates("codex"),
        "gemini-cli" => npm_prefix_candidates("gemini"),
        "opencode-cli" => npm_prefix_candidates("opencode"),
        "openclaw" => npm_prefix_candidates("openclaw"),
        "hermes" => vec![
            "venv\\Scripts\\hermes.exe".to_string(),
            "venv\\Scripts\\hermes.cmd".to_string(),
            "Scripts\\hermes.exe".to_string(),
            "Scripts\\hermes.cmd".to_string(),
        ],
        _ => vec![],
    }
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

fn choose_npm_registry_source(network: &NetworkStatus) -> NpmRegistrySource {
    if network.npm_reachable {
        NpmRegistrySource::Official
    } else {
        NpmRegistrySource::Mirror
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

    /// 检测网络连通性
    pub async fn check_network() -> NetworkStatus {
        let client = reqwest::Client::new();

        let github_ok = client
            .get("https://github.com")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        let npm_ok = client
            .get("https://registry.npmjs.org")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        let youtube_ok = client
            .get("https://www.youtube.com")
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        let all_ok = github_ok && npm_ok && youtube_ok;
        let error_message = if all_ok {
            None
        } else if !github_ok && !npm_ok && !youtube_ok {
            Some("网络连接异常，请检查网络设置。".to_string())
        } else if !github_ok && !npm_ok && youtube_ok {
            Some("GitHub 和 npm 源连接异常，但国际网络正常，可能是站点被屏蔽。".to_string())
        } else if !github_ok {
            Some("GitHub 连接异常，部分工具可能无法下载。".to_string())
        } else if !npm_ok {
            Some("npm 源连接异常，CLI 工具可能无法安装。".to_string())
        } else {
            Some("部分站点连接异常（YouTube 不可达），请检查网络。".to_string())
        };

        NetworkStatus {
            github_reachable: github_ok,
            npm_reachable: npm_ok,
            youtube_reachable: youtube_ok,
            error_message,
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
        let network = Self::check_network().await;
        let npm_registry_source = choose_npm_registry_source(&network);
        // 确保 bin 目录和 PATH 已就绪
        self.path_manager.ensure_bin_dir()?;
        log::info!(
            "[InstallerService] bin 目录已就绪: {}",
            self.path_manager.get_tool_install_dir("").display()
        );
        self.path_manager.register_in_path()?;
        log::info!("[InstallerService] PATH 注册完成");

        for step in &plan.steps {
            log::info!(
                "[InstallerService] 处理步骤: tool_id={}, tool_name={}, is_installed={}",
                step.tool_id,
                step.tool_name,
                step.is_installed
            );
            // 跳过已安装
            if step.is_installed {
                let plugin = get_plugin_by_id(&step.tool_id)
                    .ok_or_else(|| format!("未知工具: {}", step.tool_id))?;
                if matches!(plugin.install_strategy(), InstallStrategy::GlobalNpm) {
                    self.path_manager
                        .remove_windows_cli_shims(&get_exe_name(&step.tool_id))?;
                }
                let detect = plugin.detect(Some(&self.root_path));
                if should_publish_managed_shims(plugin.install_strategy(), &self.root_path, &detect)
                {
                    let exe_name = get_exe_name(&step.tool_id);
                    let candidates = managed_executable_candidates(&step.tool_id);
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
                continue;
            }

            // 开始安装
            let install_tool_name = step.tool_name.clone();
            let install_log = InstallLogEmitter::new(
                app_handle.clone(),
                step.tool_id.clone(),
                install_tool_name.clone(),
            );
            install_log.emit_session_started();
            install_log.emit_phase("starting", "Preparing install");
            let _ = app_handle.emit(
                "install-progress",
                InstallProgress {
                    tool_id: step.tool_id.clone(),
                    tool_name: install_tool_name.clone(),
                    phase: "starting".to_string(),
                    percent: 0,
                    message: "准备安装...".to_string(),
                },
            );

            let plugin = get_plugin_by_id(&step.tool_id)
                .ok_or_else(|| format!("未知工具: {}", step.tool_id))?;
            log::info!(
                "[InstallerService] 找到插件: {} -> {}",
                step.tool_id,
                plugin.metadata().name
            );

            let target_dir = self.path_manager.get_tool_install_dir(&step.tool_id);
            log::info!("[InstallerService] 目标安装目录: {}", target_dir.display());

            // 创建进度通道
            let (tx, rx) = mpsc::channel::<InstallProgress>(32);

            let install_target = target_dir.clone();
            let install_tool_id = step.tool_id.clone();
            let install_tool_name = install_tool_name.clone();
            let _install_category = step.category.clone();
            let install_root = self.root_path.clone();

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

            // 重新获取插件以进行后续操作
            let post_plugin = get_plugin_by_id(&install_tool_id)
                .ok_or_else(|| format!("未知工具: {}", install_tool_id))?;

            match install_result {
                Ok(()) => {
                    if matches!(post_plugin.install_strategy(), InstallStrategy::GlobalNpm) {
                        self.path_manager
                            .remove_windows_cli_shims(&get_exe_name(&install_tool_id))?;
                    }
                    // 检测已安装版本（传入安装根目录）
                    let detect = post_plugin.detect(Some(&self.root_path));
                    let publish_managed_shims = should_publish_managed_shims(
                        post_plugin.install_strategy(),
                        &self.root_path,
                        &detect,
                    );
                    let (version, install_path) =
                        detect_successful_install(&install_tool_id, &target_dir, detect)?;

                    // 创建 shim
                    if publish_managed_shims {
                        let exe_name = get_exe_name(&install_tool_id);
                        let candidates = managed_executable_candidates(&install_tool_id);
                        let candidate_refs =
                            candidates.iter().map(String::as_str).collect::<Vec<_>>();
                        let exe_path = find_managed_executable(
                            &self.root_path,
                            &install_tool_id,
                            &candidate_refs,
                        )
                        .ok_or_else(|| {
                            format!("安装完成后未找到 {} 的可执行文件", install_tool_id)
                        })?;
                        self.path_manager
                            .create_windows_cli_shims(&exe_name, &exe_path)?;
                    }

                    // 更新数据库
                    let now = chrono::Utc::now().timestamp();
                    db.upsert_installed_tool(&InstalledToolRecord {
                        id: install_tool_id.clone(),
                        name: install_tool_name.clone(),
                        version,
                        install_path,
                        install_root: self.root_path.to_string_lossy().to_string(),
                        category: step.category.clone(),
                        status: "installed".to_string(),
                        installed_at: Some(now),
                        updated_at: Some(now),
                    })
                    .map_err(|e| format!("保存安装记录失败: {e}"))?;

                    install_log.emit_result("complete", "Install completed", Some(0), true);
                    let _ = app_handle.emit(
                        "install-progress",
                        InstallProgress {
                            tool_id: install_tool_id.clone(),
                            tool_name: install_tool_name.clone(),
                            phase: "complete".to_string(),
                            percent: 100,
                            message: "安装完成".to_string(),
                        },
                    );

                    let _ = app_handle.emit("install-complete", &install_tool_id);
                }
                Err(e) => {
                    // 记录错误状态
                    let now = chrono::Utc::now().timestamp();
                    db.upsert_installed_tool(&InstalledToolRecord {
                        id: install_tool_id.clone(),
                        name: install_tool_name.clone(),
                        version: None,
                        install_path: target_dir.to_string_lossy().to_string(),
                        install_root: self.root_path.to_string_lossy().to_string(),
                        category: step.category.clone(),
                        status: "error".to_string(),
                        installed_at: None,
                        updated_at: Some(now),
                    })
                    .ok();

                    install_log.emit_result("error", e.clone(), None, false);
                    let _ = app_handle.emit(
                        "install-progress",
                        InstallProgress {
                            tool_id: install_tool_id.clone(),
                            tool_name: install_tool_name.clone(),
                            phase: "error".to_string(),
                            percent: 0,
                            message: e.clone(),
                        },
                    );

                    let _ = app_handle.emit(
                        "install-error",
                        serde_json::json!({
                            "toolId": install_tool_id,
                            "error": e
                        }),
                    );

                    return Err(format!("安装 {} 失败", install_tool_name));
                }
            }
        }

        Ok(())
    }

    /// 卸载工具
    pub fn uninstall_tool(
        &self,
        tool_id: &str,
        db: &Arc<Database>,
    ) -> Result<(), String> {
        let plugin =
            get_plugin_by_id(tool_id).ok_or_else(|| format!("unknown tool: {tool_id}"))?;
        let strategy = plugin.install_strategy();
        let record = db
            .get_installed_tool(tool_id)
            .map_err(|e| format!("failed to load tool record: {e}"))?;

        let target_dir = record
            .as_ref()
            .map(|record| PathBuf::from(&record.install_path))
            .unwrap_or_else(|| self.path_manager.get_tool_install_dir(tool_id));
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
            self.path_manager
                .remove_windows_cli_shims(&get_exe_name(tool_id))?;
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
        let plugin = get_plugin_by_id(tool_id).ok_or_else(|| format!("鏈煡宸ュ叿: {tool_id}"))?;
        let _strategy = plugin.install_strategy();
        let record = db
            .get_installed_tool(tool_id)
            .map_err(|e| format!("查询工具记录失败: {e}"))?
            .ok_or_else(|| format!("未找到已安装工具: {tool_id}"))?;
        let plugin = get_plugin_by_id(tool_id).ok_or_else(|| format!("未知工具: {tool_id}"))?;

        let target_dir = Path::new(&record.install_path);
        let strategy = plugin.install_strategy();
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
            self.path_manager
                .remove_windows_cli_shims(&get_exe_name(tool_id))?;
        }

        if should_delete_install_dir(strategy, owned_by_root) && target_dir.exists() {
            std::fs::remove_dir_all(target_dir).map_err(|e| format!("删除安装目录失败: {e}"))?;
        }

        db.delete_installed_tool(tool_id)
            .map_err(|e| format!("删除工具记录失败: {e}"))?;

        Ok(())
    }

    /// 检查工具更新
    pub fn check_tool_updates(db: &Arc<Database>) -> Result<Vec<ToolUpdateInfo>, String> {
        let tools = db
            .get_installed_tools()
            .map_err(|e| format!("查询已安装工具失败: {e}"))?;

        let mut updates = Vec::new();

        for tool in tools {
            // 只检查用户工具（非依赖项）
            if tool.category != "tool" {
                continue;
            }

            if let Some(plugin) = get_plugin_by_id(&tool.id) {
                let detect = plugin.detect(Some(Path::new(&tool.install_root)));
                if detect.installed {
                    if let (Some(current), Some(new)) = (&tool.version, &detect.version) {
                        if current != new {
                            updates.push(ToolUpdateInfo {
                                tool_id: tool.id,
                                current_version: current.clone(),
                                latest_version: new.clone(),
                            });
                        }
                    }
                }
            }
        }

        Ok(updates)
    }
}

/// 根据工具 ID 推断可执行文件名
fn get_exe_name(tool_id: &str) -> String {
    match tool_id {
        "nodejs" => "node".to_string(),
        "claude-code-cli" | "claude-code-desktop" => "claude".to_string(),
        "codex-cli" | "codex-desktop" => "codex".to_string(),
        "gemini-cli" => "gemini".to_string(),
        "opencode-cli" | "opencode-desktop" => "opencode".to_string(),
        "openclaw" => "openclaw".to_string(),
        "hermes" => "hermes".to_string(),
        _ => tool_id.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        allows_uninstall_outside_managed_root, choose_npm_registry_source,
        detect_successful_install, forward_install_progress_with, get_exe_name,
        is_install_owned_by_root, should_delete_install_dir, should_ignore_uninstall_error,
        should_publish_managed_shims, should_remove_shim,
    };
    use crate::plugin::NpmRegistrySource;
    use crate::tool_types::InstallProgress;
    use crate::tool_types::{DetectResult, InstallStrategy, NetworkStatus};
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;
    use tokio::sync::mpsc;

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
        assert!(!should_remove_shim(InstallStrategy::ManagedPrefix, false));
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
        assert_eq!(get_exe_name("nodejs"), "node");
    }

    #[test]
    fn npm_registry_source_prefers_official_when_npm_is_reachable() {
        let network = NetworkStatus {
            github_reachable: true,
            npm_reachable: true,
            youtube_reachable: false,
            error_message: None,
        };

        assert_eq!(
            choose_npm_registry_source(&network),
            NpmRegistrySource::Official
        );
    }

    #[test]
    fn npm_registry_source_falls_back_to_mirror_when_npm_is_unreachable() {
        let network = NetworkStatus {
            github_reachable: false,
            npm_reachable: false,
            youtube_reachable: false,
            error_message: Some("npm unavailable".to_string()),
        };

        assert_eq!(
            choose_npm_registry_source(&network),
            NpmRegistrySource::Mirror
        );
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
