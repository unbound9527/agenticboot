use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{
    find_command_on_path, find_managed_paths, read_command_version,
};
use crate::tool_types::{
    DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct HermesPlugin;

#[cfg(target_os = "windows")]
const PYTHON_RUNTIME_VERSION: &str = "3.13.13";
#[cfg(target_os = "windows")]
const PYTHON_RUNTIME_DIR_NAME: &str = "python-runtime";

impl ToolPlugin for HermesPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "hermes".into(),
            name: "Hermes (Web UI)".into(),
            description: "Multi-provider AI coding assistant with Web UI".into(),
            icon: "hermes".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::PythonPackage
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        if let Some(root) = install_root {
            let detect_paths = find_managed_paths(
                root,
                "hermes",
                &[
                    "venv\\Scripts\\hermes.exe",
                    "venv\\Scripts\\hermes.cmd",
                    "Scripts\\hermes.exe",
                    "Scripts\\hermes.cmd",
                ],
            );
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

        if let Ok(output) = Command::new("hermes").arg("--version").output() {
            if output.status.success() {
                return DetectResult {
                    installed: true,
                    version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
                    install_path: find_command_on_path("hermes").and_then(|path| {
                        path.parent()
                            .map(|dir| dir.to_string_lossy().to_string())
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
        target_dir: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "hermes".into(),
            tool_name: "Hermes".into(),
            phase: "installing".into(),
            percent: 0,
            message: "Preparing managed Python runtime...".into(),
        });

        let python_exe = ensure_managed_python_runtime(target_dir, &progress)?;
        let venv_dir = target_dir.join("venv");
        let hermes_python = venv_dir.join("Scripts").join("python.exe");

        let venv_output = Command::new(&python_exe)
            .args(["-m", "venv", &venv_dir.to_string_lossy()])
            .output()
            .map_err(|e| format!("failed to create Hermes venv: {e}"))?;
        if !venv_output.status.success() {
            return Err(format!(
                "failed to create Hermes venv: {}",
                String::from_utf8_lossy(&venv_output.stderr)
            ));
        }

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "hermes".into(),
            tool_name: "Hermes".into(),
            phase: "installing".into(),
            percent: 55,
            message: "Upgrading pip in Hermes venv...".into(),
        });

        run_python_module(
            &hermes_python,
            &["-m", "pip", "install", "--upgrade", "pip"],
            "upgrade pip",
        )?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "hermes".into(),
            tool_name: "Hermes".into(),
            phase: "installing".into(),
            percent: 80,
            message: "Installing hermes-agent[web,pty]...".into(),
        });

        run_python_module(
            &hermes_python,
            &["-m", "pip", "install", "hermes-agent[web,pty]"],
            "install hermes-agent",
        )?;

        let hermes_exe = venv_dir.join("Scripts").join("hermes.exe");
        if !hermes_exe.exists() {
            return Err("Hermes install did not produce venv\\Scripts\\hermes.exe".into());
        }
        run_python_module(&hermes_exe, &["--help"], "verify Hermes command")?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "hermes".into(),
            tool_name: "Hermes".into(),
            phase: "complete".into(),
            percent: 100,
            message: "Hermes install complete".into(),
        });
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        _target_dir: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("Automatic Hermes install is currently implemented only for Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        let managed_venv = target_dir.join("venv").join("pyvenv.cfg");
        if managed_venv.exists() {
            std::fs::remove_dir_all(target_dir)
                .map_err(|e| format!("failed to remove Hermes environment: {e}"))?;
            return Ok(());
        }

        Err("Automatic uninstall only supports AgenticBoot-managed Hermes environments".into())
    }
}

