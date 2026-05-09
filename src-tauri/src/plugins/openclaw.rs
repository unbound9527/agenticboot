use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{
    find_command_on_path, find_managed_paths, npm_prefix_candidates, read_command_version,
    run_command_checked, run_detection_command_output, run_with_node_env,
};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

const OPENCLAW_UNINSTALL_ARGS: &[&str] = &["uninstall", "--all", "--yes", "--non-interactive"];

const OPENCLAW_NPX_UNINSTALL_ARGS: &[&str] = &[
    "-y",
    "openclaw",
    "uninstall",
    "--all",
    "--yes",
    "--non-interactive",
];

pub struct OpenClawPlugin;

fn emit_openclaw_progress(
    progress: &Sender<InstallProgress>,
    phase: &str,
    percent: u8,
    message: &str,
) {
    let _ = progress.blocking_send(InstallProgress {
        tool_id: "openclaw".into(),
        tool_name: "OpenClaw".into(),
        phase: phase.to_string(),
        percent,
        message: message.to_string(),
    });
}

fn run_official_openclaw_install<F>(
    progress: &Sender<InstallProgress>,
    mut runner: F,
) -> Result<(), String>
where
    F: FnMut() -> Result<(), String>,
{
    emit_openclaw_progress(
        progress,
        "starting",
        5,
        "Preparing the official OpenClaw PowerShell installer...",
    );
    emit_openclaw_progress(
        progress,
        "installing",
        25,
        "Invoking the official OpenClaw install script...",
    );
    emit_openclaw_progress(
        progress,
        "configuring",
        70,
        "Waiting for the official OpenClaw installer to finish...",
    );

    runner()
}

impl ToolPlugin for OpenClawPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "openclaw".into(),
            name: "OpenClaw".into(),
            description: "鍙紪绋?AI 缂栫爜寮曟搸".into(),
            icon: "openclaw".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::OfficialScript
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        if let Some(root) = install_root {
            let candidates = npm_prefix_candidates("openclaw");
            let candidate_refs = candidates.iter().map(String::as_str).collect::<Vec<_>>();
            let detect_paths = find_managed_paths(root, "openclaw", &candidate_refs);
            if let Some(executable) = detect_paths.executable.as_ref() {
                return DetectResult {
                    installed: true,
                    version: read_command_version(executable, &["--version"]),
                    install_path: detect_paths
                        .install_root
                        .map(|path| path.to_string_lossy().to_string()),
                };
            }
        }

        if let Some(output) = run_with_node_env(Path::new("openclaw"), &["--version"]) {
            if output.status.success() {
                return DetectResult {
                    installed: true,
                    version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
                    install_path: find_command_on_path("openclaw").and_then(|path| {
                        path.parent().map(|dir| dir.to_string_lossy().to_string())
                    }),
                };
            }
        }

        let mut command = Command::new("openclaw");
        command.arg("--version");
        if let Ok(output) = run_detection_command_output(&mut command, "openclaw") {
            if output.status.success() {
                return DetectResult {
                    installed: true,
                    version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
                    install_path: find_command_on_path("openclaw").and_then(|path| {
                        path.parent().map(|dir| dir.to_string_lossy().to_string())
                    }),
                };
            }
        }

        DetectResult {
            installed: false,
            version: None,
            install_path: None,
        }
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
        run_official_openclaw_install(&progress, || {
            let output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    "& ([scriptblock]::Create((Invoke-RestMethod https://openclaw.ai/install.ps1))) -NoOnboard",
                ])
                .output()
                .map_err(|e| format!("failed to launch OpenClaw install script: {e}"))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let details = if !stderr.is_empty() {
                    stderr
                } else if !stdout.is_empty() {
                    stdout
                } else {
                    format!("exit code: {:?}", output.status.code())
                };

                return Err(format!("OpenClaw install script failed: {details}"));
            }

            Ok(())
        })
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("OpenClaw 鑷姩瀹夎鐩墠浠呮敮鎸?Windows".into())
    }

    fn uninstall(&self, _target_dir: &Path) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            return run_official_openclaw_uninstall(|program, args| {
                run_command_checked(program, args, "OpenClaw 鍗歌浇澶辫触")
            });
        }

        #[allow(unreachable_code)]
        Ok(())
    }
}

