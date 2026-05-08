//! ToolPlugin trait 定义和插件注册表
//!
//! 定义 AI 编程工具的安装/卸载/检测生命周期接口。
//! 所有可安装的工具都必须实现此 trait 并注册到 get_all_plugins()。

use crate::tool_types::{DetectResult, InstallProgress, ToolDependency, ToolMeta};
use std::path::Path;
use tokio::sync::mpsc::Sender;

/// 工具插件 trait — 定义工具的完整生命周期
pub trait ToolPlugin: Send + Sync {
    /// 返回工具元信息
    fn metadata(&self) -> ToolMeta;

    /// 检测系统上是否已安装该工具
    fn detect(&self) -> DetectResult;

    /// 安装工具到指定目录
    ///
    /// # Arguments
    /// * `target_dir` - 工具专属子目录（如 `<root>/claude-code-cli/`）
    /// * `progress` - 进度上报通道
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
///
/// 包含依赖项（Node.js、Git）、CLI 版工具和桌面版工具。
/// 新增工具只需在此函数中添加对应的 Box::new() 即可。
pub fn get_all_plugins() -> Vec<Box<dyn ToolPlugin>> {
    vec![
        // 依赖项（占位，具体实现在后续任务中完成）
        // Box::new(NodeJsPlugin),
        // Box::new(GitPlugin),
        // CLI 版工具
        // Box::new(ClaudeCodeCliPlugin),
        // Box::new(CodexCliPlugin),
        // Box::new(GeminiCliPlugin),
        // Box::new(OpenCodeCliPlugin),
        // Box::new(OpenClawPlugin),
        // Box::new(HermesPlugin),
        // 桌面版工具
        // Box::new(ClaudeCodeDesktopPlugin),
        // Box::new(CodexDesktopPlugin),
        // Box::new(OpenCodeDesktopPlugin),
    ]
}

/// 按 ID 查找插件
pub fn get_plugin_by_id(id: &str) -> Option<Box<dyn ToolPlugin>> {
    get_all_plugins().into_iter().find(|p| p.metadata().id == id)
}
