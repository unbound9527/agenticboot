use crate::services::installer::logging::InstallLogEmitter;
use crate::tool_types::{
    DetectResult, InstallProgress, InstallStrategy, ToolCapabilities, ToolCatalogItem,
    ToolDependency, ToolMeta, ToolPlatformSupport, ToolUpdateSource,
};
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

    fn command_name(&self) -> Option<&'static str> {
        None
    }

    fn managed_shim_name(&self) -> Option<&'static str> {
        self.command_name()
    }

    fn managed_executable_candidates(&self) -> Vec<String> {
        Vec::new()
    }

    fn update_source(&self) -> Option<ToolUpdateSource> {
        None
    }

    fn can_install(&self) -> bool {
        self.platform_support().current_platform_is_implemented()
    }

    fn can_uninstall(&self) -> bool {
        self.platform_support().current_platform_is_implemented()
    }

    fn supports_pathless_uninstall(&self) -> bool {
        false
    }

    fn can_launch(&self) -> bool {
        self.platform_support().current_platform_is_implemented()
            && matches!(self.install_strategy(), InstallStrategy::DesktopInstaller)
    }

    fn platform_support(&self) -> ToolPlatformSupport {
        ToolPlatformSupport::windows_only()
    }

    fn catalog_item(&self) -> ToolCatalogItem {
        let meta = self.metadata();
        let update_source = self.update_source();
        let install_strategy = self.install_strategy();
        let command_name = self.command_name().map(str::to_string);
        let managed_shim_name = self.managed_shim_name().map(str::to_string);
        let managed_executable_candidates = self.managed_executable_candidates();

        ToolCatalogItem {
            id: meta.id,
            name: meta.name,
            description: meta.description,
            icon: meta.icon,
            category: meta.category,
            install_strategy: install_strategy.as_kebab_case().to_string(),
            dependencies: self.dependencies(),
            update_source: update_source.clone(),
            platform_support: self.platform_support(),
            capabilities: ToolCapabilities {
                can_install: self.can_install(),
                can_uninstall: self.can_uninstall(),
                can_launch: self.can_launch(),
                can_update: self.platform_support().current_platform_is_implemented()
                    && update_source.is_some(),
                supports_pathless_uninstall: self.supports_pathless_uninstall(),
                command_name,
                managed_shim_name,
                managed_executable_candidates,
            },
        }
    }
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

pub fn get_tool_catalog() -> Vec<ToolCatalogItem> {
    get_all_plugins()
        .into_iter()
        .map(|plugin| plugin.catalog_item())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{get_plugin_by_id, get_tool_catalog, ToolPlugin};
    use crate::services::installer::logging::InstallLogEmitter;
    use crate::tool_types::InstallStrategy;
    use crate::tool_types::{
        DetectResult, InstallProgress, ToolDependency, ToolMeta, ToolPlatformSupport,
    };
    use std::path::Path;
    use tokio::sync::mpsc::{self, Sender};

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
    fn tool_catalog_has_unique_plugin_owned_capabilities() {
        let catalog = get_tool_catalog();
        let mut ids = std::collections::HashSet::new();

        for tool in &catalog {
            assert!(ids.insert(tool.id.clone()), "duplicate tool id {}", tool.id);
        }

        let codex = catalog
            .iter()
            .find(|tool| tool.id == "codex-cli")
            .expect("codex cli in catalog");
        assert_eq!(codex.name, "Codex (CLI)");
        assert_eq!(codex.category, "ai-cli");
        assert_eq!(codex.capabilities.command_name.as_deref(), Some("codex"));
        assert_eq!(
            codex.capabilities.managed_shim_name.as_deref(),
            Some("codex")
        );
        assert!(codex.capabilities.can_install);
        assert!(codex.capabilities.can_uninstall);
        assert!(codex.capabilities.can_update);
        assert_eq!(
            codex
                .update_source
                .as_ref()
                .map(|source| source.kind.as_str()),
            Some("npm")
        );
        assert_eq!(
            codex
                .update_source
                .as_ref()
                .map(|source| source.id.as_str()),
            Some("@openai/codex")
        );
    }

    #[test]
    fn desktop_catalog_declares_pathless_uninstall_support() {
        let catalog = get_tool_catalog();
        let codex_desktop = catalog
            .iter()
            .find(|tool| tool.id == "codex-desktop")
            .expect("codex desktop in catalog");

        assert!(codex_desktop.capabilities.supports_pathless_uninstall);
        assert!(codex_desktop.capabilities.can_launch);
        assert_eq!(codex_desktop.install_strategy, "desktop-installer");
    }

    #[test]
    fn catalog_capabilities_respect_platform_support() {
        struct PlannedPlatformPlugin;

        impl ToolPlugin for PlannedPlatformPlugin {
            fn metadata(&self) -> ToolMeta {
                ToolMeta {
                    id: "future-tool".into(),
                    name: "Future Tool".into(),
                    description: "planned elsewhere".into(),
                    icon: "tool".into(),
                    category: "ai-cli".into(),
                }
            }

            fn install_strategy(&self) -> InstallStrategy {
                InstallStrategy::DesktopInstaller
            }

            fn detect(&self, _install_root: Option<&Path>) -> DetectResult {
                DetectResult::not_installed()
            }

            fn install(
                &self,
                _target_dir: &Path,
                _install_root: &Path,
                _progress: Sender<InstallProgress>,
            ) -> Result<(), String> {
                Ok(())
            }

            fn uninstall(&self, _target_dir: &Path) -> Result<(), String> {
                Ok(())
            }

            fn dependencies(&self) -> Vec<ToolDependency> {
                Vec::new()
            }

            fn platform_support(&self) -> ToolPlatformSupport {
                ToolPlatformSupport {
                    windows: "planned".into(),
                    macos: "implemented".into(),
                    linux: "implemented".into(),
                }
            }
        }

        let plugin = PlannedPlatformPlugin;
        let catalog_item = plugin.catalog_item();

        #[cfg(target_os = "windows")]
        {
            assert!(!catalog_item.capabilities.can_install);
            assert!(!catalog_item.capabilities.can_uninstall);
            assert!(!catalog_item.capabilities.can_launch);
            assert!(!catalog_item.capabilities.can_update);
        }

        #[cfg(not(target_os = "windows"))]
        {
            assert!(catalog_item.capabilities.can_install);
            assert!(catalog_item.capabilities.can_uninstall);
            assert!(catalog_item.capabilities.can_launch);
        }
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
