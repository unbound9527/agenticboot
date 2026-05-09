use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{find_uninstall_entry_ex, run_winget, winget_exists};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use log::debug;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct ClaudeCodeDesktopPlugin;

impl ToolPlugin for ClaudeCodeDesktopPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "claude-code-desktop".into(),
            name: "Claude Code (桌面版)".into(),
            description: "Claude 官方 Windows 桌面应用".into(),
            icon: "claude".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::DesktopInstaller
    }

    fn detect(&self, _install_root: Option<&Path>) -> DetectResult {
        if let Some(entry) =
            find_uninstall_entry_ex(&["Claude", "AnthropicClaude"], &["CLI", "npm"])
        {
            // Skip CLI installations (identified by AnthropicClaude path)
            if entry
                .install_location
                .as_ref()
                .is_some_and(|p| p.to_string_lossy().contains("AnthropicClaude"))
            {
                debug!(
                    "detected Claude but skipping CLI at {:?}",
                    entry.install_location
                );
                return DetectResult::not_installed();
            }

            let install_path = entry.install_location.or(entry
                .display_icon
                .and_then(|path| path.parent().map(PathBuf::from)));

            debug!(
                "detected Claude desktop: version={:?}, path={:?}",
                entry.display_version, install_path
            );
            return DetectResult {
                installed: true,
                version: entry.display_version,
                install_path: install_path.map(|dir| dir.to_string_lossy().to_string()),
            };
        }

        debug!("Claude desktop not found in registry");
        DetectResult::not_installed()
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![]
    }

    #[cfg(target_os = "windows")]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "claude-code-desktop".into(),
            tool_name: "Claude 桌面版".into(),
            phase: "installing".into(),
            percent: 0,
            message: "正在安装 Claude 官方桌面应用...".into(),
        });

        if winget_exists()
            && run_winget(&[
                "install",
                "--id",
                "Anthropic.Claude",
                "-e",
                "--accept-package-agreements",
                "--accept-source-agreements",
            ])
            .is_ok()
        {
            return Ok(());
        }

        let installer = crate::services::downloader::temp_path("claude-desktop-setup.exe");
        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建 runtime 失败: {e}"))?;
        rt.block_on(async {
            crate::services::downloader::download_file(windows_download_url(), &installer, None)
                .await
        })?;

        let status = Command::new(&installer)
            .spawn()
            .map_err(|e| format!("启动 Claude 安装程序失败: {e}"))?
            .wait()
            .map_err(|e| format!("等待 Claude 安装程序结束失败: {e}"))?;
        if !status.success() {
            return Err(format!(
                "Claude 安装程序异常退出，code: {:?}",
                status.code()
            ));
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("Claude 桌面版自动安装目前仅支持 Windows".into())
    }

    fn uninstall(&self, _target_dir: &Path) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            if winget_exists()
                && run_winget(&[
                    "uninstall",
                    "--id",
                    "Anthropic.Claude",
                    "-e",
                    "--accept-source-agreements",
                ])
                .is_ok()
            {
                return Ok(());
            }

            if let Some(entry) =
                find_uninstall_entry_ex(&["Claude", "AnthropicClaude"], &["CLI", "npm"])
            {
                if let Some(uninstall_string) = entry.uninstall_string {
                    let status = Command::new("cmd")
                        .args(["/C", &uninstall_string])
                        .spawn()
                        .map_err(|e| format!("启动 Claude 卸载程序失败: {e}"))?
                        .wait()
                        .map_err(|e| format!("等待 Claude 卸载程序结束失败: {e}"))?;
                    if !status.success() {
                        return Err(format!(
                            "Claude 卸载程序异常退出，code: {:?}",
                            status.code()
                        ));
                    }
                    return Ok(());
                }
            }

            return Err("未找到可自动卸载的 Claude 官方桌面应用。".into());
        }

        #[allow(unreachable_code)]
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn windows_download_url() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "https://claude.ai/api/desktop/win32/arm64/setup/latest/redirect"
    } else {
        "https://claude.ai/api/desktop/win32/x64/setup/latest/redirect"
    }
}
