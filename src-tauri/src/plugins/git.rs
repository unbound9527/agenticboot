use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct GitPlugin;

impl ToolPlugin for GitPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "git".into(),
            name: "Git".into(),
            description: "版本控制系统，部分工具的依赖".into(),
            icon: "git".into(),
            category: "dependency".into(),
        }
    }

    fn detect(&self) -> DetectResult {
        match Command::new("git").arg("--version").output() {
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
        vec![]
    }

    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("Git 安装需要连接网络下载官方安装包，此功能将在后续版本实现".into())
    }

    fn uninstall(&self, _target_dir: &Path) -> Result<(), String> {
        Err("暂不支持卸载系统级依赖".into())
    }
}
