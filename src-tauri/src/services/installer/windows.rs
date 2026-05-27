use crate::services::command_util::hide_console;
use crate::services::installer::logging::InstallLogEmitter;
use crate::tool_types::InstallLogLevel;
use once_cell::sync::Lazy;
use regex::Regex;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const DETECT_COMMAND_TIMEOUT: Duration = Duration::from_secs(6);
static VERSION_TOKEN_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)\bv?\d+\.\d+\.\d+(?:\.\d+)*\b").expect("valid version regex"));

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
        let mut command = Command::new(program);
        command.args(args);
        let output = run_detection_command_output(&mut command, program)?;

        Ok(ShellCommandOutput {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }
}

pub fn detect_windows_cli_version(command_name: &str) -> Option<String> {
    if let Some(command_path) = find_fast_windows_cli_command(command_name) {
        if let Some(version) = read_command_version(&command_path, &["--version"]) {
            return Some(version);
        }
    }

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
        log::info!("[Windows CLI Detect] Direct command succeeded for {command_name}: {version}");
        return Some(version);
    }

    let nvm_root = match nvm_root_override {
        Some(root) => root.to_path_buf(),
        None => find_nvm_root(shell)?,
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
        log::info!("[Windows CLI Detect] Trying {command_name} under nvm version {version}");
        if let Some(found) =
            run_windows_cli_version_with_nvm_path(shell, command_name, &nvm_root.join(&version))
        {
            log::info!(
                "[Windows CLI Detect] nvm fallback succeeded for {command_name} on {version}: {found}"
            );
            return Some(found);
        }
    }

    log::info!("[Windows CLI Detect] Exhausted nvm versions without finding {command_name}");
    None
}

fn run_windows_cli_version<S: WindowsShell>(shell: &mut S, command_name: &str) -> Option<String> {
    let command_path = resolve_windows_cli_command(shell, command_name)?;
    let result = shell
        .run(
            "cmd",
            &[
                String::from("/C"),
                format!("\"{}\" --version", command_path.replace('/', "\\")),
            ],
        )
        .ok()?;
    if !result.success {
        return None;
    }
    extract_version_output(&result.stdout).or_else(|| extract_version_output(&result.stderr))
}

fn resolve_windows_cli_command<S: WindowsShell>(
    shell: &mut S,
    command_name: &str,
) -> Option<String> {
    let result = shell.run("where", &[command_name.to_string()]).ok()?;
    if !result.success {
        return None;
    }

    select_preferred_where_result(&result.stdout)
        .or_else(|| select_preferred_where_result(&result.stderr))
}

fn run_windows_cli_version_with_nvm_path<S: WindowsShell>(
    shell: &mut S,
    command_name: &str,
    version_dir: &Path,
) -> Option<String> {
    let path_prefix = nvm_path_prefix(version_dir);
    let invocations = cli_invocation_candidates(command_name, version_dir);

    if invocations.is_empty() {
        return None;
    }

    for invocation in invocations {
        let command = format!("set \"PATH={path_prefix};%PATH%\" && {invocation} --version");
        let result = shell.run("cmd", &[String::from("/C"), command]).ok()?;
        if !result.success {
            continue;
        }
        if let Some(found) = extract_version_output(&result.stdout) {
            return Some(found);
        }
        if let Some(found) = extract_version_output(&result.stderr) {
            return Some(found);
        }
    }

    None
}

fn find_nvm_root<S: WindowsShell>(shell: &mut S) -> Option<PathBuf> {
    nvm_root_from_env()
        .or_else(nvm_root_from_settings_or_path)
        .or_else(|| nvm_root_via_cmd(shell))
}

pub fn find_windows_cli_install_dir_with_shell<S: WindowsShell>(
    shell: &mut S,
    command_name: &str,
) -> Option<PathBuf> {
    resolve_windows_cli_command(shell, command_name)
        .and_then(|command_path| Path::new(&command_path).parent().map(PathBuf::from))
        .or_else(|| {
            let nvm_root = find_nvm_root(shell)?;
            find_windows_cli_install_dir_in_nvm_root(command_name, &nvm_root)
        })
}

pub fn find_windows_cli_install_dir_in_nvm_root(
    command_name: &str,
    nvm_root: &Path,
) -> Option<PathBuf> {
    let versions = list_nvm_version_directories(nvm_root);
    for version in versions {
        let version_dir = nvm_root.join(&version);
        if let Some(candidate) = cli_shim_candidates(command_name, &version_dir)
            .into_iter()
            .next()
        {
            if candidate.starts_with(&version_dir) {
                return Some(version_dir);
            }
            if let Some(parent) = candidate.parent() {
                return Some(parent.to_path_buf());
            }
        }
    }

    None
}

