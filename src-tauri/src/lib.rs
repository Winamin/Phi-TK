prpr::tl_file!("main" mtl);

mod common;
mod gpu_info;
mod ipc;
mod preview;
pub mod render_config;
mod vulkan_readback;

#[cfg(target_os = "android")]
mod android_media;
#[cfg(target_os = "android")]
pub mod android_render;
#[cfg(target_os = "android")]
pub mod android_uri;
#[cfg(target_os = "android")]
mod android_platform;
#[cfg(target_os = "android")]
mod android_preview;
mod task;

use anyhow::{bail, Context, Result};
use common::{ensure_dir, respack_dir, CONFIG_DIR, DATA_DIR};
use gpu_info::get_msaa_sample_counts;
use macroquad::prelude::set_pc_assets_folder;
use prpr::{
    fs::{self, FileSystem},
    info::ChartInfo,
};
use render_config::{RenderConfig, RenderParams};
use serde::Serialize;
use std::{
    collections::HashMap,
    fs::File,
    future::Future,
    io::{BufRead, BufReader, BufWriter},
    ops::DerefMut,
    path::{Path, PathBuf},
    sync::OnceLock,
    time::SystemTime,
};
use task::{TaskQueue, TaskView};
use tauri::{ipc::InvokeError, Manager, State};
#[cfg(not(target_os = "android"))]
use std::process::Stdio;
#[cfg(not(target_os = "android"))]
use tokio::{io::AsyncWriteExt, process::Command};

pub(crate) static ASSET_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn set_data_dir(dir: PathBuf) {
    let _ = DATA_DIR.set(dir);
}

pub fn set_asset_path(dir: PathBuf) {
    let _ = ASSET_PATH.set(dir);
}

#[inline]
async fn wrap_async<R>(f: impl Future<Output = Result<R>>) -> Result<R, InvokeError> {
    f.await.map_err(|e| {
        eprintln!("{e:?}");
        InvokeError::from_anyhow(e)
    })
}

#[cfg(not(target_os = "android"))]
pub async fn run_wrapped(f: impl Future<Output = Result<()>>) -> ! {
    if let Err(err) = f.await {
        eprintln!("{err:?}");
        std::process::exit(1);
    }
    std::process::exit(0);
}

#[cfg(target_os = "android")]
#[no_mangle]
pub extern "C" fn quad_main() { android_preview::start(); }

#[cfg(target_os = "android")]
pub fn install_panic_logger() {
    std::panic::set_hook(Box::new(|info| android_uri::debug_log(&format!("PHITK_PANIC {info}"))));
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() -> Result<()> {
    #[cfg(target_os = "android")]
    install_panic_logger();

    //dynamic cpu core
    let num_workers = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_workers)
        .enable_all()
        .build()?;
    let _guard = rt.enter();

    #[cfg(not(target_os = "android"))]
    if std::env::args().len() > 1 {
        match std::env::args().skip(1).next().as_deref() {
            Some("preview") => {
                run_wrapped(preview::main()).await;
            }
            cmd => {
                eprintln!("Unknown subcommand: {cmd:?}");
                std::process::exit(1);
            }
        }
    }

    let builder = tauri::Builder::default();
    #[cfg(target_os = "android")]
    let builder = builder.plugin(android_platform::init());
    let app = builder
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(TaskQueue::new())
        .invoke_handler(tauri::generate_handler![
            pick_chart_files,
            import_chart,
            view_video,
            save_text_file,
            save_binary_file,
            preview_chart,
            parse_chart,
            load_chart_illustration,
            batch_add_tasks,
            post_render,
            get_tasks,
            cancel_task,
            get_respacks,
            open_respack_folder,
            get_msaa_sample_counts,
            get_presets,
            add_preset,
            remove_preset,
            set_rpe_dir,
            unset_rpe_dir,
            get_rpe_charts,
            open_app_folder,
            list_chart_files,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application");

    let resolver = app.path();

    #[cfg(not(target_os = "android"))]
    let exe = std::env::current_exe()?;

    #[cfg(not(target_os = "android"))]
    let exe_dir = exe.parent().context("Executable has no parent directory")?.to_path_buf();

    CONFIG_DIR.set(ensure_dir(resolver.app_config_dir()?)).unwrap();
    DATA_DIR.set(ensure_dir(resolver.app_data_dir()?)).unwrap();
    #[cfg(target_os = "android")]
    prpr::fs::set_cache_dir(ensure_dir(resolver.app_cache_dir()?));

    #[cfg(target_os = "linux")]
    let asset_dir = {
        PathBuf::from("/usr/lib/Phi-TK/assets")
        // exe_dir.parent().unwrap().join("assets")
    };

    #[cfg(target_os = "windows")]
    let asset_dir = exe_dir.join("assets");

    #[cfg(target_os = "android")]
    let asset_dir = DATA_DIR.get().unwrap().join("assets");

    #[cfg(not(target_os = "android"))]
    if !asset_dir.exists() {
        eprintln!("错误：资源目录不存在 - {:?}", asset_dir);
        bail!("资源目录未找到，请检查安装路径");
    }

    #[cfg(target_os = "android")]
    let asset_dir = {
        let dir = asset_dir.clone();
        ensure_dir(dir)
    };
    ASSET_PATH.set(asset_dir.clone()).unwrap();
    set_pc_assets_folder(&asset_dir.display().to_string());

    app.run(|_, _| {});

    Ok(())
}

fn open_dir(path: &Path) -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(path).spawn()?;
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn()?;
    }
    Ok(())
}

