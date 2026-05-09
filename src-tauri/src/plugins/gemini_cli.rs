use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{
    detect_windows_cli_version, find_managed_paths, npm_prefix_candidates, read_command_version,
    run_npm_command_checked,
};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use std::path::Path;
use tokio::sync::mpsc::Sender;

pub struct GeminiCliPlugin;

impl ToolPlugin for GeminiCliPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "gemini-cli".into(),
            name: "Gemini CLI".into(),
            description: "Google 官方 Gemini CLI AI 编程助手".into(),
            icon: "gemini".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::ManagedPrefix
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        log::info!(
            "[Gemini CLI] Starting detect, install_root={:?}",
            install_root.map(|p| p.to_string_lossy().to_string())
        );

        if let Some(root) = install_root {
            let candidates = npm_prefix_candidates("gemini");
            let candidate_refs = candidates.iter().map(String::as_str).collect::<Vec<_>>();
            let detect_paths = find_managed_paths(root, "gemini-cli", &candidate_refs);
            if let Some(executable) = detect_paths.executable.as_ref() {
                return DetectResult {
                    installed: true,
                    version: read_command_version(executable, &["--version"]),
                    install_path: detect_paths
                        .install_root
                        .as_ref()
                        .map(|path| path.to_string_lossy().to_string()),
                };
            }
        }

        if let Some(version) = detect_windows_cli_version("gemini") {
            log::info!(
                "[Gemini CLI] Detected via Windows shell fallback, version={}",
                version
            );
            return DetectResult {
                installed: true,
                version: Some(version),
                install_path: None,
            };
        }

        DetectResult::not_installed()
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
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        run_npm_command_checked(
            install_root,
            &[
                "install",
                "-g",
                "@google/gemini-cli",
                "--prefix",
                &target_dir.to_string_lossy(),
            ],
            "npm install failed",
        )
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("Gemini CLI 自动安装目前仅支持 Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        run_npm_command_checked(
            target_dir.parent().unwrap_or(target_dir),
            &[
                "uninstall",
                "-g",
                "@google/gemini-cli",
                "--prefix",
                &target_dir.to_string_lossy(),
            ],
            "npm uninstall failed",
        )
    }
}
