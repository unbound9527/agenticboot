use crate::plugin::{ToolInstallContext, ToolPlugin};
use crate::plugins::npm_cli::{detect_npm_cli, uninstall_npm_cli};
use crate::services::installer::logging::InstallLogEmitter;
use crate::services::installer::windows::{
    npm_prefix_candidates, run_command_checked_with_streaming_logs_for_command,
};
use crate::tool_types::{
    DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta, ToolUpdateSource,
};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct OpenClawPlugin;

const OPENCLAW_INSTALLER_URL: &str = "https://openclaw.ai/install.ps1";
const OPENCLAW_INSTALLER_ENV: [(&str, &str); 1] = [("SHARP_IGNORE_GLOBAL_LIBVIPS", "1")];

fn emit_openclaw_progress(
    progress: &Sender<InstallProgress>,
    phase: &str,
    percent: u8,
    message: &str,
) {
    let _ = progress.blocking_send(InstallProgress {
        tool_id: "openclaw".into(),
        tool_name: "OpenClaw".into(),
        phase: phase.into(),
        percent,
        message: message.into(),
    });
}

fn openclaw_installer_command() -> String {
    format!("& ([scriptblock]::Create((Invoke-RestMethod {OPENCLAW_INSTALLER_URL}))) -NoOnboard")
}

fn run_openclaw_installer(
    progress: &Sender<InstallProgress>,
    install_log: Option<&InstallLogEmitter>,
) -> Result<(), String> {
    emit_openclaw_progress(
        progress,
        "installing",
        20,
        "Running the official OpenClaw Windows installer...",
    );
    if let Some(install_log) = install_log {
        install_log.emit_phase(
            "installing",
            "Running official OpenClaw PowerShell installer",
        );
        install_log.emit_command("installing", openclaw_installer_command());
    }

    emit_openclaw_progress(
        progress,
        "installing",
        85,
        "Waiting for the official OpenClaw installer to finish...",
    );

    let mut command = Command::new("powershell");
    command.args([
        "-NoProfile",
        "-ExecutionPolicy",
        "Bypass",
        "-Command",
        &openclaw_installer_command(),
    ]);
    for (key, value) in OPENCLAW_INSTALLER_ENV {
        command.env(key, value);
    }

    if let Some(install_log) = install_log {
        return run_command_checked_with_streaming_logs_for_command(
            install_log,
            "installing",
            &mut command,
            "OpenClaw installer failed",
        );
    }

    let status = command
        .status()
        .map_err(|e| format!("failed to launch OpenClaw installer: {e}"))?;
    if status.success() {
        return Ok(());
    }

    Err(format!(
        "OpenClaw installer failed: exit code: {:?}",
        status.code()
    ))
}

impl ToolPlugin for OpenClawPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "openclaw".into(),
            name: "OpenClaw".into(),
            description: "可编程 AI 代码生成引擎".into(),
            icon: "openclaw".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::OfficialScript
    }

    fn command_name(&self) -> Option<&'static str> {
        Some("openclaw")
    }

    fn managed_executable_candidates(&self) -> Vec<String> {
        npm_prefix_candidates("openclaw")
    }

    fn update_source(&self) -> Option<ToolUpdateSource> {
        Some(ToolUpdateSource {
            kind: "npm".into(),
            id: "openclaw".into(),
        })
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        detect_npm_cli(install_root, "openclaw", "openclaw", "OpenClaw")
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![]
    }

    #[cfg(target_os = "windows")]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        run_openclaw_installer(&progress, None)
    }

    #[cfg(target_os = "windows")]
    fn install_with_context(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
        context: ToolInstallContext,
    ) -> Result<(), String> {
        let _ = context.npm_registry_source();
        run_openclaw_installer(&progress, Some(context.install_log()))
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
        uninstall_npm_cli(target_dir, "openclaw")?;

        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir)
                .map_err(|e| format!("删除 OpenClaw 安装目录失败: {e}"))?;
        }
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn update_with_context(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
        context: ToolInstallContext,
    ) -> Result<(), String> {
        let _ = context.npm_registry_source();
        run_openclaw_installer(&progress, Some(context.install_log()))
    }
}

#[cfg(test)]
mod tests {
    use super::{openclaw_installer_command, OpenClawPlugin};
    use crate::plugin::ToolPlugin;
    use crate::tool_types::InstallStrategy;

    #[test]
    fn native_windows_openclaw_uses_official_script_strategy() {
        assert_eq!(
            OpenClawPlugin.install_strategy(),
            InstallStrategy::OfficialScript
        );
    }

    #[test]
    fn openclaw_installer_command_uses_no_onboard_flag() {
        let command = openclaw_installer_command();
        assert!(command.contains("Invoke-RestMethod https://openclaw.ai/install.ps1"));
        assert!(command.contains("-NoOnboard"));
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
    fn native_windows_openclaw_uses_official_installer_without_node_dependency() {
        let deps = OpenClawPlugin.dependencies();
        assert!(
            deps.is_empty(),
            "official OpenClaw installer should manage Node.js itself on Windows"
        );
    }
}
