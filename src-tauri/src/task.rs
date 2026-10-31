use crate::{common::output_dir, render_config::RenderParams};
use anyhow::Result;
use chrono::Local;
use prpr::fs;
use serde::Serialize;
use std::{
    io::Write,
    ops::DerefMut,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Instant,
};
use tempfile::NamedTempFile;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tracing::{error, info};

#[derive(Serialize, Clone)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum TaskStatus {
    Pending,
    Loading,
    Mixing,
    Saving,
    Rendering {
        encoder_fps: u64,
        progress: f64,
        fps: u64,
        estimate: f64,
    },
    Done {
        duration: f64,
        output: String,
        video_uri: String,
    },
    Canceled,
    Failed {
        error: String,
    },
}

pub struct Task {
    id: u32,
    name: String,
    cover: NamedTempFile,
    output: PathBuf,

    params: RenderParams,
    status: Mutex<TaskStatus>,
    request_cancel: Arc<AtomicBool>,
}

impl Task {
    async fn new(id: u32, mut params: RenderParams) -> Result<Self> {
        params.config.normalize()?;
        let mut fs = fs::fs_from_file(&params.path)?;
        let info = fs::load_info(fs.deref_mut()).await?;
        let mut cover = prpr::fs::create_temp_file()?;
        let video = "mp4";
        cover.write_all(&fs.load_file(&info.illustration).await?)?;

        let level: String = info
            .level
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        let safe_name: String = info
            .name
            .chars()
            .filter(|&it| it == '-' || it == '_' || it == ' ' || it.is_alphanumeric())
            .collect();
        let file_name = format!(
            "{}-{id} {safe_name}_{level}.{video}",
            Local::now().format("%Y-%m-%d %H-%M-%S")
        );

        let output = output_dir()?.join(file_name);

        Ok(Self {
            id,
            name: info.name,
            cover,
            output,

            params,
            status: Mutex::new(TaskStatus::Pending),
            request_cancel: Arc::<AtomicBool>::default(),
        })
    }

