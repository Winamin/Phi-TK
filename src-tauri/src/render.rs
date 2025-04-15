// Prevents additional console window on Windows in release, DO NOT REMOVE!!
prpr::tl_file!("render");

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
use crate::Path;
use macroquad::texture::Texture2D; 

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RenderConfig {
    resolution: (u32, u32),
    ffmpeg_preset: String,
    ending_length: f64,
    disable_loading: bool,
    chart_debug: bool,
    flid_x: bool,
    chart_ratio: f32,
    buffer_size: f32,
    combo: String,
    fps: u32,
    hardware_accel: bool,
    hevc: bool,
    show_progress_text: bool,
    show_time_text : bool,
    target_audio: u32,
    autoplay: Option<bool>,
    bitrate_control: String,
    bitrate: String,
    watermark: String,

    aggressive: bool,
    challenge_color: ChallengeModeColor,
    challenge_rank: u32,
    disable_effect: bool,
    double_hint: bool,
    fxaa: bool,
    note_scale: f32,
    particle: bool,
    player_avatar: Option<String>,
    player_name: String,
    player_rks: f32,
    sample_count: u32,
    res_pack_path: Option<String>,
    speed: f32,
    volume_music: f32,
    volume_sfx: f32,
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
}

#[cfg(target_os = "windows")]
mod hw_detect {
    use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};
    use std::path::Path;

    pub fn detect_nvidia() -> bool {
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(r"SOFTWARE\NVIDIA Corporation\Global\NvControlPanel")
            .is_ok()
    }

    pub fn detect_intel_qsv() -> bool {
        let mut found = false;
        let classes = [
            "{4d36e968-e325-11ce-bfc1-08002be10318}", // Display adapters
            "{4d36e97d-e325-11ce-bfc1-08002be10318}"  // System devices
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
        Path::new(r"C:\Windows\System32\amdvlk64.dll").exists() ||
            Path::new(r"C:\Windows\System32\amfrt64.dll").exists()
    }
}

#[cfg(target_os = "linux")]
mod hw_detect {
    use std::path::Path;
    use std::process::Command;

    pub fn detect_nvidia() -> bool {
        Path::new("/dev/nvidia0").exists() ||
            Command::new("nvidia-smi").status().is_ok()
    }

    pub fn detect_intel_qsv() -> bool {
        Path::new("/dev/dri/renderD128").exists() &&
            Command::new("vainfo")
                .output()
                .map(|out| String::from_utf8_lossy(&out.stdout).contains("VAProfileH264"))
                .unwrap_or(false)
    }

    pub fn detect_amd() -> bool {
        Path::new("/dev/kfd").exists() &&
            Command::new("vainfo")
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
                SafeTexture::from(Texture2D::from_file_with_format(
                    &tokio::fs::read(path)
                        .await
                        .with_context(|| tl!("load-avatar-failed"))?,
                    None,
                ))
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
    let target_sample_rate = params.config.target_audio as u32;
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
        for i in 0..count {
            let position = i as f64 * ratio;
            let frame = music.sample(position as f32).unwrap_or_default();
            output[start_index + i * 2] += frame.0 * volume_music;
            output[start_index + i * 2 + 1] += frame.1 * volume_music;
        }
        info!("music Time:{:?}", start_time.elapsed());
    }

    let mut place = |pos: f64, clip: &AudioClip, volume: f32| {
        let position = (pos * sample_rate_f64).round() as usize * 2;
        if position >= output.len() {
            return 0;
        }
        let slice = &mut output[position..];
        let len = (slice.len() / 2).min(clip.frame_count());

        let frames = clip.frames();
        for i in 0..len {
            let sample = frames[i].0 * volume;
            slice[i * 2] += sample;
            slice[i * 2 + 1] += sample;
        }

        len
    };

    if volume_sfx != 0.0 {
        let start_time = Instant::now();
        for line in &chart.lines {
            for note in &line.notes {
                if !note.fake {
                    let sfx = match note.kind {
                        NoteKind::Click | NoteKind::Hold { .. } => &sfx_click,
                        NoteKind::Drag => &sfx_drag,
                        NoteKind::Flick => &sfx_flick,
                    };
                    place(O + note.time as f64 + offset as f64, sfx, volume_sfx);
                }
            }
        }
        info!("sfx Time:{:?}", start_time.elapsed());
    }

