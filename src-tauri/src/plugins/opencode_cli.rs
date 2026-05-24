use crate::plugin::ToolInstallContext;
use crate::plugin::ToolPlugin;
use crate::plugins::npm_cli::{
    detect_npm_cli, install_npm_cli, install_npm_cli_with_registry, uninstall_npm_cli,
};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use std::path::Path;
use tokio::sync::mpsc::Sender;

pub struct OpenCodeCliPlugin;

impl ToolPlugin for OpenCodeCliPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "opencode-cli".into(),
            name: "OpenCode (CLI)".into(),
            description: "OpenCode 瀹樻柟 CLI".into(),
            icon: "opencode".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::GlobalNpm
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        detect_npm_cli(install_root, "opencode-cli", "opencode", "OpenCode CLI")
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![ToolDependency {
            tool_id: "nodejs".into(),
            min_version: Some(">= 18.0.0".into()),
        }]
    }

    #[cfg(target_os = "windows")]
    fn install(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-cli".into(),
            tool_name: "OpenCode CLI".into(),
            phase: "installing".into(),
            percent: 0,
            message: "姝ｅ湪閫氳繃瀹樻柟 npm 鍖呭畨瑁?OpenCode CLI...".into(),
        });

        install_npm_cli(
            target_dir,
            install_root,
            "opencode-cli",
            "OpenCode CLI",
            progress,
            "opencode-ai",
        )
    }

    #[cfg(target_os = "windows")]
    fn install_with_context(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
        context: ToolInstallContext,
    ) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-cli".into(),
            tool_name: "OpenCode CLI".into(),
            phase: "installing".into(),
            percent: 0,
            message: "姝ｅ湪閫氳繃瀹樻柟 npm 鍖呭畨瑁?OpenCode CLI...".into(),
        });

        install_npm_cli_with_registry(
            target_dir,
            install_root,
            "opencode-cli",
            "OpenCode CLI",
            progress,
            "opencode-ai",
            context.npm_registry_source(),
        )
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-cli".into(),
            tool_name: "OpenCode CLI".into(),
            phase: "downloading".into(),
            percent: 0,
            message: "姝ｅ湪涓嬭浇 OpenCode...".into(),
        });

        let bin_dir = target_dir.join("bin");
        std::fs::create_dir_all(&bin_dir).map_err(|e| format!("鍒涘缓 bin 鐩綍澶辫触: {e}"))?;

        let version = Self::fetch_latest_version()?;
        let (os, arch) = Self::get_os_arch();
        let tar_path = target_dir.join("opencode.tar.gz");
        let url = format!(
            "https://github.com/opencode-ai/opencode/releases/download/v{}/opencode-{}-{}.tar.gz",
            version, os, arch
        );

        let rt =
            tokio::runtime::Runtime::new().map_err(|e| format!("鍒涘缓 runtime 澶辫触: {e}"))?;
        rt.block_on(async {
            crate::services::downloader::download_file(&url, &tar_path, None).await
        })?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-cli".into(),
            tool_name: "OpenCode CLI".into(),
            phase: "extracting".into(),
            percent: 50,
            message: "姝ｅ湪瑙ｅ帇...".into(),
        });

        crate::services::downloader::extract_tar_gz(&tar_path, target_dir)?;
        std::fs::remove_file(&tar_path).ok();

        let extracted_bin = target_dir.join("opencode");
        let final_bin = bin_dir.join("opencode");
        if !extracted_bin.exists() {
            return Err("OpenCode download failed: extracted binary was not found".to_string());
        }
        std::fs::rename(&extracted_bin, &final_bin)
            .or_else(|_| std::fs::copy(&extracted_bin, &final_bin).map(|_| ()))
            .map_err(|e| format!("failed to move OpenCode binary: {e}"))?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-cli".into(),
            tool_name: "OpenCode CLI".into(),
            phase: "complete".into(),
            percent: 100,
            message: "OpenCode install complete".into(),
        });
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        let package_name = "opencode-ai";
        let target_dir_arg = target_dir.to_string_lossy();
        let output = std::process::Command::new("npm")
            .args(["uninstall", "-g", package_name, "--prefix", &target_dir_arg])
            .output()
            .map_err(|e| format!("npm uninstall failed: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("[OpenCode CLI] npm uninstall warning: {}", stderr);
        }
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir).map_err(|e| format!("删除失败: {e}"))?;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        uninstall_npm_cli(target_dir, "opencode-ai")?;

        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir).map_err(|e| format!("删除失败: {e}"))?;
        }
        Ok(())
    }
}

impl OpenCodeCliPlugin {
    #[cfg(not(target_os = "windows"))]
    fn fetch_latest_version() -> Result<String, String> {
        let output = std::process::Command::new("curl")
            .args([
                "-s",
                "https://api.github.com/repos/opencode-ai/opencode/releases/latest",
            ])
            .output()
            .map_err(|e| format!("鑾峰彇鐗堟湰澶辫触: {e}"))?;
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
        Err("鏃犳硶瑙ｆ瀽 OpenCode 鏈€鏂扮増鏈彿".to_string())
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
}

#[cfg(test)]
mod tests {
    use super::OpenCodeCliPlugin;
    use crate::plugin::ToolPlugin;

    #[test]
    fn native_windows_opencode_cli_detects_existing_managed_windows_command() {
        let tmp = tempfile::tempdir().unwrap();
        let tool_dir = tmp.path().join("opencode-cli");
        std::fs::create_dir_all(&tool_dir).unwrap();
        std::fs::write(
            tool_dir.join("opencode.cmd"),
            "@echo off\r\necho opencode 1.2.3\r\n",
        )
        .unwrap();

        let detect = OpenCodeCliPlugin.detect(Some(tmp.path()));
        assert!(detect.installed);
        assert_eq!(detect.version.as_deref(), Some("opencode 1.2.3"));
        assert_eq!(
            detect.install_path.as_deref(),
            Some(tool_dir.to_string_lossy().as_ref())
        );
    }
}
