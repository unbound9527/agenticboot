//! PATH 管理器（Windows）
//!
//! 管理统一安装根目录下的 bin/ 文件夹和 Windows 注册表 PATH。
//! 自动创建 shim 脚本使工具可从任意终端调用。

use std::fs;
use std::path::{Path, PathBuf};

/// PATH 和 shim 管理器
pub struct PathManager {
    root_dir: PathBuf,
}

impl PathManager {
    /// 创建新的 PATH 管理器
    pub fn new(root_dir: &Path) -> Self {
        Self {
            root_dir: root_dir.to_path_buf(),
        }
    }

    /// 确保 <root>/bin/ 目录存在
    pub fn ensure_bin_dir(&self) -> Result<PathBuf, String> {
        let bin_dir = self.root_dir.join("bin");
        fs::create_dir_all(&bin_dir)
            .map_err(|e| format!("创建 bin 目录失败: {e}"))?;
        Ok(bin_dir)
    }

    /// 将 <root>/bin 注册到 Windows PATH
    #[cfg(target_os = "windows")]
    pub fn register_in_path(&self) -> Result<(), String> {
        use winreg::enums::*;
        use winreg::RegKey;

        let bin_dir = self.ensure_bin_dir()?;
        let bin_str = bin_dir.to_string_lossy().to_string();

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu
            .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
            .map_err(|e| format!("无法打开注册表 Environment 键: {e}"))?;

        let current_path: String = env_key
            .get_value("PATH")
            .unwrap_or_default();

        if !current_path.split(';').any(|p| p == bin_str) {
            let new_path = if current_path.is_empty() {
                bin_str
            } else {
                format!("{current_path};{bin_str}")
            };
            env_key
                .set_value("PATH", &new_path)
                .map_err(|e| format!("设置 PATH 失败: {e}"))?;

            // 广播 WM_SETTINGCHANGE 通知系统环境变量已变更
            broadcast_env_change();
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn register_in_path(&self) -> Result<(), String> {
        // macOS/Linux: 后续实现
        // 将 export PATH 追加到 ~/.bashrc / ~/.zshrc
        let _ = self.ensure_bin_dir()?;
        Ok(())
    }

    /// 从 Windows PATH 中移除 <root>/bin
    #[cfg(target_os = "windows")]
    pub fn unregister_from_path(&self) -> Result<(), String> {
        use winreg::enums::*;
        use winreg::RegKey;

        let bin_dir = self.root_dir.join("bin");
        let bin_str = bin_dir.to_string_lossy().to_string();

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env_key = hkcu
            .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
            .map_err(|e| format!("无法打开注册表 Environment 键: {e}"))?;

        let current_path: String = env_key
            .get_value("PATH")
            .unwrap_or_default();

        let new_path = current_path
            .split(';')
            .filter(|p| *p != bin_str)
            .collect::<Vec<&str>>()
            .join(";");

        env_key
            .set_value("PATH", &new_path)
            .map_err(|e| format!("移除 PATH 失败: {e}"))?;

        broadcast_env_change();
        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn unregister_from_path(&self) -> Result<(), String> {
        Ok(())
    }

    /// 创建 shim 脚本（Windows .cmd 文件）
    pub fn create_shim(&self, shim_name: &str, target_exe: &str) -> Result<(), String> {
        let bin_dir = self.ensure_bin_dir()?;
        let shim_path = bin_dir.join(format!("{shim_name}.cmd"));

        let content = format!("@echo off\r\n\"{target_exe}\" %*\r\n");
        fs::write(&shim_path, content)
            .map_err(|e| format!("创建 shim 失败: {e}"))?;

        Ok(())
    }

    /// 移除 shim 脚本
    pub fn remove_shim(&self, shim_name: &str) -> Result<(), String> {
        let shim_path = self.root_dir.join("bin").join(format!("{shim_name}.cmd"));
        if shim_path.exists() {
            fs::remove_file(&shim_path)
                .map_err(|e| format!("删除 shim 失败: {e}"))?;
        }
        Ok(())
    }

    /// 获取工具的安装目录
    pub fn get_tool_install_dir(&self, tool_id: &str) -> PathBuf {
        self.root_dir.join(tool_id)
    }
}

/// 广播环境变量变更通知（Windows）
#[cfg(target_os = "windows")]
fn broadcast_env_change() {
    use std::ptr;
    unsafe {
        // SendMessageTimeoutW(HWND_BROADCAST, WM_SETTINGCHANGE, 0, "Environment", ...)
        // 通知所有窗口环境变量已变更
        extern "system" {
            fn SendMessageTimeoutW(
                hwnd: isize,
                msg: u32,
                wparam: usize,
                lparam: *const u16,
                flags: u32,
                timeout: u32,
                result: *mut usize,
            ) -> isize;
        }

        let lparam: Vec<u16> = "Environment\0".encode_utf16().collect();
        let _ = SendMessageTimeoutW(
            -1isize, // HWND_BROADCAST = 0xffff = -1 on x64
            0x001A,  // WM_SETTINGCHANGE
            0,
            lparam.as_ptr(),
            0x0002,  // SMTO_ABORTIFHUNG
            5000,
            ptr::null_mut(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_ensure_bin_dir_creates_directory() {
        let tmp = TempDir::new().unwrap();
        let pm = PathManager::new(tmp.path());
        let bin = pm.ensure_bin_dir().unwrap();
        assert!(bin.exists());
        assert!(bin.is_dir());
    }

    #[test]
    fn test_get_tool_install_dir() {
        let pm = PathManager::new(Path::new("D:\\AITools"));
        let dir = pm.get_tool_install_dir("claude-code-cli");
        assert_eq!(dir, Path::new("D:\\AITools\\claude-code-cli"));
    }

    #[test]
    fn test_create_and_remove_shim() {
        let tmp = TempDir::new().unwrap();
        let pm = PathManager::new(tmp.path());
        let shim_path = tmp.path().join("bin").join("test-tool.cmd");

        pm.create_shim("test-tool", "D:\\AITools\\claude-code-cli\\claude.exe")
            .unwrap();
        assert!(shim_path.exists());

        let content = fs::read_to_string(&shim_path).unwrap();
        // For now just verify the file is non-empty and a valid cmd script
        assert!(content.contains("claude.exe"));

        pm.remove_shim("test-tool").unwrap();
        assert!(!shim_path.exists());
    }
}
