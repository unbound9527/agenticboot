use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{
    find_command_on_path, find_managed_paths, read_command_version, run_detection_command_output,
};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct HermesPlugin;

/// Matches NousResearch `scripts/install.ps1`: Python **3.11** via `uv python` / embeddable zip.
/// Hermes declares `requires-python >= 3.11`; 3.11 is the best-tested Windows path upstream.
#[cfg(target_os = "windows")]
const PYTHON_RUNTIME_VERSION: &str = "3.11.9";
#[cfg(target_os = "windows")]
const PYTHON_RUNTIME_DIR_NAME: &str = "python-runtime";

/// Source tarball — same repo as https://hermes-agent.nousresearch.com/docs/getting-started/installation
/// (Windows installer clones this; we use the ZIP so users do not need `git` on PATH).
///
/// Use **`main`** branch archives: release tags like `v0.13.0` are not guaranteed to exist on GitHub
/// (404 on `refs/tags/...`), while `refs/heads/main` matches upstream `install.ps1` default `$Branch`.
#[cfg(target_os = "windows")]
const HERMES_AGENT_SOURCE_ZIP_URL: &str =
    "https://github.com/NousResearch/hermes-agent/archive/refs/heads/main.zip";
#[cfg(target_os = "windows")]
const HERMES_AGENT_CHECKOUT_DIR: &str = "hermes-agent-checkout";
#[cfg(target_os = "windows")]
const HERMES_AGENT_ZIP_NAME: &str = "hermes-agent-src.zip";

/// Tier order mirrors `scripts/install.ps1` `Install-Dependencies`: prefer dashboard + core
/// (`[web,mcp,cron,cli,messaging,dev]`), then `[web]` only, then bare package.
#[cfg(target_os = "windows")]
const HERMES_PIP_INSTALL_TIERS: &[&str] = &[".[web,mcp,cron,cli,messaging,dev]", ".[web]", "."];

impl ToolPlugin for HermesPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "hermes".into(),
            name: "Hermes (Web UI)".into(),
            description: "多供应商 AI 编程助手，带 Web 界面".into(),
            icon: "hermes".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::PythonPackage
    }

    fn command_name(&self) -> Option<&'static str> {
        Some("hermes")
    }

    fn managed_executable_candidates(&self) -> Vec<String> {
        vec![
            "venv\\Scripts\\hermes.exe".to_string(),
            "venv\\Scripts\\hermes.cmd".to_string(),
            "Scripts\\hermes.exe".to_string(),
            "Scripts\\hermes.cmd".to_string(),
        ]
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

        if let Some(executable) = find_command_on_path("hermes") {
            let install_path = detect_install_path_from_executable(&executable)
                .or_else(|| executable.parent().map(PathBuf::from))
                .map(|dir| dir.to_string_lossy().to_string());
            let version = read_python_package_version_from_executable(
                &executable,
                &["hermes_agent", "hermes-agent"],
            )
            .or_else(|| read_hermes_command_version());

            return DetectResult {
                installed: true,
                version,
                install_path,
            };
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
        _install_root: &Path,
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
            &[
                "-m",
                "pip",
                "install",
                "--upgrade",
                "pip",
                "setuptools",
                "wheel",
            ],
            "upgrade pip",
        )?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "hermes".into(),
            tool_name: "Hermes".into(),
            phase: "downloading".into(),
            percent: 62,
            message: "Downloading Hermes Agent source (GitHub)...".into(),
        });

        let checkout_dir = target_dir.join(HERMES_AGENT_CHECKOUT_DIR);
        let zip_path = target_dir.join(HERMES_AGENT_ZIP_NAME);
        if checkout_dir.exists() {
            std::fs::remove_dir_all(&checkout_dir)
                .map_err(|e| format!("failed to clear Hermes checkout dir: {e}"))?;
        }
        std::fs::create_dir_all(&checkout_dir)
            .map_err(|e| format!("failed to create Hermes checkout dir: {e}"))?;

        let rt =
            tokio::runtime::Runtime::new().map_err(|e| format!("failed to create runtime: {e}"))?;
        rt.block_on(async {
            crate::services::downloader::download_file(HERMES_AGENT_SOURCE_ZIP_URL, &zip_path, None)
                .await
        })?;

        let _ = progress.blocking_send(InstallProgress {
            tool_id: "hermes".into(),
            tool_name: "Hermes".into(),
            phase: "extracting".into(),
            percent: 68,
            message: "Extracting Hermes Agent source...".into(),
        });

        crate::services::downloader::extract_zip(&zip_path, &checkout_dir)?;
        std::fs::remove_file(&zip_path).ok();

        let project_root = find_hermes_pyproject_root(&checkout_dir)?;
        install_hermes_from_source_with_tiers(&hermes_python, &project_root, &progress)?;

        std::fs::remove_dir_all(&checkout_dir).ok();

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
        _install_root: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("Automatic Hermes install is currently implemented only for Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        if let Some(uninstall_root) = resolve_uninstall_root(target_dir) {
            std::fs::remove_dir_all(&uninstall_root)
                .map_err(|e| format!("failed to remove Hermes environment: {e}"))?;
            return Ok(());
        }

        if remove_hermes_launchers(target_dir)? {
            return Ok(());
        }

        Err("Automatic uninstall only supports Hermes virtual environments".into())
    }
}

