use crate::plugin::{NpmRegistrySource, ToolInstallContext, ToolPlugin};
use crate::plugins::npm_cli::detect_npm_cli;
use crate::services::installer::windows::npm_prefix_candidates;
use crate::tool_types::{
    DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta, ToolUpdateSource,
};
use std::path::Path;
use tokio::sync::mpsc::Sender;

pub struct OpenCodeCliPlugin;

const OPENCODE_PACKAGE_NAME: &str = "opencode-ai";
#[cfg(target_os = "windows")]
const OPENCODE_REGISTRY_BASE: &str = "https://registry.npmjs.org";
#[cfg(target_os = "windows")]
const OPENCODE_REGISTRY_MIRROR_BASE: &str = "https://registry.npmmirror.com";

impl ToolPlugin for OpenCodeCliPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "opencode-cli".into(),
            name: "OpenCode (CLI)".into(),
            description: "OpenCode official command line tool".into(),
            icon: "opencode".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::ManagedPrefix
    }

    fn command_name(&self) -> Option<&'static str> {
        Some("opencode")
    }

    fn managed_executable_candidates(&self) -> Vec<String> {
        npm_prefix_candidates("opencode")
    }

    fn update_source(&self) -> Option<ToolUpdateSource> {
        Some(ToolUpdateSource {
            kind: "npm".into(),
            id: OPENCODE_PACKAGE_NAME.into(),
        })
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        detect_npm_cli(install_root, "opencode-cli", "opencode", "OpenCode CLI")
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
        install_windows_opencode_cli(target_dir, progress, NpmRegistrySource::Official)
    }

    #[cfg(target_os = "windows")]
    fn install_with_context(
        &self,
        target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
        context: ToolInstallContext,
    ) -> Result<(), String> {
        install_windows_opencode_cli(target_dir, progress, context.npm_registry_source())
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
            message: "Downloading OpenCode...".into(),
        });

        let bin_dir = target_dir.join("bin");
        std::fs::create_dir_all(&bin_dir).map_err(|e| format!("failed to create bin dir: {e}"))?;

        let version = Self::fetch_latest_version()?;
        let (os, arch) = Self::get_os_arch();
        let tar_path = target_dir.join("opencode.tar.gz");
        let url = format!(
            "https://github.com/opencode-ai/opencode/releases/download/v{}/opencode-{}-{}.tar.gz",
            version, os, arch
        );

        let rt =
            tokio::runtime::Runtime::new().map_err(|e| format!("failed to create runtime: {e}"))?;
        rt.block_on(async {
            crate::services::downloader::download_file(&url, &tar_path, None).await
        })?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "opencode-cli".into(),
            tool_name: "OpenCode CLI".into(),
            phase: "extracting".into(),
            percent: 50,
            message: "Extracting OpenCode...".into(),
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
        let target_dir_arg = target_dir.to_string_lossy();
        let output = std::process::Command::new("npm")
            .args([
                "uninstall",
                "-g",
                OPENCODE_PACKAGE_NAME,
                "--prefix",
                &target_dir_arg,
            ])
            .output()
            .map_err(|e| format!("npm uninstall failed: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            log::warn!("[OpenCode CLI] npm uninstall warning: {}", stderr);
        }
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir).map_err(|e| format!("delete failed: {e}"))?;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir).map_err(|e| format!("delete failed: {e}"))?;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn update_with_context(
        &self,
        target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
        context: ToolInstallContext,
    ) -> Result<(), String> {
        install_windows_opencode_cli(target_dir, progress, context.npm_registry_source())
    }
}

