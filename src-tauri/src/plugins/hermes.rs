use crate::plugin::{ToolInstallContext, ToolPlugin};
use crate::tool_types::{
    DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta, ToolUpdateSource,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct HermesPlugin;

#[cfg(target_os = "windows")]
fn hermes_process_is_running() -> bool {
    for name in &[
        "hermes-agent.exe",
        "Hermes Agent.exe",
        "Hermes Desktop.exe",
        "hermes-desktop.exe",
    ] {
        let result = std::process::Command::new("tasklist")
            .args(["/FI", &format!("IMAGENAME eq {name}"), "/NH"])
            .output();
        if let Ok(output) = result {
            if String::from_utf8_lossy(&output.stdout).contains(name) {
                return true;
            }
        }
    }
    false
}

fn hermes_desktop_download_url(version: &str, filename: &str) -> String {
    format!(
        "https://github.com/fathah/hermes-desktop/releases/download/v{}/{}",
        version, filename
    )
}

fn fetch_latest_hermes_version() -> Result<String, String> {
    let output = Command::new("curl")
        .args([
            "-s",
            "https://api.github.com/repos/fathah/hermes-desktop/releases/latest",
        ])
        .output()
        .map_err(|e| format!("获取 Hermes 最新版本失败: {e}"))?;
    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("\"tag_name\"") {
            if let Some(v) = line.split(':').nth(1) {
                let v = v.trim().trim_matches('"').trim_start_matches('v');
                return Ok(v.to_string());
            }
        }
    }
    Err("无法解析 Hermes 最新版本号".to_string())
}

#[cfg(target_os = "windows")]
fn hermes_desktop_asset_name(version: &str) -> String {
    format!("hermes-desktop-{}-setup.exe", version)
}

#[cfg(target_os = "macos")]
fn hermes_desktop_asset_name(version: &str) -> String {
    if cfg!(target_arch = "aarch64") {
        format!("hermes-desktop-{}-arm64-mac.zip", version)
    } else {
        format!("hermes-desktop-{}-x64-mac.zip", version)
    }
}

#[cfg(target_os = "linux")]
fn hermes_desktop_asset_name(version: &str) -> String {
    format!("hermes-desktop-{}.AppImage", version)
}

