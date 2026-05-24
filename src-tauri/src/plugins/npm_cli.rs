use crate::plugin::NpmRegistrySource;
use crate::services::installer::windows::{
    detect_windows_cli_version, find_command_install_dir, find_managed_paths,
    find_windows_cli_install_dir_with_shell, npm_prefix_candidates, read_command_version,
    run_npm_command_checked, run_npm_command_checked_for_uninstall, SystemWindowsShell,
};
use crate::tool_types::{DetectResult, InstallProgress};
use std::path::Path;
use tokio::sync::mpsc::Sender;

fn is_managed_bin_dir(install_root: &Path, install_path: &Path) -> bool {
    install_path == install_root.join("bin")
}

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

    let target_dir_arg = target_dir.to_string_lossy().to_string();
    let mut args = vec![
        "install".to_string(),
        "-g".to_string(),
        package_name.to_string(),
        "--prefix".to_string(),
        target_dir_arg,
    ];
    args.extend(extra_args.iter().map(|arg| (*arg).to_string()));
    log::info!("[npm_cli] npm install args: {:?}", args);

    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_command(install_root, &arg_refs, "npm install failed")?;

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

    if let Some(root) = install_root {
        let candidates = npm_prefix_candidates(command_name);
        let candidate_refs = candidates.iter().map(String::as_str).collect::<Vec<_>>();
        let detect_paths = find_managed_paths(root, tool_dir, &candidate_refs);
        if let Some(executable) = detect_paths.executable.as_ref() {
            log::info!(
                "[{log_prefix}] *** DETECTED via find_managed_paths, executable={}, install_root={:?}",
                executable.display(),
                detect_paths
                    .install_root
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
            );
            return DetectResult {
                installed: true,
                version: read_command_version(executable, &["--version"]),
                install_path: detect_paths
                    .install_root
                    .as_ref()
                    .map(|path| path.to_string_lossy().to_string()),
            };
        }

        log::info!(
            "[{log_prefix}] find_managed_paths checked {} but no executable found (root={})",
            candidate_refs.join(", "),
            root.display()
        );
    }

    // Step 1: Windows PATH / shell fallback
    if let Some(version) = detect_windows_cli_version(command_name) {
        let path = find_command_install_dir(command_name);
        log::info!(
            "[{log_prefix}] *** DETECTED via detect_windows_cli_version (PATH fallback), version={}, path={:?}",
            version,
            path
        );
        return DetectResult {
            installed: true,
            version: Some(version),
            install_path: path.map(|p| p.to_string_lossy().to_string()),
        };
    }

    #[cfg(target_os = "windows")]
    {
        let mut shell = SystemWindowsShell;
        if let Some(install_path) =
            find_windows_cli_install_dir_with_shell(&mut shell, command_name)
        {
            if install_root.is_some_and(|root| is_managed_bin_dir(root, &install_path)) {
                log::info!(
                    "[{log_prefix}] ignoring stale managed bin shim path {}",
                    install_path.display()
                );
                return DetectResult::not_installed();
            }
            log::info!(
                "[{log_prefix}] *** DETECTED via find_windows_cli_install_dir_with_shell, path={}",
                install_path.display()
            );
            return DetectResult {
                installed: true,
                version: None,
                install_path: Some(install_path.to_string_lossy().to_string()),
            };
        }
    }

    log::info!("[{log_prefix}] Not detected");
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
    log::info!("[npm_cli] install_npm_cli package={}", package_name);
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
    log::info!(
        "[uninstall_npm_cli] called with target_dir={}, package={}",
        target_dir.display(),
        package_name
    );

    // Step 1: npm uninstall -g --prefix <target_dir>
    let uninstall_result = uninstall_npm_cli_with_runner(
        target_dir,
        package_name,
        |install_dir, args, context| {
            log::info!(
                "[uninstall_npm_cli] calling run_npm_command_checked_for_uninstall: install_dir={}, args={:?}",
                install_dir.display(),
                args
            );
            run_npm_command_checked_for_uninstall(install_dir, args, context)
        },
    );

    // Step 2: If npm uninstall succeeded, clean up any leftover shim files in target_dir
    if uninstall_result.is_ok() {
        log::info!(
            "[uninstall_npm_cli] npm uninstall succeeded, checking for leftover shim files in target_dir"
        );
        if let Err(e) = cleanup_npm_shm_files(target_dir) {
            log::warn!(
                "[uninstall_npm_cli] cleanup shim files returned error (non-fatal): {}",
                e
            );
        }
    } else {
        log::warn!(
            "[uninstall_npm_cli] npm uninstall failed, skipping shim cleanup: {:?}",
            uninstall_result.as_ref().err()
        );
    }

    uninstall_result
}