    //ending
    let mut pos = O + length + A;
    while place(pos, &ending, volume_music) != 0 && params.config.ending_length > 0.1 {
        pos += ending.frame_count() as f64 / sample_rate_f64;
    }
    let args_str = if target_sample_rate != sample_rate {
        let resample_filter = format!("aresample=resampler=soxr:osr={}", target_sample_rate);
        format!(
            "-y -f f32le -ar {} -ac 2 -i - -af {} -c:a pcm_f32le -f wav",
            sample_rate, resample_filter
        )
    } else {
        format!(
            "-y -f f32le -ar {} -ac 2 -i - -c:a pcm_f32le -f wav",
            sample_rate
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

    let (vw, vh) = params.config.resolution;
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
    let frames = (video_length * fps as f64).round() as u64;
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

    fn test_encoder(ffmpeg: &Path, encoder: &str) -> Result<bool> {
        let output = cmd_hidden(&ffmpeg)
            .args(&["-f", "lavfi", "-i", "color=c=black:s=320x240:d=0", "-c:v", encoder, "-f", "null", "-"])
            .arg("-loglevel")
            .arg("fatal")
            .arg("-hide_banner")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .with_context(|| format!("Failed to test encoder {}", encoder))?;
        Ok(output.status.success())
    }

    let hw_detected = EncoderAvailability {
        h264_nvenc: params.config.hardware_accel && hw_detect::detect_nvidia(),
        hevc_nvenc: params.config.hardware_accel && params.config.hevc && hw_detect::detect_nvidia(),
        h264_qsv: params.config.hardware_accel && hw_detect::detect_intel_qsv(),
        hevc_qsv: params.config.hardware_accel && params.config.hevc && hw_detect::detect_intel_qsv(),
        h264_amf: params.config.hardware_accel && hw_detect::detect_amd(),
        hevc_amf: params.config.hardware_accel && params.config.hevc && hw_detect::detect_amd(),
    };

    let encoder_availability = EncoderAvailability {
        h264_nvenc: hw_detected.h264_nvenc && test_encoder(ffmpeg.as_ref(), "h264_nvenc")?,
        hevc_nvenc: hw_detected.hevc_nvenc && test_encoder(ffmpeg.as_ref(), "hevc_nvenc")?,
        h264_qsv: hw_detected.h264_qsv && test_encoder(ffmpeg.as_ref(), "h264_qsv")?,
        hevc_qsv: hw_detected.hevc_qsv && test_encoder(ffmpeg.as_ref(), "hevc_qsv")?,
        h264_amf: hw_detected.h264_amf && test_encoder(ffmpeg.as_ref(), "h264_amf")?,
        hevc_amf: hw_detected.hevc_amf && test_encoder(ffmpeg.as_ref(), "hevc_amf")?,
    };

    let candidates = if params.config.hevc {
        vec![
            ("hevc_nvenc", encoder_availability.hevc_nvenc),
            ("hevc_qsv", encoder_availability.hevc_qsv),
            ("hevc_amf", encoder_availability.hevc_amf),
            ("libx265", true),
        ]
    } else {
        vec![
            ("h264_nvenc", encoder_availability.h264_nvenc),
            ("h264_qsv", encoder_availability.h264_qsv),
            ("h264_amf", encoder_availability.h264_amf),
            ("libx264", true),
        ]
    };

    let ffmpeg_encoder = candidates.iter()
        .find(|&&(_name, available)| available)
        .map(|&(name, _)| name)
        .expect("At least one software encoder is available.");

    println!("Selected encoder: {}", ffmpeg_encoder);

    let ffmpeg_preset = match ffmpeg_encoder {
        "h264_amf" | "hevc_amf" => "-quality",
        _ => "-preset",
    };

    let ffmpeg_preset_name = match ffmpeg_encoder {
        "h264_nvenc" | "hevc_nvenc" => params.config.ffmpeg_preset.split_whitespace().nth(1).unwrap_or("p4"),
        "h264_qsv" | "hevc_qsv" => params.config.ffmpeg_preset.split_whitespace().next().unwrap_or("medium"),
        "h264_amf" | "hevc_amf" => params.config.ffmpeg_preset.split_whitespace().nth(2).unwrap_or("balanced"),
        _ => params.config.ffmpeg_preset.split_whitespace().next().unwrap_or("medium"),
    };

    let bitrate_control = if params.config.bitrate_control == "CRF" {
        match ffmpeg_encoder {
            "h264_nvenc" | "hevc_nvenc" => "-cq",
            "h264_qsv" | "hevc_qsv" => "-q",
            "h264_amf" | "hevc_amf" => "-qp_p",
            _ => "-crf",
        }
    } else {
        "-b:v"
    };

    if params.config.hardware_accel {
        let h264_supported = encoder_availability.h264_nvenc || encoder_availability.h264_qsv || encoder_availability.h264_amf;
        let hevc_supported = encoder_availability.hevc_nvenc || encoder_availability.hevc_qsv || encoder_availability.hevc_amf;

        if params.config.hevc && !hevc_supported {
            bail!(tl!("no-hwacc"));
        } else if !params.config.hevc && !h264_supported {
            bail!(tl!("no-hwacc"));
        }
    }

    let mut args = "-y -f rawvideo -c:v rawvideo".to_owned();
    if ffmpeg_encoder.contains("nvenc") {
        args += " -hwaccel_output_format cuda";
    }
    write!(&mut args, " -s {vw}x{vh} -r {fps} -pix_fmt rgba -i - -i")?;

    let args2 = format!(
        "-c:a copy -c:v {} -pix_fmt yuv420p {} {} {} {} -map 0:v:0 -map 1:a:0 {} -vf vflip -f mov",
        ffmpeg_encoder,
        bitrate_control,
        params.config.bitrate,
        ffmpeg_preset,
        ffmpeg_preset_name,
        if params.config.disable_loading {
            format!("-ss {}", LoadingScene::TOTAL_TIME + GameScene::BEFORE_TIME)
        } else {
            "-ss 0.1".to_string()
        },
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

    const N: usize = 180;
    let mut pbos: [GLuint; N] = [0; N];
    unsafe {
        use miniquad::gl::*;
        glGenBuffers(N as _, pbos.as_mut_ptr());
        for pbo in pbos {
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbo);
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

    let fps = fps as f64;
    for frame in 0..N {
        *my_time.borrow_mut() = (frame as f64 / fps).max(0.);
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
            //let tex = mst.output().texture.raw_miniquad_texture_handle();
            glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(mst.output()));
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[frame]);
            glReadPixels(
                0,
                0,
                vw as _,
                vh as _,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                std::ptr::null_mut(),
            );
        }
        send(IPCEvent::Frame);
    }
    let frames10 = frames / 10;
    let mut step_time = Instant::now();
    for frame in N as u64..frames {
        if frame % frames10 == 0 {
            let progress = frame as f32 / frames as f32;
            let percent = (progress * 100.).ceil() as i8;
            let bar_width = 20;
            let filled = (progress * bar_width as f32).round() as usize;
            let empty = bar_width - filled;

            info!(
                "Rendering: [{}{}] {:>3}% | Time: {:.2}s | Frames: {}/{}",
                "█".repeat(filled),
                " ".repeat(empty),
                percent,
                step_time.elapsed().as_secs_f32(),
                frame,
                frames
                );
            step_time = Instant::now();
        }
        *my_time.borrow_mut() = (frame as f64 / fps).max(0.);
        gl.quad_gl.render_pass(Some(mst.output().render_pass));
        //clear_background(BLACK);
        main.viewport = Some((0, 0, vw as _, vh as _));
        main.update()?;
        main.render(&mut painter)?;
        // TODO magic. can't remove this line.
        if *my_time.borrow() <= LoadingScene::TOTAL_TIME as f64 && !params.config.disable_loading {
            draw_rectangle(0., 0., 0., 0., Color::default());
        }

        gl.flush();

        if MSAA.load(Ordering::SeqCst) {
            mst.blit();
        }
        unsafe {
            use miniquad::gl::*;
            //let tex = mst.output().texture.raw_miniquad_texture_handle();
            glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(mst.output()));

            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[frame as usize % N]);
            glReadPixels(
                0,
                0,
                vw as _,
                vh as _,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                std::ptr::null_mut(),
            );

            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[(frame + 1) as usize % N]);
            let src = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8);
            if !src.is_null() {
                input.write_all(&std::slice::from_raw_parts(src as *const u8, byte_size))?;
                glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
            }
        }
        send(IPCEvent::Frame);
    }

    for i in 1..N {
        unsafe {
            use miniquad::gl::*;
            glBindBuffer(GL_PIXEL_PACK_BUFFER, pbos[(frames as usize + i) % N]);
            let src = glMapBuffer(GL_PIXEL_PACK_BUFFER, 0x88B8);
            if !src.is_null() {
                input.write_all(&std::slice::from_raw_parts(src as *const u8, byte_size))?;
                glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
            }
        }
    }
    input.flush()?;
    drop(input);
    info!("Render Time: {:.2?}", render_start_time.elapsed());
    info!(
        "Average FPS: {:.2}",
        frames as f64 / render_start_time.elapsed().as_secs_f64()
    );
    proc.wait()?;

    send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    Ok(())
}