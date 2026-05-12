use crate::plugin::{ToolInstallContext, ToolPlugin};
use crate::services::installer::logging::InstallLogEmitter;
use crate::services::installer::windows::{
    find_local_uninstaller_executable, find_uninstall_entry_ex, run_windows_uninstaller_with_common_args,
    run_winget, run_winget_with_logs, winget_exists, WindowsUninstallEntry,
};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use log::{debug, info, warn, error};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

pub struct ClaudeCodeDesktopPlugin;

#[cfg(target_os = "windows")]
const CLAUDE_WINGET_ARGS: &[&str] = &[
    "install",
    "--id",
    "Anthropic.Claude",
    "-e",
    "--accept-package-agreements",
    "--accept-source-agreements",
];

#[cfg(target_os = "windows")]
fn emit_claude_progress(
    progress: &Sender<InstallProgress>,
    phase: &str,
    percent: u8,
    message: &str,
) {
    let _ = progress.blocking_send(InstallProgress {
        tool_id: "claude-code-desktop".into(),
        tool_name: "Claude Code (Desktop)".into(),
        phase: phase.to_string(),
        percent,
        message: message.to_string(),
    });
}

#[cfg(all(target_os = "windows", test))]
fn quote_command_arg(arg: &str) -> String {
    if arg.contains([' ', '\t', '"']) {
        format!("\"{}\"", arg.replace('"', "\\\""))
    } else {
        arg.to_string()
    }
}

