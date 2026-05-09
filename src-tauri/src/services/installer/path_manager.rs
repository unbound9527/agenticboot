use crate::services::installer::windows::normalize_windows_exe;
use std::fs;
use std::path::{Path, PathBuf};

pub struct PathManager {
    root_dir: PathBuf,
}

impl PathManager {
    pub fn new(root_dir: &Path) -> Self {
        Self {
            root_dir: root_dir.to_path_buf(),
        }
    }

    pub fn ensure_bin_dir(&self) -> Result<PathBuf, String> {
        let bin_dir = self.root_dir.join("bin");
        fs::create_dir_all(&bin_dir).map_err(|e| format!("创建 bin 目录失败: {e}"))?;
        Ok(bin_dir)
    }

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

        let current_path: String = env_key.get_value("PATH").unwrap_or_default();

        if !current_path.split(';').any(|p| p == bin_str) {
            let new_path = if current_path.is_empty() {
                bin_str
            } else {
                format!("{current_path};{bin_str}")
            };
            env_key
                .set_value("PATH", &new_path)
                .map_err(|e| format!("设置 PATH 失败: {e}"))?;

            broadcast_env_change();
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    pub fn register_in_path(&self) -> Result<(), String> {
        let _ = self.ensure_bin_dir()?;
        Ok(())
    }

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

        let current_path: String = env_key.get_value("PATH").unwrap_or_default();
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

    pub fn create_cmd_shim(&self, shim_name: &str, target_exe: &Path) -> Result<(), String> {
        let bin_dir = self.ensure_bin_dir()?;
        let shim_path = bin_dir.join(format!("{shim_name}.cmd"));
        let content = format!(
            "@echo off\r\n\"{}\" %*\r\n",
            normalize_windows_exe(target_exe)
        );

        fs::write(&shim_path, content).map_err(|e| format!("创建 shim 失败: {e}"))?;
        Ok(())
    }

    pub fn create_shim(&self, shim_name: &str, target_exe: &str) -> Result<(), String> {
        self.create_cmd_shim(shim_name, Path::new(target_exe))
    }

    pub fn remove_shim(&self, shim_name: &str) -> Result<(), String> {
        let shim_path = self.root_dir.join("bin").join(format!("{shim_name}.cmd"));
        if shim_path.exists() {
            fs::remove_file(&shim_path).map_err(|e| format!("删除 shim 失败: {e}"))?;
        }
        Ok(())
    }

    pub fn get_tool_install_dir(&self, tool_id: &str) -> PathBuf {
        self.root_dir.join(tool_id)
    }
}

#[cfg(target_os = "windows")]
fn broadcast_env_change() {
    use std::ptr;

    unsafe {
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
            -1isize,
            0x001A,
            0,
            lparam.as_ptr(),
            0x0002,
            5000,
            ptr::null_mut(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn windows_paths_cli_shim_targets_actual_managed_executable_path() {
        let tmp = TempDir::new().unwrap();
        let pm = PathManager::new(tmp.path());
        let shim_path = tmp.path().join("bin").join("claude.cmd");

        pm.create_cmd_shim(
            "claude",
            Path::new("D:\\AITools\\claude-code-cli\\claude.cmd"),
        )
        .unwrap();
        assert!(shim_path.exists());

        let content = fs::read_to_string(&shim_path).unwrap();
        assert!(content.contains("claude-code-cli\\claude.cmd"));

        pm.remove_shim("claude").unwrap();
        assert!(!shim_path.exists());
    }
}
