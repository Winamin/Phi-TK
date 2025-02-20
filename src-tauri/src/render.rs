// Prevents additional console window on Windows in release, DO NOT REMOVE!!
prpr::tl_file!("render");

use anyhow::{bail, Context, Result};
use macroquad::{miniquad::gl::GLuint, prelude::*};
use prpr::{
    config::{ChallengeModeColor, Config, Mods},
    core::{internal_id, MSRenderTarget, NoteKind},
    fs,
    info::ChartInfo,
    scene::{BasicPlayer, GameMode, GameScene, LoadingScene, EndingScene},
    time::TimeManager,
    ui::{FontArc, TextPainter},
    Main,
};
use sasa::AudioClip;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    io::{BufRead, BufWriter, Write, self},
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
    bitrate_control: String,
    bitrate: String,

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

pub const GL_MAP_READ_BIT: u32 = 0x0001;
     pub const GL_MAP_UNSYNCHRONIZED_BIT: u32 = 0x0020;
     pub const GL_SYNC_GPU_COMMANDS_COMPLETE: u32 = 0x9117;
     pub const GL_TIMEOUT_IGNORED: u64 = 0xFFFFFFFFFFFFFFFF;
     pub const GL_BGRA: u32 = 0x80E1;
     pub const GL_PACK_ALIGNMENT: u32 = 0x0D05;
     pub const GL_READ_ONLY: u32 = 0x88B8;

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
    if let Some(path_var) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_var) {
            let candidate = dir.join(ffmpeg_exe);
            if test(&candidate) {
                return Ok(Some(candidate.to_string_lossy().into_owned()));
            }
        }
    }
    eprintln!("Failed to find global ffmpeg. Using bundled ffmpeg");
    let exe_dir = std::env::current_exe()?
        .parent()
        .expect("Executable should have parent directory")
        .to_owned();
    
    let bundled_ffmpeg = exe_dir.join(ffmpeg_exe);
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
        info!("music Time:{:?}", start_time.elapsed())

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
            slice[i * 2] += frames[i].0 * volume;
            slice[i * 2 + 1] += frames[i].1 * volume;
        }
    
        return len;
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
        info!("sfx Time:{:?}", start_time.elapsed())
        
    }

    //ending
    let mut pos = O + length + A;
    while place(pos, &ending, volume_music) != 0 && params.config.ending_length > 0.1 {
        pos += ending.frame_count() as f64 / sample_rate_f64;
    }

    let mut proc = cmd_hidden(&ffmpeg)
        .args(format!("-y -f f32le -ar {} -ac 2 -i - -c:a pcm_f32le -f wav", sample_rate).split_whitespace())
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
    let frame_delta = 1. / fps as f32;
    let frames = (video_length / frame_delta as f64).ceil() as u64;
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

    let encoder_availability = EncoderAvailability {
        h264_nvenc: params.config.hardware_accel && test_encoder(ffmpeg.as_ref(), "h264_nvenc")?,
        hevc_nvenc: params.config.hardware_accel && params.config.hevc && test_encoder(ffmpeg.as_ref(), "hevc_nvenc")?,
        h264_qsv: params.config.hardware_accel && test_encoder(ffmpeg.as_ref(), "h264_qsv")?,
        hevc_qsv: params.config.hardware_accel && params.config.hevc && test_encoder(ffmpeg.as_ref(), "hevc_qsv")?,
        h264_amf: params.config.hardware_accel && test_encoder(ffmpeg.as_ref(), "h264_amf")?,
        hevc_amf: params.config.hardware_accel && params.config.hevc && test_encoder(ffmpeg.as_ref(), "hevc_amf")?,
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
        .find(|&&(name, available)| available)
        .map(|&(name, _)| name)
        .expect("At least one software encoder is available.");
        
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

    const N: usize = 4;
    let mut pbos: [GLuint; N] = [0; N];

    type GlBufferStorage = unsafe extern "system" fn(target: u32, size: isize, data: *const std::ffi::c_void, flags: u32);
    type GlFenceSync = unsafe extern "system" fn(condition: u32, flags: u32) -> *mut std::ffi::c_void;
    type GlWaitSync = unsafe extern "system" fn(sync: *mut std::ffi::c_void, flags: u32, timeout: u64);
    type GlDeleteSync = unsafe extern "system" fn(sync: *mut std::ffi::c_void);
/*
    type GlMapBufferRange = unsafe extern "system" fn(target: u32, offset: isize, length: isize, access: u32) -> *mut std::ffi::c_void;
*/
    unsafe {
        use miniquad::gl::*;
        glGenBuffers(N as _, pbos.as_mut_ptr());
        for pbo in &pbos {
            glBindBuffer(GL_PIXEL_PACK_BUFFER, *pbo);
            glBufferStorage(
                GL_PIXEL_PACK_BUFFER,
                byte_size as _,
                std::ptr::null(),
                GL_MAP_READ_BIT
            );
        }
        glBindBuffer(GL_PIXEL_PACK_BUFFER, 0);
    
        glPixelStorei(GL_PACK_ALIGNMENT, 1);
    }
 
    let mut pending_frames = 0;
    for frame in 0..frames {
        *my_time.borrow_mut() = (frame as f32 * frame_delta).max(0.) as f64;
        gl.quad_gl.render_pass(Some(mst.output().render_pass));
        main.viewport = Some((0, 0, vw as _, vh as _));
        main.update()?;
        main.render(&mut painter)?;
        gl.flush();

        if MSAA.load(Ordering::SeqCst) {
            mst.blit();
        }

        unsafe {
            use miniquad::gl::*;
            let gl = gl::GL::get()
            gl GlBufferStorage(...)
            gl GlFenceSync(...)
            gl GlWaitSync(...)
            gl GlDeleteSync(...)
            let tex = mst.output().texture.raw_miniquad_texture_handle();
            glBindFramebuffer(GL_READ_FRAMEBUFFER, internal_id(mst.output()));

            let read_pbo = pbos[frame as usize % N];
            let write_pbo = pbos[(frame + 1) as usize % N];
        
            glBindBuffer(GL_PIXEL_PACK_BUFFER, write_pbo);
            glReadPixels(
                0,
                0,
                tex.width as _,
                tex.height as _,
                GL_RGBA,
                GL_UNSIGNED_BYTE,
                std::ptr::null_mut(),
            );

            let sync = glFenceSync(GL_SYNC_GPU_COMMANDS_COMPLETE, 0);
        
            if pending_frames >= N-1 {
                glBindBuffer(GL_PIXEL_PACK_BUFFER, read_pbo);
            
                glWaitSync(sync, 0, GL_TIMEOUT_IGNORED);
                glDeleteSync(sync);
            
            let src = glMapBuffer(GL_PIXEL_PACK_BUFFER, GL_READ_ONLY);
                if !src.is_null() {
                    input.write_all(std::slice::from_raw_parts(src as   *const u8, byte_size))?;
                    glUnmapBuffer(GL_PIXEL_PACK_BUFFER);
                }
            }
           pending_frames += 1;
        }
        send(IPCEvent::Frame);
    }

    drop(input);
    proc.wait()?;

    send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    Ok(())
}
