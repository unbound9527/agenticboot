use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{
    find_appx_install_location, find_uninstall_entry_ex, run_winget, winget_exists,
};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use log::debug;
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct CodexDesktopPlugin;

impl ToolPlugin for CodexDesktopPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "codex-desktop".into(),
            name: "Codex (桌面版)".into(),
            description: "Codex 官方 Windows 桌面应用".into(),
            icon: "codex".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::DesktopInstaller
    }

    fn detect(&self, _install_root: Option<&Path>) -> DetectResult {
        if let Some(entry) = find_uninstall_entry_ex(&["Codex", "OpenAI Codex"], &["CLI", "npm"]) {
            let install_path = entry.install_location.or(entry
                .display_icon
                .and_then(|path| path.parent().map(|p| p.to_path_buf())));

            debug!(
                "detected Codex via registry: version={:?}, path={:?}",
                entry.display_version, install_path
            );
            return DetectResult {
                installed: true,
                version: entry.display_version,
                install_path: install_path.map(|dir| dir.to_string_lossy().to_string()),
            };
        }

        if let Some((location, version)) = find_appx_install_location("OpenAI.Codex") {
            debug!(
                "detected Codex via AppX: version={:?}, path={:?}",
                version, location
            );
            return DetectResult {
                installed: true,
                version,
                install_path: Some(location),
            };
        }

        debug!("Codex desktop not found");
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
            tool_id: "codex-desktop".into(),
            tool_name: "Codex 桌面版".into(),
            phase: "installing".into(),
            percent: 0,
            message: "正在通过 Microsoft Store 安装 Codex 应用...".into(),
        });

        if !winget_exists() {
            return Err("安装 Codex 桌面版需要 Windows App Installer/winget。".into());
        }

        run_winget(&[
            "install",
            "Codex",
            "-s",
            "msstore",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ])
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("Codex 桌面版自动安装目前仅支持 Windows".into())
    }

    fn uninstall(&self, _target_dir: &Path) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let status = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    "Get-AppxPackage OpenAI.Codex | Remove-AppxPackage",
                ])
                .spawn()
                .map_err(|e| format!("启动 Codex 卸载失败: {e}"))?
                .wait()
                .map_err(|e| format!("等待 Codex 卸载完成失败: {e}"))?;
            if !status.success() {
                return Err(format!("Codex 卸载失败，code: {:?}", status.code()));
            }
            return Ok(());
        }

        #[allow(unreachable_code)]
        Ok(())
    }
}
