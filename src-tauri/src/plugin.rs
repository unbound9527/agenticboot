//! ToolPlugin trait 定义和插件注册表

use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use tokio::sync::mpsc::Sender;

/// 工具插件 trait — 定义工具的完整生命周期
pub trait ToolPlugin: Send + Sync {
    /// 返回工具元信息
    fn metadata(&self) -> ToolMeta;

    /// 检测工具是否已安装
    ///
    /// `install_root` — 自定义安装根目录，用于检查自定义路径下的安装。
    /// CLI 工具还会检查系统 PATH。
    fn detect(&self, install_root: Option<&Path>) -> DetectResult;

    /// 安装工具到指定目录
    fn install(
        &self,
        target_dir: &Path,
        progress: Sender<InstallProgress>,
    ) -> Result<(), String>;

    /// 从指定目录卸载工具
    fn uninstall(&self, target_dir: &Path) -> Result<(), String>;

    /// 返回该工具依赖的其他工具 ID 列表
    fn dependencies(&self) -> Vec<ToolDependency>;
}

/// 获取所有已注册的工具插件
pub fn get_all_plugins() -> Vec<Box<dyn ToolPlugin>> {
    vec![
        Box::new(crate::plugins::nodejs::NodeJsPlugin),
        Box::new(crate::plugins::git::GitPlugin),
        Box::new(crate::plugins::claude_code_cli::ClaudeCodeCliPlugin),
        Box::new(crate::plugins::codex_cli::CodexCliPlugin),
        Box::new(crate::plugins::gemini_cli::GeminiCliPlugin),
        Box::new(crate::plugins::opencode_cli::OpenCodeCliPlugin),
        Box::new(crate::plugins::openclaw::OpenClawPlugin),
        Box::new(crate::plugins::hermes::HermesPlugin),
        Box::new(crate::plugins::claude_code_desktop::ClaudeCodeDesktopPlugin),
        Box::new(crate::plugins::codex_desktop::CodexDesktopPlugin),
        Box::new(crate::plugins::opencode_desktop::OpenCodeDesktopPlugin),
    ]
}

/// 按 ID 查找插件
pub fn get_plugin_by_id(id: &str) -> Option<Box<dyn ToolPlugin>> {
    get_all_plugins().into_iter().find(|p| p.metadata().id == id)
}