fn emit_progress(progress: &Sender<InstallProgress>, phase: &str, percent: u8, message: &str) {
    let _ = progress.blocking_send(InstallProgress {
        tool_id: "hermes".into(),
        tool_name: "Hermes Desktop".into(),
        phase: phase.into(),
        percent,
        message: message.into(),
    });
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
fn hermes_desktop_candidate_locations() -> Vec<PathBuf> {
    let mut bases = Vec::new();

    if let Some(local_app_data) = dirs::data_local_dir() {
        bases.push(local_app_data.join("Programs").join("hermes-desktop"));
        bases.push(local_app_data.join("Programs").join("hermes-agent"));
        bases.push(local_app_data.join("Programs").join("Hermes Agent"));
        bases.push(local_app_data.join("hermes-desktop"));
        bases.push(local_app_data.join("hermes-agent"));
        bases.push(local_app_data.join("Hermes Agent"));
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        let pf: PathBuf = program_files.clone().into();
        bases.push(pf.join("Hermes Desktop"));
        bases.push(pf.join("Hermes Agent"));
        bases.push(pf.join("hermes-desktop"));
        bases.push(pf.join("hermes-agent"));
    }

    bases
}

#[cfg(target_os = "windows")]
fn find_hermes_desktop_exe_in_dir(dir: &Path) -> Option<PathBuf> {
    for name in &[
        "hermes-agent.exe",
        "Hermes Agent.exe",
        "Hermes Desktop.exe",
        "hermes-desktop.exe",
    ] {
        let exe = dir.join(name);
        if exe.exists() {
            return Some(exe);
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn managed_hermes_install_dir(install_root: &Path) -> PathBuf {
    install_root.join("hermes")
}

#[cfg(target_os = "windows")]
fn detect_hermes_desktop_system_wide() -> Option<PathBuf> {
    for base in hermes_desktop_candidate_locations() {
        if find_hermes_desktop_exe_in_dir(&base).is_some() {
            return Some(base);
        }
    }
    None
}

#[cfg(all(target_os = "windows", test))]
fn hermes_windows_install_args(install_dir: &Path) -> [String; 2] {
    [
        "/S".to_string(),
        format!("/D={}", install_dir.to_string_lossy().replace('/', "\\")),
    ]
}

#[cfg(target_os = "windows")]
fn hermes_windows_installer_path(version: &str) -> PathBuf {
    crate::services::downloader::temp_path(&hermes_desktop_asset_name(version))
}

// ---------------------------------------------------------------------------
// ToolPlugin impl
// ---------------------------------------------------------------------------

impl ToolPlugin for HermesPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "hermes".into(),
            name: "Hermes Desktop".into(),
            description: "Hermes Agent 原生桌面管理应用".into(),
            icon: "hermes".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::DesktopInstaller
    }

    fn command_name(&self) -> Option<&'static str> {
        Some("hermes-desktop")
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        // 1) Check managed install root first.
        if let Some(root) = install_root {
            let managed_dir = managed_hermes_install_dir(root);
            if find_hermes_desktop_exe_in_dir(&managed_dir).is_some() {
                return DetectResult {
                    installed: true,
                    version: None,
                    install_path: Some(managed_dir.to_string_lossy().to_string()),
                };
            }

            if find_hermes_desktop_exe_in_dir(root).is_some() {
                return DetectResult {
                    installed: true,
                    version: None,
                    install_path: Some(root.to_string_lossy().to_string()),
                };
            }
        }

        // 2) Check system-wide install locations.
        #[cfg(target_os = "windows")]
        if let Some(install_path) = detect_hermes_desktop_system_wide() {
            return DetectResult {
                installed: true,
                version: None,
                install_path: Some(install_path.to_string_lossy().to_string()),
            };
        }

        // 3) Check Windows registry uninstall entries.
        #[cfg(target_os = "windows")]
        {
            use crate::services::installer::windows::find_uninstall_entry_ex;
            if let Some(entry) = find_uninstall_entry_ex(
                &[
                    "Hermes Agent",
                    "Hermes Desktop",
                    "hermes-agent",
                    "hermes-desktop",
                ],
                &[],
            ) {
                let install_path = entry.install_location.or_else(|| {
                    entry
                        .display_icon
                        .as_ref()
                        .and_then(|path| path.parent().map(PathBuf::from))
                });
                return DetectResult {
                    installed: true,
                    version: entry.display_version,
                    install_path: install_path.map(|p| p.to_string_lossy().to_string()),
                };
            }
        }

        DetectResult::not_installed()
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![]
    }

    fn update_source(&self) -> Option<ToolUpdateSource> {
        Some(ToolUpdateSource {
            kind: "github".into(),
            id: "fathah/hermes-desktop".into(),
        })
    }

    // -----------------------------------------------------------------------
    // Windows install
    // -----------------------------------------------------------------------
    #[cfg(target_os = "windows")]
    fn install(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        install_hermes_desktop_windows(target_dir, install_root, &progress, None)
    }

    #[cfg(target_os = "windows")]
    fn install_with_context(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
        context: ToolInstallContext,
    ) -> Result<(), String> {
        install_hermes_desktop_windows(target_dir, install_root, &progress, Some(&context))
    }

    #[cfg(target_os = "windows")]
    fn update_with_context(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
        context: ToolInstallContext,
    ) -> Result<(), String> {
        // 检查 Hermes 是否在运行，避免文件锁定导致更新失败
        if hermes_process_is_running() {
            return Err("Hermes Desktop 正在运行，请先关闭应用再更新".into());
        }
        install_hermes_desktop_windows(target_dir, install_root, &progress, Some(&context))
    }

    // -----------------------------------------------------------------------
    // macOS install
    // -----------------------------------------------------------------------
    #[cfg(target_os = "macos")]
    fn install(
        &self,
        target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        install_hermes_desktop_macos(target_dir, &progress)
    }

    // -----------------------------------------------------------------------
    // Linux install
    // -----------------------------------------------------------------------
    #[cfg(target_os = "linux")]
    fn install(
        &self,
        target_dir: &Path,
        _install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        install_hermes_desktop_linux(target_dir, &progress)
    }

    // -----------------------------------------------------------------------
    // Unsupported platform fallback
    // -----------------------------------------------------------------------
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("Hermes Desktop auto-install is not supported on this platform".into())
    }

    // -----------------------------------------------------------------------
    // Uninstall
    // -----------------------------------------------------------------------
    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            use crate::services::installer::windows::{
                find_local_uninstaller_executable, find_uninstall_entry_ex,
                run_windows_uninstaller_with_common_args,
            };

            // Try registry uninstall string first.
            if let Some(entry) = find_uninstall_entry_ex(
                &[
                    "Hermes Agent",
                    "Hermes Desktop",
                    "hermes-agent",
                    "hermes-desktop",
                ],
                &[],
            ) {
                if let Some(uninstall_string) = entry.uninstall_string {
                    let status = Command::new("cmd")
                        .args(["/C", &uninstall_string])
                        .spawn()
                        .map_err(|e| format!("failed to launch uninstall: {e}"))?
                        .wait()
                        .map_err(|e| format!("uninstall failed: {e}"))?;
                    if status.success() {
                        return Ok(());
                    }
                }
            }

            // Try running uninstaller from the install directory.
            if let Some(uninstaller) = find_local_uninstaller_executable(target_dir) {
                run_windows_uninstaller_with_common_args(&uninstaller)?;
                return Ok(());
            }

            // Fallback: just remove the portable exe if present.
            if let Some(exe) = find_hermes_desktop_exe_in_dir(target_dir) {
                std::fs::remove_file(&exe)
                    .map_err(|e| format!("failed to remove {}: {e}", exe.display()))?;
                return Ok(());
            }

            return Err("Could not find Hermes Desktop uninstaller".into());
        }

        #[cfg(not(target_os = "windows"))]
        {
            let _ = target_dir;
            Err("Hermes Desktop uninstall is not yet supported on this platform".into())
        }
    }
}

