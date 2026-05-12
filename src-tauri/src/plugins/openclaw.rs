use crate::plugin::ToolPlugin;
use crate::plugins::npm_cli::{detect_npm_cli, uninstall_npm_cli};
use crate::tool_types::{
    DetectResult, InstallLogLevel, InstallProgress, InstallStrategy, ToolDependency, ToolMeta,
};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct OpenClawPlugin;

const OPENCLAW_SOURCE_ZIP_URL: &str =
    "https://github.com/openclaw/openclaw/archive/refs/heads/main.zip";
const OPENCLAW_SOURCE_ZIP_NAME: &str = "openclaw-source.zip";
const OPENCLAW_SOURCE_DIR_NAME: &str = "openclaw-source";

fn emit_openclaw_progress(
    progress: &Sender<InstallProgress>,
    phase: &str,
    percent: u8,
    message: &str,
) {
    let _ = progress.blocking_send(InstallProgress {
        tool_id: "openclaw".into(),
        tool_name: "OpenClaw".into(),
        phase: phase.into(),
        percent,
        message: message.into(),
    });
}

fn managed_node_dir(install_root: &Path) -> Option<PathBuf> {
    let node_dir = install_root.join("nodejs");
    (node_dir.join("node.exe").exists() && node_dir.join("corepack.cmd").exists())
        .then_some(node_dir)
}

fn build_path_with_managed_node(install_root: &Path) -> Option<OsString> {
    let managed_node = managed_node_dir(install_root)?;
    let current_path = std::env::var_os("PATH").unwrap_or_default();
    let mut combined = Vec::with_capacity(1 + std::env::split_paths(&current_path).count());
    combined.push(managed_node);
    combined.extend(std::env::split_paths(&current_path));
    std::env::join_paths(combined).ok()
}

fn emit_openclaw_output(phase: &str, level: InstallLogLevel, text: &[u8]) {
    let _ = (phase, level, text);
}

fn emit_openclaw_output_with_log(
    install_log: Option<&crate::services::installer::logging::InstallLogEmitter>,
    phase: &str,
    level: InstallLogLevel,
    text: &[u8],
) {
    if let Some(install_log) = install_log {
        for line in String::from_utf8_lossy(text)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
        {
            install_log.emit_output(phase, level, line.to_string());
        }
    } else {
        emit_openclaw_output(phase, level, text);
    }
}

fn format_command_for_log(program: &Path, args: &[String]) -> String {
    let mut parts = vec![quote_for_cmd(program.to_string_lossy().as_ref())];
    parts.extend(args.iter().map(|arg| quote_for_cmd(arg)));
    parts.join(" ")
}

fn quote_for_cmd(text: &str) -> String {
    if text.contains(' ') || text.contains('\t') || text.contains('"') {
        format!("\"{}\"", text.replace('"', "\\\""))
    } else {
        text.to_string()
    }
}

fn run_openclaw_command(
    install_root: &Path,
    program: &Path,
    args: &[String],
    current_dir: &Path,
    phase: &str,
    context: &str,
    install_log: Option<&crate::services::installer::logging::InstallLogEmitter>,
) -> Result<(), String> {
    let mut command = Command::new(program);
    command.args(args);
    command.current_dir(current_dir);
    command.env("NPM_CONFIG_SCRIPT_SHELL", "cmd.exe");

    if let Some(path_with_managed_node) = build_path_with_managed_node(install_root) {
        command.env("PATH", path_with_managed_node);
    }

    if let Some(install_log) = install_log {
        install_log.emit_command(phase, format_command_for_log(program, args));
    }

    let output = command
        .output()
        .map_err(|e| format!("{context} failed: {e}"))?;
    emit_openclaw_output_with_log(install_log, phase, InstallLogLevel::Stdout, &output.stdout);
    emit_openclaw_output_with_log(install_log, phase, InstallLogLevel::Stderr, &output.stderr);

    if output.status.success() {
        if let Some(install_log) = install_log {
            install_log.emit_output(phase, InstallLogLevel::Success, "Command completed");
        }
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        Err(format!(
            "{context} failed: exit code {:?}",
            output.status.code()
        ))
    } else {
        Err(format!("{context} failed: {stderr}"))
    }
}

