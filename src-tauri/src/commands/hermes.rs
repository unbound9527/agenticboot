use std::process::Command;

use crate::hermes_config;
use crate::store::AppState;

/// Error string returned when the Hermes Desktop executable cannot be found
/// or launched. Kept in sync with the `HERMES_DESKTOP_NOT_FOUND_ERROR` constant
/// in `src/hooks/useHermes.ts` so the frontend can branch on it.
const HERMES_DESKTOP_NOT_FOUND_ERROR: &str = "hermes_desktop_not_found";
const HERMES_OFFICIAL_DIR_NAMES: &[&str] = &["Hermes Agent", "Hermes Desktop"];
const HERMES_OFFICIAL_EXE_NAMES: &[&str] = &["Hermes Agent.exe", "Hermes Desktop.exe"];

// ============================================================================
// Hermes Provider Commands
// ============================================================================

/// Import providers from Hermes live config to database.
///
/// Hermes uses additive mode — users may already have providers
/// configured in config.yaml.
#[tauri::command]
pub fn import_hermes_providers_from_live(
    state: tauri::State<'_, AppState>,
) -> Result<usize, String> {
    crate::services::provider::import_hermes_providers_from_live(state.inner())
        .map_err(|e| e.to_string())
}

/// Get provider names in the Hermes live config.
#[tauri::command]
pub fn get_hermes_live_provider_ids() -> Result<Vec<String>, String> {
    hermes_config::get_providers()
        .map(|providers| providers.keys().cloned().collect())
        .map_err(|e| e.to_string())
}

/// Get a single Hermes provider fragment from live config.
#[tauri::command]
pub fn get_hermes_live_provider(
    #[allow(non_snake_case)] providerId: String,
) -> Result<Option<serde_json::Value>, String> {
    hermes_config::get_provider(&providerId).map_err(|e| e.to_string())
}

// ============================================================================
// Model Configuration Commands
// ============================================================================

/// Get Hermes model config (model section of config.yaml). Read-only — writes
/// happen implicitly through `apply_switch_defaults` when switching providers.
#[tauri::command]
pub fn get_hermes_model_config() -> Result<Option<hermes_config::HermesModelConfig>, String> {
    hermes_config::get_model_config().map_err(|e| e.to_string())
}

// ============================================================================
// Memory Files Commands
// ============================================================================

