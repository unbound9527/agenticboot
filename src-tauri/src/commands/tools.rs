//! AgenticBoot tool-management Tauri commands.

use crate::database::{Database, InstalledToolRecord};
use crate::services::installer::dependency_resolver::resolve_install_plan as resolve_plan;
use crate::services::installer::is_install_owned_by_root;
use crate::services::installer::InstallerService;
use crate::store::AppState;
use crate::tool_types::{
    DetectResult, InstallPlan, InstalledTool, ResolveProgress, ToolCatalogItem, ToolUpdateInfo,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use tauri::Emitter;

fn should_use_db_fallback(install_root: Option<&str>) -> bool {
    install_root.is_none()
}

fn cache_matches_install_root(record: &InstalledToolRecord, install_root: Option<&str>) -> bool {
    if record.state_source == "external_detected" {
        return true;
    }

    match install_root {
        Some(root) => record.install_root == root,
        None => true,
    }
}

fn can_reuse_detect_cache(record: &InstalledToolRecord, install_root: Option<&str>) -> bool {
    match record.status.as_str() {
        "installed" | "not_installed" => true,
        "detected" => {
            if record.state_source == "external_detected" {
                return false;
            }
            let Some(root) = install_root else {
                return false;
            };
            let install_path = record.install_path.trim();
            if install_path.is_empty() {
                return false;
            }
            is_install_owned_by_root(Path::new(root), Path::new(install_path))
        }
        _ => false,
    }
}

fn detect_result_from_cache_record(record: &InstalledToolRecord) -> Option<DetectResult> {
    match record.status.as_str() {
        "installed" | "detected" => Some(DetectResult {
            installed: true,
            version: record.version.clone(),
            install_path: Some(record.install_path.clone()),
        }),
        "not_installed" => Some(DetectResult::not_installed()),
        _ => None,
    }
}

fn cache_status_for_detect_result(
    previous: Option<&InstalledToolRecord>,
    result: &DetectResult,
) -> &'static str {
    if result.installed {
        if previous.is_some_and(|record| record.status == "installed") {
            "installed"
        } else {
            "detected"
        }
    } else {
        "not_installed"
    }
}

fn cache_state_source_for_detect_result(
    previous: Option<&InstalledToolRecord>,
    install_root: Option<&str>,
    result: &DetectResult,
) -> &'static str {
    if previous.is_some_and(|record| record.status == "installed") {
        return "managed";
    }

    let install_path = result.install_path.as_deref().unwrap_or_default();
    if let Some(root) = install_root {
        if !install_path.is_empty() && is_install_owned_by_root(Path::new(root), Path::new(install_path))
        {
            return "managed";
        }
    }

    "external_detected"
}

fn persist_detect_result_cache(
    db: &Arc<Database>,
    tool_id: &str,
    install_root: Option<&str>,
    previous: Option<&InstalledToolRecord>,
    result: &DetectResult,
) {
    let next_status = cache_status_for_detect_result(previous, result);
    let now = chrono::Utc::now().timestamp();
    let plugin_meta = crate::plugin::get_plugin_by_id(tool_id).map(|plugin| plugin.metadata());
    let state_source = if next_status == "not_installed" {
        previous
            .map(|record| record.state_source.as_str())
            .unwrap_or("managed")
    } else {
        cache_state_source_for_detect_result(previous, install_root, result)
    };

    let record = InstalledToolRecord {
        id: tool_id.to_string(),
        name: previous
            .map(|record| record.name.clone())
            .or_else(|| plugin_meta.as_ref().map(|meta| meta.name.clone()))
            .unwrap_or_else(|| tool_id.to_string()),
        version: result.version.clone(),
        install_path: result.install_path.clone().unwrap_or_default(),
        install_root: install_root
            .map(str::to_string)
            .or_else(|| previous.map(|record| record.install_root.clone()))
            .unwrap_or_default(),
        category: previous
            .map(|record| record.category.clone())
            .or_else(|| plugin_meta.as_ref().map(|meta| meta.category.clone()))
            .unwrap_or_else(|| "tool".to_string()),
        status: next_status.to_string(),
        state_source: state_source.to_string(),
        installed_at: previous.and_then(|record| record.installed_at),
        last_seen_at: result.installed.then_some(now),
        updated_at: Some(now),
    };

    if let Err(error) = db.upsert_installed_tool(&record) {
        log::warn!("failed to persist detect cache for {tool_id}: {error}");
    }
}