/// Cleans up npm-generated shim files (/*.cmd,/*.exe,/*.bat,/*.ps1) in the target tool directory.
fn cleanup_npm_shm_files(target_dir: &Path) -> Result<(), String> {
    if !target_dir.exists() {
        log::info!("[cleanup_npm_shm_files] target_dir does not exist, nothing to clean");
        return Ok(());
    }

    let extensions = ["cmd", "exe", "bat", "ps1"];
    let mut removed = 0;
    let mut errors = Vec::new();

    cleanup_dir_recursive(target_dir, &extensions, &mut removed, &mut errors);

    log::info!(
        "[cleanup_npm_shm_files] removed {} shim files, {} errors",
        removed,
        errors.len()
    );
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.join("; "))
    }
}

fn cleanup_dir_recursive(
    dir: &Path,
    extensions: &[&str],
    removed: &mut usize,
    errors: &mut Vec<String>,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) => {
            log::warn!(
                "[cleanup_dir_recursive] failed to read dir {}: {}",
                dir.display(),
                e
            );
            errors.push(format!("failed to read {}: {}", dir.display(), e));
            return;
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if path.is_dir() {
            cleanup_dir_recursive(&path, extensions, removed, errors);
        } else if let Some(ext) = path.extension() {
            if extensions.contains(&ext.to_string_lossy().to_lowercase().as_str()) {
                log::info!("[cleanup_dir_recursive] removing shim file: {:?}", path);
                if let Err(e) = std::fs::remove_file(&path) {
                    let msg = format!("failed to remove {}: {}", path.display(), e);
                    log::warn!("[cleanup_dir_recursive] {}", msg);
                    errors.push(msg);
                } else {
                    *removed += 1;
                }
            }
        }
    }
}

fn uninstall_npm_cli_with_runner<F>(
    target_dir: &Path,
    package_name: &str,
    run_command: F,
) -> Result<(), String>
where
    F: FnOnce(&Path, &[&str], &str) -> Result<(), String>,
{
    log::info!(
        "[uninstall_npm_cli_with_runner] target_dir={}, package_name={}",
        target_dir.display(),
        package_name
    );
    let target_dir_arg = target_dir.to_string_lossy().to_string();
    let args = [
        "uninstall",
        "-g",
        package_name,
        "--prefix",
        target_dir_arg.as_str(),
    ];
    let result = run_command(target_dir, &args, "npm uninstall failed");
    match &result {
        Ok(()) => log::info!("[uninstall_npm_cli_with_runner] npm uninstall succeeded"),
        Err(e) => log::error!(
            "[uninstall_npm_cli_with_runner] npm uninstall failed: {}",
            e
        ),
    }
    result
}

#[cfg(test)]
mod tests {
    use super::{
        install_npm_cli_with_extra_args_and_registry_and_runner,
        install_npm_cli_with_extra_args_and_runner, install_npm_cli_with_registry_and_runner,
        is_managed_bin_dir, uninstall_npm_cli_with_runner,
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
                assert_eq!(args[3], "--prefix");
                assert_eq!(args[4], target_dir.to_string_lossy());
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
                assert_eq!(args[3], "--prefix");
                assert_eq!(args[4], target_dir.to_string_lossy());
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
                assert_eq!(args[3], "--prefix");
                assert_eq!(args[4], target_dir.to_string_lossy());
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
    fn npm_cli_uninstall_targets_the_managed_prefix_directory() {
        let temp = tempfile::tempdir().unwrap();
        let target_dir = temp.path().join("codex-cli");

        uninstall_npm_cli_with_runner(
            &target_dir,
            "@openai/codex",
            |_install_root, args, context| {
                assert_eq!(context, "npm uninstall failed");
                assert_eq!(
                    args,
                    [
                        "uninstall",
                        "-g",
                        "@openai/codex",
                        "--prefix",
                        target_dir.to_string_lossy().as_ref(),
                    ]
                );
                Ok(())
            },
        )
        .unwrap();
    }

    #[test]
    fn npm_cli_detect_ignores_agenticboot_bin_directory() {
        let temp = tempfile::tempdir().unwrap();
        let install_root = temp.path().join("AgenticTools");
        let managed_bin = install_root.join("bin");

        assert!(is_managed_bin_dir(&install_root, &managed_bin));
        assert!(!is_managed_bin_dir(
            &install_root,
            &install_root.join("openclaw")
        ));
    }
}
