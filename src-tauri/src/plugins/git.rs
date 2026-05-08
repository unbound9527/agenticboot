use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{
    find_executable_in_dir, find_managed_paths, read_command_version,
};
use crate::tool_types::{
    DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta,
};
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

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::ManagedPrefix
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        if let Some(root) = install_root {
            let detect_paths = find_managed_paths(root, "git", &["cmd\\git.exe", "bin\\git.exe"]);
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

        if let Ok(output) = Command::new("git").arg("--version").output() {
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
    fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
        let git_zip = target_dir.join("MinGit.zip");
        let url = "https://github.com/git-for-windows/git/releases/download/v2.51.0.windows.1/MinGit-2.51.0-64-bit.zip";
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "git".into(),
            tool_name: "Git".into(),
            phase: "downloading".into(),
            percent: 0,
            message: "正在下载 Git...".into(),
        });

        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| format!("创建 runtime 失败: {e}"))?;
        rt.block_on(async { crate::services::downloader::download_file(url, &git_zip, None).await })?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "git".into(),
            tool_name: "Git".into(),
            phase: "extracting".into(),
            percent: 50,
            message: "正在解压 Git...".into(),
        });

        crate::services::downloader::extract_zip(&git_zip, target_dir)?;
        std::fs::remove_file(&git_zip).ok();

        flatten_mingit_dir(target_dir)?;

        if find_executable_in_dir(target_dir, &["cmd\\git.exe", "bin\\git.exe"]).is_none() {
            return Err(format!(
                "Git 安装异常：未在 {} 找到 git.exe",
                target_dir.display()
            ));
        }

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "git".into(),
            tool_name: "Git".into(),
            phase: "complete".into(),
            percent: 100,
            message: "Git 安装完成".into(),
        });
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("Git 自动安装目前仅支持 Windows，macOS/Linux 请手动安装".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir)
                .map_err(|e| format!("删除失败: {e}"))?;
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn flatten_mingit_dir(target_dir: &Path) -> Result<(), String> {
    let entries: Vec<_> = std::fs::read_dir(target_dir)
        .map_err(|e| format!("读取安装目录失败: {e}"))?
        .filter_map(|e| e.ok())
        .collect();

    for entry in entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with("MinGit-")
            && entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
        {
            let sub_dir = entry.path();
            for item in std::fs::read_dir(&sub_dir)
                .map_err(|e| format!("读取 MinGit 目录失败: {e}"))?
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
    use super::GitPlugin;
    use crate::plugin::ToolPlugin;

    #[test]
    fn windows_paths_git_detects_cmd_level_git_exe_after_zip_extract() {
        let tmp = tempfile::tempdir().unwrap();
        let git_dir = tmp.path().join("git").join("cmd");
        std::fs::create_dir_all(&git_dir).unwrap();
        std::fs::write(git_dir.join("git.exe"), b"").unwrap();

        let detect = GitPlugin.detect(Some(tmp.path()));
        assert!(detect.installed);
        assert_eq!(
            detect.install_path.as_deref(),
            Some(tmp.path().join("git").to_string_lossy().as_ref())
        );
    }
}
