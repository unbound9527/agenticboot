use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct GitPlugin;

impl ToolPlugin for GitPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "git".into(), name: "Git".into(),
            description: "版本控制系统，部分工具的依赖".into(),
            icon: "git".into(), category: "dependency".into(),
        }
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        if let Ok(output) = Command::new("git").arg("--version").output() {
            if output.status.success() {
                return DetectResult {
                    installed: true,
                    version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
                    install_path: None,
                };
            }
        }
        if let Some(root) = install_root {
            let git_exe = root.join("git").join("bin").join("git.exe");
            if git_exe.exists() {
                return DetectResult { installed: true, version: None, install_path: Some(root.join("git").to_string_lossy().to_string()) };
            }
        }
        DetectResult { installed: false, version: None, install_path: None }
    }

    fn dependencies(&self) -> Vec<ToolDependency> { vec![] }

    #[cfg(target_os = "windows")]
    fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
        let git_zip = target_dir.join("MinGit.zip");
        let url = "https://github.com/git-for-windows/git/releases/download/v2.51.0.windows.1/MinGit-2.51.0-64-bit.zip";
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "git".into(), tool_name: "Git".into(),
            phase: "downloading".into(), percent: 0,
            message: "正在下载 Git...".into(),
        });
        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建 runtime 失败: {e}"))?;
        rt.block_on(async { crate::services::downloader::download_file(url, &git_zip, None).await })?;
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "git".into(), tool_name: "Git".into(),
            phase: "extracting".into(), percent: 50,
            message: "正在解压 Git...".into(),
        });
        crate::services::downloader::extract_zip(&git_zip, target_dir)?;
        std::fs::remove_file(&git_zip).ok();
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "git".into(), tool_name: "Git".into(),
            phase: "complete".into(), percent: 100,
            message: "Git 安装完成".into(),
        });
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("Git 自动安装目前仅支持 Windows，macOS/Linux 请手动安装".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        if target_dir.exists() { std::fs::remove_dir_all(target_dir).map_err(|e| format!("删除失败: {e}"))?; }
        Ok(())
    }
}
