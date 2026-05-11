use crate::plugin::ToolInstallContext;
use crate::plugin::ToolPlugin;
use crate::plugins::npm_cli::{
    detect_npm_cli, install_npm_cli_with_extra_args, install_npm_cli_with_extra_args_and_registry,
    uninstall_npm_cli,
};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use std::path::Path;
use tokio::sync::mpsc::Sender;

pub struct OpenClawPlugin;

#[cfg(target_os = "windows")]
fn openclaw_extra_args() -> [&'static str; 0] {
    []
}

impl ToolPlugin for OpenClawPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "openclaw".into(),
            name: "OpenClaw".into(),
            description: "Programmable AI coding engine".into(),
            icon: "openclaw".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::GlobalNpm
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        detect_npm_cli(install_root, "openclaw", "openclaw", "OpenClaw")
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![
            ToolDependency {
                tool_id: "nodejs".into(),
                min_version: Some(">= 22.14.0".into()),
            },
            ToolDependency {
                tool_id: "git".into(),
                min_version: None,
            },
        ]
    }

    #[cfg(target_os = "windows")]
    fn install(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "openclaw".into(),
            tool_name: "OpenClaw".into(),
            phase: "installing".into(),
            percent: 0,
            message: "Installing OpenClaw into the managed tool directory...".into(),
        });

        install_npm_cli_with_extra_args(
            target_dir,
            install_root,
            "openclaw",
            "OpenClaw",
            progress,
            "openclaw@latest",
            &openclaw_extra_args(),
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
            tool_id: "openclaw".into(),
            tool_name: "OpenClaw".into(),
            phase: "installing".into(),
            percent: 0,
            message: "Installing OpenClaw into the managed tool directory...".into(),
        });

        install_npm_cli_with_extra_args_and_registry(
            target_dir,
            install_root,
            "openclaw",
            "OpenClaw",
            progress,
            "openclaw@latest",
            &openclaw_extra_args(),
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
        Err("OpenClaw auto-install is currently supported only on Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        uninstall_npm_cli(target_dir, "openclaw")
    }
}

#[cfg(test)]
mod tests {
    use super::OpenClawPlugin;
    use crate::plugin::ToolPlugin;
    use crate::tool_types::InstallStrategy;

    #[test]
    fn native_windows_openclaw_uses_global_npm_strategy() {
        assert_eq!(OpenClawPlugin.install_strategy(), InstallStrategy::GlobalNpm);
    }

    #[test]
    fn native_windows_openclaw_detects_existing_managed_windows_command() {
        let tmp = tempfile::tempdir().unwrap();
        let tool_dir = tmp.path().join("openclaw");
        std::fs::create_dir_all(&tool_dir).unwrap();
        std::fs::write(
            tool_dir.join("openclaw.cmd"),
            "@echo off\r\necho openclaw 1.2.3\r\n",
        )
        .unwrap();

        let detect = OpenClawPlugin.detect(Some(tmp.path()));
        assert!(detect.installed);
        assert_eq!(detect.version.as_deref(), Some("openclaw 1.2.3"));
        assert_eq!(
            detect.install_path.as_deref(),
            Some(tool_dir.to_string_lossy().as_ref())
        );
    }

    #[test]
    fn native_windows_openclaw_declares_node_and_git_dependencies() {
        let deps = OpenClawPlugin.dependencies();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].tool_id, "nodejs");
        assert_eq!(deps[0].min_version.as_deref(), Some(">= 22.14.0"));
        assert_eq!(deps[1].tool_id, "git");
        assert_eq!(deps[1].min_version, None);
    }
}
