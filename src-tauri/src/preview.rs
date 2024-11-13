use crate::render::{build_player, RenderParams};
use anyhow::Result;
use macroquad::prelude::*;
use prpr::{
    config::{Config, Mods},
    fs,
    scene::{show_error, GameMode, LoadingScene, NextScene, Scene},
    time::TimeManager,
    ui::{FontArc, TextPainter, Ui},
    Main,
};
use std::io::BufRead;
use tokio::task;
use futures::future::join_all;

struct RenderTask {
    params: RenderParams,
    config: Config,
}

struct BaseScene(Option<NextScene>, bool);
impl Scene for BaseScene {
    fn on_result(&mut self, _tm: &mut TimeManager, result: Box<dyn std::any::Any>) -> Result<()> {
        show_error(
            result
                .downcast::<anyhow::Error>()
                .unwrap()
                .context("加载谱面失败"),
        );
        self.1 = true;
        Ok(())
    }
    fn enter(&mut self, _tm: &mut TimeManager, _target: Option<RenderTarget>) -> Result<()> {
        if self.0.is_none() && !self.1 {
            self.0 = Some(NextScene::Exit);
        }
        Ok(())
    }
    fn update(&mut self, _tm: &mut TimeManager) -> Result<()> {
        Ok(())
    }
    fn render(&mut self, _tm: &mut TimeManager, _ui: &mut Ui) -> Result<()> {
        Ok(())
    }
    fn next_scene(&mut self, _tm: &mut TimeManager) -> prpr::scene::NextScene {
        self.0.take().unwrap_or_default()
    }
}

async fn render_task(task: RenderTask) -> Result<()> {
    let fs = fs::fs_from_file(&task.params.path)?;
    let info = task.params.info;
    let mut config = task.config.to_config();
    config.mods |= Mods::AUTOPLAY;

    let font = FontArc::try_from_vec(load_file("font.ttf").await?)?;
    let mut painter = TextPainter::new(font);
    let player = build_player(&task.config).await?;

    let tm = TimeManager::default();
    let ctm = TimeManager::from_config(&config);
    let mut main = Main::new(
        Box::new(BaseScene(
            Some(NextScene::Overlay(Box::new(
                LoadingScene::new(GameMode::Normal, info, config, fs, Some(player), None, None).await?,
            ))),
            false,
        )),
        ctm,
        None,
    )
    .await?;

    let mut fps_time = -1;
    'app: loop {
        let frame_start = tm.real_time();
        main.update()?;
        main.render(&mut painter)?;
        if main.should_exit() {
            break 'app;
        }

        let t = tm.real_time();
        let fps_now = t as i32;
        if fps_now != fps_time {
            fps_time = fps_now;
            info!("| {}", (1. / (t - frame_start)) as u32);
        }

        next_frame().await;
    }
    Ok(())
}

#[tokio::main]
pub async fn main() -> Result<()> {
    set_pc_assets_folder(&std::env::args().nth(2).unwrap());

    let mut stdin = std::io::stdin().lock();
    let stdin = &mut stdin;

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    let params_list: Vec<RenderParams> = serde_json::from_str(line.trim())?;

    let mut tasks = Vec::new();
    for params in params_list {
        let config = params.config.clone();
        tasks.push(RenderTask { params, config });
    }

    let futures: Vec<_> = tasks.into_iter().map(|task| {
        task::spawn(async move {
            render_task(task).await.unwrap();
        })
    }).collect();

    join_all(futures).await;

    Ok(())
}
