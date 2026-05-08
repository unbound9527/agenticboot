use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use tokio::sync::mpsc::Sender;

pub struct CodexDesktopPlugin;

impl ToolPlugin for CodexDesktopPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "codex-desktop".into(),
            name: "Codex (桌面版)".into(),
            description: "OpenAI Codex 桌面应用".into(),
            icon: "codex".into(),
            category: "ai-cli".into(),
        }
    }

    fn detect(&self) -> DetectResult {
        let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let possible_paths = vec![
            Path::new(&local_app_data).join("Programs").join("Codex"),
            Path::new("C:\\Program Files\\Codex").to_path_buf(),
        ];
        for p in &possible_paths {
            if p.exists() && p.join("Codex.exe").exists() {
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
        vec![]
    }

    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("Codex 桌面版需要下载安装包，此功能将在后续版本实现".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir)
                .map_err(|e| format!("删除目录失败: {e}"))?;
        }
        Ok(())
    }
}
