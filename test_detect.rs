use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn find_appx_install_location(package_name: &str) -> Option<String> {
    let script = format!(
        "$pkg = Get-AppxPackage {package_name} | Select-Object -First 1; if ($pkg) {{ $pkg.InstallLocation }}"
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let location = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!location.is_empty()).then_some(location)
}

fn find_exe_recursive(dir: &Path, exe_name: &str, current_depth: usize, max_depth: usize) -> Option<PathBuf> {
    if current_depth >= max_depth {
        return None;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return None,
    };

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_dir() {
            let candidate = path.join(exe_name);
            if candidate.exists() {
                return Some(candidate);
            }
            if let Some(found) = find_exe_recursive(&path, exe_name, current_depth + 1, max_depth) {
                return Some(found);
            }
        }
    }
    None
}

fn find_appx_exe_in_localcache(package_name: &str, exe_name: &str) -> Option<PathBuf> {
    let local = env::var_os("LOCALAPPDATA")?;
    let packages_dir = PathBuf::from(local).join("Packages");
    if !packages_dir.is_dir() {
        return None;
    }

    let entries = std::fs::read_dir(&packages_dir).ok()?;
    for entry in entries.filter_map(|e| e.ok()) {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(package_name) {
            let localcache = entry.path().join("LocalCache");
            if let Some(found) = find_exe_recursive(&localcache, exe_name, 0, 5) {
                return Some(found);
            }
        }
    }

    None
}

fn main() {
    println!("=== Testing Codex Desktop detection ===\n");

    // 1. Test find_appx_install_location
    println!("1. find_appx_install_location(\"OpenAI.Codex\"):");
    match find_appx_install_location("OpenAI.Codex") {
        Some(loc) => println!("   Found: {}", loc),
        None => println!("   Not found"),
    }

    // 2. Test find_appx_exe_in_localcache
    println!("\n2. find_appx_exe_in_localcache(\"OpenAI.Codex\", \"Codex.exe\"):");
    match find_appx_exe_in_localcache("OpenAI.Codex", "Codex.exe") {
        Some(exe) => println!("   Found: {}", exe.display()),
        None => println!("   Not found"),
    }

    println!("\n=== Done ===");
}
