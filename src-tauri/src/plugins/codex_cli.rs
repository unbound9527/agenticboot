use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct CodexCliPlugin;

impl ToolPlugin for CodexCliPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "codex-cli".into(),
            name: "Codex (CLI)".into(),
            description: "OpenAI 官方 CLI AI 编程助手".into(),
            icon: "codex".into(),
            category: "ai-cli".into(),
        }
    }

    fn detect(&self) -> DetectResult {
        match Command::new("codex").arg("--version").output() {
            Ok(output) if output.status.success() => {
                let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
                DetectResult {
                    installed: true,
                    version: Some(version),
                    install_path: None,
                }
            }
            _ => DetectResult {
                installed: false,
                version: None,
                install_path: None,
            },
        }
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![ToolDependency {
            tool_id: "nodejs".into(),
            min_version: Some(">= 18.0.0".into()),
        }]
    }

    fn install(&self, target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        let output = Command::new("npm")
            .args([
                "install", "-g", "@anthropic-ai/codex",
                "--prefix", &target_dir.to_string_lossy(),
            ])
            .output()
            .map_err(|e| format!("执行 npm install 失败: {e}"))?;

        if !output.status.success() {
            return Err(format!("npm install 失败: {}", String::from_utf8_lossy(&output.stderr)));
        }
        Ok(())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        let output = Command::new("npm")
            .args([
                "uninstall", "-g", "@anthropic-ai/codex",
                "--prefix", &target_dir.to_string_lossy(),
            ])
            .output()
            .map_err(|e| format!("执行 npm uninstall 失败: {e}"))?;

        if !output.status.success() {
            return Err(format!("npm uninstall 失败: {}", String::from_utf8_lossy(&output.stderr)));
        }
        Ok(())
    }
}
