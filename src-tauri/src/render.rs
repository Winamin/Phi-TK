prpr::tl_file!("render");

use crate::Path;
use anyhow::{bail, Context, Result};
use macroquad::{miniquad::{gl::{GLuint, GL_RGB, GL_RED, GL_RG}, RenderPass as MQRenderPass, Texture, TextureFormat, TextureParams, TextureWrap}, prelude::*};
use prpr::{
    config::{ChallengeModeColor, Config, Mods},
    core::{internal_id, MSRenderTarget},
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
    cell::RefCell,
    collections::VecDeque,
    io::{BufRead, BufWriter, Write},
    ops::DerefMut,
    path::PathBuf,
    process::{Command, Stdio},
    rc::Rc,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc,
    },
    time::Instant,
};
use std::{ffi::OsStr, fmt::Write as _};
use tempfile::NamedTempFile;



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
            sample_count: 1,
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

const GL_PACK_ALIGNMENT: u32 = 0x0D05;


pub fn drain_stderr<R>(mut stderr: R) -> std::thread::JoinHandle<String>
where
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
    let gl = unsafe { get_internal_gl() };
    let texture = Texture::new_render_texture(
        gl.quad_context,
        TextureParams {
            width: w,
            height: h,
            format,
            filter: FilterMode::Linear,
            wrap: TextureWrap::Clamp,
        },
    );
    RenderTarget {
        texture: Texture2D::from_miniquad_texture(texture),
        render_pass: MQRenderPass::new(gl.quad_context, texture, None),
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
    outputs: Vec<RenderTarget>,
    materials: Vec<Material>,
    planar: bool,
}

impl YuvTarget {
    fn new(w: u32, h: u32, planar: bool) -> Result<Self> {
        let hd = h > 576;
        let mut outputs = vec![new_plane(w, h, TextureFormat::Alpha)];
        let mut materials = vec![yuv_material(&yuv_shader::y(hd)).context("failed to compile Y plane shader")?];
        if planar {
            outputs.push(new_plane(w / 2, h / 2, TextureFormat::Alpha));
            materials.push(yuv_material(&yuv_shader::u(hd)).context("failed to compile U plane shader")?);
            outputs.push(new_plane(w / 2, h / 2, TextureFormat::Alpha));
            materials.push(yuv_material(&yuv_shader::v(hd)).context("failed to compile V plane shader")?);
        } else {
            outputs.push(new_plane(w / 2, h / 2, TextureFormat::LuminanceAlpha));
            materials.push(yuv_material(&yuv_shader::uv(hd)).context("failed to compile UV plane shader")?);
        }
        Ok(Self {
            outputs,
            materials,
            planar,
        })
    }

    fn plane_layout(&self, w: i32, h: i32) -> Vec<(GLuint, usize, i32, i32, u32)> {
        let (cw, ch) = (w / 2, h / 2);
        let y = (
            internal_id(&self.outputs[0]),
            (w * h) as usize,
            w,
            h,
            GL_RED,
        );
        if self.planar {
            let c = |i: usize| {
                (
                    internal_id(&self.outputs[i]),
                    (cw * ch) as usize,
                    cw,
                    ch,
                    GL_RED,
                )
            };
            vec![y, c(1), c(2)]
        } else {
            vec![
                y,
                (
                    internal_id(&self.outputs[1]),
                    (cw * ch * 2) as usize,
                    cw,
                    ch,
                    GL_RG,
                ),
            ]
        }
    }