fn nvm_root_from_env() -> Option<PathBuf> {
    std::env::var_os("NVM_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_dir())
}

fn nvm_root_from_settings_or_path() -> Option<PathBuf> {
    let nvm_path = find_command_on_path("nvm")?;
    read_nvm_root_from_settings(
        &nvm_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("settings.txt"),
    )
    .filter(|path| path.is_dir())
    .or_else(|| {
        nvm_path
            .parent()
            .map(PathBuf::from)
            .filter(|path| path.is_dir())
    })
}

fn nvm_root_via_cmd<S: WindowsShell>(shell: &mut S) -> Option<PathBuf> {
    let result = shell
        .run("cmd", &[String::from("/C"), String::from("nvm root")])
        .ok()?;
    if !result.success {
        log::info!("[Windows CLI Detect] `nvm root` failed");
        return None;
    }
    let root = extract_version_output(&result.stdout)
        .or_else(|| extract_version_output(&result.stderr))?;
    (!root.trim().is_empty()).then_some(PathBuf::from(root.trim()))
}

fn read_nvm_root_from_settings(settings_path: &Path) -> Option<PathBuf> {
    let contents = std::fs::read_to_string(settings_path).ok()?;
    contents
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("root:").map(str::trim))
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

fn find_fast_windows_cli_command(command_name: &str) -> Option<PathBuf> {
    dirs::data_dir()
        .and_then(|appdata| find_command_in_directory(&appdata.join("npm"), command_name))
        .or_else(|| find_command_in_path_env(command_name))
        .or_else(|| find_command_on_path(command_name))
}

fn find_command_in_directory(directory: &Path, command_name: &str) -> Option<PathBuf> {
    ["exe", "cmd", "bat", ""]
        .iter()
        .map(|extension| {
            if extension.is_empty() {
                directory.join(command_name)
            } else {
                directory.join(format!("{command_name}.{extension}"))
            }
        })
        .find(|path| path.exists())
}

fn find_command_in_path_env(command_name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .find_map(|directory| find_command_in_directory(&directory, command_name))
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
    preferred_output_line(output.lines())
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

    let mut command = Command::new(program);
    command.args(args).env("PATH", new_path);
    run_detection_command_output(&mut command, &program.to_string_lossy()).ok()
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

    if let Some(nvm_home) = std::env::var_os("NVM_HOME") {
        let windows_nvm = PathBuf::from(nvm_home).join("versions").join("node");
        if windows_nvm.exists() {
            if let Ok(entries) = std::fs::read_dir(&windows_nvm) {
                for entry in entries.flatten() {
                    let node_exe = entry.path().join("node.exe");
                    if node_exe.exists() {
                        return Some(node_exe);
                    }
                }
            }
        }
    }

    if let Some(nvm_symlink) = std::env::var_os("NVM_SYMLINK") {
        let node_exe = PathBuf::from(nvm_symlink).join("node.exe");
        if node_exe.exists() {
            return Some(node_exe);
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

pub fn find_managed_executable(
    root: &Path,
    tool_dir: &str,
    candidates: &[&str],
) -> Option<PathBuf> {
    find_managed_paths(root, tool_dir, candidates).executable
}

pub fn npm_prefix_candidates(cmd_name: &str) -> Vec<String> {
    vec![
        format!("{cmd_name}.cmd"),
        format!("{cmd_name}.exe"),
        format!("bin\\{cmd_name}.cmd"),
        format!("bin\\{cmd_name}.exe"),
        format!("node_modules\\.bin\\{cmd_name}.cmd"),
        format!("node_modules\\.bin\\{cmd_name}.exe"),
    ]
}

pub fn find_command_install_dir(command: &str) -> Option<PathBuf> {
    find_command_on_path(command).and_then(|path| path.parent().map(PathBuf::from))
}

pub fn find_command_on_path(command: &str) -> Option<PathBuf> {
    if let Some(found) = find_command_in_path_env(command) {
        return Some(found);
    }

    let mut where_command = Command::new("where");
    where_command.arg(command);
    let output = run_detection_command_output(&mut where_command, "where").ok()?;
    if !output.status.success() {
        return None;
    }

    select_preferred_where_result(&String::from_utf8_lossy(&output.stdout))
        .or_else(|| select_preferred_where_result(&String::from_utf8_lossy(&output.stderr)))
        .map(PathBuf::from)
}

pub fn read_command_version(command: &Path, args: &[&str]) -> Option<String> {
    let mut version_command = Command::new(command);
    version_command.args(args);
    let output =
        run_detection_command_output(&mut version_command, &command.to_string_lossy()).ok()?;
    if !output.status.success() {
        return None;
    }

    first_non_empty_output_line(&output)
}

pub fn winget_exists() -> bool {
    find_command_on_path("winget").is_some()
}

pub fn run_winget(args: &[&str]) -> Result<(), String> {
    let mut command = Command::new("winget");
    command.args(args);
    hide_console(&mut command);
    let output = command
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

#[allow(dead_code)]
pub fn run_winget_with_logs(
    install_log: &InstallLogEmitter,
    phase: &str,
    args: &[&str],
) -> Result<(), String> {
    run_command_checked_with_logs(install_log, phase, "winget", args, "winget 执行失败")
}

#[allow(dead_code)]
pub fn run_spawned_command_with_logs<P: AsRef<Path>>(
    install_log: &InstallLogEmitter,
    phase: &str,
    program: P,
    args: &[&str],
    failure_context: &str,
) -> Result<(), String> {
    let program = program.as_ref();
    install_log.emit_command(
        phase,
        format_command_for_log(&program.to_string_lossy(), args),
    );

    let mut command = Command::new(program);
    command.args(args);
    hide_console(&mut command);
    let status = command
        .spawn()
        .map_err(|e| {
            let error = format!("{failure_context}: {e}");
            install_log.emit_output(phase, InstallLogLevel::Error, error.clone());
            error
        })?
        .wait()
        .map_err(|e| {
            let error = format!("{failure_context}: {e}");
            install_log.emit_output(phase, InstallLogLevel::Error, error.clone());
            error
        })?;

    if status.success() {
        install_log.emit_output(phase, InstallLogLevel::Success, "Command completed");
        return Ok(());
    }

    let details = format!("exit code: {:?}", status.code());
    install_log.emit_output(phase, InstallLogLevel::Error, details.clone());
    Err(format!("{failure_context}: {details}"))
}

pub fn run_command_checked(
    program: &str,
    args: &[&str],
    failure_context: &str,
) -> Result<(), String> {
    let mut command = Command::new(program);
    command.args(args);
    run_command_checked_with_command(&mut command, failure_context)
}

#[allow(dead_code)]
pub fn run_command_checked_with_logs(
    install_log: &InstallLogEmitter,
    phase: &str,
    program: &str,
    args: &[&str],
    failure_context: &str,
) -> Result<(), String> {
    let command_line = format_command_for_log(program, args);
    install_log.emit_command(phase, command_line);

    let mut command = Command::new(program);
    command.args(args);
    run_command_checked_with_logs_for_command(install_log, phase, &mut command, failure_context)
}

fn run_command_checked_with_command(
    command: &mut Command,
    failure_context: &str,
) -> Result<(), String> {
    log::info!("[run_command_checked_with_command] 执行命令: {:?}", command);
    let output = run_command_output(command, failure_context)?;

    if output.status.success() {
        log::info!(
            "[run_command_checked_with_command] 成功, stdout: {}",
            String::from_utf8_lossy(&output.stdout)
                .trim()
                .to_string()
                .lines()
                .take(5)
                .collect::<Vec<_>>()
                .join("\n")
        );
        return Ok(());
    }

    let details = command_failure_details(&output);
    log::error!(
        "[run_command_checked_with_command] 失败: {}, stderr: {}",
        details,
        String::from_utf8_lossy(&output.stderr).trim().to_string()
    );
    Err(format!("{failure_context}: {details}"))
}

fn run_command_output(command: &mut Command, failure_context: &str) -> Result<Output, String> {
    hide_console(command);
    command
        .output()
        .map_err(|e| format!("{failure_context}: {e}"))
}

fn run_command_checked_with_logs_for_command(
    install_log: &InstallLogEmitter,
    phase: &str,
    command: &mut Command,
    failure_context: &str,
) -> Result<(), String> {
    let output = match run_command_output(command, failure_context) {
        Ok(output) => output,
        Err(error) => {
            install_log.emit_output(phase, InstallLogLevel::Error, error.clone());
            return Err(error);
        }
    };
    emit_output_lines(install_log, phase, &output);

    if output.status.success() {
        install_log.emit_output(phase, InstallLogLevel::Success, "Command completed");
        return Ok(());
    }

    let details = command_failure_details(&output);
    install_log.emit_output(phase, InstallLogLevel::Error, details.clone());
    Err(format!("{failure_context}: {details}"))
}

#[allow(dead_code)]
pub fn run_command_checked_with_streaming_logs_for_command(
    install_log: &InstallLogEmitter,
    phase: &str,
    command: &mut Command,
    failure_context: &str,
) -> Result<(), String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    hide_console(command);

    let mut child = command.spawn().map_err(|e| {
        let error = format!("{failure_context}: {e}");
        install_log.emit_output(phase, InstallLogLevel::Error, error.clone());
        error
    })?;

    let stdout_reader = child.stdout.take().map(|stdout| {
        spawn_stream_reader(
            stdout,
            install_log.clone(),
            phase.to_string(),
            InstallLogLevel::Stdout,
        )
    });
    let stderr_reader = child.stderr.take().map(|stderr| {
        spawn_stream_reader(
            stderr,
            install_log.clone(),
            phase.to_string(),
            InstallLogLevel::Stderr,
        )
    });

    let status = child.wait().map_err(|e| {
        let error = format!("{failure_context}: {e}");
        install_log.emit_output(phase, InstallLogLevel::Error, error.clone());
        error
    })?;

    let stdout = join_stream_reader(stdout_reader, install_log, phase, failure_context, "stdout")?;
    let stderr = join_stream_reader(stderr_reader, install_log, phase, failure_context, "stderr")?;

    if status.success() {
        install_log.emit_output(phase, InstallLogLevel::Success, "Command completed");
        return Ok(());
    }

    let details = command_failure_details_from_text(&stdout, &stderr, status.code());
    install_log.emit_output(phase, InstallLogLevel::Error, details.clone());
    Err(format!("{failure_context}: {details}"))
}

fn spawn_stream_reader<R: Read + Send + 'static>(
    reader: R,
    install_log: InstallLogEmitter,
    phase: String,
    level: InstallLogLevel,
) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut collected = Vec::new();
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            install_log.emit_output(&phase, level, trimmed.to_string());
            collected.push(trimmed.to_string());
        }
        collected.join("\n")
    })
}