fn read_hermes_command_version() -> Option<String> {
    let mut command = Command::new("hermes");
    command.arg("--version");
    let output = run_detection_command_output(&mut command, "hermes").ok()?;
    if !output.status.success() {
        return None;
    }

    extract_hermes_version(&String::from_utf8_lossy(&output.stdout))
        .or_else(|| extract_hermes_version(&String::from_utf8_lossy(&output.stderr)))
}

fn detect_install_path_from_executable(executable: &Path) -> Option<PathBuf> {
    let scripts_dir = executable.parent()?;
    resolve_uninstall_root(scripts_dir).or_else(|| Some(scripts_dir.to_path_buf()))
}

fn resolve_uninstall_root(target_dir: &Path) -> Option<PathBuf> {
    let mut candidates = vec![target_dir.to_path_buf()];
    if let Some(parent) = target_dir.parent() {
        candidates.push(parent.to_path_buf());
        if let Some(grandparent) = parent.parent() {
            candidates.push(grandparent.to_path_buf());
        }
    }

    for candidate in candidates {
        if candidate.join("venv").join("pyvenv.cfg").exists() {
            return Some(candidate);
        }

        if candidate.join("pyvenv.cfg").exists() {
            if let Some(parent) = candidate.parent() {
                if parent.join("python-runtime").exists() {
                    return Some(parent.to_path_buf());
                }
            }
            return Some(candidate);
        }
    }

    None
}

fn remove_hermes_launchers(target_path: &Path) -> Result<bool, String> {
    if target_path.is_file() {
        let is_hermes_launcher = target_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(is_hermes_launcher_name)
            .unwrap_or(false);

        if is_hermes_launcher {
            let parent = target_path.parent().unwrap_or(target_path);
            return remove_hermes_launchers_from_dir(parent);
        }

        return Ok(false);
    }

    if !target_path.is_dir() {
        return Ok(false);
    }

    remove_hermes_launchers_from_dir(target_path).and_then(|removed| {
        if removed {
            Ok(true)
        } else {
            remove_hermes_launchers_from_dir(&target_path.join("Scripts"))
        }
    })
}

fn remove_hermes_launchers_from_dir(target_dir: &Path) -> Result<bool, String> {
    if !target_dir.is_dir() {
        return Ok(false);
    }

    let mut removed_any = false;
    for launcher in HERMES_LAUNCHERS {
        let path = target_dir.join(launcher);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| format!("failed to remove Hermes launcher {}: {e}", path.display()))?;
            removed_any = true;
        }
    }

    Ok(removed_any)
}

fn is_hermes_launcher_name(file_name: &str) -> bool {
    HERMES_LAUNCHERS
        .iter()
        .any(|launcher| launcher.eq_ignore_ascii_case(file_name))
}

const HERMES_LAUNCHERS: &[&str] = &["hermes", "hermes.exe", "hermes.cmd", "hermes.bat"];

fn extract_hermes_version(output: &str) -> Option<String> {
    let first_line = output
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())?;
    extract_semver_like(first_line).or_else(|| Some(first_line.to_string()))
}

