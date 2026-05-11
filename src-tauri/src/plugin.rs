use crate::services::installer::logging::InstallLogEmitter;
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use std::path::Path;
use tokio::sync::mpsc::Sender;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpmRegistrySource {
    Official,
    Mirror,
}

impl NpmRegistrySource {
    pub fn install_args(self) -> &'static [&'static str] {
        match self {
            NpmRegistrySource::Official => &[],
            NpmRegistrySource::Mirror => &["--registry", "https://registry.npmmirror.com"],
        }
    }
}

#[derive(Clone)]
pub struct ToolInstallContext {
    install_log: InstallLogEmitter,
    npm_registry_source: NpmRegistrySource,
}

impl ToolInstallContext {
    pub fn new(install_log: InstallLogEmitter, npm_registry_source: NpmRegistrySource) -> Self {
        Self {
            install_log,
            npm_registry_source,
        }
    }

    pub fn install_log(&self) -> &InstallLogEmitter {
        &self.install_log
    }

    pub fn npm_registry_source(&self) -> NpmRegistrySource {
        self.npm_registry_source
    }
}

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

    fn install_with_context(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
        _context: ToolInstallContext,
    ) -> Result<(), String> {
        self.install(target_dir, install_root, progress)
    }

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
    get_all_plugins()
        .into_iter()
        .find(|p| p.metadata().id == id)
}

#[cfg(test)]
mod tests {
    use super::{get_plugin_by_id, ToolPlugin};
    use crate::services::installer::logging::InstallLogEmitter;
    use crate::tool_types::InstallStrategy;
    use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
    use std::path::Path;
    use tokio::sync::mpsc;

    #[test]
    fn install_strategy_desktop_plugins_are_not_managed_prefix_tools() {
        let plugin = get_plugin_by_id("claude-code-desktop").unwrap();

        assert_eq!(plugin.install_strategy(), InstallStrategy::DesktopInstaller);
    }

    #[test]
    fn install_strategy_global_npm_plugins_use_global_npm_strategy() {
        let plugin = get_plugin_by_id("claude-code-cli").unwrap();

        assert_eq!(plugin.install_strategy(), InstallStrategy::GlobalNpm);
    }

    #[test]
    fn install_strategy_hermes_plugin_is_registered() {
        let plugin = get_plugin_by_id("hermes").unwrap();

        assert_eq!(plugin.metadata().id, "hermes");
    }

    #[test]
    fn install_with_context_defaults_to_install_behavior() {
        struct FakePlugin;

        impl ToolPlugin for FakePlugin {
            fn metadata(&self) -> ToolMeta {
                ToolMeta {
                    id: "fake".into(),
                    name: "Fake".into(),
                    description: "Fake".into(),
                    icon: "fake".into(),
                    category: "test".into(),
                }
            }

            fn install_strategy(&self) -> InstallStrategy {
                InstallStrategy::ManagedPrefix
            }

            fn detect(&self, _install_root: Option<&Path>) -> DetectResult {
                DetectResult::not_installed()
            }

            fn install(
                &self,
                _target_dir: &Path,
                _install_root: &Path,
                progress: tokio::sync::mpsc::Sender<InstallProgress>,
            ) -> Result<(), String> {
                let _ = progress.blocking_send(InstallProgress {
                    tool_id: "fake".into(),
                    tool_name: "Fake".into(),
                    phase: "installing".into(),
                    percent: 50,
                    message: "default install called".into(),
                });
                Ok(())
            }

            fn uninstall(&self, _target_dir: &Path) -> Result<(), String> {
                Ok(())
            }

            fn dependencies(&self) -> Vec<ToolDependency> {
                vec![]
            }
        }

        let (tx, mut rx) = mpsc::channel::<InstallProgress>(4);
        let install_log = InstallLogEmitter::new_for_test("fake", "Fake", |_| {});
        let result = FakePlugin.install_with_context(
            Path::new("."),
            Path::new("."),
            tx,
            super::ToolInstallContext::new(install_log, super::NpmRegistrySource::Official),
        );

        assert!(result.is_ok());
        assert_eq!(
            rx.try_recv().unwrap().message,
            "default install called".to_string()
        );
    }
}