    fn convert(&self, src: &RenderTarget) {
        unsafe { get_internal_gl() }.flush();
        for m in &self.materials {
            m.set_texture("tex", src.texture);
        }

        let vertices = [
            Vertex::new(-1., -1., 0., 0., 0., WHITE),
            Vertex::new(1., -1., 0., 1., 0., WHITE),
            Vertex::new(-1., 1., 0., 0., 1., WHITE),
            Vertex::new(1., 1., 0., 1., 1., WHITE),
        ];
        let indices = [0u16, 2, 3, 0, 1, 3];
        let prev_viewport = unsafe { get_internal_gl() }.quad_gl.get_viewport();

        for (material, target) in self.materials.iter().zip(self.outputs.iter()) {
            gl_use_material(*material);
            let quad = unsafe { get_internal_gl() }.quad_gl;
            quad.viewport(None);
            quad.render_pass(Some(target.render_pass));
            quad.draw_mode(DrawMode::Triangles);
            quad.geometry(&vertices, &indices);
            gl_use_default_material();
        }

        let quad = unsafe { get_internal_gl() }.quad_gl;
        quad.render_pass(Some(src.render_pass));
        quad.viewport(prev_viewport);
        unsafe { get_internal_gl() }.flush();
    }
}

pub async fn main() -> Result<()> {
    use crate::ipc::client::*;

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

    let gl = unsafe { get_internal_gl() };

    if std::env::var("PHITK_DX_SELFTEST").is_ok() {
        crate::zerocopy::probe_zero_copy_support();
        crate::zerocopy::probe_dx_interop(&params.config.video_codec);
    }

    let volume_music = std::mem::take(&mut config.volume_music);
    let volume_sfx = std::mem::take(&mut config.volume_sfx);

    let length = track_length - chart.offset.min(0.) as f64 + 1.;
    let video_length = O + length + A + params.config.ending_length;
    let offset = chart.offset.max(0.);

    let render_start_time = Instant::now();

    send(IPCEvent::StartMixing);
    let mixing_output = NamedTempFile::new()?;
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

    info!("=== Audio/Video Sync Configuration ===");
    info!("  Audio delay: {} frames", params.config.audio_delay_frames);
    info!("  Audio delay: {:.6} seconds", audio_delay);
    info!("  Frame duration: {:.6}s @ {}fps", frame_duration, params.config.fps);
    info!("  Sample delay: {} samples @ {}Hz", (audio_delay * sample_rate_f64).round() as i64, sample_rate);
    info!("======================================");

    let audio_buffer_length = video_length + audio_delay.abs();
    let mut output = vec![0.0_f32; (audio_buffer_length * sample_rate_f64).ceil() as usize * 2];

    if volume_music != 0.0 {
        let start_time = Instant::now();
        let original_pos = O - chart.offset.min(0.) as f64;
        let pos = original_pos + audio_delay;

        info!("Music mixing: original_pos={:.6}s, delayed_pos={:.6}s", original_pos, pos);

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

        info!("SFX mixing: offset={:.6}s (includes {:.6}s delay)", o_offset, audio_delay);

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

    let (audio_codec, output_format) = if let Some(bit) = audio_bit {
        (format!("pcm_f{}le", bit), "wav".to_string())
    } else {
        match audio_format.as_str()
        {
            "flac" => ("flac".to_string(), "flac".to_string()),
            "mp3" => ("libmp3lame".to_string(), "mp3".to_string()),
            "aac" => ("aac".to_string(), "mp4".to_string()),
            "opus" => ("libopus".to_string(), "opus".to_string()),
            "wav" => ("pcm_f16le".to_string(), "wav".to_string()),
            _ => {
                warn!("Unknown audio format '{}', using AAC/MP4 as default", audio_format);
                ("aac".to_string(), "mp4".to_string())
            }
        }
    };

    let args_str = if target_sample_rate != sample_rate {
        let resample_filter = format!("aresample=resampler=soxr:precision=33:osr={}:dither_method=triangular", target_sample_rate);
        format!(
            "-y -f f32le -ar {} -ac 2 -i - -af {} -c:a {} -f {}",
            sample_rate,
            resample_filter,
            audio_codec,
            output_format
        )
    } else {
        format!(
            "-y -f f32le -ar {} -ac 2 -i - -c:a {} -f {}",
            sample_rate,
            audio_codec,
            output_format
        )
    };

    let mut proc = cmd_hidden(&ffmpeg)
        .args(args_str.split_whitespace())
        .arg(mixing_output.path())
        .arg("-loglevel")
        .arg("warning")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| tl!("run-ffmpeg-failed"))?;
    let audio_stderr = drain_stderr(proc.stderr.take().unwrap());
    let input = proc.stdin.as_mut().unwrap();
    let mut writer = BufWriter::new(input);
    for sample in output.into_iter() {
        writer.write_all(&sample.to_le_bytes())?;
    }
    drop(writer);
    proc.wait()?;
    if let Ok(text) = audio_stderr.join() {
        if !text.trim().is_empty() {
            warn!("[ffmpeg:audio]\n{}", text);
        }
    }


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

    let mst = Rc::new(MSRenderTarget::new((vw, vh), config.sample_count));
    let my_time: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.));
    let tm = TimeManager::manual(Box::new({
        let my_time = Rc::clone(&my_time);
        move || *(*my_time).borrow()
    }));
    static MSAA: AtomicBool = AtomicBool::new(false);
    let player = build_player(&params.config).await?;
    let mut main = Main::new(
        Box::new(
            LoadingScene::new(GameMode::Normal, info, config, fs, Some(player), None, None).await?,
        ),
        tm,
        {
            let mut cnt = 0;
            let mst = Rc::clone(&mst);
            move || {
                cnt += 1;
                if cnt % 2 == 1 {
                    MSAA.store(true, Ordering::SeqCst);
                    Some(mst.input())
                } else {
                    MSAA.store(false, Ordering::SeqCst);
                    Some(mst.output())
                }
            }
        },
    )
        .await?;
    main.top_level = false;
    main.viewport = Some((0, 0, vw as _, vh as _));

    const O: f64 = LoadingScene::TOTAL_TIME as f64 + GameScene::BEFORE_TIME as f64;
    const A: f64 = 1.0;

    let fps = params.config.fps;
    let frames = (video_length * fps as f64).ceil() as u64;
    send(IPCEvent::StartRender(frames));
    

    fn test_encoder(ffmpeg: &Path, encoder: &str) -> Result<(bool, String)> {
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
    if let Ok((ok, err)) = test_encoder(ffmpeg.as_ref(), "libsvtav1") {
        encoder_availability.av1_svt = ok;
        if !ok {
            hw_errors.push(format!("libsvtav1 test failed:
{}", err.trim()));
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
                    ("libsvtav1", encoder_availability.av1_svt, &mut encoder_availability.av1_svt),
                    ("libaom-av1", true, &mut dummy_flag),
                ],
                "qsv" => vec![
                    ("av1_qsv", encoder_availability.av1_qsv, &mut encoder_availability.av1_qsv),
                    ("av1_nvenc", encoder_availability.av1_nvenc, &mut encoder_availability.av1_nvenc),
                    ("av1_amf", encoder_availability.av1_amf, &mut encoder_availability.av1_amf),
                    ("libsvtav1", encoder_availability.av1_svt, &mut encoder_availability.av1_svt),
                    ("libaom-av1", true, &mut dummy_flag),
                ],
                "amf" => vec![
                    ("av1_amf", encoder_availability.av1_amf, &mut encoder_availability.av1_amf),
                    ("av1_nvenc", encoder_availability.av1_nvenc, &mut encoder_availability.av1_nvenc),
                    ("av1_qsv", encoder_availability.av1_qsv, &mut encoder_availability.av1_qsv),
                    ("libsvtav1", encoder_availability.av1_svt, &mut encoder_availability.av1_svt),
                    ("libaom-av1", true, &mut dummy_flag),
                ],
                "vulkan" => vec![
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
                    ("libsvtav1", encoder_availability.av1_svt, &mut encoder_availability.av1_svt),
                    ("av1_nvenc", encoder_availability.av1_nvenc, &mut encoder_availability.av1_nvenc),
                    ("av1_qsv", encoder_availability.av1_qsv, &mut encoder_availability.av1_qsv),
                    ("av1_amf", encoder_availability.av1_amf, &mut encoder_availability.av1_amf),
                    ("av1_vulkan", encoder_availability.av1_vulkan, &mut encoder_availability.av1_vulkan),
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
    info!("  Selected encoder: {}", ffmpeg_encoder);
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
        info!("  Note: it is much slower — select the nvenc/amf encoder explicitly if you want speed.");
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

    if ffmpeg_encoder.ends_with("_vulkan") {
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

        let codec_unsupported = match params.config.video_codec.as_str() {
            "h264" => !h264_supported,
            "hevc" => !hevc_supported,
            "av1"  => !av1_supported,
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
    let gpu_yuv = params.config.gpu_yuv
        && std::env::var("PHITK_RGB24_READBACK").is_err()
        && vw % 2 == 0
        && vh % 2 == 0;

    let planar = matches!(
        ffmpeg_encoder,
        "libx265" | "libsvtav1" | "libaom-av1" | "librav1e"
    );
    let nv12 = if gpu_yuv {
        match YuvTarget::new(vw, vh, planar) {
            Ok(target) => {
                info!(
                    "GPU YUV: enabled (shader outputs {}, {:.2} MB/frame, was {:.2} MB/frame)",
                    if planar { "YUV420P planar" } else { "NV12" },
                    (vw as f64 * vh as f64 * 1.5) / 1048576.,
                    (vw as f64 * vh as f64 * 3.) / 1048576.
                );
                Some(target)
            }
            Err(err) => {
                warn!("GPU YUV unavailable, falling back to RGB24 readback: {err:?}");
                None
            }
        }
    } else {
        None
    };
    let gpu_yuv = nv12.is_some();

    let global_args = "-y";
    let mut input_args = String::new();
        write!(
            &mut input_args,
            "-f rawvideo -c:v rawvideo -s {vw}x{vh} -r {fps} -pix_fmt {}",
            if gpu_yuv {
                if planar { "yuv420p" } else { "nv12" }
            } else {
                "rgb24"
            }
        )?;
        if params.config.ffmpeg_thread {
            input_args.push_str(" -thread_queue_size 2048");
        }
    input_args.push_str(" -i - -i");

    let video = match params.config.video {
        true => "mov",
        false => "mp4",
    };

    let strict_flag = if params.config.audio_format == "flac" && video == "mp4" {
        "-strict -2 "
    } else {
        ""
    };
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
    let vf_arg = if dx.is_some() {
        "-vf format=nv12,hwupload".to_owned()
    } else if gpu_yuv {
        if is_vulkan_encoder {
            "-vf hwupload".to_owned()
        } else {
            String::new()
        }
    } else if is_vulkan_encoder {
        "-vf format=nv12,vflip,hwupload".to_owned()
    } else if is_hw_encoder {
        "-vf format=nv12,vflip".to_owned()
    } else {
        "-vf format=yuv420p,vflip".to_owned()
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
    let args2 = if is_vulkan_encoder {
        format!(
            "-c:a {} -c:v {} {} {} -map 0:v:0 -map 1:a:0 -shortest {} {} {} {} -f {}",
            audio_codec,
            ffmpeg_encoder,
            bitrate_control,
            params.config.bitrate,
            strict_flag,
            if params.config.disable_loading {
                format!("-ss {}", LoadingScene::TOTAL_TIME + GameScene::BEFORE_TIME)
            } else {
                "-ss 0.1".to_string()
            },
            out_extra,
            async_depth,
            video,
        )
    } else {
        format!(
            "-c:a {} -c:v {} {} {} {} {} -map 0:v:0 -map 1:a:0 -shortest {} {} {} -f {}",
            audio_codec,
            ffmpeg_encoder,
            bitrate_control,
            params.config.bitrate,
            ffmpeg_preset,
            ffmpeg_preset_name,
            strict_flag,
            if params.config.disable_loading {
                format!("-ss {}", LoadingScene::TOTAL_TIME + GameScene::BEFORE_TIME)
            } else {
                "-ss 0.1".to_string()
            },
            out_extra,
            video,
        )
    };

    let ffmpeg_frames = std::sync::Arc::new(AtomicU64::new(0));
    let mut proc = {
        let mut cmd = cmd_hidden(&ffmpeg);
        cmd.args(global_args.split_whitespace());
        cmd.args(["-progress", "pipe:2", "-nostats"]);
        if is_vulkan_encoder {
            cmd.arg("-init_hw_device").arg("vulkan=vk")
               .arg("-filter_hw_device").arg("vk");
        } else if let Some(enc) = &dx {
            cmd.arg("-init_hw_device")
                .arg(format!("{}={}", enc.hw, DX_DEVICE))
                .arg("-filter_hw_device")
                .arg(DX_DEVICE);
        }
        cmd.args(input_args.split_whitespace())
            .arg(mixing_output.path())
            .args(args2.split_whitespace())
            .arg(output_path)
            .arg("-loglevel")
            .arg("error")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| tl!("run-ffmpeg-failed"))?
    };


    let video_stderr =
        drain_stderr_with_progress(proc.stderr.take().unwrap(), ffmpeg_frames.clone());

    struct Plane {
        pbos: Vec<GLuint>,
        fbo: GLuint,
        size: usize,
        w: i32,
        h: i32,
        format: u32,
    }

    const PBO_COUNT: usize = 6;
    let n = PBO_COUNT.min(fps as usize).max(2);

    let plane_specs: Vec<(GLuint, usize, i32, i32, u32)> = if let Some(yuv) = &nv12 {
        yuv.plane_layout(vw as i32, vh as i32)
    } else {
        vec![(
            internal_id(&mst.output()),
            vw as usize * vh as usize * 3,
            vw as i32,
            vh as i32,
            GL_RGB,
        )]
    };

    let mut planes: Vec<Plane> = Vec::with_capacity(plane_specs.len());
    unsafe {
        use miniquad::gl::*;
        for (fbo, size, w, h, format) in plane_specs {
            let mut pbos: Vec<GLuint> = vec![0; n];
            glGenBuffers(n as _, pbos.as_mut_ptr());
            for pbo in &pbos {
                glBindBuffer(GL_PIXEL_PACK_BUFFER, *pbo);
                glBufferData(GL_PIXEL_PACK_BUFFER, size as _, std::ptr::null(), GL_STREAM_READ);
            }
            glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
            planes.push(Plane {
                pbos,
                fbo,
                size,
                w,
                h,
                format,
            });
        }
    }
    let frame_size: usize = planes.iter().map(|p| p.size).sum();
    info!(
        "Using {} PBOs per plane, {} plane(s), {} bytes/frame",
        n,
        planes.len(),
        frame_size
    );

    const WRITE_QUEUE: usize = 4;
    let (job_tx, job_rx) = mpsc::sync_channel::<Option<(usize, Vec<(usize, usize)>)>>(WRITE_QUEUE);
    let (done_tx, done_rx) = mpsc::channel::<usize>();
    let mut ffmpeg_stdin = proc.stdin.take().unwrap();
    let writer = std::thread::Builder::new()
        .name("ffmpeg-writer".to_owned())
        .spawn(move || -> std::io::Result<u64> {
            let mut busy: u64 = 0;
            while let Ok(job) = job_rx.recv() {
                match job {
                    Some((slot, parts)) => {
                        let t = Instant::now();
                        for (ptr, len) in parts {
                            let bytes =
                                unsafe { std::slice::from_raw_parts(ptr as *const u8, len) };
                            ffmpeg_stdin.write_all(bytes)?;
                        }
                        busy += t.elapsed().as_nanos() as u64;
                        let _ = done_tx.send(slot);
                    }
                    None => break,
                }
            }
            let t = Instant::now();
            ffmpeg_stdin.flush()?;
            busy += t.elapsed().as_nanos() as u64;
            Ok(busy)
        })
        .context("failed to spawn ffmpeg writer thread")?;

    fn submit_frame(
        planes: &[Plane],
        pbo_index: usize,
        job_tx: &mpsc::SyncSender<Option<(usize, Vec<(usize, usize)>)>>,
    ) -> Result<(f64, f64)> {

        let mut dma_wait = 0f64;
        let mut parts = Vec::with_capacity(planes.len());
        unsafe {
            use miniquad::gl::*;
            for plane in planes {
                glBindBuffer(GL_PIXEL_PACK_BUFFER, plane.pbos[pbo_index]);
                let t = Instant::now();
                let src =
                    glMapBufferRange(GL_PIXEL_PACK_BUFFER, 0, plane.size as _, GL_MAP_READ_BIT);
                if src.is_null() {
                    glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
                    bail!("Failed to map PBO");
                }
                dma_wait += t.elapsed().as_secs_f64();
                glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
                parts.push((src as usize, plane.size));
            }
        }
        let t = Instant::now();
        job_tx
            .send(Some((pbo_index, parts)))
            .map_err(|_| anyhow::anyhow!("ffmpeg writer thread exited unexpectedly"))?;
        Ok((dma_wait, t.elapsed().as_secs_f64()))
    }

    fn unmap_slot(planes: &[Plane], slot: usize) {
        unsafe {
            use miniquad::gl::*;
            for plane in planes {
                glBindBuffer(GL_PIXEL_PACK_BUFFER, plane.pbos[slot]);
                glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
                glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
            }
        }
    }

    fn fence_ready(fence: miniquad::gl::GLsync) -> bool { unsafe { miniquad::gl::glClientWaitSync(fence, 0, 0) == miniquad::gl::GL_ALREADY_SIGNALED } }
    fn fence_block(fence: miniquad::gl::GLsync) -> f64 {
        let t = Instant::now();
        unsafe { miniquad::gl::glClientWaitSync(fence, miniquad::gl::GL_SYNC_FLUSH_COMMANDS_BIT, 1_000_000_000); }
        t.elapsed().as_secs_f64()
    }
    fn fence_drop(fence: miniquad::gl::GLsync) {
        unsafe { miniquad::gl::glDeleteSync(fence) }
    }

    send(IPCEvent::StartRender(frames));

    let fps_f64 = params.config.fps as f64;
    let frame_duration = 1.0 / fps_f64;
    let total_frames = frames;

    let frames10 = (total_frames / 10).max(1);
    let mut step_time = Instant::now();
    let mut next_slot: usize = 0;
    let mut in_flight: VecDeque<usize> = VecDeque::new();
    let mut slot_busy = vec![false; n];
    let mut fences: Vec<Option<miniquad::gl::GLsync>> = vec![None; n];
    let mut dma_wait_time: f64 = 0.0;
    let mut queue_wait_time: f64 = 0.0;

    for frame in 0..total_frames {
        if frame % frames10 == 0 || frame == total_frames - 1 {
            let p = (frame as f64 / total_frames as f64).min(1.0);
            let pct = (p * 100.0).ceil() as i8;
            let w = 20;
            let fill = (p * w as f64).round() as usize;
            let pad = w - fill;

            let t = if frame == total_frames - 1 {
                "Final frame".to_string()
            } else {
                format!("{:.2}s", step_time.elapsed().as_secs_f32())
            };

            info!(
            "Rendering: [{}{}] {:>3}% | Time: {} | Frames: {}/{}",
            "█".repeat(fill),
            " ".repeat(pad),
            pct,
            t,
            frame + 1,
            total_frames
        );

            step_time = Instant::now();
        }
        let current_frame_time = frame as f64 * frame_duration;
        *my_time.borrow_mut() = current_frame_time;
        let output = mst.output();
        let render_pass: MQRenderPass = unsafe { std::mem::transmute(output.render_pass) };
        gl.quad_gl.render_pass(Some(render_pass));
        main.update()?;
        main.render(&mut painter)?;
        if current_frame_time <= LoadingScene::TOTAL_TIME as f64 && !params.config.disable_loading { draw_rectangle(0., 0., 0., 0., Color::default()); }

        if MSAA.load(Ordering::SeqCst) { mst.blit(); }
            if let Some(nv12) = &nv12 {
                nv12.convert(&mst.output());
            }

        let slot = next_slot;
        next_slot = (next_slot + 1) % n;
        while let Ok(done) = done_rx.try_recv() {
            unmap_slot(&planes, done);
            slot_busy[done] = false;
        }
        let guard_start = Instant::now();
        if slot_busy[slot] {
            loop {
                let done = done_rx.recv().context("ffmpeg writer thread exited")?;
                unmap_slot(&planes, done);
                slot_busy[done] = false;
                if !slot_busy[slot] {
                    break;
                }
            }
        }
        queue_wait_time += guard_start.elapsed().as_secs_f64();
        unsafe {
            use miniquad::gl::*;
            glPixelStorei(GL_PACK_ALIGNMENT, 1);
            for plane in &planes {
                glBindFramebuffer(GL_READ_FRAMEBUFFER, plane.fbo);
                glBindBuffer(GL_PIXEL_PACK_BUFFER, plane.pbos[slot]);
                glReadPixels(
                    0,
                    0,
                    plane.w,
                    plane.h,
                    plane.format,
                    GL_UNSIGNED_BYTE,
                    std::ptr::null_mut(),
                );
                glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
            }
            glPixelStorei(GL_PACK_ALIGNMENT, 4);
            glBindFramebuffer(GL_READ_FRAMEBUFFER, 0);
        }
        fences[slot] = unsafe { Some(miniquad::gl::glFenceSync(miniquad::gl::GL_SYNC_GPU_COMMANDS_COMPLETE, 0)) };
        slot_busy[slot] = true;
        in_flight.push_back(slot);

        while !in_flight.is_empty() {
            let oldest = *in_flight.front().unwrap();
            let ready = match fences[oldest] {
                Some(fence) => fence_ready(fence),
                None => true,
            };
            if !ready && in_flight.len() < n - 1 { break; }
            if !ready {
                if let Some(fence) = fences[oldest] {
                    dma_wait_time += fence_block(fence);
                }
            }
            in_flight.pop_front();
            if let Some(fence) = fences[oldest].take() { fence_drop(fence); }
            let (d, q) = submit_frame(&planes, oldest, &job_tx)?;
            dma_wait_time += d;
            queue_wait_time += q;
            while let Ok(done) = done_rx.try_recv() {
                unmap_slot(&planes, done);
                slot_busy[done] = false;
            }
        }
        send(IPCEvent::Frame);
    }

    while let Some(old) = in_flight.pop_front() {
        if let Some(fence) = fences[old].take() {
            dma_wait_time += fence_block(fence);
            fence_drop(fence);
        }
        let (d, q) = submit_frame(&planes, old, &job_tx)
            .context("failed to read back the final frames")?;
        dma_wait_time += d;
        queue_wait_time += q;
    }

    let _ = job_tx.send(None);
    drop(job_tx);
    let writer_busy = match writer.join() {
        Ok(Ok(busy)) => busy as f64 / 1e9,
        Ok(Err(err)) => bail!("failed to write frames to ffmpeg: {err}"),
        Err(_) => bail!("ffmpeg writer thread panicked"),
    };
    while let Ok(done) = done_rx.try_recv() {
        unmap_slot(&planes, done);
    }
    proc.wait()?;

    if let Ok(text) = video_stderr.join() {
        if !text.trim().is_empty() {
            warn!("[ffmpeg:video]\n{}", text);
        }
    }


    let elapsed = render_start_time.elapsed();
    let wall = elapsed.as_secs_f64().max(1e-9);
    info!("Render Time: {:.2?}", elapsed);
    info!(
        "Writer thread blocked on pipe+ffmpeg: {:.2}s ({:.1}%)",
        writer_busy,
        writer_busy / wall * 100.
    );
    info!(
        "Render thread waits: readback DMA wait {:.2}s ({:.1}%) | frame queue full {:.2}s ({:.1}%)",
        dma_wait_time, dma_wait_time / wall * 100.,
        queue_wait_time, queue_wait_time / wall * 100.
    );
    let written = ffmpeg_frames.load(Ordering::Relaxed);
    let busy = (wall - dma_wait_time - queue_wait_time).max(1e-9);
    info!(
        "Render Avg FPS: {:.0}  (busy {:.2}s of {:.2}s, i.e. render side is idle {:.1}%)",
        total_frames as f64 / busy,
        busy,
        wall,
        (1.0 - busy / wall) * 100.0
    );
    if written > 0 {
        info!(
            "End-to-End Avg FPS: {:.0} ({} frames written by ffmpeg)",
            written as f64 / wall,
            written
        );
    } else {
        info!("End-to-End Avg FPS: n/a (ffmpeg reported no progress)");
    }

    unsafe {
        use miniquad::gl::*;
        for plane in &planes {
            glDeleteBuffers(n as _, plane.pbos.as_ptr());
        }
    }

    send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    Ok(())
}


// GL 300
mod yuv_shader {
    pub const VERTEX: &str = r#"#version 300 es
in vec3 position;
in vec2 texcoord;
in vec4 color0;

out vec2 uv;

void main() {
    gl_Position = vec4(position.xy, 0.0, 1.0);
    uv = vec2(texcoord.x, 1.0 - texcoord.y);
}"#;
    fn coeffs(hd: bool) -> [f32; 9] {
        if hd {
            [0.2126, 0.7152, 0.0722, -0.114572, -0.385428, 0.5, 0.5, -0.454153, -0.045847]
        } else {
            [0.299, 0.587, 0.114, -0.168736, -0.331264, 0.5, 0.5, -0.418688, -0.081312]
        }
    }

    pub fn y(hd: bool) -> String {
        let c = coeffs(hd);
        format!(
            r#"#version 300 es
precision mediump float;

in vec2 uv;
out vec4 FragColor;

uniform sampler2D tex;

void main() {{
    vec3 c = texture(tex, uv).rgb;
    float y = (16.0 + 219.0 * ({:.6} * c.r + {:.6} * c.g + {:.6} * c.b)) / 255.0;
    FragColor = vec4(y, 0.0, 0.0, 1.0);
}}"#,
            c[0], c[1], c[2]
        )
    }

    pub fn uv(hd: bool) -> String {
        let c = coeffs(hd);
        format!(
            r#"#version 300 es
precision mediump float;

in vec2 uv;
out vec4 FragColor;

uniform sampler2D tex;

void main() {{
    vec3 c = texture(tex, uv).rgb;
    float u = (128.0 + 224.0 * ({:.6} * c.r + {:.6} * c.g + {:.6} * c.b)) / 255.0;
    float v = (128.0 + 224.0 * ({:.6} * c.r + {:.6} * c.g + {:.6} * c.b)) / 255.0;
    FragColor = vec4(u, v, 0.0, 1.0);
}}"#,
            c[3], c[4], c[5], c[6], c[7], c[8]
        )
    }

    pub fn u(hd: bool) -> String {
        let c = coeffs(hd);
        format!(
            r#"#version 300 es
precision mediump float;

in vec2 uv;
out vec4 FragColor;

uniform sampler2D tex;

void main() {{
    vec3 c = texture(tex, uv).rgb;
    float u = (128.0 + 224.0 * ({:.6} * c.r + {:.6} * c.g + {:.6} * c.b)) / 255.0;
    FragColor = vec4(u, 0.0, 0.0, 1.0);
}}"#,
            c[3], c[4], c[5]
        )
    }

    pub fn v(hd: bool) -> String {
        let c = coeffs(hd);
        format!(
            r#"#version 300 es
precision mediump float;

in vec2 uv;
out vec4 FragColor;

uniform sampler2D tex;

void main() {{
    vec3 c = texture(tex, uv).rgb;
    float v = (128.0 + 224.0 * ({:.6} * c.r + {:.6} * c.g + {:.6} * c.b)) / 255.0;
    FragColor = vec4(v, 0.0, 0.0, 1.0);
}}"#,
            c[6], c[7], c[8]
        )
    }
}