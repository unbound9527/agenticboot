use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{
    find_executable_in_dir, find_managed_paths, read_command_version, run_detection_command_output,
};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
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

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::ManagedPrefix
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        if let Some(root) = install_root {
            let detect_paths = find_managed_paths(root, "nodejs", &["node.exe", "bin\\node.exe"]);
            if let Some(executable) = detect_paths.executable.as_ref() {
                return DetectResult {
                    installed: true,
                    version: read_command_version(executable, &["--version"]),
                    install_path: detect_paths
                        .install_root
                        .map(|path| path.to_string_lossy().to_string()),
                };
            }
        }

        let mut command = Command::new("node");
        command.arg("--version");
        if let Ok(output) = run_detection_command_output(&mut command, "node") {
            if output.status.success() {
                return DetectResult {
                    installed: true,
                    version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
                    install_path: None,
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

    #[cfg(target_os = "windows")]
    fn install(
        &self,
        target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        let zip_path = target_dir.join("nodejs.zip");
        let url = "https://nodejs.org/dist/v22.15.0/win-x64/node-v22.15.0-win-x64.zip";
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "nodejs".into(),
            tool_name: "Node.js".into(),
            phase: "downloading".into(),
            percent: 0,
            message: "正在下载 Node.js...".into(),
        });

        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建 runtime 失败: {e}"))?;
        rt.block_on(async {
            crate::services::downloader::download_file(url, &zip_path, None).await
        })?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "nodejs".into(),
            tool_name: "Node.js".into(),
            phase: "extracting".into(),
            percent: 50,
            message: "正在解压 Node.js...".into(),
        });

        crate::services::downloader::extract_zip(&zip_path, target_dir)?;
        std::fs::remove_file(&zip_path).ok();

        flatten_nodejs_dir(target_dir)?;

        if find_executable_in_dir(target_dir, &["node.exe", "bin\\node.exe"]).is_none() {
            return Err(format!(
                "Node.js 安装异常：未在 {} 找到 node.exe",
                target_dir.display()
            ));
        }

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "nodejs".into(),
            tool_name: "Node.js".into(),
            phase: "complete".into(),
            percent: 100,
            message: "Node.js 安装完成".into(),
        });
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("Node.js 自动安装目前仅支持 Windows，macOS/Linux 请手动安装".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir).map_err(|e| format!("删除失败: {e}"))?;
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn flatten_nodejs_dir(target_dir: &Path) -> Result<(), String> {
    let entries: Vec<_> = std::fs::read_dir(target_dir)
        .map_err(|e| format!("读取安装目录失败: {e}"))?
        .filter_map(|e| e.ok())
        .collect();

    for entry in entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("node-v") && entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
        {
            let sub_dir = entry.path();
            for item in std::fs::read_dir(&sub_dir)
                .map_err(|e| format!("读取 node 目录失败: {e}"))?
                .filter_map(|e| e.ok())
            {
                let src = item.path();
                let dst = target_dir.join(item.file_name());
                std::fs::rename(&src, &dst)
                    .or_else(|_| {
                        if dst.exists() {
                            Ok(())
                        } else {
                            std::fs::copy(&src, &dst).map(|_| ())
                        }
                    })
                    .map_err(|e| format!("移动文件失败: {e}"))?;
            }
            std::fs::remove_dir(&sub_dir).ok();
            break;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::NodeJsPlugin;
    use crate::plugin::ToolPlugin;

    #[test]
    fn windows_paths_nodejs_detects_root_level_node_exe_after_zip_extract() {
        let tmp = tempfile::tempdir().unwrap();
        let node_dir = tmp.path().join("nodejs");
        std::fs::create_dir_all(&node_dir).unwrap();
        std::fs::write(node_dir.join("node.exe"), b"").unwrap();

        let detect = NodeJsPlugin.detect(Some(tmp.path()));
        assert!(detect.installed);
        assert_eq!(
            detect.install_path.as_deref(),
            Some(node_dir.to_string_lossy().as_ref())
        );
    }
}
