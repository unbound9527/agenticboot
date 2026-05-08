use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use tokio::sync::mpsc::Sender;

pub struct ClaudeCodeDesktopPlugin;

impl ToolPlugin for ClaudeCodeDesktopPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "claude-code-desktop".into(),
            name: "Claude Code (桌面版)".into(),
            description: "Anthropic 官方 Claude Code 桌面应用".into(),
            icon: "claude".into(),
            category: "ai-cli".into(),
        }
    }

    fn detect(&self) -> DetectResult {
        // Check common install paths for Claude Code Desktop
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let possible_paths = vec![
            Path::new(&local_app_data).join("Programs").join("Claude Code"),
            Path::new("C:\\Program Files\\Claude Code").to_path_buf(),
        ];
        for p in &possible_paths {
            if p.exists() && p.join("Claude Code.exe").exists() {
                return DetectResult {
                    installed: true,
                    version: None,
                    install_path: Some(p.to_string_lossy().to_string()),
                };
            }
        }
        DetectResult {
            installed: false,
            version: None,
            install_path: None,
        }
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![] // Desktop app bundles its own runtime
    }

    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("Claude Code 桌面版将通过 GitHub Release 下载安装，此功能将在实现下载引擎后可用".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir)
                .map_err(|e| format!("删除目录失败: {e}"))?;
        }
        Ok(())
    }
}
