use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct NodeJsPlugin;

impl ToolPlugin for NodeJsPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "nodejs".into(), name: "Node.js".into(),
            description: "JavaScript 运行时，CLI 工具的必要依赖".into(),
            icon: "nodejs".into(), category: "dependency".into(),
        }
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        if let Ok(output) = Command::new("node").arg("--version").output() {
            if output.status.success() {
                return DetectResult {
                    installed: true,
                    version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
                    install_path: None,
                };
            }
        }
        if let Some(root) = install_root {
            let node_exe = root.join("nodejs").join("node.exe");
            if node_exe.exists() {
                return DetectResult { installed: true, version: None, install_path: Some(root.join("nodejs").to_string_lossy().to_string()) };
            }
        }
        DetectResult { installed: false, version: None, install_path: None }
    }

    fn dependencies(&self) -> Vec<ToolDependency> { vec![] }

    #[cfg(target_os = "windows")]
    fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
        let node_exe_dest = target_dir.join("node.exe");
        let url = "https://nodejs.org/dist/latest-v22.x/win-x64/node.exe";
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "nodejs".into(), tool_name: "Node.js".into(),
            phase: "downloading".into(), percent: 0,
            message: "正在下载 Node.js...".into(),
        });
        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建 runtime 失败: {e}"))?;
        rt.block_on(async { crate::services::downloader::download_file(url, &node_exe_dest, None).await })?;
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "nodejs".into(), tool_name: "Node.js".into(),
            phase: "complete".into(), percent: 100,
            message: "Node.js 安装完成".into(),
        });
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("Node.js 自动安装目前仅支持 Windows，macOS/Linux 请手动安装".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        if target_dir.exists() { std::fs::remove_dir_all(target_dir).map_err(|e| format!("删除失败: {e}"))?; }
        Ok(())
    }
}
