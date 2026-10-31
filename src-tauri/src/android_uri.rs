use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::json;
use std::{
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

static RENDER_LOG: Mutex<Option<PathBuf>> = Mutex::new(None);
static LOGGER: AndroidLogger = AndroidLogger;

struct AndroidLogger;
impl log::Log for AndroidLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        metadata.level() <= log::Level::Info
    }
    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            debug_log(&format!("[{}] {}", record.level(), record.args()));
        }
    }
    fn flush(&self) {}
}

pub(crate) fn install_logger() {
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(log::LevelFilter::Info);
    }
}

pub(crate) struct RenderLog;
impl Drop for RenderLog {
    fn drop(&mut self) {
        if let Ok(mut path) = RENDER_LOG.lock() {
            *path = None;
        }
    }
}

pub(crate) fn start_render_log(path: PathBuf) -> Result<RenderLog> {
    std::fs::File::create(&path).with_context(|| format!("failed to create {}", path.display()))?;
    *RENDER_LOG
        .lock()
        .map_err(|error| anyhow::anyhow!("{error}"))? = Some(path);
    Ok(RenderLog)
}

pub(crate) fn debug_log(message: &str) {
    if let Ok(path) = RENDER_LOG.lock() {
        if let Some(path) = path.as_ref() {
            if let Ok(mut file) = std::fs::OpenOptions::new().append(true).open(path) {
                let _ = writeln!(file, "{message}");
            }
        }
    }
    extern "C" {
        fn __android_log_write(
            prio: i32,
            tag: *const std::os::raw::c_char,
            text: *const std::os::raw::c_char,
        ) -> i32;
    }
    if let Ok(text) = std::ffi::CString::new(message) {
        unsafe {
            __android_log_write(4, b"PhiTK\0".as_ptr() as _, text.as_ptr());
        }
    }
}

pub fn resolve(path: &Path) -> Result<PathBuf> {
    let text = path.to_string_lossy();
    if !text.starts_with("content://") && !text.starts_with("file://") {
        anyhow::ensure!(
            path.exists(),
            "The chart is no longer available. Please select it again."
        );
        return Ok(path.to_path_buf());
    }
    // Recover earlier imports even when their temporary URI grants have expired.
    if text.starts_with("content://") {
        if let Some(dir) = crate::common::DATA_DIR.get() {
            let legacy = dir
                .join("picked")
                .join(format!("picked-{:x}.zip", legacy_hash(&text)));
            if legacy.is_file() && legacy.metadata()?.len() > 0 {
                return Ok(legacy);
            }
        }
    }
    #[derive(Deserialize)]
    struct Response {
        path: PathBuf,
    }
    let result: Response = crate::android_platform::call("importPath", json!({ "path": text }))?;
    Ok(result.path)
}

pub fn publish_video(source: &Path, display_name: &str) -> Result<String> {
    #[derive(Deserialize)]
    struct Response {
        uri: String,
    }
    let result: Response = crate::android_platform::call(
        "publishVideo",
        json!({ "source": source, "name": display_name }),
    )?;
    Ok(result.uri)
}

fn legacy_hash(text: &str) -> u64 {
    text.as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ *byte as u64).wrapping_mul(0x0100_0000_01b3)
        })
}
