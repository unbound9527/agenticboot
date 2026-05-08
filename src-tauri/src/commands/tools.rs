//! AgenticBoot 工具管理 Tauri 命令
//!
//! 前端通过 invoke 调用这些命令来管理 AI 编程工具的安装和卸载。

use crate::services::installer::dependency_resolver::resolve_install_plan as resolve_plan;
use crate::services::installer::InstallerService;
use crate::store::AppState;
use crate::tool_types::{InstallPlan, InstalledTool, NetworkStatus, ToolUpdateInfo};
use std::path::Path;

/// 检测网络连通性
#[tauri::command]
pub async fn check_network() -> Result<NetworkStatus, String> {
    Ok(InstallerService::check_network().await)
}

/// 解析安装计划（传入安装根目录用于检测已有安装）
#[tauri::command]
pub fn resolve_install_plan(tool_ids: Vec<String>, install_root: Option<String>) -> Result<InstallPlan, String> {
    let root = install_root.as_deref().map(Path::new);
    resolve_plan(&tool_ids, root)
}

/// 执行安装计划
#[tauri::command]
pub async fn execute_install_plan(
    root_path: String,
    _app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let _service = InstallerService::new(Path::new(&root_path));
    state
        .db
        .set_install_root(&root_path)
        .map_err(|e| format!("保存安装根目录失败: {e}"))?;

    Err("此命令仅保存安装根目录，请使用 execute_install_plan_with_plan 传入完整安装计划".to_string())
}

/// 执行安装计划（带完整计划参数）
#[tauri::command]
pub async fn execute_install_plan_with_plan(
    plan: InstallPlan,
    root_path: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let service = InstallerService::new(Path::new(&root_path));
    state
        .db
        .set_install_root(&root_path)
        .map_err(|e| format!("保存安装根目录失败: {e}"))?;
    service
        .execute_install_plan(&plan, &app_handle, &state.db)
        .await
}

/// 卸载工具
#[tauri::command]
pub fn uninstall_tool(
    tool_id: String,
    root_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let service = InstallerService::new(Path::new(&root_path));
    service.uninstall_tool(&tool_id, &state.db)
}

/// 获取所有已安装工具
#[tauri::command]
pub fn get_installed_tools(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<InstalledTool>, String> {
    let records = state
        .db
        .get_installed_tools()
        .map_err(|e| format!("查询已安装工具失败: {e}"))?;

    Ok(records
        .into_iter()
        .map(|r| InstalledTool {
            id: r.id,
            name: r.name,
            version: r.version,
            install_path: r.install_path,
            install_root: r.install_root,
            category: r.category,
            status: r.status,
            installed_at: r.installed_at,
            updated_at: r.updated_at,
        })
        .collect())
}

/// 是否有任何已安装工具
#[tauri::command]
pub fn has_any_installed_tools(
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    state
        .db
        .has_any_installed_tools()
        .map_err(|e| format!("查询失败: {e}"))
}

/// 设置安装根目录
#[tauri::command]
pub fn set_install_root(
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state
        .db
        .set_install_root(&path)
        .map_err(|e| format!("保存安装根目录失败: {e}"))
}

/// 获取安装根目录
#[tauri::command]
pub fn get_install_root(
    state: tauri::State<'_, AppState>,
) -> Result<Option<String>, String> {
    state
        .db
        .get_install_root()
        .map_err(|e| format!("查询失败: {e}"))
}

/// 检查工具更新
#[tauri::command]
pub fn check_tool_updates(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ToolUpdateInfo>, String> {
    InstallerService::check_tool_updates(&state.db)
}
