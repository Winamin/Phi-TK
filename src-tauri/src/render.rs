// Prevents additional console window on Windows in release, DO NOT REMOVE!!
prpr::tl_file!("render");

use crate::Path;
use anyhow::{bail, Context, Result};
use macroquad::{miniquad::gl::GLuint, prelude::*};
use prpr::{
    config::{ChallengeModeColor, Config, Mods},
    core::{internal_id, MSRenderTarget, NoteKind},
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
    io::{BufRead, BufWriter, Write},
    ops::DerefMut,
    path::PathBuf,
    process::{Command, Stdio},
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
use std::{ffi::OsStr, fmt::Write as _};
use tempfile::NamedTempFile;
use rayon::prelude::*;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct RenderConfig {
    pub resolution: (u32, u32),
    pub ffmpeg_preset: String,
    pub ending_length: f64,
    pub disable_loading: bool,
    pub chart_debug: bool,
    pub flid_x: bool,
    pub chart_ratio: f32,
    pub buffer_size: f32,
    pub combo: String,
    pub fps: u32,
    pub hardware_accel: bool,
    pub video_codec: String,
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

    pub ui_score: bool,
    pub ui_combo: bool,
    pub ui_name: bool,
    pub ui_level: bool,
    pub ui_line: bool,
    pub ui_pb: bool,
    pub ui_pause: bool,

    //ffmpeg
    pub ffmpeg_thread: bool,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            ffmpeg_preset: "medium p4 balanced".to_string(),
            ending_length: -2.0,
            disable_loading: true,
            chart_debug: false,
            flid_x: false,
            chart_ratio: 1.0,
            buffer_size: 256.0,
            combo: "AUTOPLAY".to_string(),
            fps: 60,
            hardware_accel: true,
            video_codec: "h264".to_string(),
            show_progress_text: false,
            show_time_text: false,
            target_audio: 96000,
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

            //ffmpeg
            ffmpeg_thread: false,
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
            "{4d36e968-e325-11ce-bfc1-08002be10318}", // Display adapters
            "{4d36e97d-e325-11ce-bfc1-08002be10318}", // System devices
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
}

pub async fn build_player(config: &RenderConfig) -> Result<BasicPlayer> {
    Ok(BasicPlayer {
        avatar: if let Some(path) = &config.player_avatar {
            Some(
                Texture2D::from_file_with_format(
                    &tokio::fs::read(path)
                        .await
                        .with_context(|| tl!("load-avatar-failed"))?,
                    None,
                )
                    .into(),
            )
        } else {
            None
        },
        id: 0,
        rks: config.player_rks,
    })
}

pub fn cmd_hidden(program: impl AsRef<OsStr>) -> Command {
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
    dbg!(&ffmpeg);

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
    let ending = ld!("ending.ogg");
    let track_length = music.length() as f64;
    let sfx_click = ld!("click.ogg");
    let sfx_drag = ld!("drag.ogg");
    let sfx_flick = ld!("flick.ogg");

    let mut gl = unsafe { get_internal_gl() };

    let volume_music = std::mem::take(&mut config.volume_music);
    let volume_sfx = std::mem::take(&mut config.volume_sfx);

    let length = track_length - chart.offset.min(0.) as f64 + 1.;
    let video_length = O + length + A + params.config.ending_length;
    let offset = chart.offset.max(0.);

    let render_start_time = Instant::now();

    send(IPCEvent::StartMixing);
    let mixing_output = NamedTempFile::new()?;
    let target_sample_rate = params.config.target_audio;
    let sample_rate = 96000;
    let sample_rate_f64 = sample_rate as f64;
    assert_eq!(sample_rate, ending.sample_rate());
    assert_eq!(sample_rate, sfx_click.sample_rate());
    assert_eq!(sample_rate, sfx_drag.sample_rate());
    assert_eq!(sample_rate, sfx_flick.sample_rate());

    let mut output = vec![0.0_f32; (video_length * sample_rate_f64).ceil() as usize * 2];
    if volume_music != 0.0 {
        let start_time = Instant::now();
        let pos = O - chart.offset.min(0.) as f64;
        let count = (music.length() as f64 * sample_rate_f64) as usize;
        let start_index = (pos * sample_rate_f64).round() as usize * 2;
        let ratio = 1.0 / sample_rate_f64;

        let output_ptr = output.as_mut_ptr();
        for i in 0..count {
            let position = i as f64 * ratio;
            let frame = music.sample(position as f32).unwrap_or_default();
            let left = frame.0 * volume_music;
            let right = frame.1 * volume_music;

            unsafe {
                *output_ptr.add(start_index + i * 2) += left;
                *output_ptr.add(start_index + i * 2 + 1) += right;
            }
        }
        info!("music Time:{:?}", start_time.elapsed());
    }

    fn place(pos: f64, clip: &AudioClip, volume: f32, output: &mut [f32], sample_rate_f64: f64) -> usize {
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
    }

    if volume_sfx != 0.0 {
        let start_time = Instant::now();
        let offset_f64 = offset as f64;
        let o_offset = O + offset_f64;
        let sample_rate_f64 = sample_rate as f64;

        // 按音符类型分组处理
        let (clicks, drags, flicks): (Vec<_>, Vec<_>, Vec<_>) = chart.lines
            .par_iter()
            .flat_map(|line| &line.notes)
            .filter(|note| !note.fake)
            .map(|note| {
                let time = o_offset + note.time as f64;
                (time, note.kind.clone())
            })
            .fold(
                || (Vec::new(), Vec::new(), Vec::new()),
                |(mut clicks, mut drags, mut flicks), (time, kind)| {
                    match kind {
                        NoteKind::Click | NoteKind::Hold { .. } => clicks.push(time),
                        NoteKind::Drag => drags.push(time),
                        NoteKind::Flick => flicks.push(time),
                    }
                    (clicks, drags, flicks)
                },
            )
            .reduce(
                || (Vec::new(), Vec::new(), Vec::new()),
                |(mut c1, mut d1, mut f1), (c2, d2, f2)| {
                    c1.extend(c2);
                    d1.extend(d2);
                    f1.extend(f2);
                    (c1, d1, f1)
                },
            );

        // 顺序处理各组（内部可以并行化place函数）
        clicks.iter().for_each(|&time| {
            place(time, &sfx_click, volume_sfx, &mut output, sample_rate_f64);
        });

        drags.iter().for_each(|&time| {
            place(time, &sfx_drag, volume_sfx, &mut output, sample_rate_f64);
        });

        flicks.iter().for_each(|&time| {
            place(time, &sfx_flick, volume_sfx, &mut output, sample_rate_f64);
        });

        info!("sfx Time:{:?}", start_time.elapsed());
    }

    //ending
    let mut pos = O + length + A;
    while place(pos, &ending, volume_music, &mut output, sample_rate_f64) != 0 && params.config.ending_length > 0.1 {
        pos += ending.frame_count() as f64 / sample_rate_f64;
    }
    let audio_bit = params.config.audio_bit;
    let audio_format = params.config.audio_format.to_lowercase();

    // 验证输入
    let supported_formats = ["flac", "mp3", "aac", "opus", "wav"];
    if !supported_formats.contains(&audio_format.as_str()) {
        bail!(
            "Unsupported audio format: {}. Supported formats are: {}",
            audio_format,
            supported_formats.join(", ")
        );
    }

    if let Some(bit) = audio_bit {
        if ![16, 24, 32].contains(&bit) {
            bail!("Invalid audio bit depth: {}. Supported values are 16, 24, 32.", bit);
        }
        // PCM 格式强制使用 WAV 容器
        if audio_format != "wav" {
            return Err(anyhow::anyhow!("PCM audio bit depth requires WAV format, but {} was specified", audio_format));
        }
    }

    // 构建编码器和格式
    let (audio_codec, output_format) = if let Some(bit) = audio_bit {
        (format!("pcm_f{}le", bit), "wav".to_string())
    } else {
        match audio_format.as_str() {
            "flac" => ("flac".to_string(), "flac".to_string()),
            "mp3" => ("libmp3lame".to_string(), "mp3".to_string()),
            "aac" => ("aac".to_string(), "mp4".to_string()),
            "opus" => ("libopus".to_string(), "opus".to_string()),
            "wav" => ("pcm_f16le".to_string(), "wav".to_string()),
            // 修正默认值
            _ => {
                warn!("Unknown audio format '{}', using AAC/MP4 as default", audio_format);
                ("aac".to_string(), "mp4".to_string())
            }
        }
    };

    // 构建参数字符串
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
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| tl!("run-ffmpeg-failed"))?;
    let input = proc.stdin.as_mut().unwrap();
    let mut writer = BufWriter::new(input);
    for sample in output.into_iter() {
        writer.write_all(&sample.to_le_bytes())?;
    }
    drop(writer);
    proc.wait()?;

    //let (vw, vh) = params.config.resolution;

    let target_aspect = info.aspect_ratio as f64;
    let (mut vw, mut vh) = params.config.resolution;
    let (ow, oh) = (vw, vh);
    let current_aspect = vw as f64 / vh as f64;

    if (current_aspect - target_aspect).abs() > 1e-9 {
        if current_aspect > target_aspect {
            vw = (vh as f64 * target_aspect).round() as u32;
        } else {
            vh = (vw as f64 / target_aspect).round() as u32;
        }
        info!("{}x{} -> {}x{} (目标 {:.9})", ow, oh, vw, vh, target_aspect);
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
                if cnt == 1 || cnt == 3 {
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
    const A: f64 = 0.7 + 0.3 + 0.4 - 0.4;

    let fps = params.config.fps;
    //let frame_delta = 1. / fps as f32;
    let frames = (video_length * fps as f64).ceil() as u64;
    send(IPCEvent::StartRender(frames));
    /*
        let codecs = String::from_utf8(
            cmd_hidden(&ffmpeg)
                .arg("-codecs")
                .output()
                .with_context(|| tl!("run-ffmpeg-failed"))?
                .stdout,
        )?;

         let use_cuda = params.config.hardware_accel && test_encoder(ffmpeg.as_ref(), "h264_nvenc")?;
         let has_qsv = params.config.hardware_accel && test_encoder(ffmpeg.as_ref(), "h264_qsv")?;
         let has_amf = params.config.hardware_accel && test_encoder(ffmpeg.as_ref(), "h264_amf")?;

         let use_cuda_hevc = params.config.hardware_accel && params.config.hevc && test_encoder(ffmpeg.as_ref(), "hevc_nvenc")?;
         let has_qsv_hevc = params.config.hardware_accel && params.config.hevc && test_encoder(ffmpeg.as_ref(), "hevc_qsv")?;
         let has_amf_hevc = params.config.hardware_accel && params.config.hevc && test_encoder(ffmpeg.as_ref(), "hevc_amf")?;

        let ffmpeg_preset =  if !has_qsv && !use_cuda && has_amf {"-quality"} else {"-preset"};
        let mut ffmpeg_preset_name_list = params.config.ffmpeg_preset.split_whitespace();

        let (qsv, nvenc, _amf, cpu) = if params.config.hevc {
            ("hevc_qsv", "hevc_nvenc", "hevc_amf", "libx265")
        } else {
            ("h264_qsv", "h264_nvenc", "h264_amf", "libx264")
        };
        if params.config.hardware_accel && !has_qsv_hevc && !use_cuda_hevc && !has_amf_hevc {bail!(tl!("no-hwacc"));}

        let ffmpeg_preset_name = if has_qsv {ffmpeg_preset_name_list.nth(0)
        } else if use_cuda {ffmpeg_preset_name_list.nth(1)
        } else if has_amf {ffmpeg_preset_name_list.nth(2)
        } else {ffmpeg_preset_name_list.nth(0)};

        let mut args = "-y -f rawvideo -c:v rawvideo".to_owned();
        if use_cuda {
            args += " -hwaccel_output_format cuda";
        }
        write!(&mut args, " -s {vw}x{vh} -r {fps} -pix_fmt rgba -i - -i")?;

        let args2 = format!(
            "-c:a copy -c:v {} -pix_fmt yuv420p {} {} {} {} -map 0:v:0 -map 1:a:0 {} -vf vflip -f mov",
            if has_qsv {qsv}
            else if use_cuda {nvenc}
            //else if has_amf {amf}
            else if params.config.hardware_accel {bail!(tl!("no-hwacc"));}
            else {cpu},
            if params.config.bitrate_control == "CRF" {
                if has_qsv {"-q"}
                else if use_cuda {"-cq"}
                //else if has_amf {"-qp_p"}
                else {"-crf"}
            } else {
                "-b:v"
            },
            params.config.bitrate,
            ffmpeg_preset,
            ffmpeg_preset_name.unwrap(),
            if params.config.disable_loading{format!("-ss {}", LoadingScene::TOTAL_TIME + GameScene::BEFORE_TIME)}
            else{"-ss 0.1".to_string()},
        );
    */

    fn test_encoder(ffmpeg: &Path, encoder: &str) -> Result<(bool, String)> {
        let mut cmd = Command::new(ffmpeg);
        cmd.args(&[
            "-f", "lavfi",
            "-i", "color=c=black:s=320x240:d=0",
            "-c:v", encoder,
            "-f", "null", "-",
        ])
            .arg("-loglevel")
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

    };

    let mut hw_errors = Vec::new();

    // 创建编码器可用性结构
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
    };

    // 测试每个硬件编码器并收集错误
    let encoders_to_test = [
        ("h264_nvenc", hw_detected.h264_nvenc, &mut encoder_availability.h264_nvenc),
        ("hevc_nvenc", hw_detected.hevc_nvenc, &mut encoder_availability.hevc_nvenc),
        ("av1_nvenc", hw_detected.av1_nvenc, &mut encoder_availability.av1_nvenc),
        ("h264_qsv", hw_detected.h264_qsv, &mut encoder_availability.h264_qsv),
        ("hevc_qsv", hw_detected.hevc_qsv, &mut encoder_availability.hevc_qsv),
        ("av1_qsv", hw_detected.av1_qsv, &mut encoder_availability.av1_qsv),
        ("h264_amf", hw_detected.h264_amf, &mut encoder_availability.h264_amf),
        ("hevc_amf", hw_detected.hevc_amf, &mut encoder_availability.hevc_amf),
        ("av1_amf", hw_detected.av1_amf, &mut encoder_availability.av1_amf),
    ];

    for (name, detected, availability_flag) in encoders_to_test {
        if detected {
            match test_encoder(ffmpeg.as_ref(), name) {
                Ok((success, error_output)) => {
                    *availability_flag = success;

                    if !success {
                        hw_errors.push(format!(
                            "{} test failed:\n{}",
                            name,
                            error_output.trim()
                        ));
                    }
                }
                Err(e) => {
                    *availability_flag = false;
                    hw_errors.push(format!("{} test error: {}", name, e));
                }
            }
        }
    }

    // 测试 cuvid 解码器（单独处理）
    let cuvid_to_test = [
        ("h264_cuvid", hw_detected.h264_cuvid, &mut encoder_availability.h264_cuvid),
        ("hevc_cuvid", hw_detected.hevc_cuvid, &mut encoder_availability.hevc_cuvid),
        ("av1_cuvid", hw_detected.av1_cuvid, &mut encoder_availability.av1_cuvid),
    ];

    for (name, detected, availability_flag) in cuvid_to_test {
        if detected {
            // 1. 生成测试用的编码视频（内存中）
            let mut encode_cmd = Command::new(&ffmpeg);
            encode_cmd.args(&[
                "-f", "lavfi",
                "-i", "testsrc=duration=1:size=320x240:rate=30",
                "-c:v", if name == "h264_cuvid" { "libx264" } else { "libx265" },
                "-t", "0.5",  // 只编码0.5秒
                "-f", "mpegts",  // 使用TS容器
                "-"  // 输出到stdout
            ])
                .stdout(Stdio::piped())
                .stderr(Stdio::null());

            // 2. 解码测试
            let mut decode_cmd = Command::new(&ffmpeg);
            decode_cmd.args(&[
                "-hwaccel", "cuvid",
                "-hwaccel_device", "0",
                "-c:v", name,
                "-f", "mpegts",
                "-i", "-",  // 从stdin读取
                "-f", "null",
                "-"  // 输出到null
            ])
                .stdin(Stdio::piped())  // 接收编码后的视频
                .stdout(Stdio::null())
                .stderr(Stdio::piped());

            // 执行管道操作
            let encoded = match encode_cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    *availability_flag = false;
                    hw_errors.push(format!("{} test encode setup failed: {}", name, e));
                    continue;
                }
            };

            decode_cmd.stdin(encoded.stdout.unwrap());  // 连接管道

            match decode_cmd.output() {
                Ok(output) => {
                    *availability_flag = output.status.success();
                    if !output.status.success() {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        hw_errors.push(format!(
                            "{} decode test failed (code {}):\n{}",
                            name,
                            output.status.code().unwrap_or(-1),
                            stderr.trim()
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
    let mut dummy_flag = false;

    // 构建候选编码器列表
    let candidates = match params.config.video_codec.as_str() {
        "hevc" => vec![
            ("hevc_nvenc", encoder_availability.hevc_nvenc, &mut encoder_availability.hevc_nvenc),
            ("hevc_qsv", encoder_availability.hevc_qsv, &mut encoder_availability.hevc_qsv),
            ("hevc_amf", encoder_availability.hevc_amf, &mut encoder_availability.hevc_amf),
            ("libx265", true, &mut dummy_flag), // 需要定义一个 dummy_flag 变量
        ],
        "av1" => vec![
            ("av1_nvenc", encoder_availability.av1_nvenc, &mut encoder_availability.av1_nvenc),
            ("av1_qsv", encoder_availability.av1_qsv, &mut encoder_availability.av1_qsv),
            ("av1_amf", encoder_availability.av1_amf, &mut encoder_availability.av1_amf),
            ("libaom-av1", true, &mut dummy_flag), // 需要定义一个 dummy_flag 变量
        ],
        _ => vec![
            ("h264_nvenc", encoder_availability.h264_nvenc, &mut encoder_availability.h264_nvenc),
            ("h264_qsv", encoder_availability.h264_qsv, &mut encoder_availability.h264_qsv),
            ("h264_amf", encoder_availability.h264_amf, &mut encoder_availability.h264_amf),
            ("libx264", true, &mut dummy_flag), // 需要定义一个 dummy_flag 变量
        ],
    };


    let ffmpeg_encoder = candidates
        .iter()
        .find(|&&(_name, available, _)| available)
        .map(|&(name, _, _)| name)
        .expect("At least one software encoder is available.");

    println!("Selected encoder: {}", ffmpeg_encoder);

    let ffmpeg_preset = match ffmpeg_encoder {
        "h264_amf" | "hevc_amf" => "-quality",
        _ => "-preset",
    };

    let ffmpeg_preset_name = match ffmpeg_encoder {
        "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" => params
            .config
            .ffmpeg_preset
            .split_whitespace()
            .nth(1)
            .unwrap_or("p4"),
        "h264_qsv" | "hevc_qsv" | "av1_qsv" => params
            .config
            .ffmpeg_preset
            .split_whitespace()
            .next()
            .unwrap_or("medium"),
        "h264_amf" | "hevc_amf" | "av1_amf" => params
            .config
            .ffmpeg_preset
            .split_whitespace()
            .nth(2)
            .unwrap_or("balanced"),
        _ => params
            .config
            .ffmpeg_preset
            .split_whitespace()
            .next()
            .unwrap_or("medium"),
    };

    let bitrate_control = if params.config.bitrate_control == "CRF" {
        match ffmpeg_encoder {
            "h264_nvenc" | "hevc_nvenc" | "av1_nvenc" => "-cq",
            "h264_qsv" | "hevc_qsv" | "av1_qsv" => "-q",
            "h264_amf" | "hevc_amf" | "av1_amf" => "-qp_p",
            _ => "-crf",
        }
    } else {
        "-b:v"
    };

    if params.config.hardware_accel {
        let h264_supported = encoder_availability.h264_nvenc
            || encoder_availability.h264_qsv
            || encoder_availability.h264_amf;
        let hevc_supported = encoder_availability.hevc_nvenc
            || encoder_availability.hevc_qsv
            || encoder_availability.hevc_amf;
        let av1_supported = encoder_availability.av1_nvenc
            || encoder_availability.av1_qsv
            || encoder_availability.av1_amf;

        if (params.config.video_codec == "h264" && !h264_supported)
            || (params.config.video_codec == "hevc" && !hevc_supported)
            || (params.config.video_codec == "av1" && !av1_supported) {
            let mut detailed_error = String::new();
            detailed_error += &format!("{}\n", tl!("no-hwacc"));

            detailed_error += &format!(
                "Hardware detection summary:\n\
         - NVIDIA: {}\n\
         - Intel Quick Sync: {}\n\
         - AMD AMF: {}\n\n",
                hw_detected.h264_nvenc,
                hw_detected.h264_qsv,
                hw_detected.h264_amf
            );

            detailed_error += "Encoder test results:\n";
            detailed_error += &format!(
                "- h264_nvenc: {}\n",
                if encoder_availability.h264_nvenc { "SUCCESS" } else { "FAILED" }
            );
            detailed_error += &format!(
                "- hevc_nvenc: {}\n",
                if encoder_availability.hevc_nvenc { "SUCCESS" } else { "FAILED" }
            );
            detailed_error += &format!(
                "- av1_nvenc: {}\n",
                if encoder_availability.av1_nvenc { "SUCCESS" } else { "FAILED" }
            );
            detailed_error += &format!(
                "- h264_qsv: {}\n",
                if encoder_availability.h264_qsv { "SUCCESS" } else { "FAILED" }
            );
            detailed_error += &format!(
                "- hevc_qsv: {}\n",
                if encoder_availability.hevc_qsv { "SUCCESS" } else { "FAILED" }
            );
            detailed_error += &format!(
                "av1_qsv: {}\n",
                if encoder_availability.av1_qsv { "SUCCESS" } else { "FAILED" }
            );
            detailed_error += &format!(
                "- h264_amf: {}\n",
                if encoder_availability.h264_amf { "SUCCESS" } else { "FAILED" }
            );
            detailed_error += &format!(
                "- hevc_amf: {}\n",
                if encoder_availability.hevc_amf { "SUCCESS" } else { "FAILED" }
            );
            detailed_error += &format!(
                "- av1_amf: {}\n",
                if encoder_availability.av1_amf { "SUCCESS" } else { "FAILED" }
            );
            detailed_error += &format!(
                "- h264_cuvid: {}\n",
                if encoder_availability.h264_cuvid { "SUCCESS" } else { "FAILED" }
            );
            detailed_error += &format!(
                "- hevc_cuvid: {}\n\n",
                if encoder_availability.hevc_cuvid { "SUCCESS" } else { "FAILED" }
            );
            detailed_error += &format!(
                "av1_cuvid: {}\n\n",
                if encoder_availability.av1_cuvid { "SUCCESS" } else { "FAILED" }
            );

            // 详细的错误日志
            if !hw_errors.is_empty() {
                detailed_error += "Detailed error logs:\n";
                for (i, error) in hw_errors.iter().enumerate() {
                    detailed_error += &format!("{}. {}\n", i + 1, error);
                }
                detailed_error += "\n";
            } else {
                detailed_error += "No hardware encoders were tested (all detection failed).\n\n";
            }

            bail!(detailed_error);
        }
    }

    // 构建FFmpeg命令参数
    let mut args = String::new();

    // 添加硬件加速选项（如果启用）
    if params.config.hardware_accel {
        // 优先使用 cuvid 进行硬件加速解码
        if params.config.video_codec == "hevc" && encoder_availability.hevc_cuvid {
            args.push_str("-hwaccel cuvid -c:v hevc_cuvid ");
        } else if params.config.video_codec != "hevc" && encoder_availability.h264_cuvid {
            args.push_str("-hwaccel cuvid -c:v h264_cuvid ");
        }
        // 设置硬件加速输出格式
        else if ffmpeg_encoder.contains("nvenc") {
            args.push_str("-hwaccel_output_format cuda ");
        } else if ffmpeg_encoder.contains("qsv") {
            args.push_str("-hwaccel qsv ");
        }
    }

    args.push_str("-y -f rawvideo -c:v rawvideo");
    if ffmpeg_encoder.contains("nvenc") && !args.contains("cuda") {
        args.push_str(" -hwaccel_output_format cuda");
    }

    write!(&mut args, " -s {vw}x{vh} -r {fps} -pix_fmt rgba -i - -i")?;

    let ffmpeg_thread = if params.config.ffmpeg_thread {
        "-thread_queue_size 2048 "
    } else {
        ""
    };

    let video = match params.config.video {
        true => "mov",
        false => "mp4",
    };

    let strict_flag = if params.config.audio_format == "flac" && video == "mp4" {
        "-strict -2 "
    } else {
        ""
    };

    let args2 = format!(
        "-c:a {} -c:v {} -pix_fmt yuv420p {} {} {} {} -map 0:v:0 -map 1:a:0 {} {} {} -vf vflip -f {}",
        audio_codec,
        ffmpeg_encoder,
        bitrate_control,
        params.config.bitrate,
        ffmpeg_preset,
        ffmpeg_preset_name,
        strict_flag,
        ffmpeg_thread,
        if params.config.disable_loading {
            format!("-ss {}", LoadingScene::TOTAL_TIME + GameScene::BEFORE_TIME)
        } else {
            "-ss 0.1".to_string()
        },
        video,
    );

    let mut proc = cmd_hidden(&ffmpeg)
        .args(args.split_whitespace())
        .arg(mixing_output.path())
        .args(args2.split_whitespace())
        .arg(output_path)
        .arg("-loglevel")
        .arg("warning")
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| tl!("run-ffmpeg-failed"))?;
    let mut input = proc.stdin.take().unwrap();

    let byte_size = vw as usize * vh as usize * 4;

    //const N: usize = 1;
    //let mut pbos: [GLuint; N] = [0; N];

    const MAX_PBO_COUNT: usize = 10;
    let mut n = MAX_PBO_COUNT;
    while n > 0 && fps as usize % n != 0 {
        n -= 1;
    }
    let n = if n == 0 { 1 } else { n };
    let mut pbos: Vec<GLuint> = vec![0; n];
    info!("Using {} PBOs for rendering", n);

    unsafe {
        use miniquad::gl::*;
        glGenBuffers(n as _, pbos.as_mut_ptr());
        for pbo in &pbos {
            glBindBuffer(GL_PIXEL_PACK_BUFFER, *pbo);
            glBufferData(
                GL_PIXEL_PACK_BUFFER,
                (vw as u64 * vh as u64 * 4) as _,
                std::ptr::null(),
                GL_STREAM_READ,
            );
        }
        glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
    }

    send(IPCEvent::StartRender(frames));

    let fps_f64 = fps as f64;

    let frames10 = frames / 10;
    let mut step_time = Instant::now();

    //TODO: The first frame + 1 function test_encoder
    for frame in 0..frames {
        if frame % frames10 == 0 || frame == frames - 1 {
            let progress = (frame as f64 / frames as f64).min(1.0);
            let percent = (progress * 100.).ceil() as i8;
            let bar_width = 20;
            let filled = (progress * bar_width as f64).round() as usize;
            let empty = bar_width - filled;

            let time_text = if frame == frames - 1 {
                "Final frame".to_string()
            } else {
                format!("{:.2}s", step_time.elapsed().as_secs_f32())
            };

            info!(
            "Rendering: [{}{}] {:>3}% | Time: {} | Frames: {}/{}",
            "█".repeat(filled),
            " ".repeat(empty),
            percent,
            time_text,
            frame + 1,
            frames
        );
            step_time = Instant::now();
        }
        let pbo_index = (frame as usize) % n;

        *my_time.borrow_mut() = (frame as f64 / fps_f64).max(0.);
        gl.quad_gl.render_pass(Some(mst.output().render_pass));
        main.update()?;
        main.render(&mut painter)?;

        if *my_time.borrow() <= LoadingScene::TOTAL_TIME as f64 && !params.config.disable_loading {
            draw_rectangle(0., 0., 0., 0., Color::default());
        }

        gl.flush();

        if MSAA.load(Ordering::SeqCst) {
            mst.blit();
        }

        unsafe {
            use miniquad::gl::*;
            glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(&mst.output()));
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[pbo_index]);  // 使用动态索引
            glReadPixels(
                0, 0,
                vw as _, vh as _,
                GL_RGBA, GL_UNSIGNED_BYTE,
                std::ptr::null_mut()
            );

            let src = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8);
            if !src.is_null() {
                input.write_all(std::slice::from_raw_parts(
                    src as *const u8,
                    byte_size
                ))?;
                glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
            }
            glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
        }

        send(IPCEvent::Frame);
    }
    /*
    wft bro??
    input.flush()?;
    drop(input);

     */

    drop(input);
    proc.wait()?;
    info!("Render Time: {:.2?}", render_start_time.elapsed());
    info!(
        "Average FPS: {:.2}",
        frames as f64 / render_start_time.elapsed().as_secs_f64()
    );
    proc.wait()?;
    unsafe {
        use miniquad::gl::*;
        glDeleteBuffers(n as _, pbos.as_ptr());
    }

    send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    Ok(())
}