#[cfg(target_os = "windows")]
fn run_python_module(executable: &Path, args: &[&str], context: &str) -> Result<(), String> {
    let output = Command::new(executable)
        .args(args)
        .output()
        .map_err(|e| format!("{context} failed: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "{context} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn ensure_managed_python_runtime(
    target_dir: &Path,
    progress: &Sender<InstallProgress>,
) -> Result<PathBuf, String> {
    let python_exe = managed_python_executable(target_dir);
    if python_exe.exists() {
        return Ok(python_exe);
    }

    let runtime_dir = managed_python_runtime_dir(target_dir);
    std::fs::create_dir_all(&runtime_dir)
        .map_err(|e| format!("failed to create managed Python directory: {e}"))?;

    let archive_path = target_dir.join(format!("python-{PYTHON_RUNTIME_VERSION}.zip"));
    let download_url = managed_python_download_url(current_python_architecture());

    let _ = progress.blocking_send(InstallProgress {
        tool_id: "hermes".into(),
        tool_name: "Hermes".into(),
        phase: "downloading".into(),
        percent: 15,
        message: "Downloading managed Python runtime...".into(),
    });

    let runtime = tokio::runtime::Runtime::new()
        .map_err(|e| format!("failed to create runtime: {e}"))?;
    runtime.block_on(async {
        crate::services::downloader::download_file(&download_url, &archive_path, None).await
    })?;

    let _ = progress.blocking_send(InstallProgress {
        tool_id: "hermes".into(),
        tool_name: "Hermes".into(),
        phase: "extracting".into(),
        percent: 30,
        message: "Extracting managed Python runtime...".into(),
    });

    crate::services::downloader::extract_zip(&archive_path, &runtime_dir)?;
    std::fs::remove_file(&archive_path).ok();

    if !python_exe.exists() {
        return Err(format!(
            "managed Python runtime is missing {}",
            python_exe.display()
        ));
    }

    let verify = Command::new(&python_exe)
        .arg("--version")
        .output()
        .map_err(|e| format!("failed to verify managed Python runtime: {e}"))?;
    if !verify.status.success() {
        return Err(format!(
            "failed to verify managed Python runtime: {}",
            String::from_utf8_lossy(&verify.stderr)
        ));
    }

    Ok(python_exe)
}

#[cfg(target_os = "windows")]
fn managed_python_runtime_dir(target_dir: &Path) -> PathBuf {
    target_dir.join(PYTHON_RUNTIME_DIR_NAME)
}

#[cfg(target_os = "windows")]
fn managed_python_executable(target_dir: &Path) -> PathBuf {
    managed_python_runtime_dir(target_dir).join("python.exe")
}

#[cfg(target_os = "windows")]
fn managed_python_download_url(arch: &str) -> String {
    format!(
        "https://www.python.org/ftp/python/{version}/python-{version}-{arch}.zip",
        version = PYTHON_RUNTIME_VERSION,
        arch = arch
    )
}

#[cfg(target_os = "windows")]
fn current_python_architecture() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "amd64"
    }
}

#[cfg(test)]
mod tests {
    use super::HermesPlugin;
    use crate::plugin::ToolPlugin;
    use crate::tool_types::InstallStrategy;

    #[test]
    fn native_windows_hermes_uses_python_package_strategy() {
        assert_eq!(HermesPlugin.install_strategy(), InstallStrategy::PythonPackage);
    }

    #[test]
    fn native_windows_hermes_detects_managed_venv_command() {
        let tmp = tempfile::tempdir().unwrap();
        let hermes_scripts = tmp.path().join("hermes").join("venv").join("Scripts");
        std::fs::create_dir_all(&hermes_scripts).unwrap();
        std::fs::write(
            hermes_scripts.join("hermes.cmd"),
            "@echo off\r\necho hermes 0.9.0\r\n",
        )
        .unwrap();

        let detect = HermesPlugin.detect(Some(tmp.path()));
        assert!(detect.installed);
        assert_eq!(detect.version.as_deref(), Some("hermes 0.9.0"));
        assert_eq!(
            detect.install_path.as_deref(),
            Some(tmp.path().join("hermes").to_string_lossy().as_ref())
        );
    }

    #[test]
    fn native_windows_hermes_managed_python_runtime_path_uses_tool_directory() {
        let tmp = tempfile::tempdir().unwrap();

        assert_eq!(
            super::managed_python_runtime_dir(tmp.path()),
            tmp.path().join("python-runtime")
        );
        assert_eq!(
            super::managed_python_executable(tmp.path()),
            tmp.path().join("python-runtime").join("python.exe")
        );
    }

    #[test]
    fn native_windows_hermes_python_runtime_urls_match_supported_architectures() {
        let amd64 = super::managed_python_download_url("amd64");
        let arm64 = super::managed_python_download_url("arm64");

        assert!(amd64.ends_with("/python-3.13.13-amd64.zip"));
        assert!(arm64.ends_with("/python-3.13.13-arm64.zip"));
    }
}
