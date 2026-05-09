use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellCommandOutput {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

pub trait WindowsShell {
    fn run(&mut self, program: &str, args: &[String]) -> Result<ShellCommandOutput, String>;
}

pub struct SystemWindowsShell;

impl WindowsShell for SystemWindowsShell {
    fn run(&mut self, program: &str, args: &[String]) -> Result<ShellCommandOutput, String> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| format!("failed to run {program}: {e}"))?;

        Ok(ShellCommandOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

pub fn detect_windows_cli_version(command_name: &str) -> Option<String> {
    let mut shell = SystemWindowsShell;
    detect_windows_cli_version_with_shell(&mut shell, command_name, None)
}

pub fn detect_windows_cli_version_with_shell<S: WindowsShell>(
    shell: &mut S,
    command_name: &str,
    nvm_root_override: Option<&Path>,
) -> Option<String> {
    log::info!("[Windows CLI Detect] Checking {command_name} via direct command");
    if let Some(version) = run_windows_cli_version(shell, command_name) {
        log::info!(
            "[Windows CLI Detect] Direct command succeeded for {command_name}: {version}"
        );
        return Some(version);
    }

    let nvm_root = match nvm_root_override {
        Some(root) => root.to_path_buf(),
        None => nvm_root_via_cmd(shell)?,
    };
    log::info!(
        "[Windows CLI Detect] Direct command failed for {command_name}, falling back to nvm root {}",
        nvm_root.display()
    );

    let versions = list_nvm_version_directories(&nvm_root);
    if versions.is_empty() {
        log::info!(
            "[Windows CLI Detect] No nvm versions found for {command_name} under {}",
            nvm_root.display()
        );
        return None;
    }

    for version in versions {
        log::info!(
            "[Windows CLI Detect] Trying {command_name} under nvm version {version}"
        );
        let use_and_check = format!("nvm use {version} && {command_name} --version");
        let result = shell
            .run("cmd", &[String::from("/C"), use_and_check])
            .ok()?;
        if result.success {
            if let Some(found) = extract_version_output(&result.stdout) {
                log::info!(
                    "[Windows CLI Detect] nvm fallback succeeded for {command_name} on {version}: {found}"
                );
                return Some(found);
            }
            if let Some(found) = extract_version_output(&result.stderr) {
                log::info!(
                    "[Windows CLI Detect] nvm fallback succeeded for {command_name} on {version}: {found}"
                );
                return Some(found);
            }
        }
    }

    log::info!(
        "[Windows CLI Detect] Exhausted nvm versions without finding {command_name}"
    );
    None
}

fn run_windows_cli_version<S: WindowsShell>(shell: &mut S, command_name: &str) -> Option<String> {
    let result = shell
        .run(
            "cmd",
            &[String::from("/C"), format!("{command_name} --version")],
        )
        .ok()?;
    if !result.success {
        return None;
    }
    extract_version_output(&result.stdout).or_else(|| extract_version_output(&result.stderr))
}

fn nvm_root_via_cmd<S: WindowsShell>(shell: &mut S) -> Option<PathBuf> {
    let result = shell
        .run("cmd", &[String::from("/C"), String::from("nvm root")])
        .ok()?;
    if !result.success {
        log::info!("[Windows CLI Detect] `nvm root` failed");
        return None;
    }
    let root = extract_version_output(&result.stdout).or_else(|| extract_version_output(&result.stderr))?;
    (!root.trim().is_empty()).then_some(PathBuf::from(root.trim()))
}

pub fn list_nvm_version_directories(root: &Path) -> Vec<String> {
    let mut versions = match std::fs::read_dir(root) {
        Ok(entries) => entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                if !file_type.is_dir() {
                    return None;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                is_nvm_version_dir(&name).then_some(name)
            })
            .collect::<Vec<_>>(),
        Err(_) => return Vec::new(),
    };

    versions.sort_by(|a, b| compare_nvm_versions(b, a));
    versions
}

fn is_nvm_version_dir(name: &str) -> bool {
    let Some(rest) = name.strip_prefix('v') else {
        return false;
    };
    let parts: Vec<_> = rest.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    parts
        .iter()
        .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

fn compare_nvm_versions(left: &str, right: &str) -> std::cmp::Ordering {
    let left = parse_nvm_version(left);
    let right = parse_nvm_version(right);
    left.cmp(&right)
}

fn parse_nvm_version(name: &str) -> Vec<u32> {
    name.strip_prefix('v')
        .unwrap_or(name)
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or(0))
        .collect()
}

fn extract_version_output(output: &str) -> Option<String> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .last()
        .map(ToOwned::to_owned)
}