fn extract_semver_like(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        let start = if matches!(bytes[index], b'v' | b'V') {
            let next_index = index + 1;
            if next_index < bytes.len() && bytes[next_index].is_ascii_digit() {
                next_index
            } else {
                index += 1;
                continue;
            }
        } else if bytes[index].is_ascii_digit() {
            index
        } else {
            index += 1;
            continue;
        };

        let mut end = start;
        while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
            end += 1;
        }

        let candidate = &text[start..end];
        if candidate.split('.').count() >= 3
            && candidate
                .split('.')
                .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
        {
            return Some(candidate.to_string());
        }

        index = end.max(index + 1);
    }

    None
}

fn read_python_package_version_from_executable(
    executable: &Path,
    package_names: &[&str],
) -> Option<String> {
    let environment_root = executable.parent()?.parent()?;
    let site_packages_candidates = [
        environment_root.join("Lib").join("site-packages"),
        environment_root.join("lib").join("site-packages"),
    ];

    for site_packages in site_packages_candidates {
        let version = read_python_package_version_from_site_packages(&site_packages, package_names);
        if version.is_some() {
            return version;
        }
    }

    None
}

fn read_python_package_version_from_site_packages(
    site_packages: &Path,
    package_names: &[&str],
) -> Option<String> {
    if !site_packages.is_dir() {
        return None;
    }

    let normalized_names = package_names
        .iter()
        .map(|name| normalize_python_package_name(name))
        .collect::<Vec<_>>();

    let entries = std::fs::read_dir(site_packages).ok()?;
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let file_name = entry.file_name().to_string_lossy().to_string();
        let normalized = normalize_python_package_name(&file_name);
        let matches_package = normalized_names
            .iter()
            .any(|package| normalized.starts_with(package) && normalized.ends_with(".dist.info"));

        if !matches_package {
            continue;
        }

        let metadata = path.join("METADATA");
        let version = std::fs::read_to_string(metadata)
            .ok()?
            .lines()
            .map(str::trim)
            .find_map(|line| line.strip_prefix("Version:"))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);

        if version.is_some() {
            return version;
        }
    }

    None
}

fn normalize_python_package_name(value: &str) -> String {
    value.to_ascii_lowercase().replace(['-', '_'], ".")
}

/// GitHub `archive/refs/tags/...zip` unpacks to a single top-level directory containing `pyproject.toml`.
#[cfg(target_os = "windows")]
fn find_hermes_pyproject_root(checkout_dir: &Path) -> Result<PathBuf, String> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(checkout_dir)
        .map_err(|e| format!("failed to read Hermes checkout: {e}"))?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_dir() && p.join("pyproject.toml").is_file())
        .collect();

    match roots.len() {
        0 => Err(
            "Hermes source archive did not contain a project with pyproject.toml (bad ZIP?)".into(),
        ),
        1 => Ok(roots.pop().expect("len checked")),
        _ => roots
            .into_iter()
            .find(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.to_lowercase().starts_with("hermes-agent"))
            })
            .ok_or_else(|| {
                "Hermes checkout contained multiple Python projects; expected a single hermes-agent-* root"
                    .to_string()
            }),
    }
}

/// Follow `scripts/install.ps1` `Install-Dependencies`: tiered `pip install` so one flaky extra
/// does not block a slimmer successful install.
#[cfg(target_os = "windows")]
fn install_hermes_from_source_with_tiers(
    hermes_python: &Path,
    project_root: &Path,
    progress: &Sender<InstallProgress>,
) -> Result<(), String> {
    let mut last_err = String::new();
    for (i, tier) in HERMES_PIP_INSTALL_TIERS.iter().enumerate() {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "hermes".into(),
            tool_name: "Hermes".into(),
            phase: "installing".into(),
            percent: 72u8.saturating_add((i as u8).saturating_mul(2)),
            message: format!("Installing Hermes ({tier})..."),
        });
        match run_python_module_in_dir(
            hermes_python,
            project_root,
            &["-m", "pip", "install", tier],
            "install hermes-agent from source",
        ) {
            Ok(()) => {
                ensure_hermes_web_dashboard_deps(hermes_python, project_root, progress)?;
                return Ok(());
            }
            Err(e) => last_err = e,
        }
    }

    Err(format!(
        "Hermes install failed after all dependency tiers. Last error:\n{last_err}"
    ))
}