fn run_official_openclaw_uninstall<F>(mut runner: F) -> Result<(), String>
where
    F: FnMut(&str, &[&str]) -> Result<(), String>,
{
    match runner("openclaw", OPENCLAW_UNINSTALL_ARGS) {
        Ok(()) => Ok(()),
        Err(primary_err) => match runner("npx", OPENCLAW_NPX_UNINSTALL_ARGS) {
            Ok(()) => Ok(()),
            Err(fallback_err)
                if is_program_not_found_error(&primary_err)
                    && is_program_not_found_error(&fallback_err) =>
            {
                Ok(())
            }
            Err(fallback_err) => Err(format!(
                "OpenClaw 鍗歌浇澶辫触锛屼富鍛戒护閿欒: {primary_err}; npx 鍥為€€閿欒: {fallback_err}"
            )),
        },
    }
}

fn is_program_not_found_error(error: &str) -> bool {
    let normalized = error.to_ascii_lowercase();
    normalized.contains("program not found")
        || normalized.contains("notfound")
        || normalized.contains("os error 2")
        || normalized.contains("cannot find the file specified")
        || normalized.contains("file specified")
}

#[cfg(test)]
mod tests {
    use super::{run_official_openclaw_install, run_official_openclaw_uninstall, OpenClawPlugin};
    use crate::plugin::ToolPlugin;
    use crate::tool_types::{InstallProgress, InstallStrategy};
    use tokio::sync::mpsc;

    #[test]
    fn native_windows_openclaw_uses_official_script_strategy() {
        assert_eq!(
            OpenClawPlugin.install_strategy(),
            InstallStrategy::OfficialScript
        );
    }

    #[test]
    fn openclaw_official_uninstall_uses_npx_fallback_after_openclaw_failure() {
        let mut invocations = Vec::new();

        let result = run_official_openclaw_uninstall(|program, args| {
            invocations.push((
                program.to_string(),
                args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>(),
            ));

            if program == "openclaw" {
                Err("primary failed".to_string())
            } else {
                Ok(())
            }
        });

        assert!(result.is_ok());
        assert_eq!(invocations.len(), 2);
        assert_eq!(invocations[0].0, "openclaw");
        assert_eq!(invocations[1].0, "npx");
        assert_eq!(
            invocations[1].1,
            vec![
                "-y".to_string(),
                "openclaw".to_string(),
                "uninstall".to_string(),
                "--all".to_string(),
                "--yes".to_string(),
                "--non-interactive".to_string(),
            ]
        );
    }

    #[test]
    fn openclaw_official_uninstall_is_noop_when_both_programs_are_missing() {
        let mut invocations = Vec::new();

        let result = run_official_openclaw_uninstall(|program, args| {
            invocations.push((
                program.to_string(),
                args.iter().map(|arg| arg.to_string()).collect::<Vec<_>>(),
            ));

            Err("program not found".to_string())
        });

        assert!(result.is_ok());
        assert_eq!(invocations.len(), 2);
        assert_eq!(invocations[0].0, "openclaw");
        assert_eq!(invocations[1].0, "npx");
    }

    #[test]
    fn openclaw_official_install_reports_waiting_progress_before_running_script() {
        let (tx, mut rx) = mpsc::channel::<InstallProgress>(8);
        let mut invoked = false;

        let result = run_official_openclaw_install(&tx, || {
            invoked = true;
            Ok(())
        });

        assert!(result.is_ok());
        assert!(invoked);

        let mut events = Vec::new();
        while let Ok(progress) = rx.try_recv() {
            events.push(progress);
        }

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].percent, 5);
        assert_eq!(events[1].percent, 25);
        assert_eq!(events[2].percent, 70);
        assert!(events[2].message.contains("official"));
    }
}