fn join_stream_reader(
    handle: Option<thread::JoinHandle<String>>,
    install_log: &InstallLogEmitter,
    phase: &str,
    failure_context: &str,
    stream_name: &str,
) -> Result<String, String> {
    let Some(handle) = handle else {
        return Ok(String::new());
    };

    handle.join().map_err(|_| {
        let error = format!("{failure_context}: failed to collect {stream_name} output");
        install_log.emit_output(phase, InstallLogLevel::Error, error.clone());
        error
    })
}

fn command_failure_details(output: &Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    command_failure_details_from_text(&stdout, &stderr, output.status.code())
}

fn command_failure_details_from_text(stdout: &str, stderr: &str, exit_code: Option<i32>) -> String {
    if !stderr.is_empty() {
        stderr.to_string()
    } else if !stdout.is_empty() {
        stdout.to_string()
    } else {
        format!("exit code: {exit_code:?}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManagedNpmCommand {
    program: PathBuf,
    prefix_args: Vec<String>,
}

fn resolve_npm_command_from_node_dir(node_dir: &Path) -> Option<ManagedNpmCommand> {
    let npm_cmd = node_dir.join("npm.cmd");
    if npm_cmd.exists() {
        return Some(ManagedNpmCommand {
            program: npm_cmd,
            prefix_args: Vec::new(),
        });
    }

    let node_exe = node_dir.join("node.exe");
    let npm_cli = node_dir
        .join("node_modules")
        .join("npm")
        .join("bin")
        .join("npm-cli.js");
    if node_exe.exists() && npm_cli.exists() {
        return Some(ManagedNpmCommand {
            program: node_exe,
            prefix_args: vec![npm_cli.to_string_lossy().to_string()],
        });
    }

    None
}

fn resolve_managed_npm_command(install_root: &Path) -> Option<ManagedNpmCommand> {
    resolve_npm_command_from_node_dir(&install_root.join("nodejs"))
}

fn resolve_system_npm_command() -> Option<ManagedNpmCommand> {
    if let Some(npm_path) = find_command_on_path("npm") {
        return Some(ManagedNpmCommand {
            program: npm_path,
            prefix_args: Vec::new(),
        });
    }

    let node_exe = find_node_on_system()?;
    let node_dir = node_exe.parent()?;
    resolve_npm_command_from_node_dir(node_dir)
}

fn apply_command_extra_env(command: &mut Command, extra_env: &[(&str, &str)]) {
    for (key, value) in extra_env {
        command.env(key, value);
    }
}

/// Same as [`run_npm_command_checked`], but sets extra environment variables on the npm process
/// (inherited by install scripts). Used for OpenClaw: `SHARP_IGNORE_GLOBAL_LIBVIPS=1` per upstream docs.
pub fn run_npm_command_checked_with_env(
    install_root: &Path,
    args: &[&str],
    failure_context: &str,
    extra_env: &[(&str, &str)],
) -> Result<(), String> {
    log::info!(
        "[run_npm_command_checked] install_root={}, args={:?}, extra_env_keys={:?}",
        install_root.display(),
        args,
        extra_env.iter().map(|(k, _)| *k).collect::<Vec<_>>()
    );
    if let Some(managed) = resolve_managed_npm_command(install_root) {
        log::info!(
            "[run_npm_command_checked] 使用托管 npm: {:?}",
            managed.program
        );
        let mut command = Command::new(&managed.program);
        command.args(&managed.prefix_args).args(args);
        apply_command_extra_env(&mut command, extra_env);
        return run_command_checked_with_command(&mut command, failure_context);
    }

    if let Some(system) = resolve_system_npm_command() {
        log::info!(
            "[run_npm_command_checked] 使用系统 npm: {:?}",
            system.program
        );
        let mut command = Command::new(&system.program);
        command.args(&system.prefix_args).args(args);
        apply_command_extra_env(&mut command, extra_env);
        return run_command_checked_with_command(&mut command, failure_context);
    }

    log::info!("[run_npm_command_checked] 使用 PATH 中的 npm");
    let mut command = Command::new("npm");
    command.args(args);
    apply_command_extra_env(&mut command, extra_env);
    run_command_checked_with_command(&mut command, failure_context)
}

pub fn run_npm_command_checked(
    install_root: &Path,
    args: &[&str],
    failure_context: &str,
) -> Result<(), String> {
    run_npm_command_checked_with_env(install_root, args, failure_context, &[])
}

pub(crate) fn resolve_npm_command_for_uninstall(install_dir: &Path) -> Option<ManagedNpmCommand> {
    resolve_npm_command_from_node_dir(install_dir)
        .or_else(|| install_dir.parent().and_then(resolve_managed_npm_command))
        .or_else(resolve_system_npm_command)
}

pub fn run_npm_command_checked_for_uninstall(
    install_dir: &Path,
    args: &[&str],
    failure_context: &str,
) -> Result<(), String> {
    log::info!(
        "[run_npm_command_checked_for_uninstall] install_dir={}, args={:?}",
        install_dir.display(),
        args
    );
    if let Some(npm) = resolve_npm_command_for_uninstall(install_dir) {
        log::info!(
            "[run_npm_command_checked_for_uninstall] 使用 npm: {:?}",
            npm.program
        );
        let mut command = Command::new(&npm.program);
        command.args(&npm.prefix_args).args(args);
        return run_command_checked_with_command(&mut command, failure_context);
    }

    run_command_checked("npm", args, failure_context)
}

#[allow(dead_code)]
pub fn run_npm_command_checked_with_logs(
    install_root: &Path,
    install_log: &InstallLogEmitter,
    phase: &str,
    args: &[&str],
    failure_context: &str,
) -> Result<(), String> {
    run_npm_command_checked_with_env_and_logs(
        install_root,
        install_log,
        phase,
        args,
        failure_context,
        &[],
    )
}

#[allow(dead_code)]
pub fn run_npm_command_checked_with_env_and_logs(
    install_root: &Path,
    install_log: &InstallLogEmitter,
    phase: &str,
    args: &[&str],
    failure_context: &str,
    extra_env: &[(&str, &str)],
) -> Result<(), String> {
    if let Some(managed) = resolve_managed_npm_command(install_root) {
        let mut command = Command::new(&managed.program);
        command.args(&managed.prefix_args).args(args);
        apply_command_extra_env(&mut command, extra_env);

        let all_args = managed
            .prefix_args
            .iter()
            .map(String::as_str)
            .chain(args.iter().copied())
            .collect::<Vec<_>>();
        install_log.emit_command(
            phase,
            format_command_for_log(&managed.program.to_string_lossy(), &all_args),
        );

        let output = match run_command_output(&mut command, failure_context) {
            Ok(output) => output,
            Err(error) => {
                install_log.emit_output(phase, InstallLogLevel::Error, error.clone());
                return Err(error);
            }
        };
        emit_output_lines(install_log, phase, &output);
        if output.status.success() {
            install_log.emit_output(phase, InstallLogLevel::Success, "Command completed");
            return Ok(());
        }

        let details = command_failure_details(&output);
        install_log.emit_output(phase, InstallLogLevel::Error, details.clone());
        return Err(format!("{failure_context}: {details}"));
    }

    if let Some(system) = resolve_system_npm_command() {
        let mut command = Command::new(&system.program);
        command.args(&system.prefix_args).args(args);
        apply_command_extra_env(&mut command, extra_env);

        let all_args = system
            .prefix_args
            .iter()
            .map(String::as_str)
            .chain(args.iter().copied())
            .collect::<Vec<_>>();
        install_log.emit_command(
            phase,
            format_command_for_log(&system.program.to_string_lossy(), &all_args),
        );

        let output = match run_command_output(&mut command, failure_context) {
            Ok(output) => output,
            Err(error) => {
                install_log.emit_output(phase, InstallLogLevel::Error, error.clone());
                return Err(error);
            }
        };
        emit_output_lines(install_log, phase, &output);
        if output.status.success() {
            install_log.emit_output(phase, InstallLogLevel::Success, "Command completed");
            return Ok(());
        }

        let details = command_failure_details(&output);
        install_log.emit_output(phase, InstallLogLevel::Error, details.clone());
        return Err(format!("{failure_context}: {details}"));
    }

    let mut command = Command::new("npm");
    command.args(args);
    apply_command_extra_env(&mut command, extra_env);
    let command_line = format_command_for_log("npm", args);
    install_log.emit_command(phase, command_line);
    run_command_checked_with_logs_for_command(install_log, phase, &mut command, failure_context)
}

#[allow(dead_code)]
fn emit_output_lines(install_log: &InstallLogEmitter, phase: &str, output: &Output) {
    for line in String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        install_log.emit_output(phase, InstallLogLevel::Stdout, line.to_string());
    }

    for line in String::from_utf8_lossy(&output.stderr)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        install_log.emit_output(phase, InstallLogLevel::Stderr, line.to_string());
    }
}

