use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct OpenClawPlugin;

impl ToolPlugin for OpenClawPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "openclaw".into(),
            name: "OpenClaw".into(),
            description: "可编程 AI 编码引擎".into(),
            icon: "openclaw".into(),
            category: "ai-cli".into(),
        }
    }

    fn detect(&self) -> DetectResult {
        match Command::new("openclaw").arg("--version").output() {
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

    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("OpenClaw 需要从 GitHub Release 下载二进制文件，此功能将在后续版本实现".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir)
                .map_err(|e| format!("删除目录失败: {e}"))?;
        }
        Ok(())
    }
}
