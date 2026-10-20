prpr::tl_file!("render");

use crate::Path;
use anyhow::{bail, Context, Result};
use macroquad::{miniquad::{MipmapFilter, RenderPass as MQRenderPass, Texture, TextureFormat, TextureParams, TextureWrap}, prelude::*, window::miniquad_context};
use prpr::{
    config::{ChallengeModeColor, Config, Mods},
    core::{MSRenderTarget},
    ext::SafeTexture,
    fs,
    info::ChartInfo,
    scene::{BasicPlayer, GameMode, GameScene, LoadingScene},
    time::TimeManager,
    ui::{FontArc, TextPainter},
    Main,
};
use sasa::AudioClip;
use serde::{Deserialize, Serialize};
use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, VecDeque},
    io::{BufRead, BufWriter, Write},
    ops::DerefMut,
    path::PathBuf,
    process::{Command, Stdio},
    rc::Rc,
    sync::{
        atomic::{AtomicU64, AtomicUsize, Ordering},
        mpsc, Condvar, Mutex,
    },
    time::Instant,
};
use std::{ffi::OsStr, fmt::Write as _};
use tempfile::NamedTempFile;

const POOL_TARGET_BYTES: usize = 256 << 20;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct RenderConfig {
    pub resolution: (u32, u32),
    pub ffmpeg_preset: String,
    pub ending_length: f64,
    pub disable_loading: bool,
    pub audio_delay_frames: i32,
    pub chart_debug: bool,
    pub flid_x: bool,
    pub chart_ratio: f32,
    pub buffer_size: f32,
    pub combo: String,
    pub fps: u32,
    pub hardware_accel: bool,
    pub video_codec: String,
    pub encoder: String,
    pub show_progress_text: bool,
    pub show_time_text: bool,
    pub target_audio: u32,
    pub autoplay: Option<bool>,
    pub bitrate_control: String,
    pub bitrate: String,
    pub watermark: String,
    pub background: bool,
    pub video: bool,
    pub audio_bit: Option<u32>,
    pub audio_format: String,

    pub aggressive: bool,
    pub challenge_color: ChallengeModeColor,
    pub challenge_rank: u32,
    pub disable_effect: bool,
    pub double_hint: bool,
    pub fxaa: bool,
    pub note_scale: f32,
    pub particle: bool,
    pub player_avatar: Option<String>,
    pub player_name: String,
    pub player_rks: f32,
    pub sample_count: u32,
    pub res_pack_path: Option<String>,
    pub speed: f32,
    pub volume_music: f32,
    pub volume_sfx: f32,

    pub hand_split: bool,
    pub note_speed_factor: f32,
    pub bar: bool,
    pub score_aim: bool,

    pub ui_score: bool,
    pub ui_combo: bool,
    pub ui_name: bool,
    pub ui_level: bool,
    pub ui_line: bool,
    pub ui_pb: bool,
    pub ui_pause: bool,

    pub ffmpeg_thread: bool,

    pub gpu_yuv: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            ffmpeg_preset: "medium p4 balanced".to_string(),
            ending_length: -2.0,
            disable_loading: true,
            chart_debug: false,
            audio_delay_frames: 0,
            flid_x: false,
            chart_ratio: 1.0,
            buffer_size: 256.0,
            combo: "AUTOPLAY".to_string(),
            fps: 60,
            hardware_accel: true,
            video_codec: "h264".to_string(),
            encoder: "auto".to_string(),
            show_progress_text: false,
            show_time_text: false,
            target_audio: 44100,
            autoplay: None,
            bitrate_control: "CRF".to_string(),
            bitrate: "28".to_string(),
            watermark: "".to_string(),
            background: false,
            aggressive: false,
            challenge_color: ChallengeModeColor::Golden,
            challenge_rank: 45,
            disable_effect: false,
            double_hint: true,
            fxaa: false,
            note_scale: 1.0,
            particle: true,
            player_avatar: None,
            player_name: "".to_string(),
            player_rks: 15.0,
            sample_count: 4,
            res_pack_path: None,
            speed: 1.0,
            volume_music: 1.0,
            volume_sfx: 1.0,
            hand_split: false,
            note_speed_factor: 1.0,
            video: false,
            audio_bit: None,
            audio_format: "flac".to_string(),
            ui_score: true,
            ui_combo: true,
            ui_name: true,
            ui_level: true,
            ui_line: true,
            ui_pb: true,
            ui_pause: true,
            bar: false,
            score_aim: false,

            ffmpeg_thread: false,

            gpu_yuv: true,
        }
    }
}

impl RenderConfig {
    pub fn to_config(&self) -> Config {
        Config {
            aggressive: self.aggressive,
            challenge_color: self.challenge_color.clone(),
            challenge_rank: self.challenge_rank,
            disable_effect: self.disable_effect,
            double_hint: self.double_hint,
            fxaa: self.fxaa,
            note_scale: self.note_scale,
            particle: self.particle,
            player_name: self.player_name.clone(),
            player_rks: self.player_rks,
            sample_count: self.sample_count,
            res_pack_path: self.res_pack_path.clone(),
            speed: self.speed,
            volume_music: self.volume_music,
            volume_sfx: self.volume_sfx,
            chart_debug: self.chart_debug,
            chart_ratio: self.chart_ratio,
            buffer_size: self.buffer_size,
            combo: self.combo.clone(),
            flid_x: self.flid_x,
            show_progress_text: self.show_progress_text,
            show_time_text: self.show_time_text,
            autoplay: self.autoplay,
            watermark: self.watermark.clone(),
            background: self.background.clone(),
            disable_loading: self.disable_loading,
            hand_split: self.hand_split,
            note_speed_factor: self.note_speed_factor,

            ui_score: self.ui_score,
            ui_combo: self.ui_combo,
            ui_name: self.ui_name,
            ui_level: self.ui_level,
            ui_line: self.ui_line,
            ui_pb: self.ui_pb,
            ui_pause: self.ui_pause,
            bar: self.bar,
            score_aim: self.score_aim,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderParams {
    pub path: PathBuf,
    pub info: ChartInfo,
    pub config: RenderConfig,
}

#[derive(Serialize, Deserialize)]
pub enum IPCEvent {
    StartMixing,
    StartRender(u64),
    Frame,
    EncoderFps(f64),
    Done(f64),
}

struct FramePipeline {
    pool: Vec<Vec<u8>>,
    pool_addrs: Vec<usize>,
    free_pool: Mutex<Vec<usize>>,
    free_cv: Condvar,
    jobq: Mutex<(VecDeque<usize>, bool)>, // (queue, stopped)
    jobq_cv: Condvar,
    /// How often / how long the render thread had to wait for a free buffer.
    acquire_blocks: AtomicU64,
    acquire_wait_ns: AtomicU64,
    acquire_max_ns: AtomicU64,
    /// Deepest writer backlog ever observed, in frames.
    queue_max: AtomicUsize,
    /// Time spent inside `write_all` (i.e. blocked on the ffmpeg pipe).
    write_ns: AtomicU64,
    write_count: AtomicU64,
    write_max_ns: AtomicU64,
}

struct EncoderAvailability {
    h264_nvenc: bool,
    hevc_nvenc: bool,
    h264_qsv: bool,
    hevc_qsv: bool,
    h264_amf: bool,
    hevc_amf: bool,
    av1_nvenc: bool,
    av1_amf: bool,
    av1_qsv: bool,
    h264_cuvid: bool,
    hevc_cuvid: bool,
    av1_cuvid: bool,
    h264_vulkan: bool,
    hevc_vulkan: bool,
    av1_vulkan: bool,
    av1_svt: bool,
}

#[cfg(target_os = "windows")]
mod hw_detect {
    use std::path::Path;
    use winreg::{enums::HKEY_LOCAL_MACHINE, RegKey};

    pub fn detect_nvidia() -> bool {
        use std::process::Command;
        use winreg::enums::HKEY_LOCAL_MACHINE;
        use winreg::RegKey;
        use std::path::Path;

        if let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}")
        {
            for subkey_name in key.enum_keys().filter_map(|x| x.ok()) {
                if let Ok(subkey) = key.open_subkey(&subkey_name) {
                    if let Ok(provider) = subkey.get_value::<String, _>("ProviderName") {
                        if provider.to_lowercase().contains("nvidia") {
                            return true;
                        }
                    }
                }
            }
        }
        if Path::new(r"C:\Windows\System32\nvcuda.dll").exists() {
            return true;
        }
        Command::new("nvidia-smi").output().is_ok()
    }

    pub fn detect_intel_qsv() -> bool {
        let mut found = false;
        let classes = [
            "{4d36e968-e325-11ce-bfc1-08002be10318}",
            "{4d36e97d-e325-11ce-bfc1-08002be10318}",
        ];

        for class in classes {
            if let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE)
                .open_subkey(format!(r"SYSTEM\CurrentControlSet\Control\Class\{}", class))
            {
                for subkey in key.enum_keys().filter_map(|x| x.ok()) {
                    if let Ok(subkey) = key.open_subkey(subkey) {
                        if let Ok(provider) = subkey.get_value::<String, _>("ProviderName") {
                            if provider.contains("Intel") {
                                found = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        found
    }

    pub fn detect_amd() -> bool {
        Path::new(r"C:\Windows\System32\amdvlk64.dll").exists()
            || Path::new(r"C:\Windows\System32\amfrt64.dll").exists()
    }

    pub fn detect_vulkan() -> bool {
        Path::new(r"C:\Windows\System32\vulkan-1.dll").exists()
    }
}

#[cfg(target_os = "linux")]
mod hw_detect {
    use std::path::Path;
    use std::process::Command;

    pub fn detect_nvidia() -> bool {
        Path::new("/dev/nvidia0").exists() || Command::new("nvidia-smi").status().is_ok()
    }

    pub fn detect_intel_qsv() -> bool {
        Path::new("/dev/dri/renderD128").exists()
            && Command::new("vainfo")
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).contains("VAProfileH264"))
            .unwrap_or(false)
    }

    pub fn detect_amd() -> bool {
        Path::new("/dev/kfd").exists()
            && Command::new("vainfo")
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).contains("AMD"))
            .unwrap_or(false)
    }

    pub fn detect_vulkan() -> bool {
        Path::new("/usr/share/vulkan/icd.d").exists()
            || Path::new("/etc/vulkan/icd.d").exists()
            || Path::new("/usr/local/share/vulkan/icd.d").exists()
    }
}

#[cfg(target_os = "macos")]
mod hw_detect {
    use std::process::Command;

    pub fn detect_nvidia() -> bool {
        Command::new("system_profiler")
            .args(&["SPDisplaysDataType"])
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).contains("NVIDIA"))
            .unwrap_or(false)
    }

    pub fn detect_intel_qsv() -> bool {
        Command::new("system_profiler")
            .args(&["SPDisplaysDataType"])
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).contains("Intel"))
            .unwrap_or(false)
    }

    pub fn detect_amd() -> bool {
        Command::new("system_profiler")
            .args(&["SPDisplaysDataType"])
            .output()
            .map(|out| String::from_utf8_lossy(&out.stdout).contains("AMD"))
            .unwrap_or(false)
    }

    pub fn detect_vulkan() -> bool {
        Command::new("sh")
            .arg("-c")
            .arg("ls /usr/local/lib/libMoltenVK.dylib 2>/dev/null || ls /opt/homebrew/lib/libMoltenVK.dylib 2>/dev/null || ls ~/Library/Frameworks/libMoltenVK.dylib 2>/dev/null")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
}

pub async fn build_player(config: &RenderConfig) -> Result<BasicPlayer> {
    Ok(BasicPlayer {
        avatar: if let Some(path) = &config.player_avatar {
            Some(
                SafeTexture::from(
                    Texture2D::from_file_with_format(
                        &tokio::fs::read(path)
                            .await
                            .with_context(|| tl!("load-avatar-failed"))?,
                        None,
                    )
                ),
            )
        } else {
            None
        },
        id: 0,
        rks: config.player_rks,
    })
}

const DX_DEVICE: &str = "dx";

pub(crate) fn cmd_hidden(program: impl AsRef<OsStr>) -> Command {
    let cmd = Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = cmd;
        cmd.creation_flags(0x08000000);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    cmd
}

/// Bind a loopback TCP listener for the raw video input.
pub fn drain_stderr<R>(mut stderr: R) -> std::thread::JoinHandle<String>where
    R: std::io::Read + Send + 'static,
{
    const LIMIT: usize = 64 * 1024;
    std::thread::Builder::new()
        .name("ffmpeg-stderr".to_owned())
        .spawn(move || {
            let mut collected: Vec<u8> = Vec::new();
            let mut buf = [0u8; 8192];
            loop {
                match stderr.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if collected.len() < LIMIT {
                            let room = LIMIT - collected.len();
                            collected.extend_from_slice(&buf[..n.min(room)]);
                        }
                    }
                }
            }
            String::from_utf8_lossy(&collected).into_owned()
        })
        .expect("failed to spawn ffmpeg stderr reader")
}

