use crate::plugin::ToolPlugin;
use crate::services::installer::windows::{
    find_command_on_path, find_managed_paths, npm_prefix_candidates, read_command_version,
};
use crate::tool_types::{
    DetectResult, InstallProgress, InstallStrategy, ToolDependency, ToolMeta,
};
use std::path::Path;
use std::process::Command;
use tokio::sync::mpsc::Sender;

pub struct OpenClawPlugin;

impl ToolPlugin for OpenClawPlugin {
    fn metadata(&self) -> ToolMeta {
        ToolMeta {
            id: "openclaw".into(),
            name: "OpenClaw".into(),
            description: "可编程 AI 编码引擎".into(),
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

        if let Ok(output) = Command::new("openclaw").arg("--version").output() {
            if output.status.success() {
                return DetectResult {
                    installed: true,
                    version: Some(String::from_utf8_lossy(&output.stdout).trim().to_string()),
                    install_path: find_command_on_path("openclaw")
                        .and_then(|path| path.parent().map(|dir| dir.to_string_lossy().to_string())),
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
    fn install(&self, _target_dir: &Path, progress: Sender<InstallProgress>) -> Result<(), String> {
        let _ = progress.blocking_send(InstallProgress {
            tool_id: "openclaw".into(),
            tool_name: "OpenClaw".into(),
            phase: "installing".into(),
            percent: 0,
            message: "正在运行 OpenClaw 官方 PowerShell 安装脚本...".into(),
        });

        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                "& ([scriptblock]::Create((Invoke-RestMethod https://openclaw.ai/install.ps1))) -NoOnboard",
            ])
            .output()
            .map_err(|e| format!("启动 OpenClaw 安装脚本失败: {e}"))?;
        if !output.status.success() {
            return Err(format!(
                "OpenClaw 安装脚本失败: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    fn install(&self, _target_dir: &Path, _progress: Sender<InstallProgress>) -> Result<(), String> {
        Err("OpenClaw 自动安装目前仅支持 Windows".into())
    }

    fn uninstall(&self, target_dir: &Path) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            if target_dir.join("node_modules").exists() {
                let output = Command::new("npm")
                    .args([
                        "uninstall",
                        "-g",
                        "openclaw",
                        "--prefix",
                        &target_dir.to_string_lossy(),
                    ])
                    .output()
                    .map_err(|e| format!("npm uninstall 失败: {e}"))?;
                if !output.status.success() {
                    return Err(format!(
                        "npm uninstall 失败: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ));
                }
                if target_dir.exists() {
                    std::fs::remove_dir_all(target_dir)
                        .map_err(|e| format!("删除失败: {e}"))?;
                }
                return Ok(());
            }

            return Err("OpenClaw 官方脚本安装当前不支持自动卸载，请手动执行上游卸载流程。".into());
        }

        #[allow(unreachable_code)]
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::OpenClawPlugin;
    use crate::plugin::ToolPlugin;
    use crate::tool_types::InstallStrategy;

    #[test]
    fn native_windows_openclaw_uses_official_script_strategy() {
        assert_eq!(OpenClawPlugin.install_strategy(), InstallStrategy::OfficialScript);
    }
}
