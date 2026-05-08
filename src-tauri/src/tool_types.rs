//! AgenticBoot 工具管理核心数据类型
//!
//! 定义工具安装/卸载/检测相关的所有共享数据结构。

use serde::{Deserialize, Serialize};

/// 网络连通性检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatus {
    pub github_reachable: bool,
    pub npm_reachable: bool,
    pub error_message: Option<String>,
}

/// 工具元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolMeta {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub category: String,
}

/// 工具检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectResult {
    pub installed: bool,
    pub version: Option<String>,
    pub install_path: Option<String>,
}

/// 工具依赖声明
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDependency {
    pub tool_id: String,
    pub min_version: Option<String>,
}

/// 安装进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallProgress {
    pub tool_id: String,
    pub tool_name: String,
    pub phase: String,
    pub percent: u8,
    pub message: String,
}

/// 安装计划
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPlan {
    pub steps: Vec<InstallStep>,
}

/// 安装步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallStep {
    pub tool_id: String,
    pub tool_name: String,
    pub category: String,
    pub reason: String,
    pub is_installed: bool,
}

/// 已安装工具信息（前端展示用）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledTool {
    pub id: String,
    pub name: String,
    pub version: Option<String>,
    pub install_path: String,
    pub install_root: String,
    pub category: String,
    pub status: String,
    pub installed_at: Option<i64>,
    pub updated_at: Option<i64>,
}

/// 工具更新信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolUpdateInfo {
    pub tool_id: String,
    pub current_version: String,
    pub latest_version: String,
}
