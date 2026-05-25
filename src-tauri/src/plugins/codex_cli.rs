use crate::plugin::ToolInstallContext;
use crate::plugin::ToolPlugin;
use crate::plugins::npm_cli::{
    detect_npm_cli, install_npm_cli, install_npm_cli_with_registry, uninstall_npm_cli,
};
use crate::services::installer::windows::npm_prefix_candidates;
use crate::tool_types::{
    DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta, ToolUpdateSource,
};
use std::path::Path;
use tokio::sync::mpsc::Sender;

pub struct CodexCliPlugin;

impl ToolPlugin for CodexCliPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "codex-cli".into(),
            name: "Codex (CLI)".into(),
            description: "OpenAI Codex 官方命令行工具".into(),
            icon: "codex".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::GlobalNpm
    }

    fn command_name(&self) -> Option<&'static str> {
        Some("codex")
    }

    fn managed_executable_candidates(&self) -> Vec<String> {
        npm_prefix_candidates("codex")
    }

    fn update_source(&self) -> Option<ToolUpdateSource> {
        Some(ToolUpdateSource {
            kind: "npm".into(),
            id: "@openai/codex".into(),
        })
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        log::info!(
            "[Codex CLI] detect called, install_root={:?}",
            install_root.map(|p| p.to_string_lossy().to_string())
        );
        let result = detect_npm_cli(install_root, "codex-cli", "codex", "Codex CLI");
        log::info!(
            "[Codex CLI] detect result: installed={}, version={:?}, path={:?}",
            result.installed,
            result.version,
            result.install_path
        );
        result
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
        install_npm_cli(
            target_dir,
            install_root,
            "codex-cli",
            "Codex (CLI)",
            progress,
            "@openai/codex",
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
        install_npm_cli_with_registry(
            target_dir,
            install_root,
            "codex-cli",
            "Codex (CLI)",
            progress,
            "@openai/codex",
            context.npm_registry_source(),
        )
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("Codex CLI 自动安装目前仅支持 Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        log::info!(
            "[Codex CLI] uninstall called with target_dir={}",
            target_dir.display()
        );
        let result = uninstall_npm_cli(target_dir, "@openai/codex");
        match &result {
            Ok(()) => log::info!("[Codex CLI] uninstall completed successfully"),
            Err(e) => log::error!("[Codex CLI] uninstall failed: {}", e),
        }
        result
    }
}