#[cfg(all(target_os = "windows", test))]
fn format_command_for_preview(program: &str, args: &[&str]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().map(|arg| quote_command_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(all(target_os = "windows", test))]
fn build_claude_desktop_install_preview(installer: &Path) -> Vec<String> {
    vec![
        format_command_for_preview("winget", CLAUDE_WINGET_ARGS),
        format_command_for_preview(&installer.to_string_lossy(), &[]),
    ]
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallerProcessState {
    Running,
    Exited(Option<i32>),
}

#[cfg(target_os = "windows")]
fn wait_for_claude_desktop_detection<F, G>(
    mut detect_installed: F,
    mut poll_process: G,
    max_polls: usize,
) -> Result<(), String>
where
    F: FnMut() -> bool,
    G: FnMut() -> Result<InstallerProcessState, String>,
{
    let mut process_exited_successfully = false;
    let mut post_exit_polls = 0usize;

    for _ in 0..max_polls {
        if detect_installed() {
            return Ok(());
        }

        match poll_process()? {
            InstallerProcessState::Running => {}
            InstallerProcessState::Exited(code) => {
                if code.is_some_and(|value| value != 0) {
                    return Err(format!(
                        "Claude desktop installer exited with code {:?}",
                        code
                    ));
                }

                process_exited_successfully = true;
                post_exit_polls += 1;
                if post_exit_polls >= 5 {
                    break;
                }
            }
        }

        std::thread::sleep(Duration::from_secs(2));
    }

    if process_exited_successfully {
        Err(
            "Claude desktop installer exited, but Claude Desktop was still not detected afterward"
                .to_string(),
        )
    } else {
        Err(
            "Timed out while waiting for Claude Desktop to appear after launching the installer"
                .to_string(),
        )
    }
}

#[cfg(target_os = "windows")]
fn install_claude_desktop(
    progress: &Sender<InstallProgress>,
    install_log: Option<&InstallLogEmitter>,
) -> Result<(), String> {
    emit_claude_progress(
        progress,
        "starting",
        5,
        "Preparing Claude Code (Desktop) install...",
    );
    emit_claude_progress(
        progress,
        "installing",
        15,
        "Trying winget install for Claude Code (Desktop)...",
    );

    if winget_exists() {
        let winget_result = if let Some(install_log) = install_log {
            install_log.emit_phase("installing", "Trying winget install for Claude desktop");
            run_winget_with_logs(install_log, "installing", CLAUDE_WINGET_ARGS)
        } else {
            run_winget(CLAUDE_WINGET_ARGS)
        };

        if winget_result.is_ok() {
            return Ok(());
        }

        if let Some(install_log) = install_log {
            install_log.emit_phase(
                "downloading",
                "winget failed, falling back to the official Claude desktop installer",
            );
        }
    } else if let Some(install_log) = install_log {
        install_log.emit_phase(
            "downloading",
            "winget is unavailable, falling back to the official Claude desktop installer",
        );
    }

    emit_claude_progress(
        progress,
        "downloading",
        45,
        "Downloading the official Claude desktop installer...",
    );

    let installer = crate::services::downloader::temp_path("claude-desktop-setup.exe");
    let rt =
        tokio::runtime::Runtime::new().map_err(|e| format!("failed to create runtime: {e}"))?;
    rt.block_on(async {
        crate::services::downloader::download_file(windows_download_url(), &installer, None).await
    })?;

    emit_claude_progress(
        progress,
        "installing",
        80,
        "Launching the downloaded Claude desktop installer...",
    );

    if let Some(install_log) = install_log {
        install_log.emit_phase(
            "installing",
            "Launching the downloaded Claude desktop installer",
        );
        install_log.emit_command("installing", installer.to_string_lossy());
    }

    let mut child = Command::new(&installer)
        .spawn()
        .map_err(|e| format!("failed to launch Claude desktop installer: {e}"))?;

    emit_claude_progress(
        progress,
        "configuring",
        90,
        "Waiting for Claude Desktop to appear after launching the installer...",
    );

    wait_for_claude_desktop_detection(
        || ClaudeCodeDesktopPlugin.detect(None).installed,
        || match child.try_wait() {
            Ok(Some(status)) => Ok(InstallerProcessState::Exited(status.code())),
            Ok(None) => Ok(InstallerProcessState::Running),
            Err(error) => Err(format!(
                "failed while waiting for Claude desktop installer: {error}"
            )),
        },
        120,
    )
}

impl ToolPlugin for ClaudeCodeDesktopPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "claude-code-desktop".into(),
            name: "Claude Code (Desktop)".into(),
            description: "Official Claude desktop app for Windows".into(),
            icon: "claude".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::DesktopInstaller
    }

    fn detect(&self, _install_root: Option<&Path>) -> DetectResult {
        info!("[Claude Desktop] detect called, searching registry for entries matching Claude/AnthropicClaude excluding CLI/npm");
        if let Some(entry) =
            find_uninstall_entry_ex(&["Claude", "AnthropicClaude"], &["CLI", "npm"])
        {
            info!("[Claude Desktop] detect found registry entry: name={:?}, version={:?}, install_location={:?}, uninstall_string={:?}",
                entry.display_name, entry.display_version, entry.install_location, entry.uninstall_string);
            return detect_claude_desktop_from_entry(entry);
        }

        debug!("Claude desktop not found in registry");
        DetectResult::not_installed()
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
        install_claude_desktop(&progress, None)
    }

    #[cfg(target_os = "windows")]
    fn install_with_context(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
        context: ToolInstallContext,
    ) -> Result<(), String> {
        install_claude_desktop(&progress, Some(context.install_log()))
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("Claude desktop auto-install is currently supported only on Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            log::info!("[Claude Desktop] Starting uninstall via winget first");
            if winget_exists() {
                log::info!("[Claude Desktop] winget detected, attempting uninstall via winget");
                if run_winget(&[
                    "uninstall",
                    "--id",
                    "Anthropic.Claude",
                    "-e",
                    "--accept-source-agreements",
                ])
                .is_ok()
                {
                    log::info!("[Claude Desktop] winget uninstall succeeded");
                    return Ok(());
                }
                log::warn!("[Claude Desktop] winget uninstall failed, trying next method");
            } else {
                log::info!("[Claude Desktop] winget not available, skipping to registry");
            }

            log::info!("[Claude Desktop] Checking registry uninstall entries");
            if let Some(entry) =
                find_uninstall_entry_ex(&["Claude", "AnthropicClaude"], &["CLI", "npm"])
            {
                log::info!("[Claude Desktop] Found registry entry: {:?}", entry.display_name);
                log::info!("[Claude Desktop] uninstall_string = {:?}", entry.uninstall_string);
                if let Some(uninstall_string) = entry.uninstall_string {
                    log::info!("[Claude Desktop] Executing registry uninstall string: {}", uninstall_string);
                    let status = Command::new("cmd")
                        .args(["/C", &uninstall_string])
                        .spawn()
                        .map_err(|e| format!("failed to launch Claude uninstall command: {e}"))?
                        .wait()
                        .map_err(|e| format!("failed to wait for Claude uninstall command: {e}"))?;
                    log::info!("[Claude Desktop] Registry uninstall exit code: {:?}", status.code());
                    if status.success() {
                        log::info!("[Claude Desktop] Registry uninstall succeeded");
                        return Ok(());
                    }
                    log::warn!("[Claude Desktop] Registry uninstall returned non-zero exit");
                } else {
                    log::warn!("[Claude Desktop] Entry found but has no uninstall_string");
                }
            } else {
                log::info!("[Claude Desktop] No registry entry found");
            }

            log::info!("[Claude Desktop] Checking for local uninstaller in target_dir: {:?}", target_dir);
            if let Some(uninstaller) = find_local_uninstaller_executable(target_dir) {
                log::info!("[Claude Desktop] Found local uninstaller: {:?}", uninstaller);
                log::info!("[Claude Desktop] Running local uninstaller with common args");
                run_windows_uninstaller_with_common_args(&uninstaller)?;
                log::info!("[Claude Desktop] Local uninstaller succeeded");
                return Ok(());
            } else {
                log::info!("[Claude Desktop] No local uninstaller found in target_dir");
            }

            log::error!("[Claude Desktop] All uninstall methods exhausted, returning error");
            return Err("No Claude desktop uninstall command was found".into());
        }

        #[allow(unreachable_code)]
        Ok(())
    }
}