fn find_openclaw_source_root(checkout_dir: &Path) -> Result<PathBuf, String> {
    let mut roots: Vec<PathBuf> = std::fs::read_dir(checkout_dir)
        .map_err(|e| format!("failed to read OpenClaw checkout: {e}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter(|path| path.join("package.json").is_file() && path.join("openclaw.mjs").is_file())
        .collect();

    match roots.len() {
        0 => Err(
            "OpenClaw source archive did not contain a project root with package.json and openclaw.mjs"
                .into(),
        ),
        1 => Ok(roots.remove(0)),
        _ => Err("OpenClaw source archive contained multiple candidate project roots".into()),
    }
}

fn write_managed_openclaw_wrapper(
    target_dir: &Path,
    install_root: &Path,
    source_root: &Path,
) -> Result<PathBuf, String> {
    std::fs::create_dir_all(target_dir)
        .map_err(|e| format!("failed to create OpenClaw target dir: {e}"))?;

    let node_dir = managed_node_dir(install_root)
        .ok_or_else(|| "managed Node.js runtime is missing node.exe/corepack.cmd".to_string())?;
    let node_exe = node_dir.join("node.exe");
    let launcher = source_root.join("openclaw.mjs");
    if !launcher.exists() {
        return Err(format!(
            "OpenClaw source build is missing launcher {}",
            launcher.display()
        ));
    }

    let wrapper = target_dir.join("openclaw.cmd");
    let contents = format!(
        "@echo off\r\n\"{}\" \"{}\" %*\r\n",
        node_exe.display(),
        launcher.display()
    );
    std::fs::write(&wrapper, contents)
        .map_err(|e| format!("failed to write managed OpenClaw wrapper: {e}"))?;
    Ok(wrapper)
}

fn install_openclaw_from_source_archive(
    target_dir: &Path,
    install_root: &Path,
    progress: &Sender<InstallProgress>,
    install_log: Option<&crate::services::installer::logging::InstallLogEmitter>,
) -> Result<(), String> {
    let node_dir = managed_node_dir(install_root).ok_or_else(|| {
        "managed Node.js runtime is required before installing OpenClaw".to_string()
    })?;
    let corepack = node_dir.join("corepack.cmd");
    if !corepack.exists() {
        return Err(format!(
            "managed Node.js runtime is missing {}",
            corepack.display()
        ));
    }

    emit_openclaw_progress(
        progress,
        "downloading",
        18,
        "Downloading OpenClaw source archive...",
    );
    if let Some(install_log) = install_log {
        install_log.emit_phase(
            "downloading",
            "Downloading OpenClaw source archive from GitHub",
        );
        install_log.emit_output(
            "downloading",
            InstallLogLevel::Info,
            format!("Using managed Node.js from {}", node_dir.display()),
        );
    }

    if target_dir.exists() {
        std::fs::remove_dir_all(target_dir)
            .map_err(|e| format!("failed to clear managed OpenClaw directory: {e}"))?;
    }
    std::fs::create_dir_all(target_dir)
        .map_err(|e| format!("failed to create managed OpenClaw directory: {e}"))?;

    let source_checkout_dir = target_dir.join(OPENCLAW_SOURCE_DIR_NAME);
    let zip_path = target_dir.join(OPENCLAW_SOURCE_ZIP_NAME);

    let runtime =
        tokio::runtime::Runtime::new().map_err(|e| format!("failed to create runtime: {e}"))?;
    runtime.block_on(async {
        crate::services::downloader::download_file(OPENCLAW_SOURCE_ZIP_URL, &zip_path, None).await
    })?;

    emit_openclaw_progress(
        progress,
        "extracting",
        30,
        "Extracting OpenClaw source archive...",
    );
    if let Some(install_log) = install_log {
        install_log.emit_phase("extracting", "Extracting OpenClaw source archive");
    }

    std::fs::create_dir_all(&source_checkout_dir)
        .map_err(|e| format!("failed to create OpenClaw source dir: {e}"))?;
    crate::services::downloader::extract_zip(&zip_path, &source_checkout_dir)?;
    std::fs::remove_file(&zip_path).ok();

    let source_root = find_openclaw_source_root(&source_checkout_dir)?;

    emit_openclaw_progress(progress, "installing", 45, "Preparing pnpm via corepack...");
    run_openclaw_command(
        install_root,
        &corepack,
        &["pnpm".to_string(), "--version".to_string()],
        &source_root,
        "installing",
        "prepare pnpm",
        install_log,
    )?;

    emit_openclaw_progress(
        progress,
        "installing",
        55,
        "Installing OpenClaw source dependencies...",
    );
    run_openclaw_command(
        install_root,
        &corepack,
        &[
            "pnpm".to_string(),
            "-C".to_string(),
            source_root.to_string_lossy().to_string(),
            "install".to_string(),
        ],
        &source_root,
        "installing",
        "install OpenClaw source dependencies",
        install_log,
    )?;

    emit_openclaw_progress(progress, "installing", 72, "Building OpenClaw UI...");
    let ui_build_result = run_openclaw_command(
        install_root,
        &corepack,
        &[
            "pnpm".to_string(),
            "-C".to_string(),
            source_root.to_string_lossy().to_string(),
            "ui:build".to_string(),
        ],
        &source_root,
        "installing",
        "build OpenClaw UI",
        install_log,
    );
    if let Err(error) = ui_build_result {
        if let Some(install_log) = install_log {
            install_log.emit_output(
                "installing",
                InstallLogLevel::Info,
                format!("{error}; continuing because CLI may still work"),
            );
        }
    }

    emit_openclaw_progress(progress, "installing", 82, "Building OpenClaw CLI...");
    run_openclaw_command(
        install_root,
        &corepack,
        &[
            "pnpm".to_string(),
            "-C".to_string(),
            source_root.to_string_lossy().to_string(),
            "build".to_string(),
        ],
        &source_root,
        "installing",
        "build OpenClaw CLI",
        install_log,
    )?;

    emit_openclaw_progress(
        progress,
        "configuring",
        92,
        "Writing managed OpenClaw launcher...",
    );
    let wrapper = write_managed_openclaw_wrapper(target_dir, install_root, &source_root)?;
    run_openclaw_command(
        install_root,
        &wrapper,
        &["--version".to_string()],
        target_dir,
        "configuring",
        "verify managed OpenClaw launcher",
        install_log,
    )?;

    emit_openclaw_progress(progress, "complete", 100, "OpenClaw install complete");
    Ok(())
}

impl ToolPlugin for OpenClawPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "openclaw".into(),
            name: "OpenClaw".into(),
            description: "Programmable AI coding engine".into(),
            icon: "openclaw".into(),
            category: "ai-cli".into(),
        }
    }

    fn install_strategy(&self) -> InstallStrategy {
        InstallStrategy::ManagedPrefix
    }

    fn detect(&self, install_root: Option<&Path>) -> DetectResult {
        detect_npm_cli(install_root, "openclaw", "openclaw", "OpenClaw")
    }

    fn dependencies(&self) -> Vec<ToolDependency> {
        vec![
            ToolDependency {
                tool_id: "nodejs".into(),
                min_version: Some(">= 22.16.0".into()),
            },
            ToolDependency {
                tool_id: "git".into(),
                min_version: None,
            },
        ]
    }

    #[cfg(target_os = "windows")]
    fn install(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        install_openclaw_from_source_archive(target_dir, install_root, &progress, None)
    }

    #[cfg(target_os = "windows")]
    fn install_with_context(
        &self,
        target_dir: &Path,
        install_root: &Path,
        progress: Sender<InstallProgress>,
        context: crate::plugin::ToolInstallContext,
    ) -> Result<(), String> {
        install_openclaw_from_source_archive(
            target_dir,
            install_root,
            &progress,
            Some(context.install_log()),
        )
    }

    #[cfg(not(target_os = "windows"))]
    fn install(
        &self,
        _target_dir: &Path,
        _install_root: &Path,
        _progress: Sender<InstallProgress>,
    ) -> Result<(), String> {
        Err("OpenClaw auto-install is currently supported only on Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        if target_dir.join("openclaw.cmd").exists()
            && target_dir.join(OPENCLAW_SOURCE_DIR_NAME).exists()
        {
            return Ok(());
        }
        uninstall_npm_cli(target_dir, "openclaw")
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_path_with_managed_node, find_openclaw_source_root, managed_node_dir,
        write_managed_openclaw_wrapper, OpenClawPlugin, OPENCLAW_SOURCE_DIR_NAME,
    };
    use crate::plugin::ToolPlugin;
    use crate::tool_types::InstallStrategy;

    #[test]
    fn native_windows_openclaw_uses_managed_prefix_strategy() {
        assert_eq!(
            OpenClawPlugin.install_strategy(),
            InstallStrategy::ManagedPrefix
        );
    }

    #[test]
    fn openclaw_uses_managed_node_when_available() {
        let tmp = tempfile::tempdir().unwrap();
        let node_dir = tmp.path().join("nodejs");
        std::fs::create_dir_all(&node_dir).unwrap();
        std::fs::write(node_dir.join("node.exe"), b"").unwrap();
        std::fs::write(node_dir.join("corepack.cmd"), b"").unwrap();

        let managed_dir = managed_node_dir(tmp.path()).expect("managed node dir");
        let path = build_path_with_managed_node(tmp.path()).expect("path");
        let first_entry = std::env::split_paths(&path)
            .next()
            .expect("first path entry");

        assert_eq!(managed_dir, node_dir);
        assert_eq!(first_entry, node_dir);
    }

    #[test]
    fn openclaw_source_root_finds_github_archive_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("openclaw-main");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("package.json"), "{}").unwrap();
        std::fs::write(root.join("openclaw.mjs"), "").unwrap();

        let resolved = find_openclaw_source_root(tmp.path()).unwrap();
        assert_eq!(resolved, root);
    }

    #[test]
    fn openclaw_wrapper_uses_managed_node_and_source_launcher() {
        let tmp = tempfile::tempdir().unwrap();
        let install_root = tmp.path().join("root");
        let target_dir = install_root.join("openclaw");
        let source_root = target_dir
            .join(OPENCLAW_SOURCE_DIR_NAME)
            .join("openclaw-main");
        let node_dir = install_root.join("nodejs");

        std::fs::create_dir_all(&source_root).unwrap();
        std::fs::create_dir_all(&node_dir).unwrap();
        std::fs::write(node_dir.join("node.exe"), b"").unwrap();
        std::fs::write(node_dir.join("corepack.cmd"), b"").unwrap();
        std::fs::write(source_root.join("openclaw.mjs"), "").unwrap();

        let wrapper =
            write_managed_openclaw_wrapper(&target_dir, &install_root, &source_root).unwrap();
        let contents = std::fs::read_to_string(wrapper).unwrap();

        assert!(contents.contains("node.exe"));
        assert!(contents.contains("openclaw.mjs"));
    }

    #[test]
    fn native_windows_openclaw_detects_existing_managed_windows_command() {
        let tmp = tempfile::tempdir().unwrap();
        let tool_dir = tmp.path().join("openclaw");
        std::fs::create_dir_all(&tool_dir).unwrap();
        std::fs::write(
            tool_dir.join("openclaw.cmd"),
            "@echo off\r\necho openclaw 1.2.3\r\n",
        )
        .unwrap();

        let detect = OpenClawPlugin.detect(Some(tmp.path()));
        assert!(detect.installed);
        assert_eq!(detect.version.as_deref(), Some("openclaw 1.2.3"));
        assert_eq!(
            detect.install_path.as_deref(),
            Some(tool_dir.to_string_lossy().as_ref())
        );
    }

    #[test]
    fn native_windows_openclaw_declares_node_and_git_dependencies() {
        let deps = OpenClawPlugin.dependencies();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].tool_id, "nodejs");
        assert_eq!(deps[0].min_version.as_deref(), Some(">= 22.16.0"));
        assert_eq!(deps[1].tool_id, "git");
        assert_eq!(deps[1].min_version, None);
    }
}