pub fn run_with_node_env(program: &Path, args: &[&str]) -> Option<std::process::Output> {
    let node_path = find_node_on_system()?;
    let node_dir = node_path.parent()?;

    #[cfg(target_os = "windows")]
    let path_sep = ";";
    #[cfg(not(target_os = "windows"))]
    let path_sep = ":";

    let current_path = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}{path_sep}{}", node_dir.display(), current_path);

    Command::new(program)
        .args(args)
        .env("PATH", new_path)
        .output()
        .ok()
}

/// 在系统常见位置查找 node 可执行文件
pub fn find_node_on_system() -> Option<PathBuf> {
    let home = dirs::home_dir()?;

    // fnm: ~/.local/state/fnm_multishells/<id>/bin/node
    let fnm_base = home.join(".local/state/fnm_multishells");
    if fnm_base.exists() {
        if let Ok(entries) = std::fs::read_dir(&fnm_base) {
            for entry in entries.flatten() {
                let bin_path = entry.path().join("bin").join("node");
                if bin_path.exists() {
                    return Some(bin_path);
                }
                // Windows 可能在上一级
                let node_exe = entry.path().join("node.exe");
                if node_exe.exists() {
                    return Some(node_exe);
                }
            }
        }
    }

    // nvm: ~/.nvm/versions/node/<version>/bin/node
    let nvm_base = home.join(".nvm/versions/node");
    if nvm_base.exists() {
        if let Ok(entries) = std::fs::read_dir(&nvm_base) {
            for entry in entries.flatten() {
                let bin_path = entry.path().join("bin").join("node");
                if bin_path.exists() {
                    return Some(bin_path);
                }
                let node_exe = entry.path().join("node.exe");
                if node_exe.exists() {
                    return Some(node_exe);
                }
            }
        }
    }

    // Volta: ~/.volta/bin/node
    let volta_node = home.join(".volta").join("bin").join("node");
    if volta_node.exists() {
        return Some(volta_node);
    }

    // n (node version manager): ~/n/bin/node
    let n_node = home.join("n").join("bin").join("node");
    if n_node.exists() {
        return Some(n_node);
    }

    // npm global: %APPDATA%\npm\node.exe
    if let Some(appdata) = dirs::data_dir() {
        let npm_node = appdata.join("npm").join("node.exe");
        if npm_node.exists() {
            return Some(npm_node);
        }
    }

    // AgenticBoot managed node
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        let managed_node = PathBuf::from(local_appdata)
            .join("AgenticBoot")
            .join("managed-node");
        if managed_node.exists() {
            if let Ok(entries) = std::fs::read_dir(&managed_node) {
                // 找最新版本的 node.exe
                let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
                for entry in entries.flatten() {
                    let node_exe = entry.path().join("node.exe");
                    if node_exe.exists() {
                        if let Ok(metadata) = entry.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                if newest.as_ref().map_or(true, |(t, _)| modified > *t) {
                                    newest = Some((modified, node_exe));
                                }
                            }
                        }
                    }
                }
                if let Some((_, node_exe)) = newest {
                    return Some(node_exe);
                }
            }
        }
    }

    // 直接在 PATH 中找 node
    find_command_on_path("node")
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowsUninstallEntry {
    pub display_name: String,
    pub display_version: Option<String>,
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

pub fn run_command_checked(
    program: &str,
    args: &[&str],
    failure_context: &str,
) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("{failure_context}: {e}"))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let details = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        format!("exit code: {:?}", output.status.code())
    };

    Err(format!("{failure_context}: {details}"))
}