#[allow(dead_code)]
fn format_command_for_log(program: &str, args: &[&str]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().map(|arg| quote_command_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

#[allow(dead_code)]
fn quote_command_arg(arg: &str) -> String {
    if arg.contains([' ', '\t', '"']) {
        format!("\"{}\"", arg.replace('"', "\\\""))
    } else {
        arg.to_string()
    }
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

pub fn find_local_uninstaller_executable(install_dir: &Path) -> Option<PathBuf> {
    let direct_candidates = [
        "uninstall.exe",
        "Uninstall.exe",
        "unins000.exe",
        "unins001.exe",
        "remove.exe",
        "Remove.exe",
    ];

    if let Some(found) = find_executable_in_dir(install_dir, &direct_candidates) {
        return Some(found);
    }

    if let Some(found) = find_named_uninstall_executable(install_dir) {
        return Some(found);
    }

    for subdir in [
        "uninstall",
        "Uninstall",
        "uninstaller",
        "Uninstaller",
        "remove",
        "Remove",
    ] {
        let candidate_dir = install_dir.join(subdir);
        if let Some(found) = find_executable_in_dir(&candidate_dir, &direct_candidates) {
            return Some(found);
        }
        if let Some(found) = find_named_uninstall_executable(&candidate_dir) {
            return Some(found);
        }
    }

    None
}

fn find_named_uninstall_executable(install_dir: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(install_dir).ok()?;
    let mut candidates = entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let file_type = entry.file_type().ok()?;
            if !file_type.is_file() {
                return None;
            }

            let name = path.file_name()?.to_string_lossy().to_ascii_lowercase();
            (name.starts_with("uninstall") && name.ends_with(".exe")).then_some(path)
        })
        .collect::<Vec<_>>();

    candidates.sort();
    candidates.into_iter().next()
}

pub fn run_windows_uninstaller_with_common_args(uninstaller: &Path) -> Result<(), String> {
    let attempts: &[&[&str]] = &[
        &["/S"],
        &["/silent"],
        &["/VERYSILENT", "/SUPPRESSMSGBOXES", "/NORESTART"],
        &[],
    ];

    let mut last_error: Option<String> = None;

    for args in attempts {
        let mut command = Command::new(uninstaller);
        command.args(*args);
        hide_console(&mut command);
        match command.spawn() {
            Ok(mut child) => match child.wait() {
                Ok(status) if status.success() => return Ok(()),
                Ok(status) => {
                    last_error = Some(format!(
                        "{} exited with code {:?}",
                        uninstaller.display(),
                        status.code()
                    ));
                }
                Err(e) => {
                    last_error = Some(format!("failed to wait for {}: {e}", uninstaller.display()));
                }
            },
            Err(e) => {
                last_error = Some(format!("failed to launch {}: {e}", uninstaller.display()));
            }
        }
    }

    Err(last_error
        .unwrap_or_else(|| format!("failed to run uninstaller {}", uninstaller.display())))
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
                && e.file_name().to_string_lossy().starts_with("app-")
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
    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-Command", &script]);
    let output = run_detection_command_output(&mut command, "powershell").ok()?;
    if !output.status.success() {
        return None;
    }

    let trimmed = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if trimmed.is_empty() {
        return None;
    }

    let parts: Vec<&str> = trimmed.splitn(2, '|').collect();
    let location = parts[0].to_string();
    let version = parts
        .get(1)
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string());

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

fn find_exe_recursive(
    dir: &Path,
    exe_name: &str,
    current_depth: usize,
    max_depth: usize,
) -> Option<PathBuf> {
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
                    display_version: app_key.get_value::<String, _>("DisplayVersion").ok(),
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

fn nvm_path_prefix(version_dir: &Path) -> String {
    let mut parts = vec![
        normalize_windows_exe(version_dir),
        normalize_windows_exe(&version_dir.join("node_modules").join(".bin")),
    ];

    if let Some(appdata) = dirs::data_dir() {
        parts.push(normalize_windows_exe(&appdata.join("npm")));
    }

    parts.join(";")
}

fn select_preferred_where_result(output: &str) -> Option<String> {
    let mut candidates = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with("INFO:"))
        .collect::<Vec<_>>();

    candidates.sort_by_key(|line| command_path_rank(line));
    candidates.first().map(|line| (*line).to_string())
}

fn command_path_rank(path: &str) -> usize {
    let lowercase = path.to_ascii_lowercase();
    if lowercase.ends_with(".exe") {
        0
    } else if lowercase.ends_with(".cmd") {
        1
    } else if lowercase.ends_with(".bat") {
        2
    } else {
        3
    }
}

fn cli_invocation_candidates(command_name: &str, version_dir: &Path) -> Vec<String> {
    let mut invocations = Vec::new();

    for path in cli_shim_candidates(command_name, version_dir) {
        let quoted = format!("\"{}\"", normalize_windows_exe(&path));
        if !invocations.contains(&quoted) {
            invocations.push(quoted);
        }
    }

    invocations
}

fn cli_shim_candidates(command_name: &str, version_dir: &Path) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for extension in ["cmd", "exe"] {
        candidates.push(
            version_dir
                .join("node_modules")
                .join(".bin")
                .join(format!("{command_name}.{extension}")),
        );
        candidates.push(version_dir.join(format!("{command_name}.{extension}")));
    }

    if let Some(appdata) = dirs::data_dir() {
        for extension in ["cmd", "exe"] {
            candidates.push(
                appdata
                    .join("npm")
                    .join(format!("{command_name}.{extension}")),
            );
        }
    }

    candidates
        .into_iter()
        .filter(|path| path.exists())
        .collect()
}

