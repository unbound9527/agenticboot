use crate::plugin::{NpmRegistrySource, ToolInstallContext, ToolPlugin};
use crate::plugins::npm_cli::{detect_npm_cli, uninstall_npm_cli};
use crate::services::installer::logging::InstallLogEmitter;
use crate::services::installer::windows::{
    run_npm_command_checked_with_env, run_npm_command_checked_with_env_and_logs,
};
use crate::tool_types::{
    DetectResult, InstallLogLevel, InstallProgress, InstallStrategy, ToolDependency, ToolMeta,
};
use std::path::Path;
use tokio::sync::mpsc::Sender;

pub struct OpenClawPlugin;

const OPENCLAW_PACKAGE: &str = "openclaw@latest";
const OPENCLAW_EXTRA_ENV: [(&str, &str); 1] = [("SHARP_IGNORE_GLOBAL_LIBVIPS", "1")];

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

fn openclaw_install_args(target_dir: &Path, registry_source: NpmRegistrySource) -> Vec<String> {
    let mut args = vec![
        "install".to_string(),
        "-g".to_string(),
        OPENCLAW_PACKAGE.to_string(),
        "--prefix".to_string(),
        target_dir.to_string_lossy().to_string(),
    ];
    args.extend(
        registry_source
            .install_args()
            .iter()
            .map(|arg| (*arg).to_string()),
    );
    args
}

fn clear_openclaw_target_dir(target_dir: &Path) -> Result<(), String> {
    if target_dir.exists() {
        std::fs::remove_dir_all(target_dir)
            .map_err(|e| format!("failed to clear managed OpenClaw directory: {e}"))?;
    }
    Ok(())
}

fn install_openclaw_with_retry<F>(
    target_dir: &Path,
    registry_source: NpmRegistrySource,
    progress: &Sender<InstallProgress>,
    install_log: Option<&InstallLogEmitter>,
    mut run_install: F,
) -> Result<(), String>
where
    F: FnMut(&[String], Option<&InstallLogEmitter>) -> Result<(), String>,
{
    let args = openclaw_install_args(target_dir, registry_source);

    emit_openclaw_progress(
        progress,
        "downloading",
        25,
        "Installing OpenClaw from npm...",
    );
    if let Some(install_log) = install_log {
        install_log.emit_phase("downloading", "Installing OpenClaw with managed npm prefix");
        install_log.emit_output(
            "downloading",
            InstallLogLevel::Info,
            format!(
                "Cleaning managed install directory: {}",
                target_dir.display()
            ),
        );
    }
    clear_openclaw_target_dir(target_dir)?;

    match run_install(&args, install_log) {
        Ok(()) => {}
        Err(first_error) => {
            emit_openclaw_progress(
                progress,
                "installing",
                55,
                "Retrying OpenClaw after clearing stale install files...",
            );
            if let Some(install_log) = install_log {
                install_log.emit_output(
                    "installing",
                    InstallLogLevel::Info,
                    format!("First npm install failed: {first_error}"),
                );
                install_log.emit_output(
                    "installing",
                    InstallLogLevel::Info,
                    format!(
                        "Retrying OpenClaw after clearing managed install directory: {}",
                        target_dir.display()
                    ),
                );
            }
            clear_openclaw_target_dir(target_dir)?;
            run_install(&args, install_log)
                .map_err(|retry_error| format!("{retry_error} (first attempt: {first_error})"))?;
        }
    }

    emit_openclaw_progress(
        progress,
        "installing",
        90,
        "Finalizing OpenClaw installation...",
    );
    Ok(())
}