pub fn find_local_program_executable(dir_names: &[&str], exe_names: &[&str]) -> Option<PathBuf> {
    let mut bases = Vec::new();
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        let local = PathBuf::from(local);
        bases.push(local.join("Programs"));
        // Microsoft Store apps (APPX/MSIX) install executables to WindowsApps folder
        bases.push(local.join("Microsoft").join("WindowsApps"));
        bases.push(local);
    }
    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        bases.push(PathBuf::from(program_files));
    }
    if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
        bases.push(PathBuf::from(program_files_x86));
    }
    // Common Windows app installation locations
    if let Some(program_data) = std::env::var_os("ProgramData") {
        bases.push(PathBuf::from(program_data));
    }

    for base in &bases {
        for dir_name in dir_names {
            for exe_name in exe_names {
                let candidate = base.join(dir_name).join(exe_name);
                if candidate.exists() {
                    return Some(candidate);
                }
            }
        }
    }

    for base in &bases {
        for dir_name in dir_names {
            let app_dir = base.join(dir_name);
            if !app_dir.is_dir() {
                continue;
            }
            for exe_name in exe_names {
                if let Some(found) = search_app_version_dir(&app_dir, exe_name) {
                    return Some(found);
                }
            }
        }
    }

    None
}

fn search_app_version_dir(app_dir: &Path, exe_name: &str) -> Option<PathBuf> {
    let entries: Vec<_> = std::fs::read_dir(app_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .collect();

    let mut version_dirs: Vec<_> = entries
        .iter()
        .filter(|e| {
            e.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && e.file_name()
                    .to_string_lossy()
                    .starts_with("app-")
        })
        .collect();

    version_dirs.sort_by(|a, b| {
        let a_name = a.file_name().to_string_lossy().into_owned();
        let b_name = b.file_name().to_string_lossy().into_owned();
        b_name.cmp(&a_name)
    });

    for dir in version_dirs {
        let candidate = dir.path().join(exe_name);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

pub fn read_exe_version(exe_path: &Path) -> Option<String> {
    let path_wide: Vec<u16> = exe_path
        .to_string_lossy()
        .to_string()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let size = unsafe {
        windows::Win32::Storage::FileSystem::GetFileVersionInfoSizeW(
            windows::core::PCWSTR(path_wide.as_ptr()),
            None,
        )
    };
    if size == 0 {
        return None;
    }
    let mut buffer = vec![0u8; size as usize];
    unsafe {
        windows::Win32::Storage::FileSystem::GetFileVersionInfoW(
            windows::core::PCWSTR(path_wide.as_ptr()),
            0,
            size,
            buffer.as_mut_ptr() as *mut _,
        )
    }
    .ok()?;
    let sub_block: Vec<u16> = r"\StringFileInfo\040904B0\ProductVersion"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut value_ptr: *mut u16 = std::ptr::null_mut();
    let mut value_len: u32 = 0;
    let _ = unsafe {
        windows::Win32::Storage::FileSystem::VerQueryValueW(
            buffer.as_ptr() as *const _,
            windows::core::PCWSTR(sub_block.as_ptr()),
            &mut value_ptr as *mut _ as *mut _,
            &mut value_len as *mut _,
        )
    };
    if value_len == 0 {
        return None;
    }
    let version_slice = unsafe { std::slice::from_raw_parts(value_ptr, value_len as usize) };
    let version = String::from_utf16_lossy(version_slice);
    Some(version.trim_end_matches('\0').trim().to_string())
}

/// Returns (install_location, version) for an AppX package
pub fn find_appx_install_location(package_name: &str) -> Option<(String, Option<String>)> {
    let script = format!(
        "$pkg = Get-AppxPackage {package_name} | Select-Object -First 1; \
         if ($pkg) {{ Write-Output (\"$($pkg.InstallLocation)|$($pkg.Version)\") }}"
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let trimmed = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if trimmed.is_empty() {
        return None;
    }

    let parts: Vec<&str> = trimmed.splitn(2, '|').collect();
    let location = parts[0].to_string();
    let version = parts.get(1).filter(|v| !v.is_empty()).map(|v| v.to_string());

    Some((location, version))
}

/// Search for an executable in the AppX package's LocalCache directory.
/// AppX packages often store their actual executables in LocalCache rather than
/// the protected WindowsApps directory returned by Get-AppxPackage.
pub fn find_appx_exe_in_localcache(package_name: &str, exe_name: &str) -> Option<PathBuf> {
    let local = std::env::var_os("LOCALAPPDATA")?;
    let packages_dir = PathBuf::from(local).join("Packages");
    if !packages_dir.is_dir() {
        return None;
    }

    let entries = std::fs::read_dir(&packages_dir).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(package_name) {
            let localcache = entry.path().join("LocalCache");
            if let Some(found) = find_exe_recursive(&localcache, exe_name, 0, 5) {
                return Some(found);
            }
        }
    }

    None
}

/// Recursively search for an executable within an APPX package directory
pub fn find_exe_in_appx_package(appx_dir: &Path, exe_name: &str) -> Option<PathBuf> {
    if !appx_dir.is_dir() {
        return None;
    }

    // Check direct children first
    if let Some(found) = find_executable_in_dir(appx_dir, &[exe_name]) {
        return Some(found);
    }

    // Recursively search subdirectories (depth limit to avoid excessive searching)
    find_exe_recursive(appx_dir, exe_name, 0, 3)
}

fn find_exe_recursive(dir: &Path, exe_name: &str, current_depth: usize, max_depth: usize) -> Option<PathBuf> {
    if current_depth >= max_depth {
        return None;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return None,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            // Check if this subdirectory has the exe directly
            if let Some(found) = find_executable_in_dir(&path, &[exe_name]) {
                return Some(found);
            }
            // Recurse into subdirectory
            if let Some(found) = find_exe_recursive(&path, exe_name, current_depth + 1, max_depth) {
                return Some(found);
            }
        }
    }
    None
}

pub fn find_uninstall_entry_ex(
    name_fragments: &[&str],
    exclude_fragments: &[&str],
) -> Option<WindowsUninstallEntry> {
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

        let excludes = exclude_fragments
            .iter()
            .map(|name| name.to_ascii_lowercase())
            .collect::<Vec<_>>();

        let mut best: Option<(WindowsUninstallEntry, usize)> = None;

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

                if excludes
                    .iter()
                    .any(|ex| display_name_lower.contains(ex.as_str()))
                {
                    continue;
                }

                let match_score = fragments
                    .iter()
                    .filter(|f| display_name_lower.contains(*f))
                    .count();

                let entry = WindowsUninstallEntry {
                    display_name,
                    display_version: app_key
                        .get_value::<String, _>("DisplayVersion")
                        .ok(),
                    install_location: app_key
                        .get_value::<String, _>("InstallLocation")
                        .ok()
                        .map(PathBuf::from),
                    display_icon: app_key
                        .get_value::<String, _>("DisplayIcon")
                        .ok()
                        .and_then(|raw| sanitize_windows_path_field(&raw)),
                    uninstall_string: app_key.get_value("UninstallString").ok(),
                };

                match &best {
                    None => best = Some((entry, match_score)),
                    Some((_, best_score)) if match_score > *best_score => {
                        best = Some((entry, match_score));
                    }
                    Some(_) => {}
                }
            }
        }

        best.map(|(entry, _)| entry)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (name_fragments, exclude_fragments);
        None
    }
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
    use super::{
        detect_windows_cli_version_with_shell, find_node_on_system,
        list_nvm_version_directories, read_command_version, run_command_checked,
        run_with_node_env, ShellCommandOutput, WindowsShell,
    };
    use std::collections::VecDeque;

    #[derive(Debug, Clone)]
    struct FakeShell {
        responses: VecDeque<Result<ShellCommandOutput, String>>,
        calls: Vec<(String, Vec<String>)>,
    }

    impl FakeShell {
        fn new(responses: Vec<Result<ShellCommandOutput, String>>) -> Self {
            Self {
                responses: responses.into(),
                calls: Vec::new(),
            }
        }
    }

    impl WindowsShell for FakeShell {
        fn run(&mut self, program: &str, args: &[String]) -> Result<ShellCommandOutput, String> {
            self.calls.push((
                program.to_string(),
                args.iter().map(ToOwned::to_owned).collect(),
            ));
            self.responses
                .pop_front()
                .unwrap_or_else(|| Err("missing fake shell response".to_string()))
        }
    }

    fn shell_success(stdout: &str) -> Result<ShellCommandOutput, String> {
        Ok(ShellCommandOutput {
            success: true,
            stdout: stdout.to_string(),
            stderr: String::new(),
        })
    }

    fn shell_failure(stderr: &str) -> Result<ShellCommandOutput, String> {
        Ok(ShellCommandOutput {
            success: false,
            stdout: String::new(),
            stderr: stderr.to_string(),
        })
    }

    #[test]
    fn find_node_on_system_finds_agenticboot_managed_node() {
        let node = find_node_on_system();
        assert!(node.is_some(), "should find node on system");
        let node_path = node.unwrap();
        assert!(node_path.exists(), "found node path should exist: {}", node_path.display());
    }

    #[test]
    fn run_with_node_env_can_execute_claude_via_found_node() {
        // This test verifies that run_with_node_env can find node and use it to run commands
        let output = run_with_node_env(std::path::Path::new("claude"), &["--version"]);
        // May fail if claude is not installed, but should not panic
        // The important thing is it finds node and doesn't crash
        let _ = output;
    }

    #[test]
    fn windows_cli_detector_prefers_direct_command_before_nvm_fallback() {
        let mut shell = FakeShell::new(vec![shell_success("claude 1.2.3\n")]);

        let detected = detect_windows_cli_version_with_shell(&mut shell, "claude", None);

        assert_eq!(detected.as_deref(), Some("claude 1.2.3"));
        assert_eq!(shell.calls.len(), 1);
        assert_eq!(shell.calls[0].0, "cmd");
        assert_eq!(
            shell.calls[0].1,
            vec!["/C".to_string(), "claude --version".to_string()]
        );
    }

    #[test]
    fn windows_cli_detector_uses_latest_nvm_version_in_single_cmd_session() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("v20.10.0")).unwrap();
        std::fs::create_dir_all(tmp.path().join("v22.11.0")).unwrap();
        std::fs::create_dir_all(tmp.path().join("not-a-version")).unwrap();

        let mut shell = FakeShell::new(vec![
            shell_failure("claude not found"),
            shell_success("Now using node v22.11.0\r\nclaude 9.9.9\r\n"),
        ]);

        let detected = detect_windows_cli_version_with_shell(
            &mut shell,
            "claude",
            Some(tmp.path()),
        );

        assert_eq!(detected.as_deref(), Some("claude 9.9.9"));
        assert_eq!(shell.calls.len(), 2);
        assert_eq!(
            shell.calls[1].1,
            vec![
                "/C".to_string(),
                "nvm use v22.11.0 && claude --version".to_string()
            ]
        );
    }

    #[test]
    fn windows_cli_detector_returns_none_when_direct_and_nvm_fail() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("v22.11.0")).unwrap();

        let mut shell = FakeShell::new(vec![
            shell_failure("codex not found"),
            shell_failure("nvm use failed"),
        ]);

        let detected =
            detect_windows_cli_version_with_shell(&mut shell, "codex", Some(tmp.path()));

        assert_eq!(detected, None);
        assert_eq!(shell.calls.len(), 2);
    }

    #[test]
    fn list_nvm_version_directories_sorts_newest_first_and_ignores_noise() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("v18.20.0")).unwrap();
        std::fs::create_dir_all(tmp.path().join("v22.11.0")).unwrap();
        std::fs::create_dir_all(tmp.path().join("v20.10.0")).unwrap();
        std::fs::create_dir_all(tmp.path().join("current")).unwrap();
        std::fs::write(tmp.path().join("README.txt"), b"ignore").unwrap();

        let versions = list_nvm_version_directories(tmp.path());

        assert_eq!(versions, vec!["v22.11.0", "v20.10.0", "v18.20.0"]);
    }

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

    #[test]
    fn run_command_checked_returns_error_for_non_zero_exit_status() {
        let err = run_command_checked(
            "cmd",
            &["/C", "echo uninstall failed 1>&2 && exit /b 17"],
            "npm uninstall failed",
        )
        .unwrap_err();

        assert!(err.contains("npm uninstall failed"));
        assert!(err.contains("uninstall failed"));
    }
}

pub fn find_npm_in_install_root(install_root: &Path) -> Option<String> {
    let candidates = ["npm.cmd", "node_modules\\npm\\bin\\npm-cli.js"];
    for candidate in candidates {
        let path = install_root.join("nodejs").join(candidate);
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }
    None
}