/// `install.ps1` verifies `import fastapi, uvicorn` after tiers — required for `hermes dashboard`.
#[cfg(target_os = "windows")]
fn ensure_hermes_web_dashboard_deps(
    hermes_python: &Path,
    project_root: &Path,
    progress: &Sender<InstallProgress>,
) -> Result<(), String> {
    let check = Command::new(hermes_python)
        .args(["-c", "import fastapi, uvicorn"])
        .output()
        .map_err(|e| format!("hermes dashboard import check: {e}"))?;
    if check.status.success() {
        return Ok(());
    }

    let _ = progress.blocking_send(InstallProgress {
        tool_id: "hermes".into(),
        tool_name: "Hermes".into(),
        phase: "installing".into(),
        percent: 90,
        message: "Installing Hermes [web] (FastAPI / Uvicorn) for dashboard...".into(),
    });

    if run_python_module_in_dir(
        hermes_python,
        project_root,
        &["-m", "pip", "install", ".[web]"],
        "install hermes [web] extra",
    )
    .is_ok()
    {
        return Ok(());
    }

    run_python_module(
        hermes_python,
        &[
            "-m",
            "pip",
            "install",
            "fastapi>=0.104.0,<1",
            "uvicorn[standard]>=0.24.0,<1",
        ],
        "install fastapi/uvicorn for Hermes dashboard",
    )
}

