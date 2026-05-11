use crate::plugin::NpmRegistrySource;
use crate::services::installer::windows::{
    detect_windows_cli_version, find_command_install_dir, find_managed_paths,
    find_windows_cli_install_dir_with_shell, npm_prefix_candidates, read_command_version,
    run_npm_command_checked, run_npm_command_checked_for_uninstall, SystemWindowsShell,
};
use crate::tool_types::{DetectResult, InstallProgress};
use std::path::Path;
use tokio::sync::mpsc::Sender;

fn emit_install_progress(
    progress: &Sender<InstallProgress>,
    tool_id: &str,
    tool_name: &str,
    phase: &str,
    percent: u8,
    message: &str,
) {
    let _ = progress.blocking_send(InstallProgress {
        tool_id: tool_id.to_string(),
        tool_name: tool_name.to_string(),
        phase: phase.to_string(),
        percent,
        message: message.to_string(),
    });
}

fn install_npm_cli_with_extra_args_and_runner<F>(
    target_dir: &Path,
    install_root: &Path,
    tool_id: &str,
    tool_name: &str,
    progress: Sender<InstallProgress>,
    package_name: &str,
    extra_args: &[&str],
    run_command: F,
) -> Result<(), String>
where
    F: FnOnce(&Path, &[&str], &str) -> Result<(), String>,
{
    log::info!(
        "[npm_cli] install_npm_cli_with_extra_args: package={}, target_dir={}, install_root={}",
        package_name,
        target_dir.display(),
        install_root.display()
    );

    emit_install_progress(
        &progress,
        tool_id,
        tool_name,
        "downloading",
        25,
        "Downloading npm package...",
    );

    let mut args = vec!["install", "-g", package_name];
    args.extend_from_slice(extra_args);
    log::info!("[npm_cli] npm 命令参数: {:?}", args);

    run_command(install_root, &args, "npm install failed")?;

    emit_install_progress(
        &progress,
        tool_id,
        tool_name,
        "installing",
        90,
        "Finalizing npm package...",
    );

    Ok(())
}

pub(crate) fn install_npm_cli_with_extra_args(
    target_dir: &Path,
    install_root: &Path,
    tool_id: &str,
    tool_name: &str,
    progress: Sender<InstallProgress>,
    package_name: &str,
    extra_args: &[&str],
) -> Result<(), String> {
    install_npm_cli_with_extra_args_and_runner(
        target_dir,
        install_root,
        tool_id,
        tool_name,
        progress,
        package_name,
        extra_args,
        run_npm_command_checked,
    )
}

pub(crate) fn install_npm_cli_with_registry(
    target_dir: &Path,
    install_root: &Path,
    tool_id: &str,
    tool_name: &str,
    progress: Sender<InstallProgress>,
    package_name: &str,
    registry_source: NpmRegistrySource,
) -> Result<(), String> {
    install_npm_cli_with_extra_args_and_runner(
        target_dir,
        install_root,
        tool_id,
        tool_name,
        progress,
        package_name,
        registry_source.install_args(),
        run_npm_command_checked,
    )
}

pub(crate) fn install_npm_cli_with_extra_args_and_registry(
    target_dir: &Path,
    install_root: &Path,
    tool_id: &str,
    tool_name: &str,
    progress: Sender<InstallProgress>,
    package_name: &str,
    extra_args: &[&str],
    registry_source: NpmRegistrySource,
) -> Result<(), String> {
    install_npm_cli_with_extra_args_and_registry_and_runner(
        target_dir,
        install_root,
        tool_id,
        tool_name,
        progress,
        package_name,
        extra_args,
        registry_source,
        run_npm_command_checked,
    )
}