    #[cfg(target_os = "android")]
    async fn run_android(&self) -> Result<()> {
        use std::sync::{
            atomic::{AtomicU64, Ordering as AtomicOrdering},
            Arc,
        };

        *self.status.lock().await = TaskStatus::Loading;
        let progress = Arc::new(AtomicU64::new(0));
        let current_fps = Arc::new(AtomicU64::new(0));
        let progress_out = Arc::clone(&progress);
        let fps_out = Arc::clone(&current_fps);
        let request = crate::android_render::RenderRequest {
            params: self.params.clone(),
            output_path: self.output.clone(),
            asset_dir: crate::ASSET_PATH
                .get()
                .cloned()
                .unwrap_or_else(|| PathBuf::from("assets")),
            request_cancel: Some(Arc::clone(&self.request_cancel)),
        };

        *self.status.lock().await = TaskStatus::Mixing;
        let finished = std::sync::Arc::new(std::sync::Mutex::new(None::<Result<()>>));
        let finished_out = std::sync::Arc::clone(&finished);
        tokio::task::spawn_blocking(move || {
            let result = crate::android_render::run(request, move |value, fps| {
                progress_out.store((value * 10000.0) as u64, AtomicOrdering::Relaxed);
                fps_out.store(fps.max(0.) as u64, AtomicOrdering::Relaxed);
            });
            if let Ok(mut guard) = finished_out.lock() {
                *guard = Some(result);
            }
        });

        let start = Instant::now();
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            let value = progress.load(AtomicOrdering::Relaxed) as f64 / 10000.0;
            let fps = current_fps.load(AtomicOrdering::Relaxed);
            let done = finished.lock().ok().and_then(|mut guard| guard.take());
            let elapsed = start.elapsed().as_secs_f64();
            let estimate = if value > 0.01 {
                elapsed / value - elapsed
            } else {
                0.0
            };
            match done {
                Some(Ok(())) => {
                    if self.request_cancel.load(Ordering::Relaxed) {
                        let _ = tokio::fs::remove_file(&self.output).await;
                        *self.status.lock().await = TaskStatus::Canceled;
                        return Ok(());
                    }
                    let log = tokio::fs::read_to_string(self.output.with_extension("log"))
                        .await
                        .unwrap_or_default();
                    let name = self
                        .output
                        .file_name()
                        .map(|value| value.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "output.mp4".to_string());
                    let source = self.output.clone();
                    *self.status.lock().await = TaskStatus::Saving;
                    let published = tokio::task::spawn_blocking(move || {
                        crate::android_uri::publish_video(&source, &name)
                    })
                    .await
                    .unwrap_or_else(|error| Err(anyhow::anyhow!("publish task failed: {error}")));

                    let video_uri = published.map_err(|error| {
                        anyhow::anyhow!("Cannot save video to the gallery: {error:#}\n\n{log}")
                    })?;
                    let _ = tokio::fs::remove_file(&self.output).await;
                    let _ = tokio::fs::remove_file(self.output.with_extension("log")).await;
                    *self.status.lock().await = TaskStatus::Done {
                        duration: elapsed,
                        output: log,
                        video_uri,
                    };
                    return Ok(());
                }
                Some(Err(error)) => {
                    let log = tokio::fs::read_to_string(self.output.with_extension("log"))
                        .await
                        .unwrap_or_default();
                    let _ = tokio::fs::remove_file(&self.output).await;
                    *self.status.lock().await = if self.request_cancel.load(Ordering::Relaxed) {
                        TaskStatus::Canceled
                    } else {
                        TaskStatus::Failed {
                            error: format!("{error:?}\n\n{log}"),
                        }
                    };
                    return Ok(());
                }
                None => {
                    *self.status.lock().await = TaskStatus::Rendering {
                        encoder_fps: fps,
                        progress: value,
                        fps,
                        estimate,
                    };
                }
            }
        }
    }

    pub async fn run(&self) -> Result<()> {
        if self.request_cancel.load(Ordering::Relaxed) {
            *self.status.lock().await = TaskStatus::Canceled;
            return Ok(());
        }
        info!("Task #{} started ({})", self.id, self.params.path.display());

        #[cfg(target_os = "android")]
        return self.run_android().await;

        #[cfg(not(target_os = "android"))]
        anyhow::bail!("rendering requires Android");
    }

    pub fn cancel(&self) {
        self.request_cancel.store(true, Ordering::Relaxed);
    }

    pub async fn to_view(&self) -> TaskView {
        let status = self.status.lock().await.clone();
        TaskView {
            id: self.id,
            name: self.name.clone(),
            output: match &status {
                TaskStatus::Done { video_uri, .. } => video_uri.clone(),
                _ => String::new(),
            },
            path: self.params.path.display().to_string(),
            cover: self.cover.path().display().to_string(),
            status,
        }
    }
}

#[derive(Serialize)]
pub struct TaskView {
    id: u32,
    name: String,
    output: String,
    path: String,
    cover: String,
    status: TaskStatus,
}

pub struct TaskQueue {
    sender: mpsc::UnboundedSender<Arc<Task>>,
    worker: JoinHandle<()>,

    tasks: Mutex<Vec<Arc<Task>>>,
}
impl TaskQueue {
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<Arc<Task>>();
        let task = tokio::spawn(async move {
            while let Some(task) = receiver.recv().await {
                if let Err(err) = task.run().await {
                    error!("Failed to render: {err:?}");
                    *task.status.lock().await = TaskStatus::Failed {
                        error: format!("{err:?}"),
                    };
                }
            }
        });

        Self {
            sender,
            worker: task,

            tasks: Mutex::default(),
        }
    }

    pub async fn post(&self, params: RenderParams) -> Result<u32> {
        let mut guard = self.tasks.lock().await;
        let id = guard.len() as u32;
        let task = Arc::new(Task::new(id, params).await?);
        guard.push(Arc::clone(&task));
        self.sender.send(task)?;

        Ok(id)
    }

    pub async fn tasks(&self) -> Vec<TaskView> {
        let guard = self.tasks.lock().await;
        let mut result = Vec::with_capacity(guard.capacity());
        for task in guard.iter() {
            result.push(task.to_view().await);
        }
        result.reverse();
        result
    }

    pub async fn cancel(&self, id: u32) {
        if let Some(task) = self.tasks.lock().await.get(id as usize) {
            task.cancel();
        }
    }
}

impl Drop for TaskQueue {
    fn drop(&mut self) {
        self.worker.abort();
    }
}
