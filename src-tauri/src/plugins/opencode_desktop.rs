use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct OpenCodeDesktopPlugin;

impl ToolPlugin for OpenCodeDesktopPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta { id: "opencode-desktop".into(), name: "OpenCode (桌面版)".into(),
            description: "OpenCode 桌面独立安装（自带运行时）".into(), icon: "opencode".into(), category: "ai-cli".into() }
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        if let Some(root) = install_root {
            let exe = root.join("opencode-desktop").join("bin").join("opencode.cmd");
            if exe.exists() { return DetectResult { installed: true, version: None, install_path: Some(root.join("opencode-desktop").to_string_lossy().to_string()) }; }
        }
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let p = Path::new(&local).join("Programs").join("OpenCode");
            if p.join("OpenCode.exe").exists() { return DetectResult { installed: true, version: None, install_path: Some(p.to_string_lossy().to_string()) }; }
        }
        DetectResult { installed: false, version: None, install_path: None }
    }

    fn dependencies(&self) -> Vec<ToolDependency> { vec![] }

    #[cfg(target_os = "windows")]
    fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-desktop".into(), tool_name: "OpenCode (桌面版)".into(),
            phase: "installing".into(), percent: 0, message: "正在安装 OpenCode 桌面版...".into(),
        });
        let output = Command::new("npm").args(["install", "-g", "opencode", "--prefix", &target_dir.to_string_lossy()])
            .output().map_err(|e| format!("npm install 失败: {e}"))?;
        if !output.status.success() { return Err(format!("npm install 失败: {}", String::from_utf8_lossy(&output.stderr))); }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("OpenCode 桌面版自动安装目前仅支持 Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        Command::new("npm").args(["uninstall", "-g", "opencode", "--prefix", &target_dir.to_string_lossy()])
            .output().map_err(|e| format!("npm uninstall 失败: {e}"))?;
        if target_dir.exists() { std::fs::remove_dir_all(target_dir).ok(); }
        Ok(())
    }
}