fn install_npm_cli_with_extra_args_and_registry_and_runner<F>(
    target_dir: &Path,
    install_root: &Path,
    tool_id: &str,
    tool_name: &str,
    progress: Sender<InstallProgress>,
    package_name: &str,
    extra_args: &[&str],
    registry_source: NpmRegistrySource,
    run_command: F,
) -> Result<(), String>
where
    F: FnOnce(&Path, &[&str], &str) -> Result<(), String>,
{
    let mut combined_args = extra_args.to_vec();
    combined_args.extend_from_slice(registry_source.install_args());
    install_npm_cli_with_extra_args_and_runner(
        target_dir,
        install_root,
        tool_id,
        tool_name,
        progress,
        package_name,
        &combined_args,
        run_command,
    )
}

fn install_npm_cli_with_registry_and_runner<F>(
    target_dir: &Path,
    install_root: &Path,
    tool_id: &str,
    tool_name: &str,
    progress: Sender<InstallProgress>,
    package_name: &str,
    registry_source: NpmRegistrySource,
    run_command: F,
) -> Result<(), String>
where
    F: FnOnce(&Path, &[&str], &str) -> Result<(), String>,
{
    install_npm_cli_with_extra_args_and_runner(
        target_dir,
        install_root,
        tool_id,
        tool_name,
        progress,
        package_name,
        registry_source.install_args(),
        run_command,
    )
}

