//! 已安装工具数据访问对象
//!
//! 提供 AgenticBoot 工具安装管理相关的数据库操作。

use crate::database::{lock_conn, Database};
use crate::error::AppError;
use rusqlite::params;
use serde::{Deserialize, Serialize};

/// 已安装工具数据库记录
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledToolRecord {
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

impl Database {
    /// 获取所有已安装工具
    pub fn get_installed_tools(&self) -> Result<Vec<InstalledToolRecord>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, version, install_path, install_root, category, status, installed_at, updated_at
                 FROM installed_tools ORDER BY category, name",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(InstalledToolRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    version: row.get(2)?,
                    install_path: row.get(3)?,
                    install_root: row.get(4)?,
                    category: row.get(5)?,
                    status: row.get(6)?,
                    installed_at: row.get(7)?,
                    updated_at: row.get(8)?,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut tools = Vec::new();
        for row in rows {
            tools.push(row.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(tools)
    }

    /// 获取单个已安装工具
    pub fn get_installed_tool(&self, id: &str) -> Result<Option<InstalledToolRecord>, AppError> {
        let conn = lock_conn!(self.conn);
        let mut stmt = conn
            .prepare(
                "SELECT id, name, version, install_path, install_root, category, status, installed_at, updated_at
                 FROM installed_tools WHERE id = ?1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut rows = stmt
            .query(params![id])
            .map_err(|e| AppError::Database(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            Ok(Some(InstalledToolRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                version: row.get(2)?,
                install_path: row.get(3)?,
                install_root: row.get(4)?,
                category: row.get(5)?,
                status: row.get(6)?,
                installed_at: row.get(7)?,
                updated_at: row.get(8)?,
            }))
        } else {
            Ok(None)
        }
    }

    /// 插入或更新已安装工具记录
    pub fn upsert_installed_tool(&self, record: &InstalledToolRecord) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute(
            "INSERT OR REPLACE INTO installed_tools
             (id, name, version, install_path, install_root, category, status, installed_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                record.id,
                record.name,
                record.version,
                record.install_path,
                record.install_root,
                record.category,
                record.status,
                record.installed_at,
                record.updated_at,
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 更新工具安装状态
    pub fn update_tool_status(&self, id: &str, status: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "UPDATE installed_tools SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status, now, id],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 删除已安装工具记录
    pub fn delete_installed_tool(&self, id: &str) -> Result<(), AppError> {
        let conn = lock_conn!(self.conn);
        conn.execute("DELETE FROM installed_tools WHERE id = ?1", params![id])
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    /// 检查是否有任何已安装工具
    pub fn has_any_installed_tools(&self) -> Result<bool, AppError> {
        let conn = lock_conn!(self.conn);
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM installed_tools WHERE status IN ('installed', 'detected')",
                [],
                |row| row.get(0),
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(count > 0)
    }

    /// 获取安装根目录路径设置
    pub fn get_install_root(&self) -> Result<Option<String>, AppError> {
        self.get_setting("install_root")
    }

    /// 设置安装根目录路径
    pub fn set_install_root(&self, path: &str) -> Result<(), AppError> {
        self.set_setting("install_root", path)
    }
}

#[cfg(test)]
mod tests {
    use super::InstalledToolRecord;
    use crate::database::Database;

    #[test]
    fn has_any_installed_tools_ignores_error_records() {
        let db = Database::memory().expect("create db");
        db.upsert_installed_tool(&InstalledToolRecord {
            id: "openclaw".into(),
            name: "OpenClaw".into(),
            version: None,
            install_path: "D:\\AITools\\openclaw".into(),
            install_root: "D:\\AITools".into(),
            category: "ai-cli".into(),
            status: "error".into(),
            installed_at: None,
            updated_at: Some(1),
        })
        .expect("seed error record");

        assert!(!db.has_any_installed_tools().expect("query installed tools"));
    }

    #[test]
    fn has_any_installed_tools_counts_installed_records() {
        let db = Database::memory().expect("create db");
        db.upsert_installed_tool(&InstalledToolRecord {
            id: "openclaw".into(),
            name: "OpenClaw".into(),
            version: None,
            install_path: "D:\\AITools\\openclaw".into(),
            install_root: "D:\\AITools".into(),
            category: "ai-cli".into(),
            status: "installed".into(),
            installed_at: Some(1),
            updated_at: Some(1),
        })
        .expect("seed installed record");

        assert!(db.has_any_installed_tools().expect("query installed tools"));
    }

    #[test]
    fn has_any_installed_tools_counts_detected_records() {
        let db = Database::memory().expect("create db");
        db.upsert_installed_tool(&InstalledToolRecord {
            id: "claude-code-cli".into(),
            name: "Claude Code (CLI)".into(),
            version: Some("1.2.3".into()),
            install_path: "C:\\Users\\me\\AppData\\Roaming\\npm".into(),
            install_root: "D:\\AITools".into(),
            category: "ai-cli".into(),
            status: "detected".into(),
            installed_at: None,
            updated_at: Some(1),
        })
        .expect("seed detected record");

        assert!(db.has_any_installed_tools().expect("query installed tools"));
    }
}
