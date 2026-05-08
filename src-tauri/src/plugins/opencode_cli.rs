use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{
    find_command_on_path, find_managed_paths, npm_prefix_candidates, read_command_version,
};
use crate::tool_types::{
    DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta,
};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct OpenCodeCliPlugin;

impl ToolPlugin for OpenCodeCliPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "opencode-cli".into(),
            name: "OpenCode (CLI)".into(),
            description: "OpenCode 官方 CLI".into(),
            icon: "opencode".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::ManagedPrefix
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        if let Some(root) = install_root {
            let candidates = npm_prefix_candidates("opencode");
            let candidate_refs = candidates.iter().map(String::as_str).collect::<Vec<_>>();
            let detect_paths = find_managed_paths(root, "opencode-cli", &candidate_refs);
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

        if let Ok(output) = Command::new("opencode").arg("--version").output() {
            if output.status.success() {
                return DetectResult {
                    installed: true,
                    version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
                    install_path: find_command_on_path("opencode")
                        .and_then(|path| path.parent().map(|dir| dir.to_string_lossy().to_string())),
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
        vec![ToolDependency {
            tool_id: "nodejs".into(),
            min_version: Some(">= 18.0.0".into()),
        }]
    }

    #[cfg(target_os = "windows")]
    fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-cli".into(),
            tool_name: "OpenCode CLI".into(),
            phase: "installing".into(),
            percent: 0,
            message: "正在通过官方 npm 包安装 OpenCode CLI...".into(),
        });

        let output = Command::new("npm")
            .args([
                "install",
                "-g",
                "opencode-ai",
                "--prefix",
                &target_dir.to_string_lossy(),
            ])
            .output()
            .map_err(|e| format!("npm install 失败: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "npm install 失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(&self, target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-cli".into(),
            tool_name: "OpenCode CLI".into(),
            phase: "downloading".into(),
            percent: 0,
            message: "正在下载 OpenCode...".into(),
        });

        let bin_dir = target_dir.join("bin");
        std::fs::create_dir_all(&bin_dir).map_err(|e| format!("创建 bin 目录失败: {e}"))?;

        let version = Self::fetch_latest_version()?;
        let (os, arch) = Self::get_os_arch();
        let tar_path = target_dir.join("opencode.tar.gz");
        let url = format!(
            "https://github.com/opencode-ai/opencode/releases/download/v{}/opencode-{}-{}.tar.gz",
            version, os, arch
        );

        let rt = tokio::runtime::Runtime::new().map_err(|e| format!("创建 runtime 失败: {e}"))?;
        rt.block_on(async { crate::services::downloader::download_file(&url, &tar_path, None).await })?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-cli".into(),
            tool_name: "OpenCode CLI".into(),
            phase: "extracting".into(),
            percent: 50,
            message: "正在解压...".into(),
        });

        crate::services::downloader::extract_tar_gz(&tar_path, target_dir)?;
        std::fs::remove_file(&tar_path).ok();

        let extracted_bin = target_dir.join("opencode");
        let final_bin = bin_dir.join("opencode");
        if !extracted_bin.exists() {
            return Err("OpenCode 下载异常：未找到解压后的二进制文件".to_string());
        }
        std::fs::rename(&extracted_bin, &final_bin)
            .or_else(|_| std::fs::copy(&extracted_bin, &final_bin).map(|_| ()))
            .map_err(|e| format!("移动 OpenCode 二进制文件失败: {e}"))?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-cli".into(),
            tool_name: "OpenCode CLI".into(),
            phase: "complete".into(),
            percent: 100,
            message: "OpenCode 安装完成".into(),
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
        #[cfg(target_os = "windows")]
        {
            let output = Command::new("npm")
                .args([
                    "uninstall",
                    "-g",
                    "opencode-ai",
                    "--prefix",
                    &target_dir.to_string_lossy(),
                ])
                .output()
                .map_err(|e| format!("npm uninstall 失败: {e}"))?;
            if !output.status.success() {
                return Err(format!(
                    "npm uninstall 失败: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }

        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir)
                .map_err(|e| format!("删除失败: {e}"))?;
        }
        Ok(())
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
