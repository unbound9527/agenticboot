//! 安装引擎服务
//!
//! AgenticBoot 的核心安装引擎，封装网络检测、安装计划执行和工具卸载。
//! 通过 Tauri events 向推送安装进度。

pub mod dependency_resolver;
pub mod path_manager;

use crate::database::InstalledToolRecord;
use crate::database::Database;
use crate::plugin::get_plugin_by_id;
use crate::tool_types::{InstallPlan, InstallProgress, NetworkStatus, ToolUpdateInfo};
use path_manager::PathManager;
use std::path::Path;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;

/// npm 包名映射表（tool_id → npm package name）
const NPM_TOOL_MAP: &[(&str, &str)] = &[
    ("claude-code-cli", "@anthropic-ai/claude-code"),
    ("claude-code-desktop", "@anthropic-ai/claude-code"),
    ("codex-cli", "@openai/codex"),
    ("codex-desktop", "@openai/codex"),
    ("gemini-cli", "@google/gemini-cli"),
];

fn get_npm_package(tool_id: &str) -> Option<&'static str> {
    NPM_TOOL_MAP.iter().find(|(id, _)| *id == tool_id).map(|(_, pkg)| *pkg)
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
        // 确保 bin 目录和 PATH 已就绪
        self.path_manager.ensure_bin_dir()?;
        self.path_manager.register_in_path()?;

        for step in &plan.steps {
            // 跳过已安装
            if step.is_installed {
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

            let target_dir = self.path_manager.get_tool_install_dir(&step.tool_id);

            // 创建进度通道
            let (tx, mut rx) = mpsc::channel::<InstallProgress>(32);

            let install_target = target_dir.clone();
            let install_handle = app_handle.clone();
            let install_tool_id = step.tool_id.clone();
            let install_tool_name = install_tool_name.clone();
            let _install_category = step.category.clone();

            // 在独立线程中执行安装
            let install_result = tokio::task::spawn_blocking(move || {
                plugin.install(&install_target, tx)
            })
            .await
            .map_err(|e| format!("安装线程错误: {e}"))?;

            // 消费进度通道（即使安装已完成，也需要排空）
            while let Ok(progress) = rx.try_recv() {
                let _ = install_handle.emit("install-progress", progress);
            }

            // 重新获取插件以进行后续操作
            let post_plugin = get_plugin_by_id(&install_tool_id)
                .ok_or_else(|| format!("未知工具: {}", install_tool_id))?;

            match install_result {
                Ok(()) => {
                    // 检测已安装版本（传入安装根目录）
                    let detect = post_plugin.detect(Some(&self.root_path));
                    let version = detect.version;

                    // 创建 shim
                    if let Some(npm_pkg) = get_npm_package(&install_tool_id) {
                        self.path_manager.create_npm_shim(&install_tool_id, npm_pkg).ok();
                    } else {
                        let exe_name = get_exe_name(&install_tool_id);
                        let exe_path = target_dir
                            .join("bin")
                            .join(&exe_name)
                            .to_string_lossy()
                            .to_string();
                        self.path_manager.create_shim(&install_tool_id, &exe_path).ok();
                    }

                    // 更新数据库
                    let now = chrono::Utc::now().timestamp();
                    db.upsert_installed_tool(&InstalledToolRecord {
                        id: install_tool_id.clone(),
                        name: install_tool_name.clone(),
                        version,
                        install_path: target_dir.to_string_lossy().to_string(),
                        install_root: self.root_path.to_string_lossy().to_string(),
                        category: step.category.clone(),
                        status: "installed".to_string(),
                        installed_at: Some(now),
                        updated_at: Some(now),
                    })
                    .map_err(|e| format!("保存安装记录失败: {e}"))?;

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
    pub fn uninstall_tool(&self, tool_id: &str, db: &Arc<Database>) -> Result<(), String> {
        let record = db
            .get_installed_tool(tool_id)
            .map_err(|e| format!("查询工具记录失败: {e}"))?
            .ok_or_else(|| format!("未找到已安装工具: {tool_id}"))?;

        let plugin = get_plugin_by_id(tool_id)
            .ok_or_else(|| format!("未知工具: {tool_id}"))?;

        let target_dir = Path::new(&record.install_path);

        // 调用插件卸载
        plugin.uninstall(target_dir)?;

        // 移除 shim
        self.path_manager.remove_shim(tool_id)?;

        // 删除安装目录
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir)
                .map_err(|e| format!("删除安装目录失败: {e}"))?;
        }

        // 删除数据库记录
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
        "claude-code-cli" | "claude-code-desktop" => "claude".to_string(),
        "codex-cli" | "codex-desktop" => "codex".to_string(),
        "gemini-cli" => "gemini".to_string(),
        "opencode-cli" | "opencode-desktop" => "opencode".to_string(),
        "openclaw" => "openclaw".to_string(),
        "hermes" => "hermes".to_string(),
        _ => tool_id.to_string(),
    }
}
