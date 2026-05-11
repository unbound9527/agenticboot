use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{
    find_local_uninstaller_executable, find_uninstall_entry_ex,
    run_windows_uninstaller_with_common_args,
};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use log::debug;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct OpenCodeDesktopPlugin;

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
            if let Some(entry) = find_uninstall_entry_ex(&["OpenCode"], &["CLI", "npm"]) {
                if let Some(uninstall_string) = entry.uninstall_string {
                    let status = Command::new("cmd")
                        .args(["/C", &uninstall_string])
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
                    return Ok(());
                }
            }

            if let Some(uninstaller) = find_local_uninstaller_executable(target_dir) {
                run_windows_uninstaller_with_common_args(&uninstaller)?;
                return Ok(());
            }

            return Err("未找到可自动卸载的 OpenCode 官方桌面应用。".into());
        }

        #[allow(unreachable_code)]
        Ok(())
    }
}
