use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::sync::OnceLock;
use tauri::{
    plugin::{Builder, PluginHandle, TauriPlugin},
    Wry,
};

static PLATFORM: OnceLock<PluginHandle<Wry>> = OnceLock::new();

pub fn init() -> TauriPlugin<Wry> {
    Builder::new("android-platform")
        .setup(|_, api| {
            let handle = api.register_android_plugin("com.sc.phi.tk", "PlatformPlugin")?;
            let _ = PLATFORM.set(handle);
            Ok(())
        })
        .build()
}

pub fn call<T: DeserializeOwned>(command: &str, payload: Value) -> Result<T> {
    PLATFORM
        .get()
        .context("Android platform is not ready")?
        .run_mobile_plugin(command, payload)
        .map_err(Into::into)
}

pub async fn pick_charts(directory: bool, multiple: bool) -> Result<Vec<String>> {
    #[derive(serde::Deserialize)]
    struct Response {
        paths: Vec<String>,
    }
    let result: Response = PLATFORM
        .get()
        .context("Android platform is not ready")?
        .run_mobile_plugin_async(
            "pickCharts",
            json!({ "directory": directory, "multiple": multiple }),
        )
        .await?;
    Ok(result.paths)
}

pub async fn launch_preview(params: crate::render_config::RenderParams) -> Result<()> {
    PLATFORM
        .get()
        .context("Android platform is not ready")?
        .run_mobile_plugin_async::<Value>(
            "launchPreview",
            json!({ "params": serde_json::to_string(&params)? }),
        )
        .await?;
    Ok(())
}