pub(crate) fn detect_npm_cli(
    install_root: Option<&Path>,
    tool_dir: &str,
    command_name: &str,
    log_prefix: &str,
) -> DetectResult {
    log::info!(
        "[{log_prefix}] Starting detect, install_root={:?}",
        install_root.map(|path| path.to_string_lossy().to_string())
    );

    if let Some(version) = detect_windows_cli_version(command_name) {
        log::info!("[{log_prefix}] Detected via Windows shell fallback, version={version}");
        return DetectResult {
            installed: true,
            version: Some(version),
            install_path: find_command_install_dir(command_name)
                .map(|path| path.to_string_lossy().to_string()),
        };
    }

    #[cfg(target_os = "windows")]
    {
        let mut shell = SystemWindowsShell;
        if let Some(install_path) =
            find_windows_cli_install_dir_with_shell(&mut shell, command_name)
        {
            log::info!(
                "[{log_prefix}] Detected Windows CLI path without version, path={}",
                install_path.display()
            );
            return DetectResult {
                installed: true,
                version: None,
                install_path: Some(install_path.to_string_lossy().to_string()),
            };
        }
    }

    if let Some(root) = install_root {
        let candidates = npm_prefix_candidates(command_name);
        let candidate_refs = candidates.iter().map(String::as_str).collect::<Vec<_>>();
        let detect_paths = find_managed_paths(root, tool_dir, &candidate_refs);
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

    DetectResult::not_installed()
}

pub(crate) fn install_npm_cli(
    target_dir: &Path,
    install_root: &Path,
    tool_id: &str,
    tool_name: &str,
    progress: Sender<InstallProgress>,
    package_name: &str,
) -> Result<(), String> {
    log::info!("[npm_cli] install_npm_cli 调用: package={}", package_name);
    install_npm_cli_with_extra_args(
        target_dir,
        install_root,
        tool_id,
        tool_name,
        progress,
        package_name,
        &[],
    )
}

pub(crate) fn uninstall_npm_cli(target_dir: &Path, package_name: &str) -> Result<(), String> {
    uninstall_npm_cli_with_runner(target_dir, package_name, |install_dir, args, context| {
        run_npm_command_checked_for_uninstall(install_dir, args, context)
    })
}

fn uninstall_npm_cli_with_runner<F>(
    target_dir: &Path,
    package_name: &str,
    run_command: F,
) -> Result<(), String>
where
    F: FnOnce(&Path, &[&str], &str) -> Result<(), String>,
{
    run_command(
        target_dir.parent().unwrap_or(target_dir),
        &["uninstall", "-g", package_name],
        "npm uninstall failed",
    )
}

#[cfg(test)]
mod tests {
    use super::{
        install_npm_cli_with_extra_args_and_registry_and_runner,
        install_npm_cli_with_extra_args_and_runner, install_npm_cli_with_registry_and_runner,
        uninstall_npm_cli_with_runner,
    };
    use crate::plugin::NpmRegistrySource;
    use crate::tool_types::InstallProgress;
    use tokio::sync::mpsc;

    #[test]
    fn npm_cli_install_reports_progress_phases() {
        let temp = tempfile::tempdir().unwrap();
        let target_dir = temp.path().join("gemini-cli");
        let (tx, mut rx) = mpsc::channel::<InstallProgress>(4);

        install_npm_cli_with_extra_args_and_runner(
            &target_dir,
            temp.path(),
            "gemini-cli",
            "Gemini CLI",
            tx,
            "@google/gemini-cli",
            &[],
            |_install_root, args, context| {
                assert_eq!(context, "npm install failed");
                assert!(args.starts_with(&["install", "-g", "@google/gemini-cli"]));
                assert!(!args.contains(&"--prefix"));
                Ok(())
            },
        )
        .unwrap();

        let first = rx.try_recv().unwrap();
        let second = rx.try_recv().unwrap();

        assert_eq!(first.tool_id, "gemini-cli");
        assert_eq!(first.phase, "downloading");
        assert_eq!(first.percent, 25);
        assert_eq!(second.phase, "installing");
        assert_eq!(second.percent, 90);
    }

    #[test]
    fn npm_cli_install_uses_mirror_registry_when_requested() {
        let temp = tempfile::tempdir().unwrap();
        let target_dir = temp.path().join("gemini-cli");
        let (tx, mut rx) = mpsc::channel::<InstallProgress>(4);

        install_npm_cli_with_registry_and_runner(
            &target_dir,
            temp.path(),
            "gemini-cli",
            "Gemini CLI",
            tx,
            "@google/gemini-cli",
            NpmRegistrySource::Mirror,
            |_install_root, args, context| {
                assert_eq!(context, "npm install failed");
                assert!(args.starts_with(&["install", "-g", "@google/gemini-cli"]));
                assert!(!args.contains(&"--prefix"));
                assert!(args.contains(&"--registry"));
                assert!(args.contains(&"https://registry.npmmirror.com"));
                Ok(())
            },
        )
        .unwrap();

        let first = rx.try_recv().unwrap();
        let second = rx.try_recv().unwrap();

        assert_eq!(first.phase, "downloading");
        assert_eq!(second.phase, "installing");
    }

    #[test]
    fn npm_cli_install_combines_extra_args_with_mirror_registry() {
        let temp = tempfile::tempdir().unwrap();
        let target_dir = temp.path().join("openclaw");
        let (tx, mut rx) = mpsc::channel::<InstallProgress>(4);

        install_npm_cli_with_extra_args_and_registry_and_runner(
            &target_dir,
            temp.path(),
            "openclaw",
            "OpenClaw",
            tx,
            "openclaw@latest",
            &["--ignore-scripts"],
            NpmRegistrySource::Mirror,
            |_install_root, args, context| {
                assert_eq!(context, "npm install failed");
                assert!(args.starts_with(&["install", "-g", "openclaw@latest"]));
                assert!(!args.contains(&"--prefix"));
                assert!(args.contains(&"--ignore-scripts"));
                assert!(args.contains(&"--registry"));
                assert!(args.contains(&"https://registry.npmmirror.com"));
                Ok(())
            },
        )
        .unwrap();

        let first = rx.try_recv().unwrap();
        let second = rx.try_recv().unwrap();

        assert_eq!(first.phase, "downloading");
        assert_eq!(second.phase, "installing");
    }

    #[test]
    fn npm_cli_uninstall_uses_global_uninstall_without_prefix() {
        let temp = tempfile::tempdir().unwrap();
        let target_dir = temp.path().join("codex-cli");

        uninstall_npm_cli_with_runner(&target_dir, "@openai/codex", |_install_root, args, context| {
            assert_eq!(context, "npm uninstall failed");
            assert_eq!(args, ["uninstall", "-g", "@openai/codex"]);
            Ok(())
        })
        .unwrap();
    }
}
