use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct NodeJsPlugin;

impl ToolPlugin for NodeJsPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "nodejs".into(),
            name: "Node.js".into(),
            description: "JavaScript 运行时，CLI 工具的必要依赖".into(),
            icon: "nodejs".into(),
            category: "dependency".into(),
        }
    }

    fn detect(&self) -> DetectResult {
        match Command::new("node").arg("--version").output() {
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
        Err("Node.js 安装需要连接网络下载官方安装包，此功能将在后续版本实现".into())
    }

    fn uninstall(&self, _target_dir: &Path) -> Result<(), String> {
        Err("暂不支持卸载系统级依赖".into())
    }
}
