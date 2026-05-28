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

pub struct ClaudeCodeCliPlugin;

impl ToolPlugin for ClaudeCodeCliPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "claude-code-cli".into(),
            name: "Claude Code (CLI)".into(),
            description: "Anthropic Claude Code 官方命令行工具".into(),
            icon: "claude".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::GlobalNpm
    }

    fn command_name(&self) -> Option<&'static str> {
        Some("claude")
    }

    fn managed_executable_candidates(&self) -> Vec<String> {
        npm_prefix_candidates("claude")
    }

    fn update_source(&self) -> Option<ToolUpdateSource> {
        Some(ToolUpdateSource {
            kind: "npm".into(),
            id: "@anthropic-ai/claude-code".into(),
        })
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        detect_npm_cli(install_root, "claude-code-cli", "claude", "Claude Code CLI")
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
            "claude-code-cli",
            "Claude Code (CLI)",
            progress,
            "@anthropic-ai/claude-code",
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
            "claude-code-cli",
            "Claude Code (CLI)",
            progress,
            "@anthropic-ai/claude-code",
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
        Err("Claude Code CLI 自动安装目前仅支持 Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        uninstall_npm_cli(target_dir, "@anthropic-ai/claude-code")
    }

    #[cfg(target_os = "windows")]
    fn update_with_context(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
        context: ToolInstallContext,
    ) -> Result<(), String> {
        install_npm_cli_with_registry(
            target_dir,
            install_root,
            "claude-code-cli",
            "Claude Code (CLI)",
            progress,
            "@anthropic-ai/claude-code",
            context.npm_registry_source(),
        )
    }
}
