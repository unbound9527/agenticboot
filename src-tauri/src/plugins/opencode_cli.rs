use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct OpenCodeCliPlugin;

impl ToolPlugin for OpenCodeCliPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta { id: "opencode-cli".into(), name: "OpenCode (CLI)".into(),
            description: "开源 AI 编程 CLI 工具".into(), icon: "opencode".into(), category: "ai-cli".into() }
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        if let Ok(output) = Command::new("opencode").arg("--version").output() {
            if output.status.success() {
                return DetectResult { installed: true, version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()), install_path: None };
            }
        }
        if let Some(root) = install_root {
            let exe = root.join("opencode-cli").join("bin").join("opencode.cmd");
            if exe.exists() { return DetectResult { installed: true, version: None, install_path: Some(root.join("opencode-cli").to_string_lossy().to_string()) }; }
        }
        DetectResult { installed: false, version: None, install_path: None }
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![ToolDependency { tool_id: "nodejs".into(), min_version: Some(">= 18.0.0".into()) }]
    }

    #[cfg(target_os = "windows")]
    fn install(&self, target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        let output = Command::new("npm").args(["install", "-g", "opencode", "--prefix", &target_dir.to_string_lossy()])
            .output().map_err(|e| format!("npm install 失败: {e}"))?;
        if !output.status.success() { return Err(format!("npm install 失败: {}", String::from_utf8_lossy(&output.stderr))); }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("OpenCode CLI 自动安装目前仅支持 Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        Command::new("npm").args(["uninstall", "-g", "opencode", "--prefix", &target_dir.to_string_lossy()])
            .output().map_err(|e| format!("npm uninstall 失败: {e}"))?;
        Ok(())
    }
}