pub fn drain_stderr_with_progress<R>(
    mut stderr: R,
    frames_written: std::sync::Arc<std::sync::atomic::AtomicU64>,
) -> std::thread::JoinHandle<String>
where
    R: std::io::Read + Send + 'static,
{
    const LIMIT: usize = 64 * 1024;
    std::thread::Builder::new()
        .name("ffmpeg-progress".to_owned())
        .spawn(move || {
            let mut collected: Vec<u8> = Vec::new();
            let mut line: Vec<u8> = Vec::new();
            let mut buf = [0u8; 8192];
            let mut last_sent = f64::NAN;
            loop {
                match stderr.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        for &b in &buf[..n] {
                            if b != b'\n' && b != b'\r' {
                                line.push(b);
                                continue;
                            }
                            if let Some(n) = parse_progress_u64(&line, "frame=") {
                                frames_written.store(n, Ordering::Relaxed);
                            }
                            if let Some(fps) = parse_progress_fps(&line) {
                                if fps > 0.0 && fps != last_sent {
                                    last_sent = fps;
                                    crate::ipc::client::send(IPCEvent::EncoderFps(fps));
                                }
                            } else if !is_progress_line(&line) && collected.len() < LIMIT {
                                let room = LIMIT - collected.len();
                                collected.extend_from_slice(&line[..line.len().min(room)]);
                                collected.push(b'\n');
                            }
                            line.clear();
                        }
                    }
                }
            }
            String::from_utf8_lossy(&collected).into_owned()
        })
        .expect("failed to spawn ffmpeg progress reader")
}

fn parse_progress_fps(line: &[u8]) -> Option<f64> {
    let text = std::str::from_utf8(line).ok()?;
    let rest = text.strip_prefix("fps=")?;
    let digits: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

fn parse_progress_u64(line: &[u8], key: &str) -> Option<u64> {
    let text = std::str::from_utf8(line).ok()?;
    let rest = text.strip_prefix(key)?;
    let digits: String = rest
        .trim_start()
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    if digits.is_empty() { None } else { digits.parse().ok() }
}

fn is_progress_line(line: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(line) else { return false; };
    let Some((key, _)) = text.split_once('=') else { return false; };
    matches!(
        key,
        "frame"
            | "fps"
            | "bitrate"
            | "total_size"
            | "out_time_us"
            | "out_time_ms"
            | "out_time"
            | "dup_frames"
            | "drop_frames"
            | "speed"
            | "progress"
    ) || key.starts_with("stream_")
}


pub fn find_ffmpeg() -> Result<Option<String>> {
    fn test(path: impl AsRef<OsStr>) -> bool {
        matches!(cmd_hidden(path).arg("-version").output(), Ok(_))
    }

    let ffmpeg_exe = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    let exe_dir = std::env::current_exe()?
        .parent()
        .expect("Executable should have parent directory")
        .to_owned();
    let bundled_ffmpeg = exe_dir.join(ffmpeg_exe);
    if test(&bundled_ffmpeg) {
        return Ok(Some(bundled_ffmpeg.to_string_lossy().into_owned()));
    }

    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(ffmpeg_exe);
            if test(&candidate) {
                return Ok(Some(candidate.to_string_lossy().into_owned()));
            }
        }
    }

    eprintln!("Failed to find global ffmpeg. Using bundled ffmpeg");
    Ok(if test(&bundled_ffmpeg) {
        Some(bundled_ffmpeg.to_string_lossy().into_owned())
    } else {
        None
    })
}




pub(crate) fn new_plane(w: u32, h: u32, format: TextureFormat) -> RenderTarget {
    let quad_context = miniquad_context();
    let texture = Texture::new_render_texture(
        quad_context,
        TextureParams {
            width: w,
            height: h,
            format,
            filter: FilterMode::Nearest,
            wrap: TextureWrap::Clamp,
            mipmap_filter: MipmapFilter::None,
        },
    );
    RenderTarget {
        texture: Texture2D::from_miniquad_texture(texture),
        render_pass: MQRenderPass::new(quad_context, texture, None),
    }
}

fn yuv_material(fragment: &str) -> Result<Material> {
    load_material(
        yuv_shader::VERTEX,
        fragment,
        MaterialParams {
            pipeline_params: PipelineParams::default(),
            uniforms: Vec::new(),
            textures: vec!["tex".to_owned()],
        },
    )
    .map_err(|e| anyhow::anyhow!("{e:?}"))
}

struct YuvTarget {
    y: RenderTarget,
    uv: RenderTarget,
    y_material: Material,
    uv_material: Material,
    y_index: usize,
    uv_index: usize,
}

impl YuvTarget {
    fn new(w: u32, h: u32) -> Result<Self> {
        let y = new_plane(w, h, TextureFormat::Alpha);
        let uv = new_plane(w / 2, h / 2, TextureFormat::LuminanceAlpha);
        Ok(Self {
            y_index: y.texture.raw_miniquad_texture_handle().gl_internal_id() as usize,
            uv_index: uv.texture.raw_miniquad_texture_handle().gl_internal_id() as usize,
            y,
            uv,
            y_material: yuv_material(&yuv_shader::Y)?,
            uv_material: yuv_material(&yuv_shader::UV)?,
        })
    }

    fn convert(&self, src: &RenderTarget) {
        self.y_material.set_texture("tex", src.texture);
        self.uv_material.set_texture("tex", src.texture);
        let vertices = [
            Vertex::new(-1., -1., 0., 0., 0., WHITE),
            Vertex::new(1., -1., 0., 1., 0., WHITE),
            Vertex::new(-1., 1., 0., 0., 1., WHITE),
            Vertex::new(1., 1., 0., 1., 1., WHITE),
        ];
        let indices = [0u16, 2, 3, 0, 1, 3];
        let previous = unsafe { get_internal_gl() }.quad_gl.get_viewport();
        for (material, target) in [(&self.y_material, &self.y), (&self.uv_material, &self.uv)] {
            gl_use_material(*material);
            let quad = unsafe { get_internal_gl() }.quad_gl;
            quad.scissor(None);
            quad.viewport(None);
            quad.render_pass(Some(target.render_pass));
            quad.draw_mode(DrawMode::Triangles);
            quad.geometry(&vertices, &indices);
            unsafe { get_internal_gl() }.flush();
            gl_use_default_material();
        }
        let quad = unsafe { get_internal_gl() }.quad_gl;
        quad.render_pass(Some(src.render_pass));
        quad.viewport(previous);
    }
}

struct SendVideoContext(crate::vulkan_video::VideoContext);
unsafe impl Send for SendVideoContext {}

struct VulkanPipe {
    job_tx: mpsc::Sender<(u64, usize)>,
    nv12_raw: Vec<u64>,
    free: VecDeque<usize>,
    free_rx: mpsc::Receiver<usize>,
}

struct Nv12Pipe {
    readback: crate::vulkan_readback::Nv12Readback,
    free: VecDeque<usize>,
    ptrs: Vec<usize>,
    frame_bytes: usize,
    pending: [Option<(u64, usize)>; 2],
    current_read: Option<usize>,
}

impl FramePipeline {
    fn new(frame_size: usize) -> Self {
        let pool_size = (POOL_TARGET_BYTES / frame_size.max(1)).clamp(8, 32);
        Self {
            pool: (0..pool_size).map(|_| vec![0u8; frame_size]).collect(),
            pool_addrs: Vec::new(),
            free_pool: Mutex::new((0..pool_size).collect()),
            free_cv: Condvar::new(),
            jobq: Mutex::new((VecDeque::new(), false)),
            jobq_cv: Condvar::new(),

            acquire_blocks: AtomicU64::new(0),
            acquire_wait_ns: AtomicU64::new(0),
            acquire_max_ns: AtomicU64::new(0),

            queue_max: AtomicUsize::new(0),

            write_ns: AtomicU64::new(0),
            write_count: AtomicU64::new(0),
            write_max_ns: AtomicU64::new(0),
        }
    }

    fn finish_init(&mut self) {
        self.pool_addrs = self.pool.iter_mut().map(|b| b.as_mut_ptr() as usize).collect();
    }

    fn acquire_buf(&self) -> usize {
        let mut free = self.free_pool.lock().unwrap();
        let mut wait_start: Option<Instant> = None;
        loop {
            if let Some(idx) = free.pop() {
                if let Some(start) = wait_start {
                    let d = start.elapsed().as_nanos() as u64;
                    self.acquire_blocks.fetch_add(1, Ordering::Relaxed);
                    self.acquire_wait_ns.fetch_add(d, Ordering::Relaxed);
                    self.acquire_max_ns.fetch_max(d, Ordering::Relaxed);
                }
                return idx;
            }
            wait_start.get_or_insert_with(Instant::now);
            free = self.free_cv.wait(free).unwrap();
        }
    }

    fn release_buf(&self, idx: usize) {
        self.free_pool.lock().unwrap().push(idx);
        self.free_cv.notify_one();
    }

    fn push_job(&self, idx: usize) {
        let mut jobq = self.jobq.lock().unwrap();
        jobq.0.push_back(idx);
        let queued = jobq.0.len();
        drop(jobq);
        self.queue_max.fetch_max(queued, Ordering::Relaxed);
        self.jobq_cv.notify_one();
    }

    fn push_stop(&self) {
        self.jobq.lock().unwrap().1 = true;
        self.jobq_cv.notify_one();
    }

    fn pop_job(&self) -> Option<usize> {
        let mut jobq = self.jobq.lock().unwrap();
        loop {
            if let Some(idx) = jobq.0.pop_front() {
                return Some(idx);
            }
            if jobq.1 {
                return None;
            }
            jobq = self.jobq_cv.wait(jobq).unwrap();
        }
    }
}

/// Parse an ffmpeg-style bitrate (`8000k`, `8M`, `8000kbps`, or plain bits/s)
fn parse_bitrate_bps(raw: &str) -> Option<f64> {
    let s = raw.trim().to_ascii_lowercase().replace(' ', "");
    if s.is_empty() {
        return None;
    }
    let (num, mult) = if let Some(v) = s.strip_suffix("kbps") {
        (v, 1e3)
    } else if let Some(v) = s.strip_suffix("mbps") {
        (v, 1e6)
    } else if let Some(v) = s.strip_suffix('k') {
        (v, 1e3)
    } else if let Some(v) = s.strip_suffix('m') {
        (v, 1e6)
    } else {
        (s.as_str(), 1.0)
    };
    let num: f64 = num.parse().ok()?;
    (num > 0.0).then_some(num * mult)
}

fn encoder_qp(config: &RenderConfig, width: u32, height: u32) -> u8 {
    let is_cbr = config.bitrate_control.eq_ignore_ascii_case("CBR");
    if is_cbr {
        if let Some(bps) = parse_bitrate_bps(&config.bitrate) {
            let pixels = width as f64 * height as f64 * config.fps.max(1) as f64;
            let bpp = bps / pixels.max(1.0);
            let qp = 28.0 - 6.0 * (bpp / 0.040).log2();
            return qp.round().clamp(1.0, 51.0) as u8;
        }
        warn!(
            "Invalid CBR bitrate '{}', falling back to QP 28",
            config.bitrate
        );
        return 28;
    }
    let crf = config
        .bitrate
        .trim()
        .parse::<f64>()
        .unwrap_or(28.0)
        .round();
    crf.clamp(1.0, 51.0) as u8
}

const KEYFRAME_INTERVAL_SECS: u32 = 2;

