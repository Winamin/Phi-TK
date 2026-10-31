use crate::{
    android_media::{pcm_f32_to_i16, Encoder, Muxer},
    render_config::{build_player, RenderParams},
};
use anyhow::{bail, Context, Result};
use log::info;
use macroquad::{
    miniquad::{
        MipmapFilter, RenderPass as MQRenderPass, Texture, TextureFormat, TextureParams,
        TextureWrap,
    },
    prelude::*,
    window::{miniquad_context, Conf},
};
use prpr::{
    config::Mods,
    core::MSRenderTarget,
    fs,
    scene::{GameMode, GameScene, LoadingScene},
    time::TimeManager,
    ui::{FontArc, TextPainter},
    Main,
};
use sasa::AudioClip;
use std::{
    cell::{Cell, RefCell},
    ops::DerefMut,
    path::PathBuf,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};

const SAMPLE_RATE: usize = 44100;

pub fn build_conf(width: u32, height: u32) -> Conf {
    Conf {
        window_title: "Phi-TK".to_string(),
        window_width: width as i32,
        window_height: height as i32,
        headless: true,
        platform: macroquad::miniquad::conf::Platform {
            rendering_backend: macroquad::miniquad::conf::RenderingBackend::Vulkan,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub struct RenderRequest {
    pub params: RenderParams,
    pub output_path: PathBuf,
    pub asset_dir: PathBuf,
    pub request_cancel: Option<Arc<AtomicBool>>,
}

pub fn run(request: RenderRequest, on_progress: impl Fn(f32, f64) + 'static) -> Result<()> {
    crate::android_uri::install_logger();
    let _log = crate::android_uri::start_render_log(request.output_path.with_extension("log"))?;
    let mut request = request;
    request.params.config.normalize()?;
    let (mut vw, mut vh) = request.params.config.resolution;
    let aspect = request.params.info.aspect_ratio as f64;
    crate::android_uri::debug_log(&format!(
        "render: requested {}x{} aspect={aspect:.6}",
        vw, vh
    ));
    if vw == 0 || vh == 0 {
        bail!("resolution must not be zero")
    }
    if aspect.is_finite() && (0.2..=5.0).contains(&aspect) {
        if (vw as f64 / vh as f64 - aspect).abs() > 1e-9 {
            if vw as f64 / vh as f64 > aspect {
                vw = (vh as f64 * aspect).round() as u32;
            } else {
                vh = (vw as f64 / aspect).round() as u32;
            }
        }
    } else {
        crate::android_uri::debug_log(&format!(
            "render: ignoring implausible aspect {aspect:.6}, keeping {}x{}",
            vw, vh
        ));
    }
    vw &= !1;
    vh &= !1;
    if vw < 16 || vh < 16 {
        bail!("refusing to render at {vw}x{vh}");
    }
    crate::android_uri::debug_log(&format!("render: using {vw}x{vh}"));
    let conf = build_conf(vw, vh);
    macroquad::miniquad::fs::set_asset_dir(request.asset_dir.clone());
    set_pc_assets_folder(&request.asset_dir.display().to_string());

    let shared: std::sync::Arc<std::sync::Mutex<Option<Result<()>>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let slot = std::sync::Arc::clone(&shared);
    macroquad::Window::from_config_headless(conf, async move {
        let result = render_async(request, vw, vh, on_progress).await;
        if let Err(error) = &result {
            log::error!("Render failed: {error:?}");
        }
        if let Ok(mut guard) = slot.lock() {
            *guard = Some(result);
        }
    });
    shared
        .lock()
        .ok()
        .and_then(|mut guard| guard.take())
        .unwrap_or_else(|| Err(anyhow::anyhow!("render thread did not report a result")))
}

async fn render_async(
    request: RenderRequest,
    vw: u32,
    vh: u32,
    on_progress: impl Fn(f32, f64) + 'static,
) -> Result<()> {
    let process_start = Instant::now();
    let mut phase = process_start;
    macro_rules! mark {
        ($name:literal) => {{
            let elapsed = phase.elapsed().as_secs_f64();
            info!("setup {}: {:.3}s", $name, elapsed);
            phase = Instant::now();
            elapsed
        }};
    }
    let check_cancel = || -> Result<()> {
        if request
            .request_cancel
            .as_ref()
            .is_some_and(|value| value.load(Ordering::Relaxed))
        {
            bail!("render canceled");
        }
        Ok(())
    };
    check_cancel()?;
    let params = request.params;
    let path = params.path.clone();
    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;
    let mut painter = TextPainter::new(font);

    let mut config = params.config.to_config();
    config.mods = Mods::AUTOPLAY;
    config.interactive = false;

    let mut info = params.info.clone();
    let mut chart_fs = fs::fs_from_file(&path)?;
    crate::android_uri::debug_log(&format!(
        "render: path={} chart={:?} music={:?} illustration={:?}",
        path.display(),
        info.chart,
        info.music,
        info.illustration
    ));
    fs::fix_info(chart_fs.deref_mut(), &mut info)
        .await
        .context("failed to infer the chart files")?;
    crate::android_uri::debug_log(&format!(
        "render: fixed chart={:?} music={:?} illustration={:?}",
        info.chart, info.music, info.illustration
    ));
    let (chart, ..) = GameScene::load_chart(chart_fs.deref_mut(), &info)
        .await
        .context("failed to load the chart")?;
    crate::android_uri::debug_log("render: chart loaded");

    let music = AudioClip::new(
        chart_fs
            .load_file(&info.music)
            .await
            .with_context(|| format!("failed to read music {:?}", info.music))?,
    )
    .context("failed to decode the chart music")?;
    crate::android_uri::debug_log("render: music decoded");
    macro_rules! ld {
        ($path:literal) => {
            AudioClip::new(load_file($path).await?).context(concat!("failed to load ", $path))?
        };
    }
    let ending = ld!("ending.mp3");
    let sfx_click = ld!("click.ogg");
    let sfx_drag = ld!("drag.ogg");
    let sfx_flick = ld!("flick.ogg");

    let render_start_time = Instant::now();
    mark!("chart + audio load");
    let volume_music = config.volume_music;
    let volume_sfx = config.volume_sfx;
    config.volume_music = 0.;
    config.volume_sfx = 0.;

    let track_length = music.length() as f64;
    const O: f64 = LoadingScene::TOTAL_TIME as f64 + GameScene::BEFORE_TIME as f64;
    const A: f64 = 1.0;
    let length = track_length - chart.offset.min(0.) as f64 + 1.;
    let video_length = O + length + A + params.config.ending_length;
    let offset = chart.offset.max(0.);

    let fps = params.config.fps.max(1);
    crate::android_uri::debug_log(&format!("render: fps={fps}"));
    let fps_f64 = fps as f64;
    let frame_duration = 1.0 / fps_f64;
    let audio_delay = params.config.audio_delay_frames as f64 * frame_duration;

    crate::android_uri::debug_log(&format!(
        "render: track_length={track_length:.3} chart_offset={:.3} length={length:.3} O={O:.3} A={A} ending={:.3} video_length={video_length:.3}",
        chart.offset, params.config.ending_length
    ));
    if !video_length.is_finite() || video_length <= 0. {
        bail!("render duration must be positive and finite");
    }
    let audio_buffer_length = video_length + audio_delay.abs();
    let mut output = vec![0.0_f32; (audio_buffer_length * SAMPLE_RATE as f64).ceil() as usize * 2];

    let ratio = 1.0 / SAMPLE_RATE as f64;
    let music_start = Instant::now();
    if volume_music != 0.0 {
        let pos = O - chart.offset.min(0.) as f64 + audio_delay;
        let start_index = (pos * SAMPLE_RATE as f64).round() as usize * 2;
        if start_index < output.len() {
            let output_ptr = output.as_mut_ptr();
            let max_i = (output.len() - start_index) / 2;
            let effective = ((music.length() as f64 * SAMPLE_RATE as f64) as usize).min(max_i);
            let mut time = 0.0_f64;
            for i in 0..effective {
                let frame = music.sample(time as f32).unwrap_or_default();
                unsafe {
                    let idx = start_index + i * 2;
                    *output_ptr.add(idx) += frame.0 * volume_music;
                    *output_ptr.add(idx + 1) += frame.1 * volume_music;
                }
                time += ratio;
            }
        }
    }

    info!("music Time:{:?}", music_start.elapsed());
    let mut place = |pos: f64, clip: &AudioClip, volume: f32| {
        let position = (pos * SAMPLE_RATE as f64).round() as usize * 2;
        if position >= output.len() {
            return 0;
        }
        let len = clip.frame_count();
        let output_len = output.len() - position;
        let valid = (output_len / 2).min(len);
        let output_ptr = unsafe { output.as_mut_ptr().add(position) };
        let frames_ptr = clip.frames().as_ptr();
        for i in 0..valid {
            unsafe {
                let sample = (*frames_ptr.add(i)).0 * volume;
                *output_ptr.add(i * 2) += sample;
                *output_ptr.add(i * 2 + 1) += sample;
            }
        }
        valid
    };

    let sfx_start = Instant::now();
    if volume_sfx != 0.0 {
        let o_offset = O + offset as f64 + audio_delay;
        let sfx = [
            &sfx_click as *const _,
            &sfx_drag as *const _,
            &sfx_click as *const _,
            &sfx_flick as *const _,
        ];
        unsafe {
            for line in chart.lines.iter() {
                for note in line.notes.iter() {
                    if !note.above {
                        continue;
                    }
                    let clip = &*sfx[note.kind.order() as usize];
                    place(o_offset + note.time as f64, clip, volume_sfx);
                }
            }
        }
    }

    info!("sfx Time:{:?}", sfx_start.elapsed());
    let mut pos = O + length + A + audio_delay;
    info!("Ending music start: {:.6}s", pos);
    if params.config.ending_length > 0.1 {
        while place(pos, &ending, volume_music) != 0 {
            pos += ending.length() as f64;
        }
    }
    let mixing_s = mark!("mixing");
    check_cancel()?;

    let sample_rate = params.config.target_audio.max(8000) as usize;
    let pcm = resample_pcm(&output, SAMPLE_RATE, sample_rate);
    let pcm16 = pcm_f32_to_i16(&pcm);
    info!(
        "audio ready: {} samples @ {} Hz ({:.1}s)",
        pcm.len() / 2,
        sample_rate,
        pcm.len() as f64 / 2. / sample_rate as f64
    );

    let ss = if params.config.disable_loading {
        O
    } else {
        0.1
    };
    let ss_samples = (ss * sample_rate as f64).round() as usize * 2;
    let start_bytes = (ss_samples * 2).min(pcm16.len());
    let end_bytes = (((video_length * sample_rate as f64) as usize) * 4).min(pcm16.len());
    let pcm16 = &pcm16[start_bytes..end_bytes.max(start_bytes)];
    crate::android_uri::debug_log(&format!(
        "audio: skipped {ss:.3}s, kept {} frames ({:.3}s) from {:.3}s",
        pcm16.len() / 4,
        pcm16.len() as f64 / 4. / sample_rate as f64,
        start_bytes as f64 / 4. / sample_rate as f64
    ));

    let mut muxer = Muxer::new(&request.output_path)?;
    let bitrate_audio = 192_000u32;
    let mut audio_encoder = Encoder::audio(sample_rate as u32, 2, bitrate_audio)?;
    let chunk = (sample_rate / 25 * 4).max(2048);
    let mut audio_packets = Vec::new();
    let mut written = 0usize;
    while written < pcm16.len() {
        check_cancel()?;
        let end = (written + chunk).min(pcm16.len());
        let slice = &pcm16[written..end];
        let time_us = (written as i64 / 4) * 1_000_000 / sample_rate as i64;
        audio_encoder.submit(slice, time_us, false)?;
        written = end;
        for packet in audio_encoder.drain()? {
            audio_packets.push(packet);
        }
    }
    for packet in audio_encoder.flush()? {
        audio_packets.push(packet);
    }
    let audio_format = audio_encoder
        .take_format()
        .context("the AAC encoder did not report an output format")?;
    let audio_track = muxer.set_audio_track(audio_format)?;
    let audio_s = mark!("audio encode + resample");
    info!(
        "audio encoded: {} packets in {:.2}s",
        audio_packets.len(),
        audio_s
    );

    let bitrate = params.config.bitrate_bps()?;
    let keyframe_interval = fps.saturating_mul(2).max(1);
    let mut video_encoder = Encoder::video(vw, vh, bitrate, fps, 2, &params.config.video_codec)?;
    info!("video encoder: {vw}x{vh} {}bps @ {fps}fps", bitrate);
    info!("Video codec: {}", params.config.video_codec);
    info!("Bitrate: CBR {} bps", bitrate);
    info!("Audio format: AAC, 192000 bps, {} Hz", sample_rate);
    let probe_s = mark!("encoder probe");
    let yuvs: Vec<YuvTarget> = (0..2)
        .map(|_| YuvTarget::new(vw, vh))
        .collect::<Result<Vec<_>>>()?;
    let msts: Vec<Rc<MSRenderTarget>> = (0..2)
        .map(|_| Rc::new(MSRenderTarget::new((vw, vh), params.config.sample_count)))
        .collect();
    let mst_slot = Rc::new(Cell::new(0usize));
    let my_time: Rc<RefCell<f64>> = Rc::new(RefCell::new(0.));
    let tm = TimeManager::manual(Box::new({
        let my_time = Rc::clone(&my_time);
        move || *(*my_time).borrow()
    }));

    let mut readback = crate::vulkan_readback::Nv12Readback::new(vw, vh, 3)
        .context("failed to create the Vulkan NV12 readback ring")?;
    let frame_bytes = readback.frame_size();
    let slot_count = readback.slot_count();
    let ptrs = readback.writer_ptrs();
    crate::android_uri::debug_log(&format!(
        "render: frame_bytes={frame_bytes} expected_nv12={} slots={slot_count}",
        vw as usize * vh as usize * 3 / 2
    ));
    if !crate::vulkan_readback::enable_offscreen_video() {
        bail!("failed to enable the Vulkan offscreen video mode")
    }

    mark!("offscreen/readback setup");
    let player = build_player(&params.config).await?;
    let mut main = Main::new(
        Box::new(
            LoadingScene::new(
                GameMode::Normal,
                info.clone(),
                config,
                chart_fs,
                Some(player),
                None,
                None,
            )
            .await?,
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
    let skip_source = if params.config.disable_loading {
        O
    } else {
        0.1
    };
    let skip = ((skip_source * fps_f64 - 1e-6).ceil().max(0.)) as u64;
    let frames = (video_length * fps_f64).ceil() as u64;
    let out_frames = frames.saturating_sub(skip);
    if out_frames == 0 {
        bail!("render duration contains no output frames");
    }
    let render_start = Instant::now();
    info!("rendering {out_frames} frames");

    let mut pending: [Option<(u64, usize)>; 2] = [None, None];
    let mut free_slots: Vec<usize> = (0..slot_count).collect();
    let mut video_packets = Vec::new();
    let mut video_track: Option<usize> = None;
    let mut muxer_started = false;
    let mut last_progress = 0f32;
    let mut fps_window = Instant::now();
    let mut fps_window_frames = 0u64;
    let mut render_fps = 0f64;

    let mut fence_wait_time = 0.;
    let mut update_time = 0.;
    let mut scene_render_time = 0.;
    let mut yuv_convert_time = 0.;
    let mut nv12_submit_time = 0.;
    let mut encoder_time = 0.;
    let mut mux_time = 0.;
    let mut submitted = 0u64;
    let mut encoded = 0u64;
    let mut encode_max = 0f64;
    let mut step_time = Instant::now();
    let mut next_mark = (out_frames / 10).max(1);
    let show = |done: u64, label: &str, step_time: &mut Instant| {
        let progress = (done as f64 / out_frames as f64).min(1.);
        let fill = (progress * 20.).round() as usize;
        info!(
            "Rendering: [{}{}] {:>3}% | Time: {} | Frames: {}/{}",
            "█".repeat(fill),
            " ".repeat(20 - fill),
            (progress * 100.).ceil() as u32,
            label,
            done,
            out_frames
        );
        *step_time = Instant::now();
    };
    let loop_start = Instant::now();
    for frame in 0..frames {
        check_cancel()?;
        let current_frame_time = frame as f64 * frame_duration;
        *my_time.borrow_mut() = current_frame_time;
        let keep = frame >= skip;

        let context = miniquad_context().vulkan_context().unwrap();
        if context.is_frame_active() {
            context.finish_offscreen_frame();
        }
        let wait_start = Instant::now();
        let frame_slot = context.begin_offscreen_frame();
        fence_wait_time += wait_start.elapsed().as_secs_f64();

        if let Some((output_index, read_slot)) = pending[frame_slot].take() {
            let data =
                unsafe { std::slice::from_raw_parts(ptrs[read_slot] as *const u8, frame_bytes) };
            let time_us = output_index as i64 * 1_000_000 / fps as i64;
            let encode_start = Instant::now();
            video_encoder.submit(data, time_us, output_index % keyframe_interval as u64 == 0)?;
            free_slots.push(read_slot);
            for packet in video_encoder.drain()? {
                if video_track.is_none() {
                    if let Some(format) = video_encoder.take_format() {
                        crate::android_uri::debug_log(&format!(
                            "encoder: output format {}",
                            crate::android_media::Encoder::describe_format(&format)
                        ));
                        let index = muxer.set_video_track(format)?;
                        video_track = Some(index);
                    }
                }
                encoded += 1;
                video_packets.push(packet);
            }
            submitted += 1;
            let elapsed = encode_start.elapsed().as_secs_f64();
            encoder_time += elapsed;
            encode_max = encode_max.max(elapsed);
            let mux_start = Instant::now();
            if !muxer_started {
                if let Some(format) = video_encoder.take_format() {
                    if video_track.is_none() {
                        let index = muxer.set_video_track(format)?;
                        video_track = Some(index);
                    }
                }
                if let Some(track) = video_track {
                    muxer.start()?;
                    muxer_started = true;
                    for packet in audio_packets.iter() {
                        muxer.write(audio_track, &packet.data, packet.time_us)?;
                    }
                    for packet in video_packets.iter() {
                        muxer.write_with_flags(
                            track,
                            &packet.data,
                            packet.time_us,
                            packet.keyframe,
                        )?;
                    }
                    video_packets.clear();
                }
            } else if let Some(track) = video_track {
                for packet in video_packets.iter() {
                    muxer.write_with_flags(track, &packet.data, packet.time_us, packet.keyframe)?;
                }
                video_packets.clear();
            }
            mux_time += mux_start.elapsed().as_secs_f64();
        }

        if frame != 0 {
            let update_start = Instant::now();
            main.update()?;
            update_time += update_start.elapsed().as_secs_f64();
        }
        if !keep {
            context.finish_offscreen_frame();
            continue;
        }

        mst_slot.set(frame_slot);
        {
            let mut gl = unsafe { macroquad::window::get_internal_gl() };
            gl.quad_gl
                .render_pass(Some(msts[frame_slot].input().render_pass));
        }
        let scene_start = Instant::now();
        main.render(&mut painter)?;
        scene_render_time += scene_start.elapsed().as_secs_f64();

        let read_slot = free_slots.pop().context("NV12 readback ring exhausted")?;
        let yuv = &yuvs[frame_slot];
        let convert_start = Instant::now();
        yuv.convert(&msts[frame_slot].output());
        yuv_convert_time += convert_start.elapsed().as_secs_f64();
        let submit_start = Instant::now();
        context.record_yuv_copy_to_staging(
            yuv.y_index,
            yuv.uv_index,
            readback.buffer_raw(read_slot),
            vw,
            vh,
        );
        context.finish_offscreen_frame();
        pending[frame_slot] = Some((frame - skip, read_slot));
        nv12_submit_time += submit_start.elapsed().as_secs_f64();

        fps_window_frames += 1;
        let window_elapsed = fps_window.elapsed().as_secs_f64();
        if window_elapsed >= 0.5 {
            render_fps = fps_window_frames as f64 / window_elapsed;
            fps_window = Instant::now();
            fps_window_frames = 0;
        }

        if encoded >= next_mark {
            let label = format!("{:.2}s", step_time.elapsed().as_secs_f64());
            show(encoded, &label, &mut step_time);
            next_mark = encoded + (out_frames / 10).max(1);
        }
        let done = (frame.saturating_sub(skip) + 1) as f32 / out_frames.max(1) as f32;
        if done - last_progress >= 0.01 {
            last_progress = done;
            on_progress(done, render_fps);
        }
    }

    let loop_wall = loop_start.elapsed().as_secs_f64().max(1e-9);
    mark!("render loop");
    let backlog_at_end = submitted.saturating_sub(encoded)
        + pending.iter().filter(|slot| slot.is_some()).count() as u64;
    miniquad_context()
        .vulkan_context()
        .unwrap()
        .wait_offscreen_idle();
    let loop_encoder_time = encoder_time;
    let loop_mux_time = mux_time;
    let drain_start = Instant::now();
    let mut drained: Vec<(u64, usize)> = (0..2).filter_map(|slot| pending[slot].take()).collect();
    drained.sort_by_key(|(index, _)| *index);
    for (output_index, read_slot) in drained {
        let data = unsafe { std::slice::from_raw_parts(ptrs[read_slot] as *const u8, frame_bytes) };
        let time_us = output_index as i64 * 1_000_000 / fps as i64;
        video_encoder.submit(data, time_us, output_index % keyframe_interval as u64 == 0)?;
        submitted += 1;
        for packet in video_encoder.drain()? {
            video_packets.push(packet);
        }
    }
    for packet in video_encoder.flush()? {
        video_packets.push(packet);
    }
    encoded += video_packets.len() as u64;
    encoder_time += drain_start.elapsed().as_secs_f64();
    let mux_start = Instant::now();
    if video_track.is_none() {
        if let Some(format) = video_encoder.take_format() {
            let index = muxer.set_video_track(format)?;
            video_track = Some(index);
        }
    }
    if !muxer_started {
        if video_track.is_some() {
            muxer.start()?;
            muxer_started = true;
            for packet in audio_packets.iter() {
                muxer.write(audio_track, &packet.data, packet.time_us)?;
            }
        } else {
            bail!("no video frame was encoded");
        }
    }
    if let Some(track) = video_track {
        for packet in video_packets.iter() {
            muxer.write_with_flags(track, &packet.data, packet.time_us, packet.keyframe)?;
        }
    }
    muxer.finish()?;
    mux_time += mux_start.elapsed().as_secs_f64();
    mark!("drain + finalize");
    let elapsed = render_start_time.elapsed();
    let wall = elapsed.as_secs_f64().max(1e-9);
    info!("Render Time: {:.2?}", elapsed);
    info!("Render thread waits: offscreen fence wait {:.2}s ({:.1}%) | MediaCodec submit + drain {:.2}s ({:.1}%) | MP4 mux {:.2}s ({:.1}%)",
        fence_wait_time, fence_wait_time / wall * 100., encoder_time, encoder_time / wall * 100., mux_time, mux_time / wall * 100.);
    let swap_wait_s = miniquad_context()
        .vulkan_context()
        .map(|context| context.frame_wait_ns as f64 / 1e9)
        .unwrap_or(0.);
    info!("Swapchain frame fence wait: {:.2}s ({:.1}% of scene render) -> CPU-side scene recording {:.2}s",
        swap_wait_s, swap_wait_s / scene_render_time.max(1e-9) * 100., (scene_render_time - swap_wait_s).max(0.));
    info!("NV12 handoff: ring {} x {:.1} MiB | MediaCodec {} submissions, {} packets, {:.2}s (max {:.1}ms) | backlog when loop ended {} frames",
        slot_count, frame_bytes as f64 / (1 << 20) as f64, submitted, encoded, encoder_time, encode_max * 1000., backlog_at_end);
    info!("prpr stage times: update {:.2}s ({:.0} fps) | scene render {:.2}s ({:.0} fps) | YUV convert {:.2}s ({:.0} fps) | NV12 copy/submit {:.2}s ({:.0} fps)",
        update_time, out_frames as f64 / update_time.max(1e-9), scene_render_time, out_frames as f64 / scene_render_time.max(1e-9),
        yuv_convert_time, out_frames as f64 / yuv_convert_time.max(1e-9), nv12_submit_time, out_frames as f64 / nv12_submit_time.max(1e-9));
    show(out_frames, "Final frame", &mut step_time);
    let busy = (loop_wall - fence_wait_time - loop_encoder_time - loop_mux_time).max(1e-9);
    info!("Render loop Time: {:.2}s | prpr avg fps: {:.0} (busy {:.2}s of {:.2}s, i.e. render side is idle {:.1}%)",
        loop_wall, out_frames as f64 / busy, busy, loop_wall, (1. - busy / loop_wall) * 100.);
    let excluded_setup = mixing_s + audio_s + probe_s;
    let video_wall = (wall - excluded_setup).max(1e-9);
    info!("Video Pipeline Avg FPS: {:.0} ({} frames / {:.2}s of video pipeline; {:.2}s setup excluded: audio {:.2}s, encoder probe {:.2}s)",
        encoded as f64 / video_wall, encoded, video_wall, excluded_setup, mixing_s + audio_s, probe_s);
    info!(
        "End-to-End Time: {:.2}s",
        process_start.elapsed().as_secs_f64()
    );
    check_cancel()?;
    on_progress(1.0, render_fps);
    info!(
        "render finished in {:.2}s -> {}",
        render_start.elapsed().as_secs_f64(),
        request.output_path.display()
    );
    Ok(())
}

fn resample_pcm(input: &[f32], from: usize, to: usize) -> Vec<f32> {
    if from == to {
        return input.to_vec();
    }
    let frames = input.len() / 2;
    let ratio = from as f64 / to as f64;
    let out_frames = (frames as f64 / ratio) as usize;
    let mut out = vec![0.0f32; out_frames * 2];
    for index in 0..out_frames {
        let position = index as f64 * ratio;
        let base = position.floor() as usize;
        let frac = (position - base as f64) as f32;
        let next = (base + 1).min(frames.saturating_sub(1));
        for channel in 0..2 {
            let a = input.get(base * 2 + channel).copied().unwrap_or(0.);
            let b = input.get(next * 2 + channel).copied().unwrap_or(0.);
            out[index * 2 + channel] = a + (b - a) * frac;
        }
    }
    out
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

pub(crate) struct YuvTarget {
    y: RenderTarget,
    uv: RenderTarget,
    y_material: Material,
    uv_material: Material,
    pub(crate) y_index: usize,
    pub(crate) uv_index: usize,
}

impl YuvTarget {
    pub(crate) fn new(w: u32, h: u32) -> Result<Self> {
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

    pub(crate) fn convert(&self, src: &RenderTarget) {
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
