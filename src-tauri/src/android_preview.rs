use crate::render_config::RenderParams;
use jni::{
    objects::{GlobalRef, JObject, JString},
    JNIEnv, JavaVM,
};
use std::sync::Mutex;

static PARAMS: Mutex<Option<RenderParams>> = Mutex::new(None);
static ACTIVITY: Mutex<Option<GlobalRef>> = Mutex::new(None);

#[no_mangle]
pub extern "system" fn Java_com_sc_phi_tk_PreviewActivity_releasePreview(
    _env: JNIEnv,
    _activity: JObject,
) {
    if let Ok(mut activity) = ACTIVITY.lock() {
        *activity = None;
    }
    if let Ok(mut params) = PARAMS.lock() {
        *params = None;
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sc_phi_tk_PreviewActivity_preparePreview(
    mut env: JNIEnv,
    activity: JObject,
    json: JString,
    cache_dir: JString,
) -> jni::sys::jboolean {
    let result = (|| -> anyhow::Result<()> {
        let json: String = env.get_string(&json)?.into();
        let params: RenderParams = serde_json::from_str(&json)?;
        let cache_dir: String = env.get_string(&cache_dir)?.into();
        prpr::fs::set_cache_dir(cache_dir.into());
        *ACTIVITY
            .lock()
            .map_err(|_| anyhow::anyhow!("Preview activity unavailable"))? =
            Some(env.new_global_ref(activity)?);
        *PARAMS
            .lock()
            .map_err(|_| anyhow::anyhow!("Preview state unavailable"))? = Some(params);
        Ok(())
    })();
    match result {
        Ok(()) => 1,
        Err(error) => {
            crate::android_uri::debug_log(&format!("Cannot prepare preview: {error:#}"));
            0
        }
    }
}

pub fn start() {
    let Some(params) = PARAMS.lock().ok().and_then(|mut params| params.take()) else {
        return;
    };
    macroquad::Window::from_config(
        macroquad::window::Conf {
            window_title: "Phi-TK".into(),
            fullscreen: true,
            high_dpi: true,
            ..Default::default()
        },
        async move {
            crate::android_uri::install_logger();
            let _graphics = prpr::GraphicsSession;
            // The preview process owns its runtime and graphics/audio context.
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build();
            match runtime {
                Ok(runtime) => {
                    let _guard = runtime.enter();
                    if let Err(error) = crate::preview::run(params).await {
                        report_error(&format!("{error:#}"));
                    }
                }
                Err(error) => report_error(&format!("Cannot start preview: {error}")),
            }
        },
    );
}

fn report_error(message: &str) {
    crate::android_uri::debug_log(message);
    let context = ndk_context::android_context();
    let result = (|| -> anyhow::Result<()> {
        let vm = unsafe { JavaVM::from_raw(context.vm() as _) }?;
        let mut env = vm.attach_current_thread()?;
        if let Some(activity) = ACTIVITY.lock().ok().and_then(|activity| activity.clone()) {
            env.call_method(activity.as_obj(), "previewFailed", "()V", &[])?;
        }
        Ok(())
    })();
    if result.is_err() {
        crate::android_uri::debug_log("Cannot show preview error");
    }
}
