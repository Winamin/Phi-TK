prpr::tl_file!("render");

use anyhow::{bail, Context, Result};
use macroquad::prelude::Texture2D;
use prpr::{
    config::{ChallengeModeColor, Config},
    ext::SafeTexture,
    info::ChartInfo,
    scene::BasicPlayer,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct RenderConfig {
    pub resolution: (u32, u32),
    pub ending_length: f64,
    pub disable_loading: bool,
    pub audio_delay_frames: i32,
    pub chart_debug: bool,
    pub flid_x: bool,
    pub chart_ratio: f32,
    pub buffer_size: f32,
    pub combo: String,
    pub fps: u32,
    pub video_codec: String,
    pub show_progress_text: bool,
    pub show_time_text: bool,
    pub target_audio: u32,
    pub autoplay: Option<bool>,
    pub bitrate_control: String,
    pub bitrate: String,
    pub watermark: String,
    pub background: bool,

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
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            resolution: (1920, 1080),
            ending_length: -2.0,
            disable_loading: true,
            chart_debug: false,
            audio_delay_frames: 0,
            flid_x: false,
            chart_ratio: 1.0,
            buffer_size: 256.0,
            combo: "AUTOPLAY".to_string(),
            fps: 60,
            video_codec: "h264".to_string(),
            show_progress_text: false,
            show_time_text: false,
            target_audio: 48000,
            autoplay: None,
            bitrate_control: "CBR".to_string(),
            bitrate: "8M".to_string(),
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
            ui_score: true,
            ui_combo: true,
            ui_name: true,
            ui_level: true,
            ui_line: true,
            ui_pb: true,
            ui_pause: true,
            bar: false,
            score_aim: false,
        }
    }
}

impl RenderConfig {
    pub fn normalize(&mut self) -> Result<()> {
        if self.bitrate_control.eq_ignore_ascii_case("CRF") {
            self.bitrate = "8M".to_string();
        }
        self.bitrate_control = "CBR".to_string();
        self.target_audio = 48000;
        self.video_codec = match self.video_codec.to_ascii_lowercase().as_str() {
            "hevc" | "h265" => "hevc".to_string(),
            _ => "h264".to_string(),
        };
        self.bitrate_bps()?;
        Ok(())
    }

    pub fn bitrate_bps(&self) -> Result<u32> {
        let value = parse_bitrate_bps(&self.bitrate).context("invalid target bitrate")?;
        if !(200_000.0..=i32::MAX as f64).contains(&value) {
            bail!("target bitrate must be between 200000 and {} bps", i32::MAX);
        }
        Ok(value.round() as u32)
    }

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

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RenderParams {
    pub path: PathBuf,
    pub info: ChartInfo,
    pub config: RenderConfig,
}

pub(crate) async fn build_player(config: &RenderConfig) -> Result<BasicPlayer> {
    Ok(BasicPlayer {
        avatar: if let Some(path) = &config.player_avatar {
            Some(SafeTexture::from(Texture2D::from_file_with_format(
                &tokio::fs::read(path)
                    .await
                    .with_context(|| tl!("load-avatar-failed"))?,
                None,
            )))
        } else {
            None
        },
        id: 0,
        rks: config.player_rks,
    })
}

pub(crate) fn parse_bitrate_bps(raw: &str) -> Option<f64> {
    let s = raw.trim().to_ascii_lowercase().replace(' ', "");
    if s.is_empty() {
        return None;
    }
    let (num, mult) = if let Some(v) = s.strip_suffix("kbps") {
        (v, 1e3)
    } else if let Some(v) = s.strip_suffix("mbps") {
        (v, 1e6)
    } else if let Some(v) = s.strip_suffix("bps") {
        (v, 1.0)
    } else if let Some(v) = s.strip_suffix('k') {
        (v, 1e3)
    } else if let Some(v) = s.strip_suffix('m') {
        (v, 1e6)
    } else {
        (s.as_str(), 1.0)
    };
    if num.is_empty()
        || num.bytes().filter(|&value| value == b'.').count() > 1
        || !num
            .bytes()
            .all(|value| value.is_ascii_digit() || value == b'.')
        || !num.as_bytes().first()?.is_ascii_digit()
        || !num.as_bytes().last()?.is_ascii_digit()
    {
        return None;
    }
    let num: f64 = num.parse().ok()?;
    (num.is_finite() && num > 0.0 && (num * mult).is_finite()).then_some(num * mult)
}
