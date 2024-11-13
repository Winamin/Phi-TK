use crate::render::{build_player, RenderParams};
use crate::RenderConfig;
use anyhow::Result;
use anyhow::Context;
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
use std::iter::IntoIterator;
use tokio::task;
use futures::future::join_all;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
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

impl RenderTask {
    async fn run(&self) -> Result<()> {
        let fs = fs::fs_from_file(&self.params.path)?;
        let info = &self.params.info;
        let config = &self.config;
        let mut painter = TextPainter::new(load_font().await?);
        let render_config: RenderConfig = config.into();
        let player = build_player(&render_config).await?;

        let tm = TimeManager::default();
        let ctm = TimeManager::from_config(config); // strange variable name...
        let mut main = Main::new(
            Box::new(BaseScene(
                Some(NextScene::Overlay(Box::new(
                    LoadingScene::new(GameMode::Normal, info.clone(), config.clone(), fs, Some(player), None, None)
                        .await?,
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
}

async fn load_font() -> Result<FontArc> {
    FontArc::try_from_vec(load_file("font.ttf").await?).context("Failed to load font")
}

async fn parallel_render_tasks(tasks: Vec<RenderTask>) -> Result<()> {
    let handles: Vec<_> = tasks.into_iter().map(|task| {
        task::spawn(async move {
            task.run().await.context("Task failed")
        })
    }).collect();

    let results = join_all(handles).await;

    for result in results {
        result??;
    }

    Ok(())
}

pub async fn main() -> Result<()> {
    set_pc_assets_folder(&std::env::args().nth(2).unwrap());

    let mut stdin = std::io::stdin().lock();
    let stdin = &mut stdin;

    let mut line = String::new();
    stdin.read_line(&mut line)?;
    let params: RenderParams = serde_json::from_str(line.trim())?;

    let render_task = RenderTask {
        params: params.clone(),
        config: params.config.to_config(),
    };

    parallel_render_tasks(vec![render_task]).await?;

    Ok(())
}
