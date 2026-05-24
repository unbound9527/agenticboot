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
        InstallStrategy::GlobalNpm
    }

    fn command_name(&self) -> Option<&'static str> {
        Some("gemini")
    }

    fn managed_executable_candidates(&self) -> Vec<String> {
        npm_prefix_candidates("gemini")
    }

    fn update_source(&self) -> Option<ToolUpdateSource> {
        Some(ToolUpdateSource {
            kind: "npm".into(),
            id: "@google/gemini-cli".into(),
        })
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        detect_npm_cli(install_root, "gemini-cli", "gemini", "Gemini CLI")
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
            "gemini-cli",
            "Gemini CLI",
            progress,
            "@google/gemini-cli",
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
            "gemini-cli",
            "Gemini CLI",
            progress,
            "@google/gemini-cli",
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
        Err("Gemini CLI 自动安装目前仅支持 Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        uninstall_npm_cli(target_dir, "@google/gemini-cli")
    }
}

#[cfg(test)]
mod tests {
    use super::GeminiCliPlugin;
    use crate::plugin::ToolPlugin;

    #[test]
    fn native_windows_gemini_cli_detects_existing_managed_windows_command() {
        let tmp = tempfile::tempdir().unwrap();
        let tool_dir = tmp.path().join("gemini-cli");
        std::fs::create_dir_all(&tool_dir).unwrap();
        std::fs::write(tool_dir.join("gemini.cmd"), "@echo off\r\necho 0.41.2\r\n").unwrap();

        let detect = GeminiCliPlugin.detect(Some(tmp.path()));
        assert!(detect.installed);
        assert_eq!(detect.version.as_deref(), Some("0.41.2"));
        assert_eq!(
            detect.install_path.as_deref(),
            Some(tool_dir.to_string_lossy().as_ref())
        );
    }
}
