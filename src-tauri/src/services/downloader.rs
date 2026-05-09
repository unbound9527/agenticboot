//! 文件下载服务
//!
//! 支持下载进度上报，用于 Node.js / Git / GitHub Release 等大文件下载。

use futures::StreamExt;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// 下载进度回调
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send>;

/// 下载文件到本地路径，返回文件大小（字节）
pub async fn download_file(
    url: &str,
    dest: &Path,
    on_progress: Option<ProgressCallback>,
) -> Result<u64, String> {
    // 确保父目录存在
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建目录失败: {e}"))?;
    }

    let client = reqwest::Client::builder()
        .user_agent("AgenticBoot/0.1")
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("下载失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("下载失败，HTTP {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = fs::File::create(dest)
        .await
        .map_err(|e| format!("创建文件失败: {e}"))?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载数据错误: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("写入文件失败: {e}"))?;
        downloaded += chunk.len() as u64;

        if let Some(ref cb) = on_progress {
            cb(downloaded, total_size);
        }
    }

    file.flush()
        .await
        .map_err(|e| format!("刷新文件失败: {e}"))?;

    Ok(downloaded)
}

/// 解压 zip 文件到目标目录
pub fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let file = std::fs::File::open(zip_path).map_err(|e| format!("打开 zip 文件失败: {e}"))?;

    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("读取 zip 失败: {e}"))?;

    std::fs::create_dir_all(dest_dir).map_err(|e| format!("创建解压目录失败: {e}"))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {e}"))?;

        let out_path = match entry.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path).map_err(|e| format!("创建目录失败: {e}"))?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| format!("创建父目录失败: {e}"))?;
            }
            let mut outfile =
                std::fs::File::create(&out_path).map_err(|e| format!("创建解压文件失败: {e}"))?;
            std::io::copy(&mut entry, &mut outfile).map_err(|e| format!("解压文件失败: {e}"))?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = entry.unix_mode() {
                std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(mode)).ok();
            }
        }
    }

    Ok(())
}

/// 运行 Windows 安装程序并等待完成
pub fn run_installer(installer_path: &Path, target_dir: &Path) -> Result<(), String> {
    let exit_status = std::process::Command::new(installer_path)
        .args([
            "/VERYSILENT",
            "/SUPPRESSMSGBOXES",
            "/NORESTART",
            "/SP-",
            &format!("/DIR={}", target_dir.display()),
        ])
        .spawn()
        .map_err(|e| format!("启动安装程序失败: {e}"))?
        .wait()
        .map_err(|e| format!("等待安装完成失败: {e}"))?;

    if !exit_status.success() {
        return Err(format!("安装程序异常退出，code: {:?}", exit_status.code()));
    }

    Ok(())
}

/// 获取临时文件路径
pub fn temp_path(filename: &str) -> PathBuf {
    std::env::temp_dir().join("agenticboot").join(filename)
}

/// 解压 .tar.gz 文件到目标目录
pub fn extract_tar_gz(tar_gz_path: &Path, dest_dir: &Path) -> Result<(), String> {
    let output = std::process::Command::new("tar")
        .args([
            "xzf",
            &tar_gz_path.to_string_lossy(),
            "-C",
            &dest_dir.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("tar 解压失败: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "tar 解压失败: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}