#[tauri::command]
fn save_text_file(path: PathBuf, content: String) -> Result<(), InvokeError> {
    std::fs::write(path, content).map_err(|e| InvokeError::from_anyhow(e.into()))
}

#[tauri::command]
fn save_binary_file(path: PathBuf, data: Vec<u8>) -> Result<(), InvokeError> {
    std::fs::write(path, data).map_err(|e| InvokeError::from_anyhow(e.into()))
}

#[tauri::command]
async fn import_chart(path: PathBuf) -> Result<ImportedChart, InvokeError> {
    wrap_async(async move {
        #[cfg(target_os = "android")]
        let path = tokio::task::spawn_blocking(move || android_uri::resolve(&path)).await??;

        #[cfg(not(target_os = "android"))]
        let path = path.to_path_buf();

        #[cfg(target_os = "android")]
        android_uri::debug_log(&format!("parse_chart: {}", path.display()));

        let mut fs: Box<dyn FileSystem + Send + Sync + 'static> =
            fs::fs_from_file(&path).with_context(|| mtl!("read-chart-failed"))?;
        let info = fs::load_info(fs.deref_mut())
            .await
            .with_context(|| mtl!("load-info-failed"))?;

        #[cfg(target_os = "android")]
        android_uri::debug_log("parse_chart: ok");

        Ok(ImportedChart { path, info })
    })
    .await
}

#[derive(Serialize)]
struct ImportedChart { path: PathBuf, info: ChartInfo }

#[tauri::command]
async fn parse_chart(path: PathBuf) -> Result<ChartInfo, InvokeError> {
    Ok(import_chart(path).await?.info)
}

#[tauri::command]
async fn pick_chart_files(directory: bool, multiple: bool) -> Result<Vec<String>, InvokeError> {
    #[cfg(target_os = "android")]
    return wrap_async(android_platform::pick_charts(directory, multiple)).await;
    #[cfg(not(target_os = "android"))]
    Err(InvokeError::from("Use the desktop file picker"))
}

#[tauri::command]
async fn view_video(uri: Option<String>) -> Result<(), InvokeError> {
    #[cfg(target_os = "android")]
    return wrap_async(async move {
        tokio::task::spawn_blocking(move || android_platform::call::<serde_json::Value>("viewVideo", serde_json::json!({ "uri": uri }))).await??;
        Ok(())
    }).await;
    #[cfg(not(target_os = "android"))]
    Err(InvokeError::from("Open your gallery to view videos"))
}

#[tauri::command]
async fn load_chart_illustration(path: &Path, illustration: &str) -> Result<tauri::ipc::Response, InvokeError> {
    wrap_async(async move {
        let mut fs = fs::fs_from_file(path).with_context(|| mtl!("read-chart-failed"))?;
        let data = fs.load_file(illustration).await?;
        Ok(tauri::ipc::Response::new(data))
    })
    .await
}