#[tauri::command]
pub fn get_tool_catalog() -> Vec<ToolCatalogItem> {
    crate::plugin::get_tool_catalog()
}

#[tauri::command]
pub fn resolve_install_plan(
    tool_ids: Vec<String>,
    install_root: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<InstallPlan, String> {
    for tool_id in &tool_ids {
        let tool_name = crate::plugin::get_plugin_by_id(tool_id)
            .map(|plugin| plugin.metadata().name)
            .unwrap_or_else(|| tool_id.clone());
        let _ = app_handle.emit(
            "resolve-progress",
            ResolveProgress {
                tool_id: tool_id.clone(),
                tool_name,
                phase: "resolving".to_string(),
                message: "Resolving install plan...".to_string(),
            },
        );
    }

    let root = install_root.as_deref().map(Path::new);
    let plan = resolve_plan(&tool_ids, root);

    match &plan {
        Ok(plan) => {
            for step in &plan.steps {
                let message = if step.is_installed {
                    "Already installed. Will skip.".to_string()
                } else if step.reason == "selected" {
                    "Resolved for installation.".to_string()
                } else {
                    format!("Resolved dependency: {}", step.reason)
                };
                let _ = app_handle.emit(
                    "resolve-progress",
                    ResolveProgress {
                        tool_id: step.tool_id.clone(),
                        tool_name: step.tool_name.clone(),
                        phase: "resolved".to_string(),
                        message,
                    },
                );
            }
            let _ = app_handle.emit("resolve-complete", ());
        }
        Err(error) => {
            for tool_id in &tool_ids {
                let tool_name = crate::plugin::get_plugin_by_id(tool_id)
                    .map(|plugin| plugin.metadata().name)
                    .unwrap_or_else(|| tool_id.clone());
                let _ = app_handle.emit(
                    "resolve-progress",
                    ResolveProgress {
                        tool_id: tool_id.clone(),
                        tool_name,
                        phase: "error".to_string(),
                        message: error.clone(),
                    },
                );
            }
            let _ = app_handle.emit("resolve-complete", ());
        }
    }

    plan
}

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

    Err(
        "此命令仅保存安装根目录，请使用 execute_install_plan_with_plan 传入完整安装计划"
            .to_string(),
    )
}

#[tauri::command]
pub async fn execute_install_plan_with_plan(
    plan: InstallPlan,
    root_path: String,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "[Install] 开始执行安装计划, root_path={}, plan_steps={}",
        root_path,
        plan.steps.len()
    );
    let service = InstallerService::new(Path::new(&root_path));
    state
        .db
        .set_install_root(&root_path)
        .map_err(|e| format!("保存安装根目录失败: {e}"))?;
    log::info!("[Install] 保存安装根目录完成: {}", root_path);
    service
        .execute_install_plan(&plan, &app_handle, &state.db)
        .await
}

#[tauri::command]
pub async fn uninstall_tool(
    tool_id: String,
    root_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        let service = InstallerService::new(Path::new(&root_path));
        service.uninstall_tool(&tool_id, &db)
    })
    .await
    .map_err(|e| format!("卸载任务执行失败: {e}"))?
}

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
        .filter(|r| matches!(r.status.as_str(), "installed" | "detected"))
        .map(|r| InstalledTool {
            id: r.id,
            name: r.name,
            version: r.version,
            install_path: r.install_path,
            install_root: r.install_root,
            category: r.category,
            status: r.status,
            state_source: r.state_source,
            installed_at: r.installed_at,
            last_seen_at: r.last_seen_at,
            updated_at: r.updated_at,
        })
        .collect())
}

#[tauri::command]
pub fn has_any_installed_tools(state: tauri::State<'_, AppState>) -> Result<bool, String> {
    state
        .db
        .has_any_installed_tools()
        .map_err(|e| format!("查询失败: {e}"))
}

#[tauri::command]
pub async fn detect_tools(
    tool_ids: Vec<String>,
    install_root: Option<String>,
    force_refresh: Option<bool>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<DetectResult>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        detect_tools_sync(tool_ids, install_root, force_refresh.unwrap_or(false), &db)
    })
    .await
    .map_err(|e| format!("检测工具任务执行失败: {e}"))?
}

