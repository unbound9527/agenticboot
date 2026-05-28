use crate::plugin::{ToolInstallContext, ToolPlugin};
use crate::services::installer::windows::{
    find_appx_install_location, find_executable_in_dir, find_local_uninstaller_executable,
    find_uninstall_entry_ex, read_command_version, run_windows_uninstaller_with_common_args,
    run_winget, winget_exists,
};
use crate::tool_types::{DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta};
use log::debug;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct CodexDesktopPlugin;

impl ToolPlugin for CodexDesktopPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "codex-desktop".into(),
            name: "Codex (Desktop)".into(),
            description: "OpenAI Codex 官方 Windows 桌面应用".into(),
            icon: "codex".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::DesktopInstaller
    }

    fn command_name(&self) -> Option<&'static str> {
        Some("codex")
    }

    fn supports_pathless_uninstall(&self) -> bool {
        true
    }

    fn detect(&self, _install_root: Option<&Path>) -> DetectResult {
        if let Some((install_path, version)) = detect_local_codex_installation() {
            debug!(
                "detected Codex via local installation: version={:?}, path={:?}",
                version, install_path
            );
            return DetectResult {
                installed: true,
                version,
                install_path: Some(install_path.to_string_lossy().to_string()),
            };
        }

        if let Some(entry) = find_uninstall_entry_ex(&["Codex", "OpenAI Codex"], &["CLI", "npm"]) {
            let install_path = entry.install_location.or(entry
                .display_icon
                .and_then(|path| path.parent().map(|p| p.to_path_buf())));

            // If registry has version, use it directly
            let version = entry.display_version.or_else(|| {
                // Try to get version from executable in install location
                install_path.as_ref().and_then(|dir| {
                    find_executable_in_dir(dir, &["Codex.exe", "codex.exe"])
                        .and_then(|exe| read_command_version(&exe, &["--version"]))
                })
            });

            debug!(
                "detected Codex via registry: version={:?}, path={:?}",
                version, install_path
            );
            return DetectResult {
                installed: true,
                version,
                install_path: install_path.map(|dir| dir.to_string_lossy().to_string()),
            };
        }

        if let Some((location, version)) = find_appx_install_location("OpenAI.Codex") {
            debug!(
                "detected Codex via AppX: version={:?}, path={:?}",
                version, location
            );
            return DetectResult {
                installed: true,
                version,
                install_path: Some(location),
            };
        }

        debug!("Codex desktop not found");
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
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "codex-desktop".into(),
            tool_name: "Codex (Desktop)".into(),
            phase: "installing".into(),
            percent: 0,
            message: "Installing Codex desktop app via Microsoft Store...".into(),
        });

        if !winget_exists() {
            return Err("Codex desktop install requires winget / App Installer".into());
        }

        run_winget(&[
            "install",
            "Codex",
            "-s",
            "msstore",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ])
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("Codex desktop auto-install is currently supported only on Windows".into())
    }

    #[cfg(target_os = "windows")]
    fn update_with_context(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
        _context: ToolInstallContext,
    ) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "codex-desktop".into(),
            tool_name: "Codex (Desktop)".into(),
            phase: "installing".into(),
            percent: 0,
            message: "Updating Codex desktop app via Microsoft Store...".into(),
        });

        if !winget_exists() {
            return Err("Codex desktop update requires winget / App Installer".into());
        }

        run_winget(&[
            "upgrade",
            "Codex",
            "-s",
            "msstore",
            "--accept-package-agreements",
            "--accept-source-agreements",
        ])
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            if winget_exists()
                && run_winget(&[
                    "uninstall",
                    "Codex",
                    "-s",
                    "msstore",
                    "--accept-package-agreements",
                    "--accept-source-agreements",
                ])
                .is_ok()
            {
                return Ok(());
            }

            let status = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    "Get-AppxPackage OpenAI.Codex | Remove-AppxPackage",
                ])
                .spawn()
                .map_err(|e| format!("failed to launch Codex uninstall: {e}"))?
                .wait()
                .map_err(|e| format!("failed to wait for Codex uninstall: {e}"))?;
            if status.success() {
                return Ok(());
            }

            if let Some(entry) =
                find_uninstall_entry_ex(&["Codex", "OpenAI Codex"], &["CLI", "npm"])
            {
                if let Some(uninstall_string) = entry.uninstall_string {
                    let status = Command::new("cmd")
                        .args(["/C", &uninstall_string])
                        .spawn()
                        .map_err(|e| format!("failed to launch Codex uninstall command: {e}"))?
                        .wait()
                        .map_err(|e| format!("failed to wait for Codex uninstall command: {e}"))?;
                    if status.success() {
                        return Ok(());
                    }
                }
            }

            if let Some(uninstaller) = find_local_uninstaller_executable(target_dir) {
                run_windows_uninstaller_with_common_args(&uninstaller)?;
                return Ok(());
            }

            return Err("No Codex desktop uninstall command was found".into());
        }

        #[allow(unreachable_code)]
        Ok(())
    }
}

fn local_codex_candidate_bases() -> Vec<PathBuf> {
    let mut candidate_bases = Vec::new();

    if let Some(local_app_data) = dirs::data_local_dir() {
        candidate_bases.push(local_app_data);
    }

    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        candidate_bases.push(PathBuf::from(program_files));
    }

    if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
        candidate_bases.push(PathBuf::from(program_files_x86));
    }

    candidate_bases
}

fn detect_local_codex_installation() -> Option<(PathBuf, Option<String>)> {
    let candidate_bases = local_codex_candidate_bases();
    let candidate_roots = ["OpenAI\\Codex", "Programs\\Codex", "Codex"];

    for base in candidate_bases {
        if let Some(found) = candidate_roots
            .iter()
            .map(|suffix| base.join(suffix))
            .find_map(|candidate| find_codex_install_root(&candidate))
        {
            return Some(found);
        }

        if let Some(found) = find_codex_install_root(&base) {
            return Some(found);
        }
    }

    None
}

fn find_codex_install_root(candidate_root: &Path) -> Option<(PathBuf, Option<String>)> {
    // Try bin directory first (common for installed apps)
    let bin_dir = candidate_root.join("bin");
    if let Some(exe) = find_executable_in_dir(
        &bin_dir,
        &["Codex.exe", "codex.exe", "Codex.cmd", "codex.cmd"],
    ) {
        let version = read_command_version(&exe, &["--version"]);
        return Some((candidate_root.to_path_buf(), version));
    }

    // Try root directory
    if let Some(exe) = find_executable_in_dir(
        candidate_root,
        &["Codex.exe", "codex.exe", "Codex.cmd", "codex.cmd"],
    ) {
        let version = read_command_version(&exe, &["--version"]);
        return Some((candidate_root.to_path_buf(), version));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn find_codex_install_root_accepts_local_install_layout() {
        let temp = tempfile::tempdir().unwrap();
        let install_root = temp.path().join("OpenAI").join("Codex");
        let bin_dir = install_root.join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("codex.exe"), b"").unwrap();

        let result = find_codex_install_root(&install_root);
        let expected_install_path = install_root.to_string_lossy().to_string();

        assert_eq!(
            result.map(|(path, _)| path.to_string_lossy().to_string()),
            Some(expected_install_path)
        );
    }

    #[test]
    fn detect_does_not_treat_managed_install_root_as_desktop_install_location() {
        let temp = tempfile::tempdir().unwrap();
        let candidate_bases = local_codex_candidate_bases();

        assert!(!candidate_bases.iter().any(|path| path == temp.path()));
    }
}
