use crate::tool_types::{
    DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta,
};
use std::path::Path;
use tokio::sync::mpsc::Sender;

pub trait ToolPlugin: Send + Sync {
    fn metadata(&self) -> ToolMeta;
    fn install_strategy(&self) -> InstallStrategy;

    fn detect(&self, install_root: Option<&Path>) -> DetectResult;

    fn install(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String>;

    fn uninstall(&self, target_dir: &Path) -> Result<(), String>;

    fn dependencies(&self) -> Vec<ToolDependency>;
}

pub fn get_all_plugins() -> Vec<Box<dyn ToolPlugin>> {
    vec![
        Box::new(crate::plugins::nodejs::NodeJsPlugin),
        Box::new(crate::plugins::git::GitPlugin),
        Box::new(crate::plugins::claude_code_cli::ClaudeCodeCliPlugin),
        Box::new(crate::plugins::codex_cli::CodexCliPlugin),
        Box::new(crate::plugins::gemini_cli::GeminiCliPlugin),
        Box::new(crate::plugins::opencode_cli::OpenCodeCliPlugin),
        Box::new(crate::plugins::openclaw::OpenClawPlugin),
        Box::new(crate::plugins::hermes::HermesPlugin),
        Box::new(crate::plugins::claude_code_desktop::ClaudeCodeDesktopPlugin),
        Box::new(crate::plugins::codex_desktop::CodexDesktopPlugin),
        Box::new(crate::plugins::opencode_desktop::OpenCodeDesktopPlugin),
    ]
}

pub fn get_plugin_by_id(id: &str) -> Option<Box<dyn ToolPlugin>> {
    get_all_plugins().into_iter().find(|p| p.metadata().id == id)
}

#[cfg(test)]
mod tests {
    use super::get_plugin_by_id;
    use crate::tool_types::InstallStrategy;

    #[test]
    fn install_strategy_desktop_plugins_are_not_managed_prefix_tools() {
        let plugin = get_plugin_by_id("claude-code-desktop").unwrap();

        assert_eq!(plugin.install_strategy(), InstallStrategy::DesktopInstaller);
    }

    #[test]
    fn install_strategy_managed_prefix_plugins_use_managed_prefix_strategy() {
        let plugin = get_plugin_by_id("claude-code-cli").unwrap();

        assert_eq!(plugin.install_strategy(), InstallStrategy::ManagedPrefix);
    }

    #[test]
    fn install_strategy_hermes_plugin_is_registered() {
        let plugin = get_plugin_by_id("hermes").unwrap();

        assert_eq!(plugin.metadata().id, "hermes");
    }
}