#[tauri::command]
pub fn get_hermes_memory(kind: hermes_config::MemoryKind) -> Result<String, String> {
    hermes_config::read_memory(kind).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_hermes_memory(kind: hermes_config::MemoryKind, content: String) -> Result<(), String> {
    hermes_config::write_memory(kind, &content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_hermes_memory_limits() -> Result<hermes_config::HermesMemoryLimits, String> {
    hermes_config::read_memory_limits().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_hermes_memory_enabled(
    kind: hermes_config::MemoryKind,
    enabled: bool,
) -> Result<hermes_config::HermesWriteOutcome, String> {
    hermes_config::set_memory_enabled(kind, enabled).map_err(|e| e.to_string())
}

// ============================================================================
// Hermes Desktop launcher
// ============================================================================

/// Find the Hermes Desktop executable and launch it.
///
/// Search order:
///   1. `HERMES_DESKTOP_PATH` environment variable
///   2. Standard OS-specific install locations
///   3. Managed install root (e.g. `D:\AgenticTools\hermes`)
///
/// On Windows the app is launched detached so CC Switch stays responsive.
#[tauri::command]
pub async fn open_hermes_desktop(
    _path: Option<String>,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // 1) Environment variable override.
    if let Ok(custom_path) = std::env::var("HERMES_DESKTOP_PATH") {
        let exe = std::path::PathBuf::from(&custom_path);
        if exe.exists() {
            return launch_hermes_desktop_executable(&exe);
        }
        return Err(format!(
            "HERMES_DESKTOP_PATH is set but the file does not exist: {custom_path}"
        ));
    }

    // 2) Standard OS-specific locations.
    #[cfg(target_os = "windows")]
    {
        if let Some(exe) = find_hermes_desktop_system_wide() {
            return launch_hermes_desktop_executable(&exe);
        }
    }

    #[cfg(target_os = "macos")]
    {
        let app_path = std::path::PathBuf::from("/Applications/Hermes Desktop.app");
        if app_path.exists() {
            return launch_hermes_desktop_executable(&app_path);
        }
    }

    // 3) Managed install root (where the software manager installs tools).
    if let Ok(Some(install_root)) = state.db.get_install_root() {
        let managed_dir = std::path::PathBuf::from(&install_root).join("hermes");
        if let Some(exe) = find_hermes_desktop_exe_in_dir(&managed_dir) {
            return launch_hermes_desktop_executable(&exe);
        }
        // Also check the root directly (user may have pointed it at the exe dir).
        let root = std::path::PathBuf::from(&install_root);
        if root != managed_dir {
            if let Some(exe) = find_hermes_desktop_exe_in_dir(&root) {
                return launch_hermes_desktop_executable(&exe);
            }
        }
    }

    Err(HERMES_DESKTOP_NOT_FOUND_ERROR.to_string())
}

/// Try to find the Hermes Desktop executable in standard system locations.
#[cfg(target_os = "windows")]
fn find_hermes_desktop_system_wide_candidates() -> Vec<std::path::PathBuf> {
    let mut bases = Vec::new();

    if let Some(local_app_data) = dirs::data_local_dir() {
        for dir_name in HERMES_OFFICIAL_DIR_NAMES {
            bases.push(local_app_data.join("Programs").join(dir_name));
            bases.push(local_app_data.join(dir_name));
        }
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        let pf: std::path::PathBuf = program_files.clone().into();
        for dir_name in HERMES_OFFICIAL_DIR_NAMES {
            bases.push(pf.join(dir_name));
        }
    }

    bases
}

/// Try to find the Hermes Desktop executable in standard system locations.
#[cfg(target_os = "windows")]
fn find_hermes_desktop_system_wide() -> Option<std::path::PathBuf> {
    for base in &find_hermes_desktop_system_wide_candidates() {
        if let Some(exe) = find_hermes_desktop_exe_in_dir(base) {
            return Some(exe);
        }
    }
    None
}

/// Check a directory for the Hermes Desktop executable.
fn find_hermes_desktop_exe_in_dir(dir: &std::path::Path) -> Option<std::path::PathBuf> {
    for name in HERMES_OFFICIAL_EXE_NAMES {
        let exe = dir.join(name);
        if exe.exists() {
            return Some(exe);
        }
    }
    None
}

/// Launch the Hermes Desktop executable detached from CC Switch.
#[cfg(target_os = "windows")]
fn launch_hermes_desktop_executable(exe: &std::path::Path) -> Result<(), String> {
    let mut cmd = Command::new(exe);
    crate::services::command_util::hide_console(&mut cmd);
    cmd.spawn()
        .map_err(|e| format!("failed to launch Hermes Desktop: {e}"))?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn launch_hermes_desktop_executable(exe: &std::path::Path) -> Result<(), String> {
    Command::new(exe)
        .spawn()
        .map_err(|e| format!("failed to launch Hermes Desktop: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn system_wide_candidate_locations_use_official_dir_names_only() {
        let bases = find_hermes_desktop_system_wide_candidates();
        assert!(bases
            .iter()
            .all(|base| !base.to_string_lossy().contains("hermes-desktop")));
        assert!(bases
            .iter()
            .all(|base| !base.to_string_lossy().contains("hermes-agent")));
    }

    #[test]
    fn launcher_exe_detection_ignores_legacy_lowercase_names() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("hermes-agent.exe"), b"").unwrap();
        std::fs::write(tmp.path().join("hermes-desktop.exe"), b"").unwrap();

        let found = find_hermes_desktop_exe_in_dir(tmp.path());
        assert_eq!(found, None);
    }

    #[test]
    fn launcher_exe_detection_accepts_official_names() {
        let tmp = tempfile::tempdir().unwrap();
        let exe = tmp.path().join("Hermes Desktop.exe");
        std::fs::write(&exe, b"").unwrap();

        let found = find_hermes_desktop_exe_in_dir(tmp.path());
        assert_eq!(found, Some(exe));
    }
}