fn install_openclaw_managed(
    target_dir: &Path,
    install_root: &Path,
    progress: &Sender<InstallProgress>,
    registry_source: NpmRegistrySource,
    install_log: Option<&InstallLogEmitter>,
) -> Result<(), String> {
    install_openclaw_with_retry(
        target_dir,
        registry_source,
        progress,
        install_log,
        |args, install_log| {
            let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
            match install_log {
                Some(install_log) => run_npm_command_checked_with_env_and_logs(
                    install_root,
                    install_log,
                    "downloading",
                    &arg_refs,
                    "npm install failed",
                    &OPENCLAW_EXTRA_ENV,
                ),
                None => run_npm_command_checked_with_env(
                    install_root,
                    &arg_refs,
                    "npm install failed",
                    &OPENCLAW_EXTRA_ENV,
                ),
            }
        },
    )
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
        InstallStrategy::ManagedPrefix
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        detect_npm_cli(install_root, "openclaw", "openclaw", "OpenClaw")
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![ToolDependency {
            tool_id: "nodejs".into(),
            min_version: Some(">= 22.16.0".into()),
        }]
    }

    #[cfg(target_os = "windows")]
    fn install(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        install_openclaw_managed(
            target_dir,
            install_root,
            &progress,
            NpmRegistrySource::Official,
            None,
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
        install_openclaw_managed(
            target_dir,
            install_root,
            &progress,
            context.npm_registry_source(),
            Some(context.install_log()),
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
        uninstall_npm_cli(target_dir, "openclaw")?;

        if target_dir.exists() {
            std::fs::remove_dir_all(target_dir)
                .map_err(|e| format!("删除 OpenClaw 安装目录失败: {e}"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{install_openclaw_with_retry, openclaw_install_args, OpenClawPlugin};
    use crate::plugin::{NpmRegistrySource, ToolPlugin};
    use crate::tool_types::{InstallProgress, InstallStrategy};
    use std::sync::{Arc, Mutex};
    use tokio::sync::mpsc;

    #[test]
    fn native_windows_openclaw_uses_managed_prefix_strategy() {
        assert_eq!(
            OpenClawPlugin.install_strategy(),
            InstallStrategy::ManagedPrefix
        );
    }

    #[test]
    fn openclaw_install_args_add_mirror_registry_when_requested() {
        let tmp = tempfile::tempdir().unwrap();
        let target_dir = tmp.path().join("openclaw");
        let args = openclaw_install_args(&target_dir, NpmRegistrySource::Mirror);

        assert_eq!(args[0], "install");
        assert_eq!(args[1], "-g");
        assert_eq!(args[2], "openclaw@latest");
        assert!(args.contains(&"--registry".to_string()));
        assert!(args.contains(&"https://registry.npmmirror.com".to_string()));
    }

    #[test]
    fn openclaw_install_retries_once_after_clearing_stale_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let target_dir = tmp.path().join("openclaw");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(target_dir.join("stale.txt"), "stale").unwrap();

        let attempts = Arc::new(Mutex::new(Vec::new()));
        let attempt_log = Arc::clone(&attempts);
        let (tx, mut rx) = mpsc::channel::<InstallProgress>(8);

        let result = install_openclaw_with_retry(
            &target_dir,
            NpmRegistrySource::Official,
            &tx,
            None,
            |args, _install_log| {
                let mut attempts = attempt_log.lock().unwrap();
                attempts.push(args.to_vec());
                if attempts.len() == 1 {
                    assert!(
                        !target_dir.exists(),
                        "target dir should be cleared before install"
                    );
                    return Err("npm install failed".into());
                }
                Ok(())
            },
        );

        assert!(result.is_ok());
        assert_eq!(attempts.lock().unwrap().len(), 2);
        assert!(!target_dir.exists());

        let first = rx.try_recv().unwrap();
        let second = rx.try_recv().unwrap();
        let third = rx.try_recv().unwrap();
        assert_eq!(first.phase, "downloading");
        assert_eq!(second.phase, "installing");
        assert_eq!(third.phase, "installing");
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
    fn native_windows_openclaw_declares_node_dependency_only() {
        let deps = OpenClawPlugin.dependencies();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].tool_id, "nodejs");
        assert_eq!(deps[0].min_version.as_deref(), Some(">= 22.16.0"));
    }
}