// ============================================================================
// Platform install helpers
// ============================================================================

#[cfg(target_os = "windows")]
fn install_hermes_desktop_windows(
    target_dir: &Path,
    _install_root: &Path,
    progress: &Sender<InstallProgress>,
    context: Option<&ToolInstallContext>,
) -> Result<(), String> {
    let version = fetch_latest_hermes_version()?;
    let asset_name = hermes_desktop_asset_name(&version);
    let download_url = hermes_desktop_download_url(&version, &asset_name);
    let installer_path = hermes_windows_installer_path(&version);

    // --- Download ---
    emit_progress(
        progress,
        "downloading",
        10,
        "Downloading Hermes Desktop installer...",
    );
    if let Some(ctx) = context {
        ctx.install_log()
            .emit_phase("downloading", "Downloading Hermes Desktop installer");
        ctx.install_log().emit_output(
            "downloading",
            crate::tool_types::InstallLogLevel::Info,
            format!("Fetching {download_url}"),
        );
    }

    let rt =
        tokio::runtime::Runtime::new().map_err(|e| format!("failed to create runtime: {e}"))?;
    rt.block_on(async {
        crate::services::downloader::download_file(&download_url, &installer_path, None).await
    })?;

    // --- Install ---
    emit_progress(
        progress,
        "installing",
        50,
        "Running Hermes Desktop installer...",
    );
    if let Some(ctx) = context {
        ctx.install_log()
            .emit_phase("installing", "Running Hermes Desktop installer");
    }

    // NSIS silent install: /S for silent, /D=<path> for target directory
    // (must be the LAST argument).
    let install_dir = target_dir.to_string_lossy().replace('/', "\\");
    let status = Command::new(&installer_path)
        .args(["/S", &format!("/D={install_dir}")])
        .spawn()
        .map_err(|e| format!("failed to launch Hermes Desktop installer: {e}"))?
        .wait()
        .map_err(|e| format!("Hermes Desktop installer failed: {e}"))?;

    if !status.success() {
        // Clean up installer on failure.
        std::fs::remove_file(&installer_path).ok();
        return Err(format!(
            "Hermes Desktop installer exited with code {:?}",
            status.code()
        ));
    }

    // --- Cleanup ---
    std::fs::remove_file(&installer_path).ok();

    emit_progress(
        progress,
        "complete",
        100,
        "Hermes Desktop installation complete",
    );
    Ok(())
}