fn detect_tools_sync(
    tool_ids: Vec<String>,
    install_root: Option<String>,
    force_refresh: bool,
    db: &Arc<Database>,
) -> Result<Vec<DetectResult>, String> {
    use crate::plugin::get_plugin_by_id;

    let root = install_root.as_ref().map(PathBuf::from);
    let allow_db_fallback = should_use_db_fallback(install_root.as_deref());
    let mut results = vec![DetectResult::not_installed(); tool_ids.len()];
    let mut pending = Vec::new();

    for (index, id) in tool_ids.iter().enumerate() {
        let cached_record = db.get_installed_tool(id).ok().flatten();

        if !force_refresh {
            if let Some(record) = cached_record.as_ref() {
                if cache_matches_install_root(record, install_root.as_deref()) {
                    if can_reuse_detect_cache(record, install_root.as_deref()) {
                        if let Some(cached) = detect_result_from_cache_record(record) {
                            results[index] = cached;
                            continue;
                        }
                    }
                }
            }
        }

        pending.push((index, id.clone(), cached_record));
    }

    let detected = std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(pending.len());

        for (index, id, cached_record) in pending {
            let root = root.clone();
            handles.push(scope.spawn(move || {
                log::info!("[Tool Detect] Starting detect for {id}");
                let started_at = Instant::now();
                let result = get_plugin_by_id(&id)
                    .map(|plugin| plugin.detect(root.as_deref()))
                    .unwrap_or_else(DetectResult::not_installed);
                log::info!(
                    "[Tool Detect] Finished detect for {id} in {} ms (installed={})",
                    started_at.elapsed().as_millis(),
                    result.installed
                );
                (index, id, cached_record, result)
            }));
        }

        let mut completed = Vec::new();
        for handle in handles {
            completed.push(
                handle
                    .join()
                    .map_err(|_| "tool detection worker thread panicked".to_string())?,
            );
        }
        Ok::<_, String>(completed)
    })?;

    for (index, id, cached_record, mut result) in detected {
        if !result.installed {
            if let Some(record) = cached_record.as_ref() {
                if allow_db_fallback && record.status == "installed" {
                    result = DetectResult {
                        installed: true,
                        version: record.version.clone(),
                        install_path: Some(record.install_path.clone()),
                    };
                }
            }
        }

        persist_detect_result_cache(
            db,
            &id,
            install_root.as_deref(),
            cached_record.as_ref(),
            &result,
        );
        results[index] = result;
    }

    Ok(results)
}

#[tauri::command]
pub fn set_install_root(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state
        .db
        .set_install_root(&path)
        .map_err(|e| format!("保存安装根目录失败: {e}"))
}

#[tauri::command]
pub fn get_install_root(state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    state
        .db
        .get_install_root()
        .map_err(|e| format!("查询失败: {e}"))
}

