//! AgenticBoot 工具管理核心数据类型
//!
//! 定义工具安装/卸载/检测相关的所有共享数据结构。

use chrono::SecondsFormat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStrategy {
    ManagedPrefix,
    GlobalNpm,
    OfficialScript,
    PythonPackage,
    DesktopInstaller,
}

/// 网络连通性检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatus {
    pub github_reachable: bool,
    pub npm_reachable: bool,
    pub youtube_reachable: bool,
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

impl DetectResult {
    pub fn not_installed() -> Self {
        DetectResult {
            installed: false,
            version: None,
            install_path: None,
        }
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallLogLevel {
    Info,
    Stdout,
    Stderr,
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallLogKind {
    SessionStarted,
    Phase,
    Command,
    Output,
    Result,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallLogEvent {
    pub tool_id: String,
    pub tool_name: String,
    pub session_id: String,
    pub timestamp: String,
    pub phase: Option<String>,
    pub level: InstallLogLevel,
    pub kind: InstallLogKind,
    pub line: String,
    pub command: Option<String>,
    pub exit_code: Option<i32>,
}

#[allow(dead_code)]
impl InstallLogEvent {
    fn timestamp_now() -> String {
        chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
    }

    pub fn session_started(
        tool_id: impl Into<String>,
        tool_name: impl Into<String>,
        session_id: impl Into<String>,
    ) -> Self {
        Self {
            tool_id: tool_id.into(),
            tool_name: tool_name.into(),
            session_id: session_id.into(),
            timestamp: Self::timestamp_now(),
            phase: None,
            level: InstallLogLevel::Info,
            kind: InstallLogKind::SessionStarted,
            line: "Install session started".to_string(),
            command: None,
            exit_code: None,
        }
    }

    pub fn phase(
        tool_id: impl Into<String>,
        tool_name: impl Into<String>,
        session_id: impl Into<String>,
        phase: impl Into<String>,
        line: impl Into<String>,
    ) -> Self {
        Self {
            tool_id: tool_id.into(),
            tool_name: tool_name.into(),
            session_id: session_id.into(),
            timestamp: Self::timestamp_now(),
            phase: Some(phase.into()),
            level: InstallLogLevel::Info,
            kind: InstallLogKind::Phase,
            line: line.into(),
            command: None,
            exit_code: None,
        }
    }

    pub fn command(
        tool_id: impl Into<String>,
        tool_name: impl Into<String>,
        session_id: impl Into<String>,
        phase: impl Into<String>,
        command: impl Into<String>,
    ) -> Self {
        let command = command.into();
        Self {
            tool_id: tool_id.into(),
            tool_name: tool_name.into(),
            session_id: session_id.into(),
            timestamp: Self::timestamp_now(),
            phase: Some(phase.into()),
            level: InstallLogLevel::Info,
            kind: InstallLogKind::Command,
            line: command.clone(),
            command: Some(command),
            exit_code: None,
        }
    }

    pub fn output(
        tool_id: impl Into<String>,
        tool_name: impl Into<String>,
        session_id: impl Into<String>,
        phase: impl Into<String>,
        level: InstallLogLevel,
        line: impl Into<String>,
    ) -> Self {
        Self {
            tool_id: tool_id.into(),
            tool_name: tool_name.into(),
            session_id: session_id.into(),
            timestamp: Self::timestamp_now(),
            phase: Some(phase.into()),
            level,
            kind: InstallLogKind::Output,
            line: line.into(),
            command: None,
            exit_code: None,
        }
    }

    pub fn result(
        tool_id: impl Into<String>,
        tool_name: impl Into<String>,
        session_id: impl Into<String>,
        phase: impl Into<String>,
        line: impl Into<String>,
        exit_code: Option<i32>,
        success: bool,
    ) -> Self {
        Self {
            tool_id: tool_id.into(),
            tool_name: tool_name.into(),
            session_id: session_id.into(),
            timestamp: Self::timestamp_now(),
            phase: Some(phase.into()),
            level: if success {
                InstallLogLevel::Success
            } else {
                InstallLogLevel::Error
            },
            kind: InstallLogKind::Result,
            line: line.into(),
            command: None,
            exit_code,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{InstallLogEvent, InstallLogKind, InstallLogLevel};

    #[test]
    fn install_log_helper_marks_result_with_exit_code() {
        let event = InstallLogEvent::result(
            "codex-desktop",
            "Codex (Desktop)",
            "session-1",
            "installing",
            "winget install completed",
            Some(0),
            true,
        );

        assert_eq!(event.kind, InstallLogKind::Result);
        assert_eq!(event.exit_code, Some(0));
        assert_eq!(event.level, InstallLogLevel::Success);
    }
}