#[cfg(target_os = "windows")]
fn install_windows_opencode_cli(
    target_dir: &Path,
    progress: Sender<InstallProgress>,
    registry_source: NpmRegistrySource,
) -> Result<(), String> {
    let _ = progress.blocking_send(InstallProgress {
        tool_id: "opencode-cli".into(),
        tool_name: "OpenCode CLI".into(),
        phase: "downloading".into(),
        percent: 0,
        message: "Downloading OpenCode Windows package...".into(),
    });

    if target_dir.exists() {
        std::fs::remove_dir_all(target_dir)
            .map_err(|e| format!("failed to clear OpenCode target directory: {e}"))?;
    }

    let bin_dir = target_dir.join("bin");
    let extract_dir = target_dir.join("package-extract");
    let metadata_path = target_dir.join("opencode-package.json");
    let archive_path = target_dir.join("opencode-package.tgz");
    std::fs::create_dir_all(&bin_dir).map_err(|e| format!("failed to create bin dir: {e}"))?;

    let registry_base = match registry_source {
        NpmRegistrySource::Official => OPENCODE_REGISTRY_BASE,
        NpmRegistrySource::Mirror => OPENCODE_REGISTRY_MIRROR_BASE,
    };
    let metadata_url = format!("{registry_base}/{OPENCODE_PACKAGE_NAME}/latest");

    let rt =
        tokio::runtime::Runtime::new().map_err(|e| format!("failed to create runtime: {e}"))?;
    rt.block_on(async {
        crate::services::downloader::download_file(&metadata_url, &metadata_path, None).await
    })?;

    let metadata = std::fs::read_to_string(&metadata_path)
        .map_err(|e| format!("failed to read OpenCode package metadata: {e}"))?;
    let metadata_json: serde_json::Value = serde_json::from_str(&metadata)
        .map_err(|e| format!("failed to parse OpenCode package metadata: {e}"))?;
    let tarball_url = metadata_json
        .get("dist")
        .and_then(|dist| dist.get("tarball"))
        .and_then(|value| value.as_str())
        .ok_or_else(|| "OpenCode package metadata did not include dist.tarball".to_string())?;

    rt.block_on(async {
        crate::services::downloader::download_file(tarball_url, &archive_path, None).await
    })?;

    let _ = progress.blocking_send(InstallProgress {
        tool_id: "opencode-cli".into(),
        tool_name: "OpenCode CLI".into(),
        phase: "extracting".into(),
        percent: 55,
        message: "Extracting OpenCode Windows package...".into(),
    });

    std::fs::create_dir_all(&extract_dir)
        .map_err(|e| format!("failed to create extract dir: {e}"))?;
    crate::services::downloader::extract_tar_gz(&archive_path, &extract_dir)?;

    let extracted_exe = extract_dir.join("package").join("bin").join("opencode.exe");
    if !extracted_exe.exists() {
        return Err("OpenCode Windows package did not contain package/bin/opencode.exe".into());
    }

    std::fs::copy(&extracted_exe, bin_dir.join("opencode.exe"))
        .map_err(|e| format!("failed to install OpenCode executable: {e}"))?;

    std::fs::remove_file(&metadata_path).ok();
    std::fs::remove_file(&archive_path).ok();
    std::fs::remove_dir_all(&extract_dir).ok();

    let _ = progress.blocking_send(InstallProgress {
        tool_id: "opencode-cli".into(),
        tool_name: "OpenCode CLI".into(),
        phase: "complete".into(),
        percent: 100,
        message: "OpenCode install complete".into(),
    });
    Ok(())
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
            .map_err(|e| format!("failed to fetch version: {e}"))?;
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
        Err("failed to parse latest OpenCode version".to_string())
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
    use crate::tool_types::InstallStrategy;

    #[test]
    fn native_windows_opencode_cli_uses_managed_prefix_strategy() {
        assert_eq!(
            OpenCodeCliPlugin.install_strategy(),
            InstallStrategy::ManagedPrefix
        );
    }

    #[test]
    fn native_windows_opencode_cli_has_no_node_dependency_on_windows() {
        assert!(
            OpenCodeCliPlugin.dependencies().is_empty(),
            "native Windows OpenCode CLI installs should not require a separate Node.js step"
        );
    }

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
