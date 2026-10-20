use ash::vk;
use serde::Serialize;
use std::ffi::CStr;
use tauri::ipc::InvokeError;

const SAMPLE_COUNTS: [u32; 7] = [1, 2, 4, 8, 16, 32, 64];

#[derive(Serialize)]
pub struct MsaaSupport {
    device: String,
    counts: Vec<u32>,
}

#[tauri::command]
pub fn get_msaa_sample_counts() -> Result<MsaaSupport, InvokeError> {
    query().map_err(InvokeError::from_anyhow)
}

fn query() -> anyhow::Result<MsaaSupport> {
    unsafe {
        let entry = ash::Entry::load()?;
        let app_info =
            vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 0, 0));
        let instance_info = vk::InstanceCreateInfo::default().application_info(&app_info);
        let instance = entry.create_instance(&instance_info, None)?;

        let physical_device = instance
            .enumerate_physical_devices()?
            .into_iter()
            .find(|device| {
                instance
                    .get_physical_device_queue_family_properties(*device)
                    .iter()
                    .any(|family| family.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            });

        let Some(physical_device) = physical_device else {
            instance.destroy_instance(None);
            anyhow::bail!("no Vulkan device with a graphics queue");
        };

        let properties = instance.get_physical_device_properties(physical_device);
        let supported = properties.limits.framebuffer_color_sample_counts;
        let counts = SAMPLE_COUNTS
            .into_iter()
            .filter(|samples| {
                supported.contains(vk::SampleCountFlags::from_raw(1u32 << samples.trailing_zeros()))
            })
            .collect();
        let device = CStr::from_ptr(properties.device_name.as_ptr())
            .to_string_lossy()
            .into_owned();

        instance.destroy_instance(None);
        Ok(MsaaSupport { device, counts })
    }
}
