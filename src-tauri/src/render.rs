// Prevents additional console window on Windows in release, DO NOT REMOVE!!
prpr::tl_file!("render");

use crate::common::{ensure_dir, let_output_dir, output_dir, DATA_DIR};
use chrono::Local;
use anyhow::{bail, Context, Result};
use macroquad::{miniquad::gl::GLuint, prelude::*};
use prpr::{
    config::{ChallengeModeColor, Config, Mods},
    core::{init_assets, internal_id, MSRenderTarget, Note},
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
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
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

fn cmd_hidden(program: impl AsRef<OsStr>) -> Command {
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
    if test("ffmpeg") {
        return Ok(Some("ffmpeg".to_owned()));
    }
    eprintln!("Failed to find global ffmpeg. Using bundled ffmpeg");
    let exe_dir = std::env::current_exe()?.parent().unwrap().to_owned();
    let ffmpeg = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };
    let ffmpeg = exe_dir.join(ffmpeg);
    Ok(if test(&ffmpeg) {
        Some(ffmpeg.display().to_string())
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

    let O: f64 = if params.config.disable_loading {
        GameScene::BEFORE_TIME as f64
    } else {
        LoadingScene::TOTAL_TIME as f64 + GameScene::BEFORE_TIME as f64
    };
    let A: f64 = -0.5; // fade out time
    let musica: f64 = 0.7 + 0.3 + EndingScene::BPM_WAIT_TIME;

    let offset = chart.offset + info.offset;
    let length = track_length - offset.min(0.) as f64 + 1.;
    let video_length = O + length + A + params.config.ending_length;
    
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
    let mut output2 = vec![0.0_f32; (video_length * sample_rate_f64).ceil() as usize];

    let mut place = |pos: f64, clip: &AudioClip, volume: f32| {
        let position = (pos * sample_rate_f64).round() as usize;
        if position >= output2.len() {
            return 0;
        }
        let slice = &mut output2[position..];
        let len = (slice.len()).min(clip.frame_count());

        let frames = clip.frames();
        for i in 0..len {
            slice[i] += frames[i].0 * volume;
        }

        return len;
    };

    if volume_music != 0.0 {
        let music_time = Instant::now();
        let pos = O - chart.offset.min(0.) as f64;
        let len = ((music.length() as f64 + 1. + A + params.config.ending_length) * sample_rate_f64) as usize;
        let start_index = (pos * sample_rate_f64).round() as usize * 2;
        let ratio = 1.0 / sample_rate_f64;
        for i in 0..len {
            let position = i as f64 * ratio;
            let frame = music.sample(position as f32).unwrap_or_default();
            output[start_index + i * 2] += frame.0 * volume_music;
            output[start_index + i * 2 + 1] += frame.1 * volume_music;
        }
        //ending
        let mut pos = O + length;
        while pos < video_length && params.config.ending_length > EndingScene::BPM_WAIT_TIME {
            let start_index = (pos * sample_rate_f64).round() as usize * 2;
            let slice = &mut output[start_index..];
            let len = (slice.len() / 2).min(ending.frame_count());
            let frames = &ending.frames();
            for i in 0..len {
                slice[i * 2] += frames[i].0 * volume_music;
                slice[i * 2 + 1] += frames[i].1 * volume_music;
            }
            pos += ending.frame_count() as f64 / sample_rate_f64;
        }
    }
    if volume_sfx != 0.0 {
        let sfx_time = Instant::now();
        let offset = offset as f64;
        for line in &chart.lines {
            for note in &line.notes {
                if !note.fake {
                    let sfx = match note.kind {
                        NoteKind::Click | NoteKind::Hold { .. } => &sfx_click,
                        NoteKind::Drag => &sfx_drag,
                        NoteKind::Flick => &sfx_flick,
                    };
                    place(O + note.time as f64 + offset, sfx, volume_sfx);
                }
            }
        }
        info!("Render Hit Effects Time:{:?}", sfx_time.elapsed())
    }
    for i in 0..output2.len() {
            output[i * 2] += output2[i];
            output[i * 2 + 1] += output2[i];
    }
    let mut proc = cmd_hidden(&ffmpeg)
        .args(format!("-y -f f32le -ar {} -ac 2 -i - -c:a pcm_f32le -f wav", sample_rate).split_whitespace())
        .arg(mixing_output.path())
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

    let preparing_render_time = Instant::now();
    let (vw, vh) = config.resolution;
    let mst = Rc::new(MSRenderTarget::new((vw, vh), config.sample_count));
    let my_time: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.));
    let tm = TimeManager::manual(Box::new({
        let my_time = Rc::clone(&my_time);
        move || *(*my_time).borrow()
    }));
    static MSAA: AtomicBool = AtomicBool::new(false);
    let player = build_player(&config).await?;
    let mut main = Main::new(
        Box::new(
            LoadingScene::new(
                GameMode::Normal,
                info,
                &prpr_config,
                fs,
                Some(player),
                None,
                None,
            )
            .await?,
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

    let fps = config.fps;
    let frames = (video_length * fps as f64 + N as f64 - 1.).ceil() as u64;

    let test_encoder = |encoder: &str| -> bool {
        let output = Command::new(&ffmpeg)
            .args(&["-f", "lavfi", "-i", "color=c=black:s=320x240:d=0", "-c:v", encoder, "-f", "null", "-"])
            .arg("-loglevel")
            .arg("fatal")
            .arg("-hide_banner")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to test encoder");
    
        output.status.success()
    };

    let use_cuda = config.hardware_accel && test_encoder("h264_nvenc");
    let has_qsv = config.hardware_accel && test_encoder("h264_qsv");
    let has_amf = config.hardware_accel && test_encoder("h264_amf");

    let use_cuda_hevc = config.hardware_accel && config.hevc && test_encoder("hevc_nvenc");
    let has_qsv_hevc = config.hardware_accel && config.hevc && test_encoder("hevc_qsv");
    let has_amf_hevc = config.hardware_accel && config.hevc && test_encoder("hevc_amf");

    let ffmpeg_preset = "-preset";
    let mut ffmpeg_preset_name_list = config.ffmpeg_preset.split_whitespace();

    if config.hardware_accel && !config.mpeg4 {
        if !(use_cuda_hevc || has_qsv_hevc || has_amf_hevc) && config.hevc {
            bail!(tl!("no-hwacc"));
        } else if !(use_cuda || has_qsv || has_amf) {
            bail!(tl!("no-hwacc"));
        }
    }

    let ffmpeg_encoder = if config.mpeg4 {
        "mpeg4"
    } else if use_cuda_hevc {
        "hevc_nvenc"
    } else if use_cuda {
        "h264_nvenc"
    } else if has_qsv_hevc {
        "hevc_qsv"
    } else if has_qsv {
        "h264_qsv"
    } else if has_amf_hevc {
        "hevc_amf"
    } else if has_amf {
        "h264_amf"
    } else {
        if config.hevc {
            "libx265"
        } else {
            "libx264"
        }
    };

    info!("Encoder: {}", ffmpeg_encoder);

    let ffmpeg_preset_name = if use_cuda {
        ffmpeg_preset_name_list.nth(1).unwrap_or(
            ffmpeg_preset_name_list.nth(0).unwrap_or("p4")
        )
    } else if has_qsv {
        ffmpeg_preset_name_list.nth(2).unwrap_or("medium")
    } else if has_amf {
        ffmpeg_preset_name_list.nth(3).unwrap_or(
            ffmpeg_preset_name_list.nth(0).unwrap_or("balanced")
        )
    } else {
        ffmpeg_preset_name_list.nth(0).unwrap_or("medium")
    };

    let bitrate_control = 
    if config.bitrate_control.to_lowercase() == "crf" {
        if use_cuda && !config.mpeg4 {
            "-cq"
        } else if has_qsv || config.mpeg4 {
            "-q"
        }
        else if has_amf {
            "-qp_p"
        } else {
            "-crf"
        }
    } else {
        "-b:v"
    };

    let bitrate = config.bitrate;

    let mut args = "-probesize 50M -y -f rawvideo -c:v rawvideo".to_owned();
    if use_cuda {
        args += " -hwaccel_output_format cuda";
    }
    write!(
        &mut args,
        " -s {vw}x{vh} -r {fps} -pix_fmt rgba -thread_queue_size 1024 -i - -i"
    )?;

    let args2 = format!(
        "-c:a copy -c:v {} -pix_fmt yuv420p {} {} {} {} -map 0:v:0 -map 1:a:0 {} -vf vflip -f mov",
        if use_cuda {nvenc} 
        else if has_qsv {qsv} 
        //else if has_amf {amf}
        else if params.config.hardware_accel {bail!(tl!("no-hwacc"));} 
        else {cpu},
        if params.config.bitrate_control == "CRF" {
            if use_cuda {"-cq"}
            else if has_qsv {"-q"}
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

    const N: usize = 60;
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

    if ipc {
        send(IPCEvent::StartRender(frames));
    }

    let fps = fps as f64;
    for frame in 0..N {
        *my_time.borrow_mut() = (frame as f64 / fps).max(0.);
        gl.quad_gl.render_pass(Some(mst.output().render_pass));
        main.update()?;
        main.render(&mut painter)?;
        if *my_time.borrow() <= LoadingScene::TOTAL_TIME as f64 && !config.disable_loading {
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
        if ipc {
            send(IPCEvent::Frame);
        }
    }
    info!("Pre-Render Time:{:.2?}", pre_render_time.elapsed());

    let frames10 = frames / 10;
    let render_time = Instant::now();
    let mut step_time = Instant::now();
    for frame in N as u64..frames {
        if frame % frames10 == 0 {
            let proc = (frame as f32 / frames as f32 * 100.).ceil() as i8 / 10 * 10;
            info!(
                "Render progress: {:.0}% Time elapsed: {:.2}s",
                proc,
                step_time.elapsed().as_secs_f32()
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
        if *my_time.borrow() <= LoadingScene::TOTAL_TIME as f64 && !config.disable_loading {
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
        if ipc {
            send(IPCEvent::Frame);
        }
    }
    drop(input);
    info!("Render Time: {:.2?}", render_time.elapsed());
    info!(
        "Average FPS: {:.2}",
        frames as f64 / render_time.elapsed().as_secs_f64()
    );
    proc.wait()?;
    info!("Task done in {:.2?}", render_start_time.elapsed());
    if ipc {
        send(IPCEvent::Done(render_start_time.elapsed().as_secs_f64()));
    }
    Ok(())
}