#[tauri::command]
pub async fn check_tool_updates(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ToolUpdateInfo>, String> {
    let db = state.db.clone();
    tokio::task::spawn_blocking(move || {
        tokio::runtime::Handle::current().block_on(InstallerService::check_tool_updates(&db))
    })
    .await
    .map_err(|e| format!("检测工具更新任务执行失败: {e}"))?
}

#[cfg(test)]
mod tests {
    use super::{
        can_reuse_detect_cache, detect_tools_sync, persist_detect_result_cache,
        should_use_db_fallback,
    };
    use crate::database::{Database, InstalledToolRecord};
    use crate::tool_types::DetectResult;
    use std::sync::Arc;

    #[test]
    fn detect_tools_db_fallback_is_disabled_for_explicit_install_root() {
        assert!(!should_use_db_fallback(Some("D:\\AgenticTools")));
    }

    #[test]
    fn detect_tools_db_fallback_stays_enabled_without_install_root() {
        assert!(should_use_db_fallback(None));
    }

    #[test]
    fn detect_tools_sync_uses_db_fallback_without_explicit_install_root() {
        let db = Arc::new(Database::memory().expect("create db"));
        db.upsert_installed_tool(&InstalledToolRecord {
            id: "unknown-tool".into(),
            name: "Unknown Tool".into(),
            version: Some("1.2.3".into()),
            install_path: "D:\\Tools\\unknown-tool".into(),
            install_root: "D:\\Tools".into(),
            category: "tool".into(),
            status: "installed".into(),
            state_source: "managed".into(),
            installed_at: Some(1),
            last_seen_at: Some(1),
            updated_at: Some(1),
        })
        .expect("seed installed tool");

        let results =
            detect_tools_sync(vec!["unknown-tool".into()], None, false, &db).expect("detect tools");

        assert_eq!(results.len(), 1);
        assert!(results[0].installed);
        assert_eq!(results[0].version.as_deref(), Some("1.2.3"));
        assert_eq!(
            results[0].install_path.as_deref(),
            Some("D:\\Tools\\unknown-tool")
        );
    }

    #[test]
    fn detect_tools_sync_skips_cache_for_mismatched_install_root() {
        let db = Arc::new(Database::memory().expect("create db"));
        db.upsert_installed_tool(&InstalledToolRecord {
            id: "unknown-tool".into(),
            name: "Unknown Tool".into(),
            version: Some("1.2.3".into()),
            install_path: "D:\\Tools\\unknown-tool".into(),
            install_root: "D:\\Tools".into(),
            category: "tool".into(),
            status: "detected".into(),
            state_source: "external_detected".into(),
            installed_at: None,
            last_seen_at: Some(1),
            updated_at: Some(1),
        })
        .expect("seed detected tool");

        let results = detect_tools_sync(
            vec!["unknown-tool".into()],
            Some("D:\\AgenticTools".into()),
            false,
            &db,
        )
        .expect("detect tools");

        assert_eq!(results.len(), 1);
        assert!(!results[0].installed);
        assert_eq!(results[0].version, None);
        assert_eq!(results[0].install_path, None);
    }

    #[test]
    fn detect_tools_sync_reuses_cached_detection_for_matching_install_root() {
        let db = Arc::new(Database::memory().expect("create db"));
        db.upsert_installed_tool(&InstalledToolRecord {
            id: "unknown-tool".into(),
            name: "Unknown Tool".into(),
            version: Some("9.9.9".into()),
            install_path: "D:\\AgenticTools\\unknown-tool".into(),
            install_root: "D:\\AgenticTools".into(),
            category: "tool".into(),
            status: "detected".into(),
            state_source: "managed".into(),
            installed_at: None,
            last_seen_at: Some(1),
            updated_at: Some(1),
        })
        .expect("seed detected cache");
        let cached = db
            .get_installed_tool("unknown-tool")
            .expect("load cache")
            .expect("cache exists");
        assert!(can_reuse_detect_cache(&cached, Some("D:\\AgenticTools")));

        let results = detect_tools_sync(
            vec!["unknown-tool".into()],
            Some("D:\\AgenticTools".into()),
            false,
            &db,
        )
        .expect("detect tools");

        assert_eq!(results.len(), 1);
        assert!(results[0].installed);
        assert_eq!(results[0].version.as_deref(), Some("9.9.9"));
        assert_eq!(
            results[0].install_path.as_deref(),
            Some("D:\\AgenticTools\\unknown-tool")
        );
    }

    #[test]
    fn detect_tools_sync_does_not_reuse_external_detected_cache_for_matching_install_root() {
        let db = Arc::new(Database::memory().expect("create db"));
        db.upsert_installed_tool(&InstalledToolRecord {
            id: "unknown-tool".into(),
            name: "Unknown Tool".into(),
            version: Some("9.9.9".into()),
            install_path: "C:\\Users\\me\\AppData\\Roaming\\npm".into(),
            install_root: "D:\\AgenticTools".into(),
            category: "tool".into(),
            status: "detected".into(),
            state_source: "external_detected".into(),
            installed_at: None,
            last_seen_at: Some(1),
            updated_at: Some(1),
        })
        .expect("seed detected cache");

        let results = detect_tools_sync(
            vec!["unknown-tool".into()],
            Some("D:\\AgenticTools".into()),
            false,
            &db,
        )
        .expect("detect tools");

        assert_eq!(results.len(), 1);
        assert!(!results[0].installed);
        assert_eq!(results[0].version, None);
        assert_eq!(results[0].install_path, None);
    }

    #[test]
    fn persist_detect_result_cache_keeps_external_detected_record_for_unmanaged_path() {
        let db = Arc::new(Database::memory().expect("create db"));
        let result = DetectResult {
            installed: true,
            version: Some("1.2.3".into()),
            install_path: Some("C:\\Users\\me\\AppData\\Roaming\\npm".into()),
        };

        persist_detect_result_cache(
            &db,
            "codex-cli",
            Some("D:\\AgenticTools"),
            None,
            &result,
        );

        let cached = db
            .get_installed_tool("codex-cli")
            .expect("load cached tool")
            .expect("cached tool exists");
        assert_eq!(cached.status, "detected");
        assert_eq!(cached.install_path, "C:\\Users\\me\\AppData\\Roaming\\npm");
    }
}
