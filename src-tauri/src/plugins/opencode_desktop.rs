use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{
    find_local_uninstaller_executable, find_uninstall_entry_ex,
    run_windows_uninstaller_with_common_args, WindowsUninstallEntry,
};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use log::debug;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct OpenCodeDesktopPlugin;

#[cfg(target_os = "windows")]
fn run_open_code_registry_uninstall(uninstall_string: &str) -> Result<(), String> {
    let status = Command::new("cmd")
        .args(["/C", uninstall_string])
        .spawn()
        .map_err(|e| format!("启动 OpenCode 卸载程序失败: {e}"))?
        .wait()
        .map_err(|e| format!("等待 OpenCode 卸载程序结束失败: {e}"))?;
    if !status.success() {
        return Err(format!(
            "OpenCode 卸载程序异常退出，code: {:?}",
            status.code()
        ));
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn uninstall_opencode_desktop_with<F, G>(
    candidate_dirs: &[PathBuf],
    registry_uninstall_string: Option<&str>,
    run_registry_uninstall: F,
    mut run_local_uninstaller: G,
) -> Result<(), String>
where
    F: FnOnce(&str) -> Result<(), String>,
    G: FnMut(&Path) -> Result<(), String>,
{
    if let Some(uninstall_string) = registry_uninstall_string {
        match run_registry_uninstall(uninstall_string) {
            Ok(()) => return Ok(()),
            Err(error) => {
                log::warn!(
                    "[OpenCode Desktop] registry uninstall failed, falling back to local uninstaller: {}",
                    error
                );
            }
        }
    }

    let mut last_error = None;
    for candidate_dir in candidate_dirs {
        match run_local_uninstaller(candidate_dir) {
            Ok(()) => return Ok(()),
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| "未找到可自动卸载的 OpenCode 官方桌面应用。".into()))
}

fn opencode_uninstall_candidate_dirs(
    target_dir: &Path,
    entry: Option<&WindowsUninstallEntry>,
) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    push_unique_dir(&mut dirs, target_dir.to_path_buf());

    if let Some(entry) = entry {
        if let Some(install_location) = entry.install_location.as_ref() {
            push_unique_dir(&mut dirs, install_location.clone());
        }
        if let Some(display_icon) = entry.display_icon.as_ref().and_then(|path| path.parent()) {
            push_unique_dir(&mut dirs, display_icon.to_path_buf());
        }
        if let Some(uninstaller_dir) = entry
            .uninstall_string
            .as_deref()
            .and_then(extract_uninstall_exe_path)
            .and_then(|path| path.parent().map(PathBuf::from))
        {
            push_unique_dir(&mut dirs, uninstaller_dir);
        }
    }

    dirs
}

fn push_unique_dir(dirs: &mut Vec<PathBuf>, dir: PathBuf) {
    if !dirs.iter().any(|existing| existing == &dir) {
        dirs.push(dir);
    }
}

fn extract_uninstall_exe_path(command: &str) -> Option<PathBuf> {
    let trimmed = command.trim();
    if let Some(rest) = trimmed.strip_prefix('"') {
        let end = rest.find('"')?;
        let path = rest[..end].trim();
        return (!path.is_empty()).then(|| PathBuf::from(path));
    }

    let lower = trimmed.to_ascii_lowercase();
    let exe_end = lower.find(".exe")? + ".exe".len();
    let path = trimmed[..exe_end].trim();
    (!path.is_empty()).then(|| PathBuf::from(path))
}

impl ToolPlugin for OpenCodeDesktopPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "opencode-desktop".into(),
            name: "OpenCode (桌面版)".into(),
            description: "OpenCode 官方 Windows 桌面应用".into(),
            icon: "opencode".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::DesktopInstaller
    }

    fn detect(&self, _install_root: Option<&Path>) -> DetectResult {
        if let Some(entry) = find_uninstall_entry_ex(&["OpenCode"], &["CLI", "npm"]) {
            let install_path = entry.install_location.or(entry
                .display_icon
                .and_then(|path| path.parent().map(PathBuf::from)));

            debug!(
                "detected OpenCode desktop: version={:?}, path={:?}",
                entry.display_version, install_path
            );
            return DetectResult {
                installed: true,
                version: entry.display_version,
                install_path: install_path.map(|dir| dir.to_string_lossy().to_string()),
            };
        }

        debug!("OpenCode desktop not found in registry");
        DetectResult::not_installed()
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![]
    }

    #[cfg(target_os = "windows")]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-desktop".into(),
            tool_name: "OpenCode 桌面版".into(),
            phase: "installing".into(),
            percent: 0,
            message: "正在下载安装 OpenCode 官方桌面应用...".into(),
        });

        let installer = crate::services::downloader::temp_path("opencode-desktop-setup.exe");
        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建 runtime 失败: {e}"))?;
        rt.block_on(async {
            crate::services::downloader::download_file(
                "https://opencode.ai/download/stable/windows-x64-nsis",
                &installer,
                None,
            )
            .await
        })?;

        let status = Command::new(&installer)
            .args(["/S"])
            .spawn()
            .map_err(|e| format!("启动 OpenCode 安装程序失败: {e}"))?
            .wait()
            .map_err(|e| format!("等待 OpenCode 安装程序结束失败: {e}"))?;
        if !status.success() {
            return Err(format!(
                "OpenCode 安装程序异常退出，code: {:?}",
                status.code()
            ));
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-desktop".into(),
            tool_name: "OpenCode 桌面版".into(),
            phase: "downloading".into(),
            percent: 0,
            message: "正在下载 OpenCode 桌面版...".into(),
        });

        let bin_dir = target_dir.join("bin");
        std::fs::create_dir_all(&bin_dir).map_err(|e| format!("创建 bin 目录失败: {e}"))?;

        let version = Self::fetch_latest_version()?;
        let (os, arch) = Self::get_os_arch();
        let tar_path = target_dir.join("opencode-desktop.tar.gz");
        let url = format!(
            "https://github.com/opencode-ai/opencode/releases/download/v{}/opencode-{}-{}.tar.gz",
            version, os, arch
        );

        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建 runtime 失败: {e}"))?;
        rt.block_on(async {
            crate::services::downloader::download_file(&url, &tar_path, None).await
        })?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-desktop".into(),
            tool_name: "OpenCode 桌面版".into(),
            phase: "extracting".into(),
            percent: 50,
            message: "正在解压...".into(),
        });

        crate::services::downloader::extract_tar_gz(&tar_path, target_dir)?;
        std::fs::remove_file(&tar_path).ok();

        let extracted_bin = target_dir.join("opencode");
        let final_bin = bin_dir.join("opencode");
        if !extracted_bin.exists() {
            return Err("OpenCode 桌面版下载异常：未找到解压后的二进制文件".to_string());
        }
        std::fs::rename(&extracted_bin, &final_bin)
            .or_else(|_| std::fs::copy(&extracted_bin, &final_bin).map(|_| ()))
            .map_err(|e| format!("移动 OpenCode 二进制文件失败: {e}"))?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-desktop".into(),
            tool_name: "OpenCode 桌面版".into(),
            phase: "complete".into(),
            percent: 100,
            message: "OpenCode 桌面版安装完成".into(),
        });
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn fetch_latest_version() -> Result<String, String> {
        let output = std::process::Command::new("curl")
            .args([
                "-s",
                "https://api.github.com/repos/opencode-ai/opencode/releases/latest",
            ])
            .output()
            .map_err(|e| format!("获取版本失败: {e}"))?;
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let line = line.trim();
            if line.starts_with("\"tag_name\"") {
                if let Some(v) = line.split(':').nth(1) {
                    let v = v.trim().trim_matches('"').trim_start_matches('v');
                    return Ok(v.to_string());
                }
            }
        }
        Err("无法解析 OpenCode 最新版本号".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    fn get_os_arch() -> (&'static str, &'static str) {
        #[cfg(target_os = "macos")]
        {
            (
                "mac",
                if cfg!(target_arch = "aarch64") {
                    "arm64"
                } else {
                    "x86_64"
                },
            )
        }
        #[cfg(target_os = "linux")]
        {
            (
                "linux",
                if cfg!(target_arch = "aarch64") {
                    "arm64"
                } else {
                    "x86_64"
                },
            )
        }
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            let entry = find_uninstall_entry_ex(&["OpenCode"], &["CLI", "npm"]);
            let candidate_dirs = opencode_uninstall_candidate_dirs(target_dir, entry.as_ref());
            let registry_uninstall_string = entry
                .as_ref()
                .and_then(|entry| entry.uninstall_string.as_deref());

            return uninstall_opencode_desktop_with(
                &candidate_dirs,
                registry_uninstall_string,
                run_open_code_registry_uninstall,
                |target_dir| {
                    if let Some(uninstaller) = find_local_uninstaller_executable(target_dir) {
                        run_windows_uninstaller_with_common_args(&uninstaller)
                    } else {
                        Err("未找到可自动卸载的 OpenCode 官方桌面应用。".into())
                    }
                },
            );
        }

        #[allow(unreachable_code)]
        Ok(())
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    use super::{
        extract_uninstall_exe_path, opencode_uninstall_candidate_dirs,
        uninstall_opencode_desktop_with,
    };
    use crate::services::installer::windows::WindowsUninstallEntry;
    use std::path::{Path, PathBuf};

    #[test]
    fn uninstall_falls_back_to_local_uninstaller_when_registry_command_exits_non_zero() {
        let target_dir = PathBuf::from("C:\\AgenticTools\\opencode-desktop");
        let candidate_dirs = vec![target_dir.clone(), PathBuf::from("D:\\opencode")];
        let mut registry_called = false;
        let mut local_attempts = Vec::new();

        let result = uninstall_opencode_desktop_with(
            &candidate_dirs,
            Some("\"C:\\Users\\me\\AppData\\Local\\Programs\\OpenCode\\Uninstall.exe\""),
            |_uninstall_string| {
                registry_called = true;
                Err("OpenCode 卸载程序异常退出，code: Some(1)".to_string())
            },
            |path| {
                local_attempts.push(path.to_path_buf());
                if path == Path::new("D:\\opencode") {
                    Ok(())
                } else {
                    Err("not found".to_string())
                }
            },
        );

        assert!(result.is_ok());
        assert!(registry_called);
        assert_eq!(local_attempts, candidate_dirs);
    }

    #[test]
    fn opencode_uninstall_candidates_include_display_icon_and_uninstall_string_dirs() {
        let target_dir = Path::new("D:\\AgenticTools\\opencode-desktop");
        let entry = WindowsUninstallEntry {
            display_name: "OpenCode 1.14.41".into(),
            display_version: Some("1.14.41".into()),
            install_location: None,
            display_icon: Some(PathBuf::from("D:\\opencode\\OpenCode.exe")),
            uninstall_string: Some("\"E:\\OpenCode\\Uninstall OpenCode.exe\" /allusers".into()),
        };

        let dirs = opencode_uninstall_candidate_dirs(target_dir, Some(&entry));

        assert_eq!(
            dirs,
            vec![
                PathBuf::from("D:\\AgenticTools\\opencode-desktop"),
                PathBuf::from("D:\\opencode"),
                PathBuf::from("E:\\OpenCode"),
            ]
        );
    }

    #[test]
    fn extract_uninstall_exe_path_handles_quoted_commands_with_args() {
        assert_eq!(
            extract_uninstall_exe_path("\"D:\\opencode\\Uninstall OpenCode.exe\" /allusers")
                .as_deref(),
            Some(Path::new("D:\\opencode\\Uninstall OpenCode.exe"))
        );
    }
}