pub async fn main() -> Result<()> {
    use crate::ipc::client::*;

    let process_start = Instant::now();

    set_pc_assets_folder(&std::env::args().nth(2).unwrap());

    let mut stdin = std::io::stdin().lock();
    let stdin = &mut stdin;

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    let params: RenderParams = serde_json::from_str(line.trim())?;
    let path = params.path;

    line.clear();
    stdin.read_line(&mut line)?;
    let output_path: PathBuf = serde_json::from_str(line.trim())?;

    let mut fs = fs::fs_from_file(&path)?;

    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;

    let Some(ffmpeg) = find_ffmpeg()? else {
        bail!("FFmpeg not found")
    };
    info!("Using ffmpeg: {}", ffmpeg);

    let mut painter = TextPainter::new(font);

    let mut config = params.config.to_config();
    config.mods = Mods::AUTOPLAY;
    config.interactive = false;

    let info = params.info;

    let (chart, ..) = GameScene::load_chart(fs.deref_mut(), &info)
        .await
        .with_context(|| tl!("load-chart-failed"))?;
    macro_rules! ld {
            ($path:literal) => {
                AudioClip::new(load_file($path).await?)
                    .with_context(|| tl!("load-sfx-failed", "name" => $path))?
            };
        }
    let music: Result<_> = async { AudioClip::new(fs.load_file(&info.music).await?) }.await;
    let music = music.with_context(|| tl!("load-music-failed"))?;
    let ending = ld!("ending.mp3");
    let track_length = music.length() as f64;
    let sfx_click = ld!("click.ogg");
    let sfx_drag = ld!("drag.ogg");
    let sfx_flick = ld!("flick.ogg");


    let volume_music = std::mem::take(&mut config.volume_music);
    let volume_sfx = std::mem::take(&mut config.volume_sfx);

    let length = track_length - chart.offset.min(0.) as f64 + 1.;
    let video_length = O + length + A + params.config.ending_length;
    let offset = chart.offset.max(0.);

    let render_start_time = Instant::now();
    info!(
        "setup chart + audio load: {:.3}s",
        process_start.elapsed().as_secs_f64()
    );
    let mut phase = render_start_time;
    macro_rules! mark {
        ($name:literal) => {{
            let d = phase.elapsed().as_secs_f64();
            info!("setup {}: {:.3}s", $name, d);
            phase = Instant::now();
            d
        }};
    }

    send(IPCEvent::StartMixing);
    let target_sample_rate = params.config.target_audio;
    let sample_rate = 44100;
    let sample_rate_f64 = sample_rate as f64;
    assert_eq!(sample_rate, ending.sample_rate());
    assert_eq!(sample_rate, sfx_click.sample_rate());
    assert_eq!(sample_rate, sfx_drag.sample_rate());
    assert_eq!(sample_rate, sfx_flick.sample_rate());

    let fps_f64 = params.config.fps as f64;
    let frame_duration = 1.0 / fps_f64;
    let audio_delay = params.config.audio_delay_frames as f64 * frame_duration;

    //info!("=== Audio/Video Sync Configuration ===");
    //info!("  Audio delay: {} frames", params.config.audio_delay_frames);
    //info!("  Audio delay: {:.6} seconds", audio_delay);
    //info!("  Frame duration: {:.6}s @ {}fps", frame_duration, params.config.fps);
    //info!("  Sample delay: {} samples @ {}Hz", (audio_delay * sample_rate_f64).round() as i64, sample_rate);
    //info!("======================================");

    let audio_buffer_length = video_length + audio_delay.abs();
    let mut output = vec![0.0_f32; (audio_buffer_length * sample_rate_f64).ceil() as usize * 2];
    
    let ss = if params.config.disable_loading { O } else { 0.1 };
    let ss_samples = (ss * sample_rate_f64).round() as usize * 2;
    let skip = ((ss * params.config.fps as f64 - 1e-6).ceil().max(0.)) as u64;

    if volume_music != 0.0 {
        let start_time = Instant::now();
        let original_pos = O - chart.offset.min(0.) as f64;
        let pos = original_pos + audio_delay;

        //info!("Music mixing: original_pos={:.6}s, delayed_pos={:.6}s", original_pos, pos);

        let start_index = (pos * sample_rate_f64).round() as usize * 2;
        let ratio = 1.0 / sample_rate_f64;

        if start_index >= output.len() {
            warn!("Music start position {} exceeds output buffer length {}", start_index, output.len());
        } else {
            let output_ptr = output.as_mut_ptr();
            let max_i = (output.len() - start_index) / 2;
            let effective_count = ((music.length() as f64 * sample_rate_f64) as usize).min(max_i);
            let mut time = 0.0_f64;
            for i in 0..effective_count {
                let frame = music.sample(time as f32).unwrap_or_default();
                let left = frame.0 * volume_music;
                let right = frame.1 * volume_music;

                unsafe {
                    let idx = start_index + i * 2;
                    *output_ptr.add(idx) += left;
                    *output_ptr.add(idx + 1) += right;
                }
                time += ratio;
            }
        }
        info!("music Time:{:?}", start_time.elapsed());
    }

    let mut place = |pos: f64, clip: &AudioClip, volume: f32| {
        let position = (pos * sample_rate_f64).round() as usize * 2;
        if position >= output.len() {
            return 0;
        }
        let len = clip.frame_count();
        let output_len = output.len() - position;
        let valid_frames = (output_len / 2).min(len);

        let output_ptr = unsafe { output.as_mut_ptr().add(position) };
        let frames_ptr = clip.frames().as_ptr();

        for i in 0..valid_frames {
            unsafe {
                let sample = (*frames_ptr.add(i)).0 * volume;
                *output_ptr.add(i * 2) += sample;
                *output_ptr.add(i * 2 + 1) += sample;
            }
        }
        valid_frames
    };

    if volume_sfx != 0.0 {
        let start_time = Instant::now();

        let offset_f64 = offset as f64;
        let o_offset = O + offset_f64 + audio_delay;

        //info!("SFX mixing: offset={:.6}s (includes {:.6}s delay)", o_offset, audio_delay);

        let sfx_lut =
        [
            &sfx_click as *const _,
            &sfx_drag as *const _,
            &sfx_click as *const _,
            &sfx_flick as *const _,
        ];

        unsafe {
            let lines_ptr = chart.lines.as_ptr();
            let lines_len = chart.lines.len();

            for i in 0..lines_len {
                let line = &*lines_ptr.add(i);
                let notes_ptr = line.notes.as_ptr();
                let notes_len = line.notes.len();
                for j in 0..notes_len {
                    let note = &*notes_ptr.add(j);
                    if !note.fake {
                        let sfx = &*sfx_lut[note.kind.order() as usize];
                        let time = o_offset + note.time as f64;
                        place(time, sfx, volume_sfx);
                    }
                }
            }
        }

        info!("sfx Time:{:?}", start_time.elapsed());
    }

    let mut pos = O + length + A + audio_delay;
    info!("Ending music start: {:.6}s", pos);

    while place(pos, &ending, volume_music) != 0 && params.config.ending_length > 0.1 {
        pos += ending.frame_count() as f64 / sample_rate_f64;
    }
    let mixing_s = mark!("mixing");

    let audio_bit = params.config.audio_bit;
    let audio_format = params.config.audio_format.to_lowercase();

    let supported_formats = ["flac", "mp3", "aac", "opus", "wav"];
    if !supported_formats.contains(&audio_format.as_str()) {
        bail!
        ("Unsupported audio format: {}. Supported formats are: {}",

            audio_format,
            supported_formats.join(", ")

        );
    }

    if let Some(bit) = audio_bit {
        if ![16, 24, 32].contains(&bit)
        { bail!("Invalid audio bit depth: {}. Supported values are 16, 24, 32.", bit); }
        if audio_format != "wav"
        { return Err(anyhow::anyhow!("PCM audio bit depth requires WAV format, but {} was specified", audio_format)); }
    }

    let (audio_mux_args, output_format) = if let Some(bit) = audio_bit {
        (
            format!("-c:a pcm_f{}le", bit),
            "wav".to_string(),
        )
    } else {
        match audio_format.as_str()
        {
            "flac" => (
                "-c:a flac -sample_fmt s32 -bits_per_raw_sample 24 -strict -2".to_string(),
                "flac".to_string(),
            ),
            "mp3" => ("-c:a libmp3lame".to_string(), "mp3".to_string()),
            "aac" => ("-c:a aac".to_string(), "mp4".to_string()),
            "opus" => ("-c:a libopus".to_string(), "opus".to_string()),
            "wav" => ("-c:a pcm_f16le".to_string(), "wav".to_string()),
            _ => {
                warn!("Unknown audio format '{}', using AAC/MP4 as default", audio_format);
                ("-c:a aac".to_string(), "mp4".to_string())
            }
        }
    };

    let args_str = if target_sample_rate != sample_rate {
        let resample_filter = format!("aresample=resampler=soxr:precision=33:osr={}:dither_method=triangular", target_sample_rate);
        format!(
            "-y -f f32le -ar {} -ac 2 -i - -af {} {} -f {}",
            sample_rate,
            resample_filter,
            audio_mux_args,
            output_format
        )
    } else {
        format!(
            "-y -f f32le -ar {} -ac 2 -i - {} -f {}",
            sample_rate,
            audio_mux_args,
            output_format
        )
    };

    let audio_suffix = format!(".{output_format}");
    let audio_output_file = NamedTempFile::with_suffix(&audio_suffix)?;
    let mut proc = cmd_hidden(&ffmpeg)
        .args(args_str.split_whitespace())
        .arg(audio_output_file.path())
        .arg("-loglevel")
        .arg("warning")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| tl!("run-ffmpeg-failed"))?;
    let audio_stderr = drain_stderr(proc.stderr.take().unwrap());
    let input = proc.stdin.as_mut().unwrap();
    let mut writer = BufWriter::new(input);
    for sample in output.into_iter().skip(ss_samples) {
        writer.write_all(&sample.to_le_bytes())?;
    }
    drop(writer);
    let audio_status = proc.wait()?;
    if let Ok(text) = audio_stderr.join() {
        if !text.trim().is_empty() {
            warn!("[ffmpeg:audio]\n{}", text);
        }
    }
    if !audio_status.success() {
        bail!("audio encoding failed: {audio_status}");
    }
    let audio_s = mark!("audio encode + resample");



    let target_aspect = info.aspect_ratio as f64;
    let (mut vw, mut vh) = params.config.resolution;
    let (ow, oh) = (vw, vh);
    let aspect = vw as f64 / vh as f64;

    if (aspect - target_aspect).abs() > 1e-9 {
        if aspect > target_aspect {
            vw = (vh as f64 * target_aspect).round() as u32;
        } else {
            vh = (vw as f64 / target_aspect).round() as u32;
        }
        info!("{}x{} -> {}x{} (target {:.9})", ow, oh, vw, vh, target_aspect);
    }

    let qp = encoder_qp(&params.config, vw, vh);
    // The Rust Vulkan encoder implements H.264 only, so the existing
    // "Hardware Encoder", "Video Codec" and "Hardware Acceleration" controls
    // decide whether it is used at all.  Other selections keep using the
    // ffmpeg encoder path.
    let want_vulkan_video = params.config.hardware_accel
        && params.config.video_codec == "h264"
        && matches!(params.config.encoder.as_str(), "auto" | "vulkan");
    let mut video_context = if want_vulkan_video {
        crate::vulkan_video::VideoContext::new(vw, vh, qp, 16)
    } else {
        None
    };
    let vulkan_video = video_context.is_some();
    if want_vulkan_video && !vulkan_video {
        warn!("Rust Vulkan encoder unavailable; falling back to the ffmpeg encoder");
    }
    info!(
        "Vulkan Video H.264: {} ({} source images, {}x{}, qp={})",
        if vulkan_video { "enabled" } else { "disabled" },
        video_context
            .as_ref()
            .map(|context| context.source_count())
            .unwrap_or(0),
        vw,
        vh,
        qp
    );
    if want_vulkan_video {
        if params.config.bitrate_control.eq_ignore_ascii_case("CBR") {
            info!(
                "Vulkan encoder is constant-QP: CBR target '{}' mapped to qp={}",
                params.config.bitrate, qp
            );
        } else {
            info!(
                "Vulkan encoder quality: CRF {} -> qp={}",
                params.config.bitrate, qp
            );
        }
    }

    let yuv_count = if !vulkan_video && vw % 2 == 0 && vh % 2 == 0 {
        2
    } else {
        1
    };
    let yuvs: Vec<YuvTarget> = (0..yuv_count)
        .filter_map(|_| YuvTarget::new(vw, vh).ok())
        .collect();
    let mut ffmpeg_nv12 = if !vulkan_video && yuvs.len() == 2 {
        match crate::vulkan_readback::Nv12Readback::new(vw, vh, 8) {
            Some(readback) => {
                info!(
                    "ffmpeg NV12: Vulkan async readback, {} slots, {:.2} MB/frame",
                    readback.slot_count(),
                    readback.frame_size() as f64 / 1048576.0
                );
                Some(readback)
            }
            None => {
                warn!(
                    "ffmpeg NV12: readback ring creation failed, falling back to RGBA readback"
                );
                None
            }
        }
    } else {
        None
    };
    let nv12_mode = ffmpeg_nv12.is_some();
    let mst_count = if nv12_mode { 2 } else { 1 };
    let msts: Vec<Rc<MSRenderTarget>> = (0..mst_count)
        .map(|_| Rc::new(MSRenderTarget::new((vw, vh), config.sample_count)))
        .collect();
    let mst_slot = Rc::new(Cell::new(0usize));
    let my_time: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.));
    let tm = TimeManager::manual(Box::new({
        let my_time = Rc::clone(&my_time);
        move || *(*my_time).borrow()
    }));
    if nv12_mode && !crate::vulkan_readback::enable_offscreen_video() {
        bail!("failed to enable the Vulkan offscreen video mode");
    }
    mark!("offscreen/readback setup");
    //static MSAA: AtomicBool = AtomicBool::new(false);
    let player = build_player(&params.config).await?;
    let mut main = Main::new(
        Box::new(
            LoadingScene::new(GameMode::Normal, info, config, fs, Some(player), None, None).await?,
        ),
        tm,
        {
            let msts = Rc::new(msts.iter().cloned().collect::<Vec<_>>());
            let mst_slot = Rc::clone(&mst_slot);
            move || Some(msts[mst_slot.get()].input())
        },
    )
        .await?;
    main.top_level = false;
    main.viewport = Some((0, 0, vw as _, vh as _));

    *my_time.borrow_mut() = 0.;
    main.update()?;
    mark!("player + scene load");

    const O: f64 = LoadingScene::TOTAL_TIME as f64 + GameScene::BEFORE_TIME as f64;
    const A: f64 = 1.0;

    let fps = params.config.fps;
    let frames = (video_length * fps as f64).ceil() as u64;
    let out_frames = frames.saturating_sub(skip);
    send(IPCEvent::StartRender(out_frames));

    static VULKAN_PROBE_LOCK: Mutex<()> = Mutex::new(());

    fn test_encoder(ffmpeg: &Path, encoder: &str) -> Result<(bool, String)> {
        let _vulkan_guard = if encoder.ends_with("_vulkan") {
            Some(VULKAN_PROBE_LOCK.lock().unwrap_or_else(|e| e.into_inner()))
        } else {
            None
        };
        let mut cmd = Command::new(ffmpeg);

        if encoder.ends_with("_vulkan") {
            cmd.args(&[
                "-init_hw_device", "vulkan=vk",
                "-f", "lavfi",
                "-i", "testsrc=duration=0.1:size=320x240:rate=30",
                "-filter_hw_device", "vk",
                "-vf", "format=nv12,hwupload",
                "-c:v", encoder,
                "-f", "null", "-",
            ]);
        } else {
            cmd.args(&[
                "-f", "lavfi",
                "-i", "testsrc2=size=320x240:rate=30:duration=0.3",
                "-c:v", encoder,
                "-f", "null", "-",
            ]);
        }

        cmd.arg("-loglevel")
            .arg("warning")
            .arg("-hide_banner")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = cmd
            .output()
            .with_context(|| format!("Failed to start encoder test for {}", encoder))?;

        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        Ok((output.status.success(), stderr))
    }

    #[derive(Clone, Copy)]
    struct DxEncoder {
        name: &'static str,
        hw: &'static str,
        pix: &'static str,
    }

    fn probe_dx_encoder(ffmpeg: &Path, enc: DxEncoder) -> bool {
        let device = format!("{}={}", enc.hw, DX_DEVICE);
        let mut cmd = Command::new(ffmpeg);
        cmd.args([
            "-init_hw_device",
            device.as_str(),
            "-filter_hw_device",
            DX_DEVICE,
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=640x360:rate=30:duration=0.3",
            "-vf",
            "format=nv12,hwupload",
            "-c:v",
            enc.name,
            "-pix_fmt",
            enc.pix,
        ]);
        cmd.args(["-bf", "0"]);
        cmd.args(["-f", "null", "-", "-loglevel", "error"]);
        cmd.arg("-hide_banner")
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        matches!(cmd.output(), Ok(out) if out.status.success())
    }

    let hw_detected = EncoderAvailability {
        h264_nvenc: params.config.hardware_accel && hw_detect::detect_nvidia(),
        hevc_nvenc: params.config.hardware_accel
            && params.config.video_codec == "hevc"
            && hw_detect::detect_nvidia(),
        h264_qsv: params.config.hardware_accel && hw_detect::detect_intel_qsv(),
        hevc_qsv: params.config.hardware_accel
            && params.config.video_codec == "hevc"
            && hw_detect::detect_intel_qsv(),
        h264_amf: params.config.hardware_accel && hw_detect::detect_amd(),
        hevc_amf: params.config.hardware_accel && params.config.video_codec == "hevc" && hw_detect::detect_amd(),
        h264_cuvid: params.config.hardware_accel && hw_detect::detect_nvidia(),
        hevc_cuvid: params.config.hardware_accel && params.config.video_codec == "hevc" && hw_detect::detect_nvidia(),
        av1_cuvid: params.config.hardware_accel && params.config.video_codec == "av1" && hw_detect::detect_nvidia(),
        av1_nvenc: params.config.hardware_accel
            && params.config.video_codec == "av1"
            && hw_detect::detect_nvidia(),
        av1_qsv: params.config.hardware_accel
            && params.config.video_codec == "av1"
            && hw_detect::detect_intel_qsv(),
        av1_amf: params.config.hardware_accel
            && params.config.video_codec == "av1"
            && hw_detect::detect_amd(),
        h264_vulkan: params.config.hardware_accel && hw_detect::detect_vulkan(),
        hevc_vulkan: params.config.hardware_accel
            && params.config.video_codec == "hevc"
            && hw_detect::detect_vulkan(),
        av1_vulkan: params.config.hardware_accel
            && params.config.video_codec == "av1"
            && hw_detect::detect_vulkan(),
        av1_svt: false,
    };

    let hw_errors = Vec::new();

    let mut encoder_availability = EncoderAvailability {
        h264_nvenc: false,
        hevc_nvenc: false,
        av1_nvenc: false,
        h264_qsv: false,
        hevc_qsv: false,
        av1_qsv: false,
        h264_amf: false,
        hevc_amf: false,
        av1_amf: false,
        h264_cuvid: false,
        hevc_cuvid: false,
        av1_cuvid: false,
        h264_vulkan: false,
        hevc_vulkan: false,
        av1_vulkan: false,
        av1_svt: false,
    };

    fn set_avail(a: &mut EncoderAvailability, name: &str, ok: bool) {
        match name {
            "h264_nvenc" => a.h264_nvenc = ok,
            "hevc_nvenc" => a.hevc_nvenc = ok,
            "av1_nvenc" => a.av1_nvenc = ok,
            "h264_qsv" => a.h264_qsv = ok,
            "hevc_qsv" => a.hevc_qsv = ok,
            "av1_qsv" => a.av1_qsv = ok,
            "h264_amf" => a.h264_amf = ok,
            "hevc_amf" => a.hevc_amf = ok,
            "av1_amf" => a.av1_amf = ok,
            "h264_vulkan" => a.h264_vulkan = ok,
            "hevc_vulkan" => a.hevc_vulkan = ok,
            "av1_vulkan" => a.av1_vulkan = ok,
            "libsvtav1" => a.av1_svt = ok,
            _ => {}
        }
    }

    let codec_idx = match params.config.video_codec.as_str() {
        "hevc" => 1usize,
        "av1" => 2,
        _ => 0,
    };
    const V_NV: [&str; 3] = ["h264_nvenc", "hevc_nvenc", "av1_nvenc"];
    const V_QSV: [&str; 3] = ["h264_qsv", "hevc_qsv", "av1_qsv"];
    const V_AMF: [&str; 3] = ["h264_amf", "hevc_amf", "av1_amf"];
    const V_VK: [&str; 3] = ["h264_vulkan", "hevc_vulkan", "av1_vulkan"];
    let vendor_detected = |first: &str| -> bool {
        match first {
            "h264_nvenc" => hw_detected.h264_nvenc,
            "h264_qsv" => hw_detected.h264_qsv,
            "h264_amf" => hw_detected.h264_amf,
            _ => hw_detected.h264_vulkan,
        }
    };
    let vendors: Vec<[&str; 3]> = match params.config.encoder.as_str() {
        "cpu" => vec![],
        "amf" => vec![V_AMF, V_NV, V_QSV, V_VK],
        "qsv" => vec![V_QSV, V_NV, V_AMF, V_VK],
        "vulkan" => vec![V_VK, V_NV, V_QSV, V_AMF],
        _ => vec![V_NV, V_QSV, V_AMF, V_VK],
    };
    let mut hw_errors: Vec<String> = hw_errors;
    for v in vendors {
        // The built-in Vulkan encoder produces the stream itself; probing
        // ffmpeg's encoders would only mislabel the run and waste ~0.5s.
        if vulkan_video {
            break;
        }
        if !vendor_detected(v[0]) {
            continue;
        }
        let names: Vec<&str> = v.iter().copied().filter(|n| !n.trim().is_empty()).collect();
        let handles: Vec<_> = names
            .iter()
            .map(|name| {
                let name = name.to_string();
                let ff = ffmpeg.clone();
                std::thread::spawn(move || {
                    let (ok, err) = match test_encoder(ff.as_ref(), &name) {
                        Ok((ok, err)) => (ok, err),
                        Err(e) => (false, format!("test error: {e}")),
                    };
                    (name, ok, err)
                })
            })
            .collect();
        let mut ok_for_codec = false;
        for handle in handles {
            match handle.join() {
                Ok((name, ok, err)) => {
                    if ok && name == v[codec_idx] {
                        ok_for_codec = true;
                    }
                    set_avail(&mut encoder_availability, &name, ok);
                    if !ok {
                        hw_errors.push(format!("{} test failed:\n{}", name, err.trim()));
                    }
                }
                Err(_) => hw_errors.push("encoder probe thread panicked".to_owned()),
            }
        }
        if ok_for_codec { break; }
    }
    if params.config.video_codec == "av1" {
        match test_encoder(ffmpeg.as_ref(), "libsvtav1") {
            Ok((ok, err)) => {
                encoder_availability.av1_svt = ok;
                if !ok {
                    hw_errors.push(format!("libsvtav1 test failed:\n{}", err.trim()));
                }
            }
            Err(e) => hw_errors.push(format!("libsvtav1 test error: {e}")),
        }
    }

    // This will be a future feature (cuvid)
    let cuvid_to_test = [
        ("h264_cuvid", hw_detected.h264_cuvid, &mut encoder_availability.h264_cuvid),
        ("hevc_cuvid", hw_detected.hevc_cuvid, &mut encoder_availability.hevc_cuvid),
        ("av1_cuvid", hw_detected.av1_cuvid, &mut encoder_availability.av1_cuvid),
    ];

    for (name, detected, availability_flag) in cuvid_to_test {
        if detected {
            let (encoder_name, container_format) = if name == "h264_cuvid" {
                ("libx264", "mpegts")
            } else if name == "hevc_cuvid" {
                ("libx265", "mpegts")
            } else {
                ("libsvtav1", "matroska")
            };

            let mut encode_cmd = Command::new(&ffmpeg);
            encode_cmd.args(&[
                "-f", "lavfi",
                "-i", "testsrc=duration=1:size=320x240:rate=30",
                "-vf", "format=yuv420p",
                "-c:v", encoder_name,
                "-t", "0.5",
                "-f", container_format,
                "-"
            ])
                .stdout(Stdio::piped())
                .stderr(Stdio::null());

            let mut decode_cmd = Command::new(&ffmpeg);
            decode_cmd.args(&[
                "-hwaccel", "cuvid",
                "-hwaccel_device", "0",
                "-c:v", name,
                "-f", container_format,
                "-i", "-",
                "-f", "null",
                "-"
            ])
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped());

            let mut encoded = match encode_cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    *availability_flag = false;
                    hw_errors.push(format!("{} test encode setup failed: {}", name, e));
                    continue;
                }
            };

            decode_cmd.stdin(encoded.stdout.unwrap());

            match decode_cmd.output() {
                Ok(output) => {
                    *availability_flag = output.status.success();
                    if !output.status.success() {
                        let mut enc_err = String::new();
                        if let Some(mut e) = encoded.stderr.take() {
                            let _ = std::io::Read::read_to_string(&mut e, &mut enc_err);
                        }
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        hw_errors.push(format!(
                            "{} decode test failed (code {}):\n{}{}",
                            name,
                            output.status.code().unwrap_or(-1),
                            stderr.trim(),
                            if enc_err.trim().is_empty() {
                                String::new()
                            } else {
                                format!("\n[encoder side]\n{}", enc_err.trim())
                            }
                        ));
                    }
                }
                Err(e) => {
                    *availability_flag = false;
                    hw_errors.push(format!("{} decode test execution failed: {}", name, e));
                }
            }
        }
    }
    if !vulkan_video {
        if let Ok((ok, err)) = test_encoder(ffmpeg.as_ref(), "libsvtav1") {
            encoder_availability.av1_svt = ok;
            if !ok {
                hw_errors.push(format!("libsvtav1 test failed:
{}", err.trim()));
            }
        }
    }

    let mut dummy_flag = false;
    let encoder_type = params.config.encoder.as_str();
    let any_vendor_hw = match params.config.video_codec.as_str() {
        "hevc" => {
            encoder_availability.hevc_nvenc
                || encoder_availability.hevc_qsv
                || encoder_availability.hevc_amf
                || encoder_availability.hevc_vulkan
        }
        "av1" => {
            encoder_availability.av1_nvenc
                || encoder_availability.av1_qsv
                || encoder_availability.av1_amf
                || encoder_availability.av1_vulkan
        }
        _ => {
            encoder_availability.h264_nvenc
                || encoder_availability.h264_qsv
                || encoder_availability.h264_amf
                || encoder_availability.h264_vulkan
        }
    };

    let want_dx = params.config.hardware_accel
        && !vulkan_video
        && match encoder_type {
            "dx12" => true,
            "auto" => !any_vendor_hw,
            _ => false,
        };
    const DX12_H264: [DxEncoder; 1] = [DxEncoder { name: "h264_d3d12va", hw: "d3d12va", pix: "d3d12" }];
    const DX12_HEVC: [DxEncoder; 1] = [DxEncoder { name: "hevc_d3d12va", hw: "d3d12va", pix: "d3d12" }];
    const DX12_AV1: [DxEncoder; 1] = [DxEncoder { name: "av1_d3d12va", hw: "d3d12va", pix: "d3d12" }];
    let dx_candidates: Vec<DxEncoder> = match params.config.video_codec.as_str() {
        "hevc" => vec![DX12_HEVC[0]],
        "av1" => vec![DX12_AV1[0]],
        _ => vec![DX12_H264[0]],
    };
    let mut dx_selected: Option<DxEncoder> = None;
    let mut dx_codec_switched: Option<&str> = None;
    if want_dx {
        let mut tried = String::new();
        for enc in &dx_candidates {
            if probe_dx_encoder(ffmpeg.as_ref(), *enc) {
                info!(
                    "  DX upload path: probe OK -> {} ({} default device, -pix_fmt {}, -bf 0)",
                    enc.name, enc.hw, enc.pix
                );
                dx_selected = Some(*enc);
                break;
            }
            tried.push_str(&format!("{}/{} ", enc.name, enc.hw));
        }
        if dx_selected.is_none() {
            for (codec, enc) in [
                ("hevc", DX12_HEVC[0]),
                ("h264", DX12_H264[0]),
                ("av1", DX12_AV1[0]),
            ] {
                if codec == params.config.video_codec.as_str() {
                    continue;
                }
                if probe_dx_encoder(ffmpeg.as_ref(), enc) {
                    info!("  DX upload path: {} unavailable ({}), switching codec to {} via {}",
                        params.config.video_codec, tried.trim_end(), codec.to_uppercase(), enc.name);
                    dx_selected = Some(enc);
                    dx_codec_switched = Some(codec);
                    break;
                }
            }
        }
        if dx_selected.is_none() {
            warn!("  DX upload path: all probes failed ({}), falling back to other encoders", tried.trim_end());
        }
        if let Some(codec) = dx_codec_switched {
            warn!(
                "  Note: this ffmpeg build has no DX12 encoder for {}, the output will be {}",
                params.config.video_codec,
                codec.to_uppercase()
            );
        }
    }
    let candidates: Vec<(&str, bool, &mut bool)> = match params.config.video_codec.as_str() {
        "hevc" => {
            match encoder_type {
                "nvenc" => vec![
                    ("hevc_nvenc", encoder_availability.hevc_nvenc, &mut encoder_availability.hevc_nvenc),
                    ("hevc_vulkan", encoder_availability.hevc_vulkan, &mut encoder_availability.hevc_vulkan),
                    ("hevc_qsv", encoder_availability.hevc_qsv, &mut encoder_availability.hevc_qsv),
                    ("hevc_amf", encoder_availability.hevc_amf, &mut encoder_availability.hevc_amf),
                    ("libx265", true, &mut dummy_flag),
                ],
                "qsv" => vec![
                    ("hevc_qsv", encoder_availability.hevc_qsv, &mut encoder_availability.hevc_qsv),
                    ("hevc_vulkan", encoder_availability.hevc_vulkan, &mut encoder_availability.hevc_vulkan),
                    ("hevc_nvenc", encoder_availability.hevc_nvenc, &mut encoder_availability.hevc_nvenc),
                    ("hevc_amf", encoder_availability.hevc_amf, &mut encoder_availability.hevc_amf),
                    ("libx265", true, &mut dummy_flag),
                ],
                "amf" => vec![
                    ("hevc_amf", encoder_availability.hevc_amf, &mut encoder_availability.hevc_amf),
                    ("hevc_vulkan", encoder_availability.hevc_vulkan, &mut encoder_availability.hevc_vulkan),
                    ("hevc_nvenc", encoder_availability.hevc_nvenc, &mut encoder_availability.hevc_nvenc),
                    ("hevc_qsv", encoder_availability.hevc_qsv, &mut encoder_availability.hevc_qsv),
                    ("libx265", true, &mut dummy_flag),
                ],
                "vulkan" => vec![
                    ("hevc_vulkan", encoder_availability.hevc_vulkan, &mut encoder_availability.hevc_vulkan),
                    ("hevc_nvenc", encoder_availability.hevc_nvenc, &mut encoder_availability.hevc_nvenc),
                    ("hevc_qsv", encoder_availability.hevc_qsv, &mut encoder_availability.hevc_qsv),
                    ("hevc_amf", encoder_availability.hevc_amf, &mut encoder_availability.hevc_amf),
                    ("libx265", true, &mut dummy_flag),
                ],
                "cpu" => vec![
                    ("libx265", true, &mut dummy_flag),
                ],
                _ => vec![
                    ("hevc_nvenc", encoder_availability.hevc_nvenc, &mut encoder_availability.hevc_nvenc),
                    ("hevc_qsv", encoder_availability.hevc_qsv, &mut encoder_availability.hevc_qsv),
                    ("hevc_amf", encoder_availability.hevc_amf, &mut encoder_availability.hevc_amf),
                    ("hevc_vulkan", encoder_availability.hevc_vulkan, &mut encoder_availability.hevc_vulkan),
                    ("libx265", true, &mut dummy_flag),
                ],
            }
        },
        "av1" => {
            match encoder_type {
                "nvenc" => vec![
                    ("av1_nvenc", encoder_availability.av1_nvenc, &mut encoder_availability.av1_nvenc),
                    ("av1_qsv", encoder_availability.av1_qsv, &mut encoder_availability.av1_qsv),
                    ("av1_amf", encoder_availability.av1_amf, &mut encoder_availability.av1_amf),
                    ("av1_vulkan", encoder_availability.av1_vulkan, &mut encoder_availability.av1_vulkan),
                    ("libsvtav1", encoder_availability.av1_svt, &mut encoder_availability.av1_svt),
                    ("libaom-av1", true, &mut dummy_flag),
                ],
                "qsv" => vec![
                    ("av1_qsv", encoder_availability.av1_qsv, &mut encoder_availability.av1_qsv),
                    ("av1_nvenc", encoder_availability.av1_nvenc, &mut encoder_availability.av1_nvenc),
                    ("av1_amf", encoder_availability.av1_amf, &mut encoder_availability.av1_amf),
                    ("av1_vulkan", encoder_availability.av1_vulkan, &mut encoder_availability.av1_vulkan),
                    ("libsvtav1", encoder_availability.av1_svt, &mut encoder_availability.av1_svt),
                    ("libaom-av1", true, &mut dummy_flag),
                ],
                "amf" => vec![
                    ("av1_amf", encoder_availability.av1_amf, &mut encoder_availability.av1_amf),
                    ("av1_nvenc", encoder_availability.av1_nvenc, &mut encoder_availability.av1_nvenc),
                    ("av1_qsv", encoder_availability.av1_qsv, &mut encoder_availability.av1_qsv),
                    ("av1_vulkan", encoder_availability.av1_vulkan, &mut encoder_availability.av1_vulkan),
                    ("libsvtav1", encoder_availability.av1_svt, &mut encoder_availability.av1_svt),
                    ("libaom-av1", true, &mut dummy_flag),
                ],
                "vulkan" => vec![
                    ("av1_vulkan", encoder_availability.av1_vulkan, &mut encoder_availability.av1_vulkan),
                    ("av1_nvenc", encoder_availability.av1_nvenc, &mut encoder_availability.av1_nvenc),
                    ("av1_qsv", encoder_availability.av1_qsv, &mut encoder_availability.av1_qsv),
                    ("av1_amf", encoder_availability.av1_amf, &mut encoder_availability.av1_amf),
                    ("libsvtav1", encoder_availability.av1_svt, &mut encoder_availability.av1_svt),
                    ("libaom-av1", true, &mut dummy_flag),
                ],
                "cpu" => vec![
                    ("libsvtav1", encoder_availability.av1_svt, &mut encoder_availability.av1_svt),
                    ("libaom-av1", true, &mut dummy_flag),
                ],
                _ => vec![
                    ("av1_nvenc", encoder_availability.av1_nvenc, &mut encoder_availability.av1_nvenc),
                    ("av1_qsv", encoder_availability.av1_qsv, &mut encoder_availability.av1_qsv),
                    ("av1_amf", encoder_availability.av1_amf, &mut encoder_availability.av1_amf),
                    ("av1_vulkan", encoder_availability.av1_vulkan, &mut encoder_availability.av1_vulkan),
                    ("libsvtav1", encoder_availability.av1_svt, &mut encoder_availability.av1_svt),
                    ("libaom-av1", true, &mut dummy_flag),
                ],
            }
        },
        _ => {
            match encoder_type {
                "nvenc" => vec![
                    ("h264_nvenc", encoder_availability.h264_nvenc, &mut encoder_availability.h264_nvenc),
                    ("h264_vulkan", encoder_availability.h264_vulkan, &mut encoder_availability.h264_vulkan),
                    ("h264_qsv", encoder_availability.h264_qsv, &mut encoder_availability.h264_qsv),
                    ("h264_amf", encoder_availability.h264_amf, &mut encoder_availability.h264_amf),
                    ("libx264", true, &mut dummy_flag),
                ],
                "qsv" => vec![
                    ("h264_qsv", encoder_availability.h264_qsv, &mut encoder_availability.h264_qsv),
                    ("h264_vulkan", encoder_availability.h264_vulkan, &mut encoder_availability.h264_vulkan),
                    ("h264_nvenc", encoder_availability.h264_nvenc, &mut encoder_availability.h264_nvenc),
                    ("h264_amf", encoder_availability.h264_amf, &mut encoder_availability.h264_amf),
                    ("libx264", true, &mut dummy_flag),
                ],
                "amf" => vec![
                    ("h264_amf", encoder_availability.h264_amf, &mut encoder_availability.h264_amf),
                    ("h264_vulkan", encoder_availability.h264_vulkan, &mut encoder_availability.h264_vulkan),
                    ("h264_nvenc", encoder_availability.h264_nvenc, &mut encoder_availability.h264_nvenc),
                    ("h264_qsv", encoder_availability.h264_qsv, &mut encoder_availability.h264_qsv),
                    ("libx264", true, &mut dummy_flag),
                ],
                "vulkan" => vec![
                    ("h264_vulkan", encoder_availability.h264_vulkan, &mut encoder_availability.h264_vulkan),
                    ("h264_nvenc", encoder_availability.h264_nvenc, &mut encoder_availability.h264_nvenc),
                    ("h264_qsv", encoder_availability.h264_qsv, &mut encoder_availability.h264_qsv),
                    ("h264_amf", encoder_availability.h264_amf, &mut encoder_availability.h264_amf),
                    ("libx264", true, &mut dummy_flag),
                ],
                "cpu" => vec![
                    ("libx264", true, &mut dummy_flag),
                ],
                _ => vec![
                    ("h264_nvenc", encoder_availability.h264_nvenc, &mut encoder_availability.h264_nvenc),
                    ("h264_qsv", encoder_availability.h264_qsv, &mut encoder_availability.h264_qsv),
                    ("h264_amf", encoder_availability.h264_amf, &mut encoder_availability.h264_amf),
                    ("h264_vulkan", encoder_availability.h264_vulkan, &mut encoder_availability.h264_vulkan),
                    ("libx264", true, &mut dummy_flag),
                ],
            }
        },
    };


    let mut ffmpeg_encoder = candidates
        .iter()
        .find(|&&(_name, available, _)| available)
        .map(|&(name, _, _)| name)
        .expect("At least one software encoder is available.");

    let dx_selected = match encoder_type {
        "dx12" => dx_selected,
        _ => dx_selected.filter(|_| matches!(ffmpeg_encoder, "libx264" | "libx265" | "libaom-av1")),
    };
    if let Some(enc) = &dx_selected {
        ffmpeg_encoder = enc.name;
    }

    if vulkan_video {
        info!("  (encoder probing skipped: the built-in Vulkan Video encoder produces the stream)");
    }
    info!(
    "=== Encoder Selection ===\n\
     Video codec: {}\n\
     User preference: {}\n\
     --- Encoder Availability ---\n\
       h264_nvenc: {}\n\
       h264_qsv: {}\n\
       h264_amf: {}\n\
       h264_vulkan: {}\n\
       hevc_nvenc: {}\n\
       hevc_qsv: {}\n\
       hevc_amf: {}\n\
       hevc_vulkan: {}\n\
       av1_nvenc: {}\n\
       av1_qsv: {}\n\
       av1_amf: {}\n\
       av1_vulkan: {}\n\
     {}",
    params.config.video_codec,
    params.config.encoder,
    encoder_availability.h264_nvenc,
    encoder_availability.h264_qsv,
    encoder_availability.h264_amf,
    encoder_availability.h264_vulkan,
    encoder_availability.hevc_nvenc,
    encoder_availability.hevc_qsv,
    encoder_availability.hevc_amf,
    encoder_availability.hevc_vulkan,
    encoder_availability.av1_nvenc,
    encoder_availability.av1_qsv,
    encoder_availability.av1_amf,
    encoder_availability.av1_vulkan,
    if !params.config.hardware_accel {
        "  (hardware acceleration is off: hardware encoders were not probed)"
    } else if params.config.encoder == "cpu" {
        "  (cpu preference: hardware encoders were not probed)"
    } else {
        ""
    }
    );
    if !hw_errors.is_empty() {
        info!("  --- Encoder Errors ---");
        for error in &hw_errors {
            info!("    {}", error);
        }
    }
    info!("  Selected encoder: {}", if vulkan_video {
        "Vulkan Video H.264 (built-in; ffmpeg only remuxes with -c:v copy)"
    } else {
        ffmpeg_encoder
    });
    let probe_s = mark!("encoder probe");
    if ffmpeg_encoder == "libsvtav1"
        && encoder_type == "auto"
        && (encoder_availability.av1_nvenc
            || encoder_availability.av1_qsv
            || encoder_availability.av1_amf
            || encoder_availability.av1_vulkan)
    {
        info!(
            "  Note: AV1 uses the CPU encoder (SVT-AV1) because hardware AV1 quality is clearly weaker."
        );
        info!("  Note: it is much slower 閳?select the nvenc/amf encoder explicitly if you want speed.");
    }
    info!("=========================");

    let ffmpeg_preset = match ffmpeg_encoder {
        "h264_amf" | "hevc_amf" | "av1_amf" => "-quality",
        "h264_vulkan" | "hevc_vulkan" | "av1_vulkan" => "-preset",
        "libaom-av1" => "-cpu-used",
        "librav1e" => "-speed",
        _ => "-preset",
    };

    fn speed_rank(word: &str) -> u32 {
        match word {
            "ultrafast" | "veryfast" => 0,
            "faster" => 1,
            "fast" => 2,
            "medium" => 3,
            "slow" => 4,
            "slower" => 5,
            "veryslow" => 6,
            _ => 3,
        }
    }

    let preset_words: Vec<&str> = params.config.ffmpeg_preset.split_whitespace().collect();
    let ffmpeg_preset_name: String = match ffmpeg_encoder {
        "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" => {
            match preset_words.get(1) {
                Some(w) if w.len() >= 2 && w.starts_with('p') && w[1..].chars().all(|c| c.is_ascii_digit()) => {
                    (*w).to_string()
                }
                Some(w) => format!("p{}", speed_rank(w) + 1),
                None => format!("p{}", speed_rank(preset_words.first().copied().unwrap_or("medium")) + 1),
            }
        }
        "h264_qsv" | "hevc_qsv" | "av1_qsv" => preset_words
            .first()
            .copied()
            .unwrap_or("medium")
            .to_string(),
        "h264_amf" | "hevc_amf" | "av1_amf" => match preset_words.get(2) {
            Some(w) => (*w).to_string(),
            None => match speed_rank(preset_words.first().copied().unwrap_or("medium")) {
                0..=2 => "speed",
                3 | 4 => "balanced",
                _ => "quality",
            }
            .to_string(),
        },
        "h264_vulkan" | "hevc_vulkan" | "av1_vulkan" => preset_words
            .first()
            .copied()
            .unwrap_or("default")
            .to_string(),
        "libsvtav1" => match speed_rank(preset_words.first().copied().unwrap_or("medium")) {
            0 | 1 => "12",
            2 => "10",
            3 => "8",
            4 => "6",
            5 => "4",
            _ => "2",
        }
        .to_string(),
        "libaom-av1" => match speed_rank(preset_words.first().copied().unwrap_or("medium")) {
            0 | 1 => "8",
            2 => "7",
            3 => "6",
            4 => "5",
            5 => "4",
            _ => "3",
        }
        .to_string(),
        "librav1e" => match speed_rank(preset_words.first().copied().unwrap_or("medium")) {
            0 | 1 => "10",
            2 => "9",
            3 => "8",
            4 => "7",
            5 => "6",
            _ => "5",
        }
        .to_string(),
        _ => preset_words
            .first()
            .copied()
            .unwrap_or("medium")
            .to_string(),
    };

    if vulkan_video {
        info!("  Speed preset: (built-in Vulkan Video encoder; the ffmpeg preset is not used)");
    } else if ffmpeg_encoder.ends_with("_vulkan") {
        info!("  Speed preset: (vulkan encoder takes no preset, ignoring the configured one)");
    } else {
        info!("  Speed preset: {} {}", ffmpeg_preset, ffmpeg_preset_name);
    }

    let bitrate_control = if params.config.bitrate_control == "CRF" {
        match ffmpeg_encoder {
            "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" => "-cq",
            "h264_qsv" | "hevc_qsv" | "av1_qsv" => "-q",
            "h264_amf" | "hevc_amf" | "av1_amf" => "-qp_p",
            "h264_vulkan" | "hevc_vulkan" | "av1_vulkan" => "-qp",
            _ => "-crf",
        }
    } else {
        "-b:v"
    };

    if params.config.hardware_accel && encoder_type != "cpu" {
        let dx_ok = dx_selected.is_some();
        let any4 = |a: bool, b: bool, c: bool, d: bool| a || b || c || d || dx_ok;

        let h264_supported = any4(
            encoder_availability.h264_nvenc,
            encoder_availability.h264_qsv,
            encoder_availability.h264_amf,
            encoder_availability.h264_vulkan,
        );
        let hevc_supported = any4(
            encoder_availability.hevc_nvenc,
            encoder_availability.hevc_qsv,
            encoder_availability.hevc_amf,
            encoder_availability.hevc_vulkan,
        );
        let av1_supported = any4(
            encoder_availability.av1_nvenc,
            encoder_availability.av1_qsv,
            encoder_availability.av1_amf,
            encoder_availability.av1_vulkan,
        );

        // The built-in Vulkan Video encoder is the hardware encoder here; the
        // ffmpeg probes above were skipped on purpose, so do not treat their
        // empty result as "no hardware acceleration".
        let codec_unsupported = !vulkan_video
            && match params.config.video_codec.as_str() {
                "h264" => !h264_supported,
                "hevc" => !hevc_supported,
                "av1" => !av1_supported,
                _ => false,
            };

        if codec_unsupported {
            let yesno = |b: bool| if b { "SUCCESS" } else { "FAILED" };
            let mut msg = format!("{}\n", tl!("no-hwacc"));

            let _ = write!(
                msg,
                "Hardware detection summary:\n\
             - NVIDIA: {}\n\
             - Intel Quick Sync: {}\n\
             - AMD AMF: {}\n\
             - Vulkan: {}\n\n",
                hw_detected.h264_nvenc,
                hw_detected.h264_qsv,
                hw_detected.h264_amf,
                hw_detected.h264_vulkan,
            );

            msg += "Encoder test results:\n";
            for (name, ok) in [
                ("h264_nvenc",  encoder_availability.h264_nvenc),
                ("hevc_nvenc",  encoder_availability.hevc_nvenc),
                ("av1_nvenc",   encoder_availability.av1_nvenc),
                ("h264_qsv",    encoder_availability.h264_qsv),
                ("hevc_qsv",    encoder_availability.hevc_qsv),
                ("av1_qsv",     encoder_availability.av1_qsv),
                ("h264_amf",    encoder_availability.h264_amf),
                ("hevc_amf",    encoder_availability.hevc_amf),
                ("av1_amf",     encoder_availability.av1_amf),
                ("h264_vulkan", encoder_availability.h264_vulkan),
                ("hevc_vulkan", encoder_availability.hevc_vulkan),
                ("av1_vulkan",  encoder_availability.av1_vulkan),
                ("h264_cuvid",  encoder_availability.h264_cuvid),
                ("hevc_cuvid",  encoder_availability.hevc_cuvid),
                ("av1_cuvid",   encoder_availability.av1_cuvid),
            ] {
                let _ = writeln!(msg, "- {}: {}", name, yesno(ok));
            }
            msg.push('\n');

            if hw_errors.is_empty() {
                msg += "No hardware encoders were tested (all detection failed).\n\n";
            } else {
                msg += "Detailed error logs:\n";
                for (i, error) in hw_errors.iter().enumerate() {
                    let _ = writeln!(msg, "{}. {}", i + 1, error);
                }
                msg.push('\n');
            }

            bail!(msg);
        }
    }


    if vulkan_video {
        info!("Readback: none (Vulkan encoder reads GPU NV12 directly)");
    } else if nv12_mode {
        info!(
            "Readback: Vulkan NV12 {:.2} MB/frame, async ring (no per-frame fence wait)",
            (vw as f64 * vh as f64 * 1.5) / 1048576.
        );
    } else {
        info!(
            "Readback: RGBA {:.2} MB/frame, one transfer per frame",
            (vw as f64 * vh as f64 * 4.) / 1048576.
        );
    }

    let video_url = "-".to_owned();
    let global_args = "-y";
    let mut input_args = String::new();
    if vulkan_video {
        write!(&mut input_args, "-f h264 -r {fps} -i {video_url} -i")?;
    } else if nv12_mode {
        write!(
            &mut input_args,
            "-f rawvideo -c:v rawvideo -s {vw}x{vh} -r {fps} -pix_fmt nv12"
        )?;
        if params.config.ffmpeg_thread {
            input_args.push_str(" -thread_queue_size 2048");
        }
        write!(&mut input_args, " -i {video_url} -i")?;
    } else {
        write!(
            &mut input_args,
            "-f rawvideo -c:v rawvideo -s {vw}x{vh} -r {fps} -pix_fmt rgba"
        )?;
        if params.config.ffmpeg_thread {
            input_args.push_str(" -thread_queue_size 2048");
        }
        write!(&mut input_args, " -i {video_url} -i")?;
    }

    let video = if audio_format == "flac" && params.config.video {
        warn!("FLAC audio is not supported in MOV; muxing into MP4 instead");
        "mp4"
    } else if params.config.video {
        "mov"
    } else {
        "mp4"
    };

    let null_sink = std::env::var_os("PHITK_NULL_SINK").is_some();
    let video_mux = if null_sink { "null" } else { video };

    let is_vulkan_encoder = ffmpeg_encoder.ends_with("_vulkan");
    let is_hw_encoder = matches!(
        ffmpeg_encoder,
        "h264_nvenc"
            | "hevc_nvenc"
            | "av1_nvenc"
            | "h264_qsv"
            | "hevc_qsv"
            | "av1_qsv"
            | "h264_amf"
            | "hevc_amf"
            | "av1_amf"
    );
    let dx = dx_selected;
    let vf_arg = if nv12_mode {
        // The pipe already carries NV12.  Only the GPU-upload encoders need a
        // filter; software encoders negotiate NV12 directly.
        if dx.is_some() || is_vulkan_encoder {
            "-vf hwupload".to_owned()
        } else {
            String::new()
        }
    } else if dx.is_some() || is_vulkan_encoder {
        "-vf format=nv12,hwupload".to_owned()
    } else if is_hw_encoder {
        "-vf format=nv12".to_owned()
    } else {
        "-vf format=yuv420p".to_owned()
    };

    let mut out_extra = vf_arg;
    if ffmpeg_encoder == "libaom-av1" {
        out_extra.push_str(" -row-mt 1");
    }
    if let Some(enc) = &dx {
        out_extra.push_str(&format!(" -pix_fmt {}", enc.pix));
        out_extra.push_str(" -bf 0");

    }

    let is_crf = params.config.bitrate_control == "CRF";
    let av1_quality_extra: &str = match ffmpeg_encoder {
        "libsvtav1" => " -svtav1-params tune=0:enable-tf=1:aq-mode=2",

        // Not using it for now because it's too slow
        "libaom-av1" => {
            if is_crf { " -b:v 0 -aq-mode 1 -enable-restoration 1 -lag-in-frames 35" }
            else {
                " -aq-mode 1 -enable-restoration 1 -lag-in-frames 35"
            }
        }
        "av1_amf" => " -bf 2",
        _ => "",
    };
    out_extra.push_str(av1_quality_extra);
    if !av1_quality_extra.is_empty() {
        info!("  AV1 quality:{}", av1_quality_extra);
        if speed_rank(preset_words.first().copied().unwrap_or("medium")) <= 3 { info!("  Note: for the best AV1 quality pick the VerySlow speed preset (SVT-AV1 preset 2)"); }
    out_extra.push_str(if vh > 576 {
        " -colorspace bt709 -color_primaries bt709 -color_trc bt709 -color_range tv" } else
    {
        " -colorspace smpte170m -color_primaries smpte170m -color_trc smpte170m -color_range tv"
    });
    if dx.is_none()
        && matches!(
            ffmpeg_encoder,
            "libsvtav1" | "libaom-av1" | "librav1e" | "av1_nvenc"
        )
    {
        out_extra.push_str(" -pix_fmt yuv420p10le");
    }
    }

    let async_depth = "-async_depth 4"; // the "4" parallel process 4 frames (vulkan encoder)
    let args2 = if null_sink {
        "-c:v rawvideo -map 0:v:0 -f null".to_string()
    } else if vulkan_video {
        let duration = out_frames as f64 / fps.max(1) as f64;
        format!(
            "{} -c:v copy -map 0:v:0 -map 1:a:0 -t {duration:.6} -f {}",
            audio_mux_args, video_mux,
        )
    } else if is_vulkan_encoder {
        format!(
            "{} -c:v {} {} {} {} -map 0:v:0 -map 1:a:0 -shortest {} {} {} -f {}",
            audio_mux_args,
            ffmpeg_encoder,
            bitrate_control,
            params.config.bitrate,
            ffmpeg_preset,
            ffmpeg_preset_name,
            out_extra,
            async_depth,
            video_mux,
        )
    } else {
        format!(
            "{} -c:v {} {} {} {} {} -map 0:v:0 -map 1:a:0 -shortest {} -f {}",
            audio_mux_args,
            ffmpeg_encoder,
            bitrate_control,
            params.config.bitrate,
            ffmpeg_preset,
            ffmpeg_preset_name,
            out_extra,
            video_mux,
        )
    };

    let ffmpeg_frames = std::sync::Arc::new(AtomicU64::new(0));
    info!("ffmpeg video args: {}", args2);
    let mut proc = {
        let mut cmd = cmd_hidden(&ffmpeg);
        cmd.args(global_args.split_whitespace());
        cmd.args(["-progress", "pipe:2", "-nostats"]);
        if is_vulkan_encoder && !vulkan_video {
            cmd.arg("-init_hw_device").arg("vulkan=vk")
               .arg("-filter_hw_device").arg("vk");
        } else if let Some(enc) = dx.as_ref().filter(|_| !vulkan_video) {
            cmd.arg("-init_hw_device")
                .arg(format!("{}={}", enc.hw, DX_DEVICE))
                .arg("-filter_hw_device")
                .arg(DX_DEVICE);
        }
        cmd.args(input_args.split_whitespace())
            .arg(audio_output_file.path())
            .args(args2.split_whitespace())
            .arg(output_path)
            .arg("-loglevel")
            .arg("error")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| tl!("run-ffmpeg-failed"))?
    };


    let video_stderr = drain_stderr_with_progress(proc.stderr.take().unwrap(), ffmpeg_frames.clone());

    let frame_size = vw as usize * vh as usize * 4;

    let mut ffmpeg_stdin = Some(proc.stdin.take().unwrap());
    let keyframe_interval = (fps.saturating_mul(KEYFRAME_INTERVAL_SECS)).max(1) as u64;

    let (mut vulkan_pipe, pipeline, writer, encoder_thread, mut nv12_pipe, nv12_pipeline) =
        if let Some(context) = video_context.take() {
            let nv12_raw = (0..context.source_count())
                .map(|slot| context.nv12_raw(slot))
                .collect::<Vec<_>>();

            let slot_count = nv12_raw.len();
            let (job_tx, job_rx) = mpsc::channel::<(u64, usize)>();
            let (result_tx, result_rx) = mpsc::channel::<(u64, Option<Vec<u8>>)>();
            let (free_tx, free_rx) = mpsc::channel::<usize>();
            let context = SendVideoContext(context);

            let encoder_thread = std::thread::Builder::new()
                .name("vulkan-encode".to_owned())
                .spawn(move || {
                    let mut context = context.0;
                    for (index, slot) in job_rx {
                        while context.encoded_in_flight() >= 2 {
                            if let Some((frame_index, data, source_slot)) =
                                context.collect_encoded()
                            {
                                result_tx.send((frame_index, Some(data))).ok();
                                free_tx.send(source_slot).ok();
                            } else {
                                break;
                            }
                        }
                        let force_keyframe = index % keyframe_interval == 0;
                        if context
                            .submit_encode_slot(slot, index, force_keyframe)
                            .is_none()
                        {
                            result_tx.send((index, None)).ok();
                            free_tx.send(slot).ok();
                        }
                    }
                    while let Some((frame_index, data, source_slot)) = context.collect_encoded()
                    {
                        result_tx.send((frame_index, Some(data))).ok();
                        free_tx.send(source_slot).ok();
                    }
                })
                .context("failed to spawn Vulkan encoder thread")?;
            let mut stdin = ffmpeg_stdin.take().unwrap();

            let writer = std::thread::Builder::new()
                .name("ffmpeg-writer".to_owned())
                .spawn(move || -> std::io::Result<u64> {
                    let mut busy: u64 = 0;
                    let mut next = 0u64;
                    let mut pending = BTreeMap::new();
                    while let Ok((index, data)) = result_rx.recv() {
                        let Some(data) = data else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Vulkan encoder produced no bitstream",
                            ));
                        };
                        pending.insert(index, data);
                        while let Some(data) = pending.remove(&next) {
                            let t = Instant::now();
                            stdin.write_all(&data)?;
                            busy += t.elapsed().as_nanos() as u64;
                            next += 1;
                        }
                    }
                    let t = Instant::now();
                    stdin.flush()?;
                    busy += t.elapsed().as_nanos() as u64;
                    Ok(busy)
                })
                .context("failed to spawn ffmpeg writer thread")?;
            (
                Some(VulkanPipe {
                    job_tx,
                    nv12_raw,
                    free: (0..slot_count).collect(),
                    free_rx,
                }),
                None,
                writer,
                Some(encoder_thread),
                None,
                None,
            )
        } else if let Some(readback) = ffmpeg_nv12.take() {
            let ptrs = readback.writer_ptrs();
            let frame_bytes = readback.frame_size();
            let slot_count = readback.slot_count();
            let mut nv12_pipeline = FramePipeline::new(frame_bytes);

            nv12_pipeline.finish_init();

            let nv12_pipeline = std::sync::Arc::new(nv12_pipeline);
            let mut stdin = ffmpeg_stdin.take().unwrap();
            let shared = std::sync::Arc::clone(&nv12_pipeline);

            let writer = std::thread::Builder::new()
                .name("ffmpeg-writer".to_owned())
                .spawn(move || -> std::io::Result<u64> {
                    let mut busy: u64 = 0;
                    while let Some(buf) = shared.pop_job() {

                        let t = Instant::now();
                        stdin.write_all(&shared.pool[buf])?;
                        let d = t.elapsed().as_nanos() as u64;

                        busy += d;

                        shared.write_ns.fetch_add(d, Ordering::Relaxed);
                        shared.write_count.fetch_add(1, Ordering::Relaxed);
                        shared.write_max_ns.fetch_max(d, Ordering::Relaxed);
                        shared.release_buf(buf);

                    }

                    let t = Instant::now();

                    stdin.flush()?;
                    busy += t.elapsed().as_nanos() as u64;
                    Ok(busy)

                })
                .context("failed to spawn ffmpeg writer thread")?;
            (
                None,
                None,
                writer,
                None,
                Some(Nv12Pipe {
                    readback,
                    free: (0..slot_count).collect(),
                    ptrs,
                    frame_bytes,
                    pending: std::array::from_fn(|_| None),
                    current_read: None,
                }),
                Some(nv12_pipeline),
            )
        } else {
            let mut pipeline = FramePipeline::new(frame_size);

            pipeline.finish_init();

            let pipeline = std::sync::Arc::new(pipeline);
            let mut stdin = ffmpeg_stdin.take().unwrap();
            let shared = std::sync::Arc::clone(&pipeline);

            let writer = std::thread::Builder::new()
                .name("ffmpeg-writer".to_owned())
                .spawn(move || -> std::io::Result<u64> {
                    let mut busy: u64 = 0;
                    while let Some(buf) = shared.pop_job() {
                        let t = Instant::now();
                        stdin.write_all(&shared.pool[buf])?;
                        let d = t.elapsed().as_nanos() as u64;
                        busy += d;
                        shared.write_ns.fetch_add(d, Ordering::Relaxed);
                        shared.write_count.fetch_add(1, Ordering::Relaxed);
                        shared.write_max_ns.fetch_max(d, Ordering::Relaxed);
                        shared.release_buf(buf);
                    }
                    let t = Instant::now();
                    stdin.flush()?;
                    busy += t.elapsed().as_nanos() as u64;
                    Ok(busy)
                })
                .context("failed to spawn ffmpeg writer thread")?;
            (None, Some(pipeline), writer, None, None, None)
        };

    fn submit_frame(pipeline: &FramePipeline, target: &RenderTarget, frame_size: usize, ) -> Result<(f64, f64)> {
        let acquire_start = Instant::now();
        let buf = pipeline.acquire_buf();
        let pool_wait = acquire_start.elapsed().as_secs_f64();
        let base = pipeline.pool_addrs[buf] as *mut u8;

        let quad_context = miniquad_context();
        let dma_start = Instant::now();
        let out = unsafe { std::slice::from_raw_parts_mut(base, frame_size) };
        target
            .texture
            .raw_miniquad_texture_handle()
            .read_pixels(quad_context, out);
        let dma_wait = dma_start.elapsed().as_secs_f64();

        pipeline.push_job(buf);
        Ok((dma_wait, pool_wait))
    }

    let total_frames = frames;

    let frames10 = (out_frames / 10).max(1);
    let mut next_mark = frames10;
    let mut step_time = Instant::now();
    let show = |done: u64, label: &str, step_time: &mut Instant| {
        let p = (done as f64 / out_frames.max(1) as f64).min(1.0);
        let pct = (p * 100.0).ceil() as i8;
        let fill = (p * 20.).round() as usize;
        info!(
            "Rendering: [{}{}] {:>3}% | Time: {} | Frames: {}/{}",
            "█".repeat(fill),
            " ".repeat(20 - fill),
            pct,
            label,
            done,
            out_frames
        );
        *step_time = Instant::now();
    };

    // log
    let mut dma_wait_time: f64 = 0.0;
    let mut output_index:u64 = 0;
    let mut pool_wait_time: f64 = 0.0;
    let mut slot_wait_time: f64 = 0.0;
    let mut fence_wait_time: f64 = 0.0;
    let mut update_time: f64 = 0.0;
    let mut scene_render_time: f64 = 0.0;
    let mut yuv_convert_time: f64 = 0.0;
    let mut nv12_submit_time: f64 = 0.0;
    let mut copy_time: f64 = 0.0;
    let gl = unsafe { get_internal_gl() };
    let spawn_s = mark!("ffmpeg spawn + threads");
    let loop_start_time = Instant::now();

    for frame in 0..total_frames {
        let current_frame_time = frame as f64 * frame_duration;
        *my_time.borrow_mut() = current_frame_time;
        let keep = frame >= skip;
        let mut nv12_frame_slot = None;

        if keep {
            let mst_index = if let Some(pipe) = &mut nv12_pipe {
                let context = miniquad_context().vulkan_context().unwrap();
                let wait_start = Instant::now();
                let frame_slot = context.begin_offscreen_frame();
                fence_wait_time += wait_start.elapsed().as_secs_f64();

                if let Some((_index, read_slot)) = pipe.pending[frame_slot].take() {
                    let pipeline = nv12_pipeline.as_ref().unwrap();
                    let acquire_start = Instant::now();
                    let buf = pipeline.acquire_buf();
                    pool_wait_time += acquire_start.elapsed().as_secs_f64();
                    let src = unsafe {
                        std::slice::from_raw_parts(
                            pipe.ptrs[read_slot] as *const u8,
                            pipe.frame_bytes,
                        )
                    };
                    let dst = unsafe {
                        std::slice::from_raw_parts_mut(
                            pipeline.pool_addrs[buf] as *mut u8,
                            pipe.frame_bytes,
                        )
                    };
                    let copy_start = Instant::now();
                    dst.copy_from_slice(src);
                    copy_time += copy_start.elapsed().as_secs_f64();
                    pipe.free.push_back(read_slot);
                    pipeline.push_job(buf);
                }
                let read_slot = pipe.free.pop_front().unwrap();
                pipe.current_read = Some(read_slot);
                nv12_frame_slot = Some(frame_slot);
                frame_slot
            } else {
                0
            };
            mst_slot.set(mst_index);
            let input = msts[mst_index].input();
            gl.quad_gl.render_pass(Some(input.render_pass));
        }
        // Frame 0 was already updated by the pre-roll above.
        if frame != 0 {
            let update_start = Instant::now();
            main.update()?;
            update_time += update_start.elapsed().as_secs_f64();
        }
        if !keep {
            continue;
        }
        let render_start = Instant::now();
        main.render(&mut painter)?;
        scene_render_time += render_start.elapsed().as_secs_f64();
        if current_frame_time <= LoadingScene::TOTAL_TIME as f64 && !params.config.disable_loading {
            draw_rectangle(0., 0., 0., 0., Color::default());
        }

        let done = ffmpeg_frames.load(Ordering::Relaxed).min(out_frames);
        if done >= next_mark {
            let label = format!("{:.2}s", step_time.elapsed().as_secs_f32());
            show(done, &label, &mut step_time);
            next_mark = done + frames10;
        }

        if let Some(pipe) = &mut nv12_pipe {
            let frame_slot = nv12_frame_slot.unwrap();
            let read_slot = pipe.current_read.take().unwrap();
            {
                let context = miniquad_context().vulkan_context().unwrap();
                let yuv = &yuvs[frame_slot];
                let convert_start = Instant::now();
                yuv.convert(&msts[frame_slot].output());
                yuv_convert_time += convert_start.elapsed().as_secs_f64();
                let submit_start = Instant::now();
                context.record_yuv_copy_to_staging(
                    yuv.y_index,
                    yuv.uv_index,
                    pipe.readback.buffer_raw(read_slot),
                    vw,
                    vh,
                );
                context.finish_offscreen_frame();
                nv12_submit_time += submit_start.elapsed().as_secs_f64();
            }
            pipe.pending[frame_slot] = Some((output_index, read_slot));
            output_index += 1;
        } else if let Some(pipe) = &mut vulkan_pipe {
            let wait_start = Instant::now();
            let slot = pipe
                .free
                .pop_front()
                .or_else(|| pipe.free_rx.recv().ok())
                .unwrap();
            slot_wait_time += wait_start.elapsed().as_secs_f64();
            if let Some(yuv) = yuvs.first() {
                yuv.convert(&msts[0].output());
                {
                    let context = miniquad_context().vulkan_context().unwrap();
                    context.copy_nv12_from_textures(
                        yuv.y_index,
                        yuv.uv_index,
                        pipe.nv12_raw[slot],
                    );
                }
                pipe.job_tx.send((output_index, slot)).ok();
                output_index += 1;
            }
        } else {
            let (dma, pool) =
                submit_frame(pipeline.as_ref().unwrap(), &msts[0].output(), frame_size)?;
            dma_wait_time += dma;
            pool_wait_time += pool;
        }
        send(IPCEvent::Frame);
    }

    let loop_wall = loop_start_time.elapsed().as_secs_f64().max(1e-9);
    mark!("render loop");
    let backlog_at_end = output_index.saturating_sub(ffmpeg_frames.load(Ordering::Relaxed) as u64);
    drop(vulkan_pipe);
    if let Some(thread) = encoder_thread {
        thread.join().ok();
    }
    if let Some(pipeline) = &pipeline {
        pipeline.push_stop();
    }
    if let Some(pipe) = &mut nv12_pipe {
        let context = miniquad_context().vulkan_context().unwrap();
        context.wait_offscreen_idle();

        if let Some(pipeline) = &nv12_pipeline {
            for slot in 0..pipe.pending.len() {
                if let Some((_index, read_slot)) = pipe.pending[slot].take() {
                    let buf = pipeline.acquire_buf();
                    let src = unsafe {
                        std::slice::from_raw_parts(
                            pipe.ptrs[read_slot] as *const u8,
                            pipe.frame_bytes,
                        )
                    };
                    let dst = unsafe {
                        std::slice::from_raw_parts_mut(
                            pipeline.pool_addrs[buf] as *mut u8,
                            pipe.frame_bytes,
                        )
                    };
                    let copy_start = Instant::now();

                    dst.copy_from_slice(src);
                    copy_time += copy_start.elapsed().as_secs_f64();
                    pipe.free.push_back(read_slot);
                    pipeline.push_job(buf);
                }
            }
            pipeline.push_stop();
        }
    }
    let writer_busy = match writer.join() {
        Ok(Ok(busy)) => busy as f64 / 1e9,
        Ok(Err(err)) => bail!("failed to write frames to ffmpeg: {err}"),
        Err(_) => bail!("ffmpeg writer thread panicked"),
    };
    let video_status = proc.wait()?;
    mark!("drain + finalize");
    let video_stderr = video_stderr.join().unwrap_or_default();
    if !video_stderr.trim().is_empty() { warn!("[ffmpeg:video]\n{}", video_stderr.trim()); }
    if !video_status.success() { bail!("ffmpeg muxing failed ({video_status})"); }
    let elapsed = render_start_time.elapsed();
    let wall = elapsed.as_secs_f64().max(1e-9);
    info!("Render Time: {:.2?}", elapsed);
    info!(
        "Writer thread blocked on video transport+ffmpeg: {:.2}s ({:.1}%)",
        writer_busy,
        writer_busy / wall * 100.
    );
    info!(
        "Render thread waits: readback DMA wait {:.2}s ({:.1}%) | frame pool exhausted {:.2}s ({:.1}%) | vulkan-encoder slot wait {:.2}s ({:.1}%) | offscreen fence wait {:.2}s ({:.1}%)",
        dma_wait_time, dma_wait_time / wall * 100.,
        pool_wait_time, pool_wait_time / wall * 100.,
        slot_wait_time, slot_wait_time / wall * 100.,
        fence_wait_time, fence_wait_time / wall * 100.
    );
    let swap_wait_s = miniquad_context()
        .vulkan_context()
        .map(|c| c.frame_wait_ns as f64 / 1e9)
        .unwrap_or(0.0);
    if swap_wait_s > 0.0 {
        info!(
            "Swapchain frame fence wait: {:.2}s ({:.1}% of scene render) -> CPU-side scene recording {:.2}s",
            swap_wait_s,
            swap_wait_s / scene_render_time.max(1e-9) * 100.,
            (scene_render_time - swap_wait_s).max(0.),
        );
    }
    if let Some(pipeline) = &nv12_pipeline {
        let frame_bytes = pipeline.pool.first().map(|b| b.len()).unwrap_or(0);
        let blocks = pipeline.acquire_blocks.load(Ordering::Relaxed);
        let acquire_ns = pipeline.acquire_wait_ns.load(Ordering::Relaxed);
        let acquire_max_ns = pipeline.acquire_max_ns.load(Ordering::Relaxed);
        let queue_max = pipeline.queue_max.load(Ordering::Relaxed);
        let writes = pipeline.write_count.load(Ordering::Relaxed).max(1);
        let write_ns = pipeline.write_ns.load(Ordering::Relaxed);
        let write_max_ns = pipeline.write_max_ns.load(Ordering::Relaxed);
        info!(
            "NV12 handoff: pool {} x {:.1} MiB | render thread blocked on a free buffer {}x, {:.2}s (max {:.1}ms) | writer queue max {} frames | staging->pool copy {:.2}s ({:.0} fps) | writer {} writes, {:.2}s inside write_all (max {:.1}ms, {:.0} MiB/s) | backlog when loop ended {} frames",
            pipeline.pool.len(),
            frame_bytes as f64 / (1 << 20) as f64,
            blocks,
            acquire_ns as f64 / 1e9,
            acquire_max_ns as f64 / 1e6,
            queue_max,
            copy_time,
            out_frames as f64 / copy_time.max(1e-9),
            writes,
            write_ns as f64 / 1e9,
            write_max_ns as f64 / 1e6,
            (writes as f64 * frame_bytes as f64) / (write_ns as f64 / 1e9).max(1e-9) / (1 << 20) as f64,
            backlog_at_end,
        );
    }
    info!(
        "prpr stage times: update {:.2}s ({:.0} fps) | scene render {:.2}s ({:.0} fps) | YUV convert {:.2}s ({:.0} fps) | NV12 copy/submit {:.2}s ({:.0} fps)",
        update_time,
        out_frames as f64 / update_time.max(1e-9),
        scene_render_time,
        out_frames as f64 / scene_render_time.max(1e-9),
        yuv_convert_time,
        out_frames as f64 / yuv_convert_time.max(1e-9),
        nv12_submit_time,
        out_frames as f64 / nv12_submit_time.max(1e-9)
    );
    let written = ffmpeg_frames.load(Ordering::Relaxed);
    if written > 0 {
        show(written.min(out_frames), "Final frame", &mut step_time);
    }
    let busy = (loop_wall - dma_wait_time - pool_wait_time - slot_wait_time - fence_wait_time)
        .max(1e-9);
    info!(
        "Render loop Time: {:.2}s | prpr avg fps: {:.0}  (busy {:.2}s of {:.2}s, i.e. render side is idle {:.1}%)",
        loop_wall,
        out_frames as f64 / busy,
        busy,
        loop_wall,
        (1.0 - busy / loop_wall) * 100.0
    );
    let excluded_setup = mixing_s + audio_s + probe_s + spawn_s;
    let video_wall = (wall - excluded_setup).max(1e-9);
    if written > 0 {
        info!(
            "Video Pipeline Avg FPS: {:.0} ({} frames / {:.2}s of video pipeline; {:.2}s setup excluded: audio {:.2}s, encoder probe {:.2}s, ffmpeg spawn {:.2}s)",
            written as f64 / video_wall,
            written,
            video_wall,
            excluded_setup,
            mixing_s + audio_s,
            probe_s,
            spawn_s,
        );
    } else {
        info!("Video Pipeline Avg FPS: n/a (ffmpeg reported no progress)");
    }
    let total_wall = process_start.elapsed().as_secs_f64().max(1e-9);
    info!(
        "End-to-End Time: {:.2}s",
        total_wall,
    );

    send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    Ok(())
}