fn detect_claude_desktop_from_entry(entry: WindowsUninstallEntry) -> DetectResult {
    let install_path = entry.install_location.or(entry
        .display_icon
        .and_then(|path| path.parent().map(PathBuf::from)));

    debug!(
        "detected Claude desktop: version={:?}, path={:?}",
        entry.display_version, install_path
    );
    DetectResult {
        installed: true,
        version: entry.display_version,
        install_path: install_path.map(|dir| dir.to_string_lossy().to_string()),
    }
}

#[cfg(target_os = "windows")]
fn windows_download_url() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "https://claude.ai/api/desktop/win32/arm64/setup/latest/redirect"
    } else {
        "https://claude.ai/api/desktop/win32/x64/setup/latest/redirect"
    }
}

#[cfg(test)]
mod tests {
    use super::{
        detect_claude_desktop_from_entry, wait_for_claude_desktop_detection, InstallerProcessState,
    };
    use crate::services::installer::windows::WindowsUninstallEntry;
    use std::path::Path;

    #[test]
    fn claude_desktop_install_preview_includes_winget_and_fallback_launcher() {
        let lines = super::build_claude_desktop_install_preview(Path::new(
            "C:\\Temp\\claude-desktop-setup.exe",
        ));

        assert!(lines
            .iter()
            .any(|line| line.contains("winget install --id Anthropic.Claude")));
        assert!(lines
            .iter()
            .any(|line| line.contains("C:\\Temp\\claude-desktop-setup.exe")));
    }

    #[test]
    fn claude_desktop_waiter_returns_when_detection_succeeds_before_process_exit() {
        let mut polls = 0usize;
        let result = wait_for_claude_desktop_detection(
            || {
                polls += 1;
                polls >= 2
            },
            || Ok(InstallerProcessState::Running),
            3,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn claude_desktop_waiter_fails_when_process_exits_without_detection() {
        let result = wait_for_claude_desktop_detection(
            || false,
            || Ok(InstallerProcessState::Exited(Some(0))),
            5,
        )
        .unwrap_err();

        assert!(result.contains("still not detected"));
    }

    #[test]
    fn claude_desktop_detects_installs_under_anthropic_local_path() {
        let detect = detect_claude_desktop_from_entry(WindowsUninstallEntry {
            display_name: "Claude".into(),
            display_version: Some("1.2.3".into()),
            install_location: Some(
                Path::new("C:\\Users\\me\\AppData\\Local\\AnthropicClaude").into(),
            ),
            display_icon: None,
            uninstall_string: None,
        });

        assert!(detect.installed);
        assert_eq!(detect.version.as_deref(), Some("1.2.3"));
        assert_eq!(
            detect.install_path.as_deref(),
            Some("C:\\Users\\me\\AppData\\Local\\AnthropicClaude")
        );
    }
}
