use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct OpenClawPlugin;

impl ToolPlugin for OpenClawPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta { id: "openclaw".into(), name: "OpenClaw".into(),
            description: "可编程 AI 编码引擎".into(), icon: "openclaw".into(), category: "ai-cli".into() }
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        if let Ok(output) = Command::new("openclaw").arg("--version").output() {
            if output.status.success() {
                return DetectResult { installed: true, version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()), install_path: None };
            }
        }
        if let Some(root) = install_root {
            let exe = root.join("openclaw").join("bin").join("openclaw.cmd");
            if exe.exists() { return DetectResult { installed: true, version: None, install_path: Some(root.join("openclaw").to_string_lossy().to_string()) }; }
        }
        DetectResult { installed: false, version: None, install_path: None }
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![ToolDependency { tool_id: "nodejs".into(), min_version: Some(">= 18.0.0".into()) }]
    }

    #[cfg(target_os = "windows")]
    fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "openclaw".into(), tool_name: "OpenClaw".into(),
            phase: "installing".into(), percent: 0, message: "正在安装 OpenClaw...".into(),
        });
        let output = Command::new("npm").args(["install", "-g", "openclaw", "--prefix", &target_dir.to_string_lossy()])
            .output().map_err(|e| format!("npm install 失败: {e}"))?;
        if !output.status.success() { return Err(format!("npm install 失败: {}", String::from_utf8_lossy(&output.stderr))); }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("OpenClaw 自动安装目前仅支持 Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        Command::new("npm").args(["uninstall", "-g", "openclaw", "--prefix", &target_dir.to_string_lossy()])
            .output().map_err(|e| format!("npm uninstall 失败: {e}"))?;
        if target_dir.exists() { std::fs::remove_dir_all(target_dir).ok(); }
        Ok(())
    }
}