mod yuv_shader {
    pub const VERTEX: &str = r#"#version 450 core
layout(location = 0) in vec3 position;
layout(location = 1) in vec2 texcoord;
layout(location = 2) in vec4 color0;

layout(set = 0, binding = 0) uniform Uniforms {
    mat4 Projection;
    mat4 Model;
    vec4 _Time;
};

layout(location = 0) out vec2 uv;

void main() {
    gl_Position = vec4(position.xy, 0.0, 1.0);
    uv = vec2(texcoord.x, 1.0 - texcoord.y);
}"#;

    pub const Y: &str = r#"#version 450 core
layout(set = 0, binding = 0) uniform Uniforms {
    mat4 Projection;
    mat4 Model;
    vec4 _Time;
};
layout(location = 0) in vec2 uv;
layout(set = 0, binding = 3) uniform sampler2D tex;
layout(location = 0) out vec4 FragColor;

void main() {
    vec2 size = vec2(textureSize(tex, 0));
    vec2 st = clamp(uv, vec2(0.5) / size, vec2(1.0) - vec2(0.5) / size);
    vec3 c = texture(tex, st).rgb;
    float y = (16.0 + 219.0 * (0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b)) / 255.0;
    FragColor = vec4(y, 0.0, 0.0, 1.0);
}"#;

    pub const UV: &str = r#"#version 450 core
layout(set = 0, binding = 0) uniform Uniforms {
    mat4 Projection;
    mat4 Model;
    vec4 _Time;
};
layout(location = 0) in vec2 uv;
layout(set = 0, binding = 3) uniform sampler2D tex;
layout(location = 0) out vec4 FragColor;

void main() {
    vec2 size = vec2(textureSize(tex, 0));
    vec2 st = clamp(uv, vec2(0.5) / size, vec2(1.0) - vec2(0.5) / size);
    vec3 c = texture(tex, st).rgb;
    float u = (128.0 + 224.0 * (-0.114572 * c.r - 0.385428 * c.g + 0.5 * c.b)) / 255.0;
    float v = (128.0 + 224.0 * (0.5 * c.r - 0.454153 * c.g - 0.045847 * c.b)) / 255.0;
    FragColor = vec4(u, v, 0.0, 1.0);
}"#;
}


