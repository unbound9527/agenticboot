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
        let path_with_npm = dirs::data_dir()
            .map(|appdata| appdata.join("npm").to_string_lossy().to_string())
            .map(|npm_bin| merge_path_with_preferred_bin(&current_path, &npm_bin))
            .unwrap_or_else(|| current_path.clone());
        let new_path = merge_path_with_preferred_bin(&path_with_npm, &bin_str);
        if new_path != current_path {
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
        let new_path = remove_path_entry(&current_path, &bin_str);

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

    pub fn create_windows_cli_shims(
        &self,
        shim_name: &str,
        target_exe: &Path,
    ) -> Result<(), String> {
        let bin_dir = self.ensure_bin_dir()?;
        let target = normalize_windows_exe(target_exe);
        let cmd_shim_path = bin_dir.join(format!("{shim_name}.cmd"));
        let cmd_content = format!("@echo off\r\n\"{}\" %*\r\n", target);
        fs::write(&cmd_shim_path, cmd_content).map_err(|e| format!("创建 shim 失败: {e}"))?;

        let shell_shim_path = bin_dir.join(shim_name);
        let shell_content = format!(
            "#!/bin/sh\nTARGET={}\nif command -v cygpath >/dev/null 2>&1; then\n  TARGET=\"$(cygpath -u \"$TARGET\")\"\nfi\nexec \"$TARGET\" \"$@\"\n",
            shell_single_quote(&target)
        );
        fs::write(&shell_shim_path, shell_content).map_err(|e| format!("创建 shim 失败: {e}"))?;
        Ok(())
    }

    pub fn remove_windows_cli_shims(&self, shim_name: &str) -> Result<(), String> {
        let shim_dir = self.root_dir.join("bin");
        for shim_path in [
            shim_dir.join(format!("{shim_name}.cmd")),
            shim_dir.join(shim_name),
        ] {
            if shim_path.exists() {
                fs::remove_file(&shim_path).map_err(|e| format!("删除 shim 失败: {e}"))?;
            }
        }
        Ok(())
    }

    pub fn create_cmd_shim(&self, shim_name: &str, target_exe: &Path) -> Result<(), String> {
        self.create_windows_cli_shims(shim_name, target_exe)
    }

    pub fn create_shim(&self, shim_name: &str, target_exe: &str) -> Result<(), String> {
        self.create_windows_cli_shims(shim_name, Path::new(target_exe))
    }

    pub fn remove_shim(&self, shim_name: &str) -> Result<(), String> {
        self.remove_windows_cli_shims(shim_name)
    }

    pub fn get_tool_install_dir(&self, tool_id: &str) -> PathBuf {
        self.root_dir.join(tool_id)
    }
}

fn merge_path_with_preferred_bin(current_path: &str, preferred_bin: &str) -> String {
    let mut entries = vec![preferred_bin.to_string()];
    entries.extend(
        current_path
            .split(';')
            .filter(|entry| !entry.trim().is_empty())
            .filter(|entry| !same_windows_path_entry(entry, preferred_bin))
            .map(str::to_string),
    );

    entries.join(";")
}

fn remove_path_entry(current_path: &str, removed_entry: &str) -> String {
    current_path
        .split(';')
        .filter(|entry| !entry.trim().is_empty())
        .filter(|entry| !same_windows_path_entry(entry, removed_entry))
        .collect::<Vec<_>>()
        .join(";")
}

fn same_windows_path_entry(left: &str, right: &str) -> bool {
    normalize_windows_path_entry(left).eq_ignore_ascii_case(&normalize_windows_path_entry(right))
}

fn normalize_windows_path_entry(path: &str) -> &str {
    path.trim().trim_end_matches(['\\', '/'])
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
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
        let pm = PathManager::new(Path::new("D:\\AgenticTools"));
        let dir = pm.get_tool_install_dir("claude-code-cli");
        assert_eq!(dir, Path::new("D:\\AgenticTools\\claude-code-cli"));
    }

    #[test]
    fn windows_paths_cli_shim_targets_actual_managed_executable_path() {
        let tmp = TempDir::new().unwrap();
        let pm = PathManager::new(tmp.path());
        let shim_path = tmp.path().join("bin").join("claude.cmd");

        pm.create_cmd_shim(
            "claude",
            Path::new("D:\\AgenticTools\\claude-code-cli\\claude.cmd"),
        )
        .unwrap();
        assert!(shim_path.exists());

        let content = fs::read_to_string(&shim_path).unwrap();
        assert!(content.contains("claude-code-cli\\claude.cmd"));
    }

    #[test]
    fn windows_paths_cli_shim_also_publishes_extensionless_launcher() {
        let tmp = TempDir::new().unwrap();
        let pm = PathManager::new(tmp.path());
        let shim_path = tmp.path().join("bin").join("gemini");

        pm.create_windows_cli_shims(
            "gemini",
            Path::new("D:\\AgenticTools\\gemini-cli\\gemini.cmd"),
        )
        .unwrap();

        assert!(shim_path.exists());

        let content = fs::read_to_string(&shim_path).unwrap();
        assert!(content.contains("#!/bin/sh"));
        assert!(content.contains("gemini-cli\\gemini.cmd"));
    }

    #[test]
    fn windows_paths_remove_shim_removes_extensionless_launcher_too() {
        let tmp = TempDir::new().unwrap();
        let pm = PathManager::new(tmp.path());
        let extensionless_shim = tmp.path().join("bin").join("claude");

        pm.create_windows_cli_shims(
            "claude",
            Path::new("D:\\AgenticTools\\claude-code-cli\\claude.cmd"),
        )
        .unwrap();
        fs::write(&extensionless_shim, "#!/bin/sh\n").unwrap();

        pm.remove_windows_cli_shims("claude").unwrap();
        assert!(!tmp.path().join("bin").join("claude.cmd").exists());
        assert!(!extensionless_shim.exists());
    }

    #[test]
    fn windows_path_registration_prefers_managed_bin_over_existing_entries() {
        let managed_bin = "D:\\AgenticTools\\bin";
        let current_path = format!(
            "C:\\Users\\Test\\AppData\\Roaming\\npm;{};C:\\Windows\\System32",
            managed_bin
        );

        let merged = merge_path_with_preferred_bin(&current_path, managed_bin);

        assert_eq!(
            merged,
            format!(
                "{};C:\\Users\\Test\\AppData\\Roaming\\npm;C:\\Windows\\System32",
                managed_bin
            )
        );
    }

    #[test]
    fn windows_path_registration_deduplicates_case_and_trailing_slash_variants() {
        let managed_bin = "D:\\AgenticTools\\bin";
        let current_path =
            "d:\\agentictools\\bin\\;C:\\Users\\Test\\AppData\\Roaming\\npm;C:\\Windows\\System32";

        let merged = merge_path_with_preferred_bin(current_path, managed_bin);

        assert_eq!(
            merged,
            format!(
                "{};C:\\Users\\Test\\AppData\\Roaming\\npm;C:\\Windows\\System32",
                managed_bin
            )
        );
    }

    #[test]
    fn windows_path_unregistration_removes_case_and_trailing_slash_variants() {
        let managed_bin = "D:\\AgenticTools\\bin";
        let current_path =
            "C:\\Users\\Test\\AppData\\Roaming\\npm;d:\\agentictools\\bin\\;C:\\Windows\\System32";

        let stripped = remove_path_entry(current_path, managed_bin);

        assert_eq!(
            stripped,
            "C:\\Users\\Test\\AppData\\Roaming\\npm;C:\\Windows\\System32"
        );
    }
}
