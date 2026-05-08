use std::process::Command;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowsUninstallEntry {
    pub display_name: String,
    pub install_location: Option<PathBuf>,
    pub display_icon: Option<PathBuf>,
    pub uninstall_string: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowsDetectPaths {
    pub executable: Option<PathBuf>,
    pub install_root: Option<PathBuf>,
}

pub fn find_executable_in_dir(base: &Path, candidates: &[&str]) -> Option<PathBuf> {
    candidates
        .iter()
        .map(|candidate| base.join(candidate))
        .find(|path| path.exists())
}

pub fn find_managed_paths(root: &Path, tool_dir: &str, candidates: &[&str]) -> WindowsDetectPaths {
    let install_root = root.join(tool_dir);

    WindowsDetectPaths {
        executable: find_executable_in_dir(&install_root, candidates),
        install_root: install_root.exists().then_some(install_root),
    }
}

pub fn find_managed_executable(root: &Path, tool_dir: &str, candidates: &[&str]) -> Option<PathBuf> {
    find_managed_paths(root, tool_dir, candidates).executable
}

pub fn npm_prefix_candidates(cmd_name: &str) -> Vec<String> {
    vec![
        format!("{cmd_name}.cmd"),
        format!("{cmd_name}.exe"),
        format!("bin\\{cmd_name}.cmd"),
        format!("bin\\{cmd_name}.exe"),
    ]
}

pub fn find_command_on_path(command: &str) -> Option<PathBuf> {
    let output = Command::new("where").arg(command).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(PathBuf::from)
}

pub fn read_command_version(command: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .chain(String::from_utf8_lossy(&output.stderr).lines())
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

pub fn winget_exists() -> bool {
    find_command_on_path("winget").is_some()
}

pub fn run_winget(args: &[&str]) -> Result<(), String> {
    let output = Command::new("winget")
        .args(args)
        .output()
        .map_err(|e| format!("启动 winget 失败: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "winget 执行失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub fn find_local_program_executable(dir_names: &[&str], exe_names: &[&str]) -> Option<PathBuf> {
    let mut bases = Vec::new();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let local = PathBuf::from(local);
        bases.push(local.join("Programs"));
        bases.push(local);
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        bases.push(PathBuf::from(program_files));
    }
    if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
        bases.push(PathBuf::from(program_files_x86));
    }

    for base in bases {
        for dir_name in dir_names {
            for exe_name in exe_names {
                let candidate = base.join(dir_name).join(exe_name);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

pub fn find_appx_install_location(package_name: &str) -> Option<String> {
    let script = format!(
        "$pkg = Get-AppxPackage {package_name} | Select-Object -First 1; if ($pkg) {{ $pkg.InstallLocation }}"
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let location = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!location.is_empty()).then_some(location)
}

pub fn find_uninstall_entry(name_fragments: &[&str]) -> Option<WindowsUninstallEntry> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
        use winreg::RegKey;

        let roots = [
            (
                RegKey::predef(HKEY_CURRENT_USER),
                "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            ),
            (
                RegKey::predef(HKEY_LOCAL_MACHINE),
                "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            ),
            (
                RegKey::predef(HKEY_LOCAL_MACHINE),
                "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
            ),
        ];

        let fragments = name_fragments
            .iter()
            .map(|name| name.to_ascii_lowercase())
            .collect::<Vec<_>>();

        for (root, path) in roots {
            let Ok(uninstall_key) = root.open_subkey(path) else {
                continue;
            };

            for subkey_name in uninstall_key.enum_keys().filter_map(Result::ok) {
                let Ok(app_key) = uninstall_key.open_subkey(&subkey_name) else {
                    continue;
                };

                let display_name: String = match app_key.get_value("DisplayName") {
                    Ok(value) => value,
                    Err(_) => continue,
                };
                let display_name_lower = display_name.to_ascii_lowercase();
                if !fragments
                    .iter()
                    .any(|fragment| display_name_lower.contains(fragment))
                {
                    continue;
                }

                return Some(WindowsUninstallEntry {
                    display_name,
                    install_location: app_key
                        .get_value::<String, _>("InstallLocation")
                        .ok()
                        .map(PathBuf::from),
                    display_icon: app_key
                        .get_value::<String, _>("DisplayIcon")
                        .ok()
                        .and_then(|raw| sanitize_windows_path_field(&raw)),
                    uninstall_string: app_key.get_value("UninstallString").ok(),
                });
            }
        }
    }

    None
}

fn sanitize_windows_path_field(raw: &str) -> Option<PathBuf> {
    let trimmed = raw.trim().trim_matches('"');
    let cleaned = trimmed.split(',').next()?.trim();
    if cleaned.is_empty() {
        return None;
    }

    Some(PathBuf::from(cleaned))
}

pub fn normalize_windows_exe(path: &Path) -> String {
    path.to_string_lossy().replace('/', "\\")
}

#[cfg(test)]
mod tests {
    use super::read_command_version;

    #[test]
    fn read_command_version_trims_first_non_empty_output_line() {
        let tmp = tempfile::tempdir().unwrap();
        let script = tmp.path().join("version.cmd");
        std::fs::write(
            &script,
            "@echo off\r\n\recho.\r\necho opencode 1.2.3\r\necho trailing\r\n",
        )
        .unwrap();

        let version_output = read_command_version(&script, &[]);
        let version = version_output.as_deref().map(str::trim);
        assert_eq!(version, Some("opencode 1.2.3"));
    }
}