#[tauri::command]
async fn preview_chart(params: RenderParams) -> Result<(), InvokeError> {
    #[cfg(target_os = "android")]
    return wrap_async(async move {
        let mut params = params;
        let path = params.path.clone();
        params.path = tokio::task::spawn_blocking(move || android_uri::resolve(&path)).await??;
        android_platform::launch_preview(params).await
    }).await;
    #[cfg(not(target_os = "android"))]
    wrap_async(async move {
        let mut child = Command::new(std::env::current_exe()?)
            .arg("preview")
            .arg(ASSET_PATH.get().unwrap())
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut stdin = child.stdin.take().unwrap();
        stdin
            .write_all(format!("{}\n", serde_json::to_string(&params)?).as_bytes())
            .await?;

        Ok(())
    })
    .await
}
#[tauri::command]
async fn post_render(queue: State<'_, TaskQueue>, params: RenderParams) -> Result<(), InvokeError> {
    wrap_async(async move {
        #[cfg(target_os = "android")]
        let mut params = params;

        #[cfg(target_os = "android")]
        {
            let path = params.path.clone();
            params.path = tokio::task::spawn_blocking(move || android_uri::resolve(&path)).await??;
        }

        queue.post(params).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
async fn get_tasks(queue: State<'_, TaskQueue>) -> Result<Vec<TaskView>, InvokeError> {
    wrap_async(async move { Ok(queue.tasks().await) }).await
}

#[tauri::command]
async fn cancel_task(queue: State<'_, TaskQueue>, id: u32) -> Result<(), InvokeError> {
    queue.cancel(id).await;
    Ok(())
}

#[derive(Serialize)]
struct RespackInfo {
    name: String,
    path: String,
}
#[tauri::command]
fn get_respacks() -> Result<Vec<RespackInfo>, InvokeError> {
    (|| {
        let dir = respack_dir()?;
        let mut names: Vec<RespackInfo> = dir
            .read_dir()?
            .filter_map(|it| {
                it.ok()
                    .filter(|it| it.path().is_file())
                    .map(|it| RespackInfo {
                        name: it.file_name().to_str().unwrap().to_owned(),
                        path: it.path().canonicalize().unwrap().display().to_string(),
                    })
            })
            .collect();
        names.sort_by(|x, y| x.name.cmp(&y.name));
        Ok(names)
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
async fn batch_add_tasks(
    queue: State<'_, TaskQueue>,
    tasks: Vec<(String, String)> // (path, preset_name)
) -> Result<(), InvokeError> {
    wrap_async(async move {
        for (path, preset_name) in tasks {
            let mut presets = get_presets().await.map_err(|e| anyhow::anyhow!("Failed to get presets: {:?}", e))?;
            let config = presets.remove(&preset_name).unwrap_or_else(|| {
                create_default_render_config()
            });
            #[cfg(target_os = "android")]
            let path = android_uri::resolve(Path::new(&path))?;

            let params = RenderParams {
                path: PathBuf::from(path),
                config,
                info: ChartInfo::default(),
            };
            queue.post(params).await.map_err(|e| anyhow::anyhow!("Failed to post task: {:?}", e))?;
        }
        Ok(())
    }).await
}

#[tauri::command]
async fn list_chart_files(path: PathBuf) -> Result<Vec<String>, InvokeError> {
    wrap_async(async move {
        let mut files = Vec::new();
        let mut dirs = vec![path];
        while let Some(dir) = dirs.pop() {
            let mut entries = tokio::fs::read_dir(dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ["zip", "json", "pez"].contains(&ext) {
                        files.push(path.to_str().context("Invalid path")?.to_owned());
                    }
                }
            }
        }
        Ok(files)
    })
        .await
}

pub fn create_default_render_config() -> RenderConfig {
    RenderConfig::default()
}

#[tauri::command]
fn open_respack_folder() -> Result<(), InvokeError> {
    (|| {
        open::that_detached(respack_dir()?)?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

fn get_presets_file() -> Result<PathBuf> {
    let file = CONFIG_DIR.get().unwrap().join("presets.json");
    if file.exists() && !file.is_file() {
        bail!("presets.json is not a file");
    }
    Ok(file)
}

#[tauri::command]
async fn get_presets() -> Result<HashMap<String, RenderConfig>, InvokeError> {
    (|| {
        let file = get_presets_file()?;
        Ok(if !file.exists() {
            HashMap::new()
        } else {
            serde_json::from_reader(BufReader::new(File::open(file)?))?
        })
    })()
    .map_err(InvokeError::from_anyhow)
}

async fn save_presets(presets: &HashMap<String, RenderConfig>) -> Result<()> {
    serde_json::to_writer(BufWriter::new(File::create(get_presets_file()?)?), presets)?;
    Ok(())
}

#[tauri::command]
async fn add_preset(name: String, config: RenderConfig) -> Result<(), InvokeError> {
    let mut presets = get_presets().await?;
    wrap_async(async move {
        if presets.insert(name, config).is_some() {
            bail!(mtl!("preset-exists"));
        }
        save_presets(&presets).await?;
        Ok(())
    })
    .await
}

#[tauri::command]
async fn remove_preset(name: String) -> Result<(), InvokeError> {
    let mut presets = get_presets().await?;
    wrap_async(async move {
        if presets.remove(&name).is_none() {
            bail!(mtl!("preset-not-found"));
        }
        save_presets(&presets).await?;
        Ok(())
    })
    .await
}

fn rpe_dir() -> Result<Option<PathBuf>> {
    let file = CONFIG_DIR.get().unwrap().join("rpe_path.txt");
    if file.exists() {
        if !file.is_file() {
            bail!("rpe_path.txt is not a file");
        }
    } else {
        return Ok(None);
    }
    let dir: PathBuf = std::fs::read_to_string(file)?.into();
    Ok(if dir.exists() { Some(dir) } else { None })
}

#[derive(Serialize)]
pub struct RPEChartInfo {
    name: String,
    id: String,
    path: String,
    illustration: String,
    charter: String,
    #[serde(skip)]
    modified: SystemTime,
}

#[tauri::command]
fn set_rpe_dir(path: PathBuf) -> Result<(), InvokeError> {
    (|| {
        if !path.is_dir()
            || ["PhiEdit.exe", "Resources"]
                .iter()
                .any(|it| !path.join(*it).exists())
        {
            bail!(mtl!("not-valid-rpe"));
        }
        std::fs::write(
            CONFIG_DIR.get().unwrap().join("rpe_path.txt"),
            path.canonicalize()?.display().to_string().as_bytes(),
        )?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn unset_rpe_dir() -> Result<(), InvokeError> {
    (|| {
        std::fs::remove_file(CONFIG_DIR.get().unwrap().join("rpe_path.txt"))?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn get_rpe_charts() -> Result<Option<Vec<RPEChartInfo>>, InvokeError> {
    (|| {
        let Some(dir) = rpe_dir()? else {
            return Ok(None);
        };
        let mut results = Vec::new();
        let mut name = None;
        let mut id = None;
        let mut chart = None;
        let mut illustration = None;
        let mut charter = None;
        macro_rules! commit {
            () => {
                (|| {
                    let id = id.take()?;
                    let path = dir.join("Resources").join(&id);
                    let metadata = path.join(chart.take()?).metadata();
                    results.push(RPEChartInfo {
                        name: name.take()?,
                        id,
                        path: path.display().to_string(),
                        illustration: path.join(illustration.take()?).display().to_string(),
                        charter: charter.take()?,
                        modified: metadata
                            .and_then(|it| it.modified())
                            .unwrap_or(SystemTime::UNIX_EPOCH),
                    });
                    Some(())
                })()
            };
        }
        if dir.join("Chartlist.txt").exists() {
            for line in BufReader::new(File::open(dir.join("Chartlist.txt"))?).lines() {
                let line = line?;
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if line == "#" {
                    commit!();
                    continue;
                }
                let Some((key, value)) = line.split_once(':') else {
                    continue;
                };
                *(match key {
                    "Name" => &mut name,
                    "Path" => &mut id,
                    "Chart" => &mut chart,
                    "Picture" => &mut illustration,
                    "Charter" => &mut charter,
                    _ => continue,
                }) = Some(value.trim().to_owned());
            }
            commit!();
        } else {
            use regex::Regex;
            let onely_num = Regex::new(r"^\d+$").unwrap();
            let mut folders = Vec::new();
            let path = dir.join("Resources");
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(folder_name) = path.file_name() {
                        if onely_num.is_match(folder_name.to_str().unwrap_or("")) {
                            folders.push(path);
                        }
                    }
                }
            }
            for folder in folders {
                println!("Found numeric folder: {:?}", folder);
                for line in BufReader::new(File::open(folder.join("info.txt"))?).lines() {
                    let line = line?;
                    let line = line.trim();
                    if line.is_empty() {
                        continue;
                    }
                    if line == "#" {
                        commit!();
                        continue;
                    }
                    let Some((key, value)) = line.split_once(':') else {
                        continue;
                    };
                    *(match key {
                        "Name" => &mut name,
                        "Path" => &mut id,
                        "Chart" => &mut chart,
                        "Picture" => &mut illustration,
                        "Charter" => &mut charter,
                        _ => continue,
                    }) = Some(value.trim().to_owned());
                }
                commit!();
            }
        }

        results.sort_by_key(|it| it.modified);
        results.reverse();

        Ok(Some(results))
    })()
    .map_err(InvokeError::from_anyhow)
}

#[tauri::command]
fn open_app_folder() -> Result<(), InvokeError> {
    (|| {
        open::that_detached(std::env::current_exe()?.parent().unwrap())?;
        Ok(())
    })()
    .map_err(InvokeError::from_anyhow)
}