#[cfg(target_os = "windows")]
fn run_python_module_in_dir(
    executable: &Path,
    current_dir: &Path,
    args: &[&str],
    context: &str,
) -> Result<(), String> {
    let output = Command::new(executable)
        .current_dir(current_dir)
        .args(args)
        .output()
        .map_err(|e| format!("{context} failed: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let tail = if stdout.trim().is_empty() {
            String::new()
        } else {
            format!("\nstdout:\n{stdout}")
        };
        return Err(format!("{context} failed: {stderr}{tail}"));
    }
    Ok(())
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

    let runtime =
        tokio::runtime::Runtime::new().map_err(|e| format!("failed to create runtime: {e}"))?;
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
    use super::{
        extract_hermes_version, read_python_package_version_from_executable, HermesPlugin,
    };
    use crate::plugin::ToolPlugin;
    use crate::tool_types::InstallStrategy;

    #[test]
    #[cfg(target_os = "windows")]
    fn find_hermes_pyproject_root_finds_github_archive_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("hermes-agent-main");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("pyproject.toml"), "[project]\n").unwrap();

        let resolved = super::find_hermes_pyproject_root(tmp.path()).unwrap();
        assert_eq!(resolved, root);
    }

    #[test]
    fn native_windows_hermes_uses_python_package_strategy() {
        assert_eq!(
            HermesPlugin.install_strategy(),
            InstallStrategy::PythonPackage
        );
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

        assert!(amd64.ends_with("/python-3.11.9-amd64.zip"));
        assert!(arm64.ends_with("/python-3.11.9-arm64.zip"));
    }

    #[test]
    fn hermes_extract_version_returns_short_semver_from_multiline_output() {
        let version = extract_hermes_version(
            "Hermes Agent v0.12.0 (2026.4.30)\r\nProject: D:\\projects\\hermes-agent\r\nPython: 3.14.0\r\n",
        );

        assert_eq!(version.as_deref(), Some("0.12.0"));
    }

    #[test]
    fn hermes_reads_version_from_adjacent_python_dist_info() {
        let tmp = tempfile::tempdir().unwrap();
        let scripts_dir = tmp.path().join("Scripts");
        let dist_info_dir = tmp
            .path()
            .join("Lib")
            .join("site-packages")
            .join("hermes_agent-0.12.0.dist-info");

        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::create_dir_all(&dist_info_dir).unwrap();
        std::fs::write(scripts_dir.join("hermes.exe"), b"").unwrap();
        std::fs::write(
            dist_info_dir.join("METADATA"),
            "Metadata-Version: 2.4\r\nName: hermes-agent\r\nVersion: 0.12.0\r\n",
        )
        .unwrap();

        let version = read_python_package_version_from_executable(
            &scripts_dir.join("hermes.exe"),
            &["hermes_agent", "hermes-agent"],
        );

        assert_eq!(version.as_deref(), Some("0.12.0"));
    }

    #[test]
    fn native_windows_hermes_uninstall_accepts_scripts_directory_for_managed_install() {
        let tmp = tempfile::tempdir().unwrap();
        let tool_dir = tmp.path().join("hermes");
        let scripts_dir = tool_dir.join("venv").join("Scripts");
        let python_runtime_dir = tool_dir.join("python-runtime");

        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::create_dir_all(&python_runtime_dir).unwrap();
        std::fs::write(tool_dir.join("venv").join("pyvenv.cfg"), "home = managed").unwrap();
        std::fs::write(scripts_dir.join("hermes.exe"), b"").unwrap();

        HermesPlugin.uninstall(&scripts_dir).unwrap();

        assert!(!tool_dir.exists());
    }

    #[test]
    fn native_windows_hermes_uninstall_accepts_venv_root_for_external_environment() {
        let tmp = tempfile::tempdir().unwrap();
        let venv_dir = tmp.path().join("hermes-venv");
        let scripts_dir = venv_dir.join("Scripts");

        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::write(venv_dir.join("pyvenv.cfg"), "home = external").unwrap();
        std::fs::write(scripts_dir.join("hermes.exe"), b"").unwrap();

        HermesPlugin.uninstall(&venv_dir).unwrap();

        assert!(!venv_dir.exists());
    }

    #[test]
    fn native_windows_hermes_uninstall_accepts_scripts_directory_for_external_environment() {
        let tmp = tempfile::tempdir().unwrap();
        let venv_dir = tmp.path().join("hermes-venv");
        let scripts_dir = venv_dir.join("Scripts");

        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::write(venv_dir.join("pyvenv.cfg"), "home = external").unwrap();
        std::fs::write(scripts_dir.join("hermes.exe"), b"").unwrap();

        HermesPlugin.uninstall(&scripts_dir).unwrap();

        assert!(!venv_dir.exists());
    }

    #[test]
    fn native_windows_hermes_uninstall_accepts_plain_scripts_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let scripts_dir = tmp.path().join("Python313").join("Scripts");

        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::write(scripts_dir.join("hermes.exe"), b"").unwrap();
        std::fs::write(scripts_dir.join("hermes.cmd"), b"").unwrap();
        std::fs::write(scripts_dir.join("other.exe"), b"").unwrap();

        HermesPlugin.uninstall(&scripts_dir).unwrap();

        assert!(!scripts_dir.join("hermes.exe").exists());
        assert!(!scripts_dir.join("hermes.cmd").exists());
        assert!(scripts_dir.join("other.exe").exists());
        assert!(scripts_dir.exists());
    }

    #[test]
    fn native_windows_hermes_uninstall_accepts_plain_launcher_path() {
        let tmp = tempfile::tempdir().unwrap();
        let scripts_dir = tmp.path().join("Python313").join("Scripts");
        let hermes_exe = scripts_dir.join("hermes.exe");

        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::write(&hermes_exe, b"").unwrap();
        std::fs::write(scripts_dir.join("hermes.cmd"), b"").unwrap();
        std::fs::write(scripts_dir.join("other.exe"), b"").unwrap();

        HermesPlugin.uninstall(&hermes_exe).unwrap();

        assert!(!hermes_exe.exists());
        assert!(!scripts_dir.join("hermes.cmd").exists());
        assert!(scripts_dir.join("other.exe").exists());
        assert!(scripts_dir.exists());
    }

    #[test]
    fn native_windows_hermes_uninstall_removes_extensionless_launcher() {
        let tmp = tempfile::tempdir().unwrap();
        let scripts_dir = tmp.path().join("Python313").join("Scripts");

        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::write(scripts_dir.join("hermes"), b"").unwrap();
        std::fs::write(scripts_dir.join("other.exe"), b"").unwrap();

        HermesPlugin.uninstall(&scripts_dir).unwrap();

        assert!(!scripts_dir.join("hermes").exists());
        assert!(scripts_dir.join("other.exe").exists());
        assert!(scripts_dir.exists());
    }

    #[test]
    fn native_windows_hermes_uninstall_accepts_python_root_with_scripts_launcher() {
        let tmp = tempfile::tempdir().unwrap();
        let python_root = tmp.path().join("Python313");
        let scripts_dir = python_root.join("Scripts");

        std::fs::create_dir_all(&scripts_dir).unwrap();
        std::fs::write(scripts_dir.join("hermes.exe"), b"").unwrap();
        std::fs::write(python_root.join("python.exe"), b"").unwrap();

        HermesPlugin.uninstall(&python_root).unwrap();

        assert!(!scripts_dir.join("hermes.exe").exists());
        assert!(python_root.join("python.exe").exists());
        assert!(python_root.exists());
    }
}