fn first_non_empty_output_line(output: &Output) -> Option<String> {
    preferred_output_line(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .chain(String::from_utf8_lossy(&output.stderr).lines()),
    )
}

fn preferred_output_line<'a>(lines: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let mut first_non_empty = None;
    let mut last_version_like = None;

    for line in lines.into_iter().map(str::trim).filter(|line| !line.is_empty()) {
        if first_non_empty.is_none() {
            first_non_empty = Some(line.to_string());
        }

        if VERSION_TOKEN_RE.is_match(line) {
            last_version_like = Some(line.to_string());
        }
    }

    last_version_like.or(first_non_empty)
}

pub fn run_detection_command_output(
    command: &mut Command,
    description: &str,
) -> Result<Output, String> {
    run_detection_command_output_with_timeout(command, description, DETECT_COMMAND_TIMEOUT)
}

fn run_detection_command_output_with_timeout(
    command: &mut Command,
    description: &str,
    timeout: Duration,
) -> Result<Output, String> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    hide_console(command);

    let mut child = command
        .spawn()
        .map_err(|e| format!("failed to spawn {description}: {e}"))?;
    let start = Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child
                    .wait_with_output()
                    .map_err(|e| format!("failed to read {description} output: {e}"));
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "{description} timed out after {} ms",
                        timeout.as_millis()
                    ));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!("failed while waiting for {description}: {e}"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        detect_windows_cli_version_with_shell, extract_version_output, find_command_in_directory,
        find_local_uninstaller_executable, find_node_on_system, list_nvm_version_directories,
        read_command_version, read_nvm_root_from_settings, resolve_managed_npm_command,
        resolve_npm_command_for_uninstall, resolve_npm_command_from_node_dir, run_command_checked,
        run_command_checked_with_logs, run_command_checked_with_streaming_logs_for_command,
        run_detection_command_output_with_timeout, run_with_node_env, ManagedNpmCommand,
        ShellCommandOutput, WindowsShell,
    };
    use crate::services::installer::logging::InstallLogEmitter;
    use crate::tool_types::{InstallLogEvent, InstallLogKind, InstallLogLevel};
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

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

    fn test_install_log_emitter() -> (InstallLogEmitter, Arc<Mutex<Vec<InstallLogEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let emitter =
            InstallLogEmitter::new_for_test("codex-desktop", "Codex (Desktop)", move |event| {
                sink.lock().unwrap().push(event);
            });

        (emitter, events)
    }

    #[test]
    fn find_node_on_system_finds_agenticboot_managed_node() {
        let node = find_node_on_system();
        assert!(node.is_some(), "should find node on system");
        let node_path = node.unwrap();
        assert!(
            node_path.exists(),
            "found node path should exist: {}",
            node_path.display()
        );
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
        let mut shell = FakeShell::new(vec![
            shell_success("C:\\Users\\me\\AppData\\Roaming\\npm\\claude.cmd\n"),
            shell_success("claude 1.2.3\n"),
        ]);

        let detected = detect_windows_cli_version_with_shell(&mut shell, "claude", None);

        assert_eq!(detected.as_deref(), Some("claude 1.2.3"));
        assert_eq!(shell.calls.len(), 2);
        assert_eq!(shell.calls[0].0, "where");
        assert_eq!(shell.calls[0].1, vec!["claude".to_string()]);
        assert_eq!(shell.calls[1].0, "cmd");
        assert_eq!(
            shell.calls[1].1,
            vec![
                "/C".to_string(),
                "\"C:\\Users\\me\\AppData\\Roaming\\npm\\claude.cmd\" --version".to_string()
            ]
        );
    }

    #[test]
    fn windows_cli_detector_uses_latest_nvm_version_in_single_cmd_session() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("v20.10.0")).unwrap();
        std::fs::create_dir_all(tmp.path().join("v22.11.0")).unwrap();
        std::fs::create_dir_all(
            tmp.path()
                .join("v22.11.0")
                .join("node_modules")
                .join(".bin"),
        )
        .unwrap();
        std::fs::write(
            tmp.path()
                .join("v22.11.0")
                .join("node_modules")
                .join(".bin")
                .join("claude.cmd"),
            b"",
        )
        .unwrap();
        std::fs::create_dir_all(tmp.path().join("not-a-version")).unwrap();

        let mut shell = FakeShell::new(vec![
            shell_failure("INFO: Could not find files for the given pattern(s)."),
            shell_success("Now using node v22.11.0\r\nclaude 9.9.9\r\n"),
        ]);

        let detected =
            detect_windows_cli_version_with_shell(&mut shell, "claude", Some(tmp.path()));

        assert_eq!(detected.as_deref(), Some("claude 9.9.9"));
        assert_eq!(shell.calls.len(), 2);
        assert_eq!(shell.calls[0].0, "where");
        assert_eq!(shell.calls[1].1[0], "/C");
        assert!(shell.calls[1].1[1].contains("set \"PATH="));
        assert!(shell.calls[1].1[1].contains("v22.11.0"));
        assert!(shell.calls[1].1[1].contains("claude.cmd"));
    }

    #[test]
    fn windows_cli_detector_returns_none_when_direct_and_nvm_fail() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("v22.11.0")).unwrap();

        let mut shell = FakeShell::new(vec![shell_failure(
            "INFO: Could not find files for the given pattern(s).",
        )]);

        let detected =
            detect_windows_cli_version_with_shell(&mut shell, "missing-cli", Some(tmp.path()));

        assert_eq!(detected, None);
        assert_eq!(shell.calls.len(), 1);
        assert_eq!(shell.calls[0].0, "where");
    }

    #[test]
    fn windows_cli_install_dir_can_be_found_without_version_output() {
        let tmp = tempfile::tempdir().unwrap();
        let nvm_root = tmp.path().join("nvm");
        let version_dir = nvm_root.join("v22.11.0");
        std::fs::create_dir_all(version_dir.join("node_modules").join(".bin")).unwrap();
        std::fs::write(
            version_dir
                .join("node_modules")
                .join(".bin")
                .join("opencode.cmd"),
            b"",
        )
        .unwrap();

        let install_dir = super::find_windows_cli_install_dir_in_nvm_root("opencode", &nvm_root);

        assert_eq!(install_dir.as_deref(), Some(version_dir.as_path()));
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
    fn read_command_version_prefers_semver_line_over_noise() {
        let tmp = tempfile::tempdir().unwrap();
        let script = tmp.path().join("version.cmd");
        std::fs::write(
            &script,
            "@echo off\r\necho i18next is made possible by our own product, Locize\r\necho Hermes Agent v0.12.0 (2026.4.30)\r\necho trailing\r\n",
        )
        .unwrap();

        let version_output = read_command_version(&script, &[]);
        let version = version_output.as_deref().map(str::trim);
        assert_eq!(version, Some("Hermes Agent v0.12.0 (2026.4.30)"));
    }

    #[test]
    fn extract_version_output_prefers_version_line_over_trailing_marketing_text() {
        let output = "Hermes Agent v0.12.0 (2026.4.30)\r\ni18next is made possible by our own product, Locize\r\n";

        let version = extract_version_output(output);

        assert_eq!(version.as_deref(), Some("Hermes Agent v0.12.0 (2026.4.30)"));
    }

    #[test]
    fn find_command_in_directory_prefers_windows_shims() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("claude"), b"").unwrap();
        std::fs::write(tmp.path().join("claude.cmd"), b"").unwrap();

        let found = find_command_in_directory(tmp.path(), "claude");

        assert_eq!(
            found.as_deref(),
            Some(tmp.path().join("claude.cmd").as_path())
        );
    }

    #[test]
    fn read_nvm_root_from_settings_extracts_root_directory() {
        let tmp = tempfile::tempdir().unwrap();
        let settings = tmp.path().join("settings.txt");
        std::fs::write(&settings, "root: D:\\nvm\r\npath: D:\\nodejs\r\n").unwrap();

        let root = read_nvm_root_from_settings(&settings);

        assert_eq!(root.as_deref(), Some(std::path::Path::new("D:\\nvm")));
    }

    #[test]
    fn run_detection_command_output_times_out_for_hanging_process() {
        let mut command = std::process::Command::new("powershell");
        command.args(["-NoProfile", "-Command", "Start-Sleep -Seconds 30"]);

        let error = run_detection_command_output_with_timeout(
            &mut command,
            "sleep",
            std::time::Duration::from_millis(100),
        )
        .unwrap_err();

        assert!(error.contains("timed out"));
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

    #[test]
    fn run_command_checked_with_logs_emits_output_error_without_terminal_result_on_failure() {
        let (install_log, events) = test_install_log_emitter();

        let err = run_command_checked_with_logs(
            &install_log,
            "installing",
            "cmd",
            &["/C", "echo command failed 1>&2 && exit /b 17"],
            "npm uninstall failed",
        )
        .unwrap_err();

        let events = events.lock().unwrap();

        assert!(err.contains("npm uninstall failed"));
        assert!(events.iter().any(|event| {
            event.kind == InstallLogKind::Command
                && event.command.as_deref()
                    == Some("cmd /C \"echo command failed 1>&2 && exit /b 17\"")
        }));
        assert!(events.iter().any(|event| {
            event.kind == InstallLogKind::Output
                && event.level == InstallLogLevel::Stderr
                && event.line.contains("command failed")
        }));
        assert!(events.iter().any(|event| {
            event.kind == InstallLogKind::Output
                && event.level == InstallLogLevel::Error
                && event.line.contains("command failed")
        }));
        assert!(!events
            .iter()
            .any(|event| event.kind == InstallLogKind::Result));
    }

    #[test]
    fn run_command_checked_with_logs_emits_raw_error_line_on_spawn_failure() {
        let (install_log, events) = test_install_log_emitter();

        let err = run_command_checked_with_logs(
            &install_log,
            "installing",
            "definitely_missing_command_12345",
            &["--version"],
            "launch failed",
        )
        .unwrap_err();

        let events = events.lock().unwrap();

        assert!(err.contains("launch failed"));
        assert!(events.iter().any(|event| {
            event.kind == InstallLogKind::Output
                && event.level == InstallLogLevel::Error
                && event.line == err
        }));
        assert!(!events
            .iter()
            .any(|event| event.kind == InstallLogKind::Result));
    }

    #[test]
    fn run_command_checked_with_streaming_logs_emits_output_before_process_exit() {
        let (install_log, events) = test_install_log_emitter();
        let worker = std::thread::spawn(move || {
            let mut command = std::process::Command::new("powershell");
            command.args([
                "-NoProfile",
                "-Command",
                "[Console]::Out.WriteLine('stream-start'); [Console]::Out.Flush(); Start-Sleep -Milliseconds 1000; [Console]::Error.WriteLine('stream-end'); [Console]::Error.Flush()",
            ]);

            run_command_checked_with_streaming_logs_for_command(
                &install_log,
                "installing",
                &mut command,
                "streaming command failed",
            )
        });

        std::thread::sleep(std::time::Duration::from_millis(600));

        let saw_early_stdout = {
            let events = events.lock().unwrap();
            events.iter().any(|event| {
                event.kind == InstallLogKind::Output
                    && event.level == InstallLogLevel::Stdout
                    && event.line.contains("stream-start")
            })
        };

        assert!(
            saw_early_stdout,
            "expected stdout to be emitted before the installer process finished"
        );
        assert!(
            !worker.is_finished(),
            "expected the process to still be running when the first line was streamed"
        );

        worker.join().unwrap().unwrap();

        let events = events.lock().unwrap();
        assert!(events.iter().any(|event| {
            event.kind == InstallLogKind::Output
                && event.level == InstallLogLevel::Stderr
                && event.line.contains("stream-end")
        }));
        assert!(events.iter().any(|event| {
            event.kind == InstallLogKind::Output
                && event.level == InstallLogLevel::Success
                && event.line == "Command completed"
        }));
    }

    #[test]
    fn resolve_managed_npm_command_prefers_npm_cmd() {
        let tmp = tempfile::tempdir().unwrap();
        let nodejs_dir = tmp.path().join("nodejs");
        std::fs::create_dir_all(&nodejs_dir).unwrap();
        std::fs::write(nodejs_dir.join("npm.cmd"), b"").unwrap();
        std::fs::write(nodejs_dir.join("node.exe"), b"").unwrap();

        let resolved = resolve_managed_npm_command(tmp.path());

        assert_eq!(
            resolved,
            Some(ManagedNpmCommand {
                program: nodejs_dir.join("npm.cmd"),
                prefix_args: Vec::new(),
            })
        );
    }

    #[test]
    fn resolve_managed_npm_command_falls_back_to_node_plus_npm_cli() {
        let tmp = tempfile::tempdir().unwrap();
        let nodejs_dir = tmp.path().join("nodejs");
        let npm_cli = nodejs_dir.join("node_modules").join("npm").join("bin");
        std::fs::create_dir_all(&npm_cli).unwrap();
        std::fs::write(nodejs_dir.join("node.exe"), b"").unwrap();
        std::fs::write(npm_cli.join("npm-cli.js"), b"").unwrap();

        let resolved = resolve_managed_npm_command(tmp.path());

        assert_eq!(
            resolved,
            Some(ManagedNpmCommand {
                program: nodejs_dir.join("node.exe"),
                prefix_args: vec![npm_cli.join("npm-cli.js").to_string_lossy().to_string()],
            })
        );
    }

    #[test]
    fn resolve_npm_command_from_node_dir_prefers_adjacent_npm_cmd() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("npm.cmd"), b"").unwrap();
        std::fs::write(tmp.path().join("node.exe"), b"").unwrap();

        let resolved = resolve_npm_command_from_node_dir(tmp.path());

        assert_eq!(
            resolved,
            Some(ManagedNpmCommand {
                program: tmp.path().join("npm.cmd"),
                prefix_args: Vec::new(),
            })
        );
    }

    #[test]
    fn resolve_npm_command_from_node_dir_uses_node_plus_local_npm_cli() {
        let tmp = tempfile::tempdir().unwrap();
        let npm_cli = tmp.path().join("node_modules").join("npm").join("bin");
        std::fs::create_dir_all(&npm_cli).unwrap();
        std::fs::write(tmp.path().join("node.exe"), b"").unwrap();
        std::fs::write(npm_cli.join("npm-cli.js"), b"").unwrap();

        let resolved = resolve_npm_command_from_node_dir(tmp.path());

        assert_eq!(
            resolved,
            Some(ManagedNpmCommand {
                program: tmp.path().join("node.exe"),
                prefix_args: vec![npm_cli.join("npm-cli.js").to_string_lossy().to_string()],
            })
        );
    }

    #[test]
    fn resolve_npm_command_for_uninstall_prefers_the_install_directory_itself() {
        let tmp = tempfile::tempdir().unwrap();
        let npm_cli = tmp.path().join("node_modules").join("npm").join("bin");
        std::fs::create_dir_all(&npm_cli).unwrap();
        std::fs::write(tmp.path().join("node.exe"), b"").unwrap();
        std::fs::write(npm_cli.join("npm-cli.js"), b"").unwrap();

        let resolved = resolve_npm_command_for_uninstall(tmp.path());

        assert_eq!(
            resolved,
            Some(ManagedNpmCommand {
                program: tmp.path().join("node.exe"),
                prefix_args: vec![npm_cli.join("npm-cli.js").to_string_lossy().to_string()],
            })
        );
    }

    #[test]
    fn find_local_uninstaller_executable_finds_common_uninstall_names() {
        let tmp = tempfile::tempdir().unwrap();
        let uninstall = tmp.path().join("unins000.exe");
        std::fs::write(&uninstall, b"").unwrap();

        assert_eq!(
            find_local_uninstaller_executable(tmp.path()).as_deref(),
            Some(uninstall.as_path())
        );
    }

    #[test]
    fn find_local_uninstaller_executable_finds_named_uninstall_exe() {
        let tmp = tempfile::tempdir().unwrap();
        let uninstall = tmp.path().join("Uninstall OpenCode.exe");
        std::fs::write(&uninstall, b"").unwrap();

        assert_eq!(
            find_local_uninstaller_executable(tmp.path()).as_deref(),
            Some(uninstall.as_path())
        );
    }
}