#[cfg(target_os = "macos")]
fn install_hermes_desktop_macos(
    target_dir: &Path,
    progress: &Sender<InstallProgress>,
) -> Result<(), String> {
    let version = fetch_latest_hermes_version()?;
    let asset_name = hermes_desktop_asset_name(&version);
    let download_url = hermes_desktop_download_url(&version, &asset_name);
    let archive_path = target_dir.join(&asset_name);

    emit_progress(progress, "downloading", 10, "Downloading Hermes Desktop...");

    let rt =
        tokio::runtime::Runtime::new().map_err(|e| format!("failed to create runtime: {e}"))?;
    rt.block_on(async {
        crate::services::downloader::download_file(&download_url, &archive_path, None).await
    })?;

    emit_progress(progress, "extracting", 50, "Extracting Hermes Desktop...");

    crate::services::downloader::extract_zip(&archive_path, target_dir)?;
    std::fs::remove_file(&archive_path).ok();

    emit_progress(
        progress,
        "complete",
        100,
        "Hermes Desktop ready — move Hermes Desktop.app to /Applications to finish",
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn install_hermes_desktop_linux(
    target_dir: &Path,
    progress: &Sender<InstallProgress>,
) -> Result<(), String> {
    let version = fetch_latest_hermes_version()?;
    let asset_name = hermes_desktop_asset_name(&version);
    let download_url = hermes_desktop_download_url(&version, &asset_name);
    let appimage_path = target_dir.join("hermes-desktop.AppImage");

    emit_progress(
        progress,
        "downloading",
        10,
        "Downloading Hermes Desktop AppImage...",
    );

    let rt =
        tokio::runtime::Runtime::new().map_err(|e| format!("failed to create runtime: {e}"))?;
    rt.block_on(async {
        crate::services::downloader::download_file(&download_url, &appimage_path, None).await
    })?;

    // Make AppImage executable.
    use std::os::unix::fs::PermissionsExt;
    let mut perms = std::fs::metadata(&appimage_path)
        .map_err(|e| format!("failed to read AppImage metadata: {e}"))?
        .permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&appimage_path, perms)
        .map_err(|e| format!("failed to make AppImage executable: {e}"))?;

    emit_progress(progress, "complete", 100, "Hermes Desktop AppImage ready");
    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hermes_desktop_plugin_uses_desktop_installer_strategy() {
        assert_eq!(
            HermesPlugin.install_strategy(),
            InstallStrategy::DesktopInstaller
        );
    }

    #[test]
    fn hermes_desktop_metadata_id_is_hermes() {
        assert_eq!(HermesPlugin.metadata().id, "hermes");
    }

    #[test]
    fn hermes_desktop_metadata_name_is_desktop() {
        assert_eq!(HermesPlugin.metadata().name, "Hermes Desktop");
    }

    #[test]
    fn hermes_desktop_download_url_contains_version() {
        let url = hermes_desktop_download_url("1.2.3", "test.exe");
        assert!(url.contains("1.2.3"));
        assert!(url.starts_with("https://github.com/fathah/hermes-desktop/releases/download/"));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn hermes_desktop_candidate_locations_includes_local_appdata_programs() {
        let bases = hermes_desktop_candidate_locations();
        // At least one candidate should include "hermes-desktop".
        assert!(bases
            .iter()
            .any(|b| b.to_string_lossy().contains("hermes-desktop")));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn find_hermes_desktop_exe_detects_real_file() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = tmp.path().join("hermes-agent.exe");
        std::fs::write(&exe, b"").unwrap();

        let found = find_hermes_desktop_exe_in_dir(tmp.path());
        assert_eq!(found, Some(exe));
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn find_hermes_desktop_exe_prefers_first_match() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("hermes-agent.exe"), b"").unwrap();
        std::fs::write(tmp.path().join("Hermes Desktop.exe"), b"").unwrap();
        std::fs::write(tmp.path().join("hermes-desktop.exe"), b"").unwrap();

        let found = find_hermes_desktop_exe_in_dir(tmp.path());
        assert_eq!(
            found.unwrap().file_name().unwrap().to_str().unwrap(),
            "hermes-agent.exe"
        );
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn detect_checks_managed_tool_subdirectory_first() {
        let tmp = tempfile::tempdir().unwrap();
        let managed_dir = tmp.path().join("hermes");
        std::fs::create_dir_all(&managed_dir).unwrap();
        std::fs::write(managed_dir.join("hermes-agent.exe"), b"").unwrap();

        let detect = HermesPlugin.detect(Some(tmp.path()));

        assert!(detect.installed);
        assert_eq!(
            detect.install_path.as_deref(),
            Some(managed_dir.to_string_lossy().as_ref())
        );
        assert_eq!(detect.version, None);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn hermes_windows_installer_targets_tool_directory() {
        let install_root = Path::new("D:\\AgenticTools");
        let target_dir = install_root.join("hermes");
        let args = hermes_windows_install_args(&target_dir);

        assert_eq!(args[0], "/S");
        assert_eq!(args[1], "/D=D:\\AgenticTools\\hermes");
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn hermes_windows_installer_downloads_to_temp_directory() {
        let version = "0.5.1";
        let installer_path = hermes_windows_installer_path(version);

        assert_eq!(
            installer_path.file_name().and_then(|name| name.to_str()),
            Some("hermes-desktop-0.5.1-setup.exe")
        );
        assert!(
            !installer_path
                .to_string_lossy()
                .contains("\\AgenticTools\\hermes\\"),
            "installer should not live inside the managed install dir: {}",
            installer_path.display()
        );
        assert!(
            installer_path.starts_with(std::env::temp_dir()),
            "installer should be downloaded under the temp dir: {}",
            installer_path.display()
        );
    }
}
