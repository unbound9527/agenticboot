//! 文件下载服务
//!
//! 支持 Node.js / Git / GitHub Release 等大文件下载。
use futures::StreamExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// 下载进度回调
pub type ProgressCallback = Box<dyn Fn(u64, u64) + Send>;

const DOWNLOAD_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const DOWNLOAD_REQUEST_TIMEOUT: Duration = Duration::from_secs(90);
const DOWNLOAD_MAX_ATTEMPTS: usize = 3;

/// 下载文件到本地路径，返回文件大小（字节）
pub async fn download_file(
    url: &str,
    dest: &Path,
    on_progress: Option<ProgressCallback>,
) -> Result<u64, String> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建目录失败: {e}"))?;
    }

    let client = reqwest::Client::builder()
        .user_agent("AgenticBoot/0.1")
        .connect_timeout(DOWNLOAD_CONNECT_TIMEOUT)
        .timeout(DOWNLOAD_REQUEST_TIMEOUT)
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;
    let on_progress_ref = on_progress.as_ref();

    retry_async("download", DOWNLOAD_MAX_ATTEMPTS, |attempt| {
        let client = client.clone();
        let partial_dest = partial_download_path(dest);
        async move {
            if attempt > 0 {
                log::warn!("[downloader] retrying attempt {} for {}", attempt + 1, url);
            }

            match download_file_once(&client, url, dest, &partial_dest, on_progress_ref).await {
                Ok(size) => Ok(size),
                Err(error) => {
                    let _ = fs::remove_file(&partial_dest).await;
                    Err(error)
                }
            }
        }
    })
    .await
}

async fn download_file_once(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    partial_dest: &Path,
    on_progress: Option<&ProgressCallback>,
) -> Result<u64, String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("下载失败: {e}"))?;

    if !response.status().is_success() {
        return Err(format!("下载失败，HTTP {}", response.status()));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded = 0_u64;
    let mut file = fs::File::create(partial_dest)
        .await
        .map_err(|e| format!("创建文件失败: {e}"))?;

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("下载数据错误: {e}"))?;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("写入文件失败: {e}"))?;
        downloaded += chunk.len() as u64;

        if let Some(cb) = on_progress {
            cb(downloaded, total_size);
        }
    }

    file.flush()
        .await
        .map_err(|e| format!("刷新文件失败: {e}"))?;
    drop(file);

    fs::rename(partial_dest, dest)
        .await
        .map_err(|e| format!("保存下载文件失败: {e}"))?;

    Ok(downloaded)
}

fn partial_download_path(dest: &Path) -> PathBuf {
    let file_name = dest
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| format!("{name}.partial"))
        .unwrap_or_else(|| "download.partial".to_string());
    dest.with_file_name(file_name)
}

async fn retry_async<T, F, Fut>(
    label: &str,
    max_attempts: usize,
    mut operation: F,
) -> Result<T, String>
where
    F: FnMut(usize) -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut last_error = None;

    for attempt in 0..max_attempts {
        match operation(attempt).await {
            Ok(value) => return Ok(value),
            Err(error) => last_error = Some(error),
        }
    }

    Err(format!(
        "{label} failed after {max_attempts} attempts: {}",
        last_error.unwrap_or_else(|| "unknown error".to_string())
    ))
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

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    #[tokio::test]
    async fn retry_helper_retries_transient_failures_until_success() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_for_closure = Arc::clone(&attempts);

        let result = super::retry_async("download", 3, move |_| {
            let attempts = Arc::clone(&attempts_for_closure);
            async move {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                if attempt < 2 {
                    Err(format!("temporary failure #{attempt}"))
                } else {
                    Ok("ok".to_string())
                }
            }
        })
        .await
        .expect("retry eventually succeeds");

        assert_eq!(result, "ok");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_helper_returns_last_error_after_exhausting_attempts() {
        let attempts = Arc::new(AtomicUsize::new(0));
        let attempts_for_closure = Arc::clone(&attempts);

        let error = super::retry_async("download", 3, move |_| {
            let attempts = Arc::clone(&attempts_for_closure);
            async move {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(format!("temporary failure #{attempt}"))
            }
        })
        .await
        .expect_err("retry should fail after last attempt");

        assert!(error.contains("download failed after 3 attempts"));
        assert!(error.contains("temporary failure #2"));
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }
}
