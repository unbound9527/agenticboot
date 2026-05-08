use crate::plugin::ToolPlugin;
use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct OpenCodeDesktopPlugin;

impl ToolPlugin for OpenCodeDesktopPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta { id: "opencode-desktop".into(), name: "OpenCode (桌面版)".into(),
            description: "OpenCode 桌面独立安装（仅 mac/Linux）".into(), icon: "opencode".into(), category: "ai-cli".into() }
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        if let Some(root) = install_root {
            let exe = root.join("opencode-desktop").join("bin").join("opencode");
            if exe.exists() { return DetectResult { installed: true, version: None, install_path: Some(root.join("opencode-desktop").to_string_lossy().to_string()) }; }
        }
        DetectResult { installed: false, version: None, install_path: None }
    }

    fn dependencies(&self) -> Vec<ToolDependency> { vec![] }

    #[cfg(target_os = "windows")]
    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("OpenCode 桌面版目前仅支持 macOS/Linux".to_string())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-desktop".into(), tool_name: "OpenCode 桌面版".into(),
            phase: "downloading".into(), percent: 0, message: "正在下载 OpenCode 桌面版...".into(),
        });

        let bin_dir = target_dir.join("bin");
        std::fs::create_dir_all(&bin_dir).map_err(|e| format!("创建 bin 目录失败: {e}"))?;

        let version = Self::fetch_latest_version()?;
        let (os, arch) = Self::get_os_arch();
        let tar_path = target_dir.join("opencode-desktop.tar.gz");
        let url = format!("https://github.com/opencode-ai/opencode/releases/download/v{}/opencode-{}-{}.tar.gz", version, os, arch);

        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建 runtime 失败: {e}"))?;
        rt.block_on(async { crate::services::downloader::download_file(&url, &tar_path, None).await })?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-desktop".into(), tool_name: "OpenCode 桌面版".into(),
            phase: "extracting".into(), percent: 50, message: "正在解压...".into(),
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
            tool_id: "opencode-desktop".into(), tool_name: "OpenCode 桌面版".into(),
            phase: "complete".into(), percent: 100, message: "OpenCode 桌面版安装完成".into(),
        });
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn fetch_latest_version() -> Result<String, String> {
        let output = std::process::Command::new("curl")
            .args(["-s", "https://api.github.com/repos/opencode-ai/opencode/releases/latest"])
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
            ("mac", if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" })
        }
        #[cfg(target_os = "linux")]
        {
            ("linux", if cfg!(target_arch = "aarch64") { "arm64" } else { "x86_64" })
        }
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        if target_dir.exists() { std::fs::remove_dir_all(target_dir).map_err(|e| format!("删除失败: {e}"))?; }
        Ok(())
    }
}
