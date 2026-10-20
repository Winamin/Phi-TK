//! Vulkan NV12 readback for the ffmpeg encoder path.
//!
//! The renderer already produces Y/UV planes on the GPU (`YuvTarget`).  This
//! module keeps a ring of NV12 images plus host-visible staging buffers.  For
//! each frame `record_nv12_copy_to_staging` in miniquad packs the two planes
//! into an NV12 image and copies it into a staging buffer as part of the frame
//! command buffer.  The exporter only waits on the frame fence when the ring
//! slot is reused, so the GPU never stalls on a per-frame readback.

use ash::vk;
use ash::vk::Handle;

struct Slot {
    image: vk::Image,
    #[allow(dead_code)]
    image_memory: vk::DeviceMemory,
    buffer: vk::Buffer,
    #[allow(dead_code)]
    buffer_memory: vk::DeviceMemory,
    mapped: *mut u8,
}

pub struct Nv12Readback {
    width: u32,
    height: u32,
    frame_size: u64,
    slots: Vec<Slot>,
}

impl Nv12Readback {
    pub fn new(width: u32, height: u32, slot_count: usize) -> Option<Self> {
        if width == 0 || height == 0 || width % 2 != 0 || height % 2 != 0 {
            return None;
        }
        let context = macroquad::window::miniquad_context().vulkan_context()?;
        let queue_family = context.queue_family_index;
        let graphics_queue =
            vk::Queue::from_raw(unsafe { std::mem::transmute(context.graphics_queue) });
        let raw_instance: u64 = unsafe { std::mem::transmute(context.instance.handle()) };
        let raw_device: u64 = unsafe { std::mem::transmute(context.device.handle()) };
        let raw_physical: u64 = unsafe { std::mem::transmute(context.physical_device) };
        let entry = unsafe { ash::Entry::load().ok()? };
        let instance = unsafe {
            ash::Instance::load(entry.static_fn(), vk::Instance::from_raw(raw_instance))
        };
        let device = unsafe {
            ash::Device::load(&instance.fp_v1_0(), vk::Device::from_raw(raw_device))
        };
        let physical_device = vk::PhysicalDevice::from_raw(raw_physical);
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let frame_size = width as u64 * height as u64 * 3 / 2;

        let mut slots = Vec::with_capacity(slot_count.max(1));
        for _ in 0..slot_count.max(1) {
            let image_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(vk::Format::G8_B8R8_2PLANE_420_UNORM)
                .extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(
                    vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST,
                )
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .initial_layout(vk::ImageLayout::UNDEFINED);
            let image = unsafe { device.create_image(&image_info, None).ok()? };
            let requirements = unsafe { device.get_image_memory_requirements(image) };
            let image_memory_type = (0..memory_properties.memory_type_count).find(|index| {
                requirements.memory_type_bits & (1 << index) != 0
                    && memory_properties.memory_types[*index as usize]
                        .property_flags
                        .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
            })?;
            let image_memory = unsafe {
                device
                    .allocate_memory(
                        &vk::MemoryAllocateInfo::default()
                            .allocation_size(requirements.size)
                            .memory_type_index(image_memory_type),
                        None,
                    )
                    .ok()?
            };
            unsafe { device.bind_image_memory(image, image_memory, 0).ok()? };

            let buffer_info = vk::BufferCreateInfo::default()
                .size(frame_size)
                .usage(vk::BufferUsageFlags::TRANSFER_DST)
                .sharing_mode(vk::SharingMode::EXCLUSIVE);
            let buffer = unsafe { device.create_buffer(&buffer_info, None).ok()? };
            let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
            let buffer_memory_type = (0..memory_properties.memory_type_count)
                .find(|index| {
                    requirements.memory_type_bits & (1 << index) != 0
                        && memory_properties.memory_types[*index as usize]
                            .property_flags
                            .contains(
                                vk::MemoryPropertyFlags::HOST_VISIBLE
                                    | vk::MemoryPropertyFlags::HOST_COHERENT
                                    | vk::MemoryPropertyFlags::HOST_CACHED,
                            )
                })
                .or_else(|| {
                    (0..memory_properties.memory_type_count).find(|index| {
                        requirements.memory_type_bits & (1 << index) != 0
                            && memory_properties.memory_types[*index as usize]
                                .property_flags
                                .contains(
                                    vk::MemoryPropertyFlags::HOST_VISIBLE
                                        | vk::MemoryPropertyFlags::HOST_COHERENT,
                                )
                    })
                })?;
            let buffer_memory = unsafe {
                device
                    .allocate_memory(
                        &vk::MemoryAllocateInfo::default()
                            .allocation_size(requirements.size)
                            .memory_type_index(buffer_memory_type),
                        None,
                    )
                    .ok()?
            };
            unsafe { device.bind_buffer_memory(buffer, buffer_memory, 0).ok()? };
            let mapped = unsafe {
                device
                    .map_memory(buffer_memory, 0, frame_size, vk::MemoryMapFlags::empty())
                    .ok()? as *mut u8
            };
            slots.push(Slot {
                image,
                image_memory,
                buffer,
                buffer_memory,
                mapped,
            });
        }

        // Move every image from UNDEFINED to GENERAL once.  Miniquad's record
        // method assumes GENERAL before and restores GENERAL afterwards.
        let command_pool = unsafe {
            device
                .create_command_pool(
                    &vk::CommandPoolCreateInfo::default()
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                        .queue_family_index(queue_family),
                    None,
                )
                .ok()?
        };
        let command_buffer = unsafe {
            device
                .allocate_command_buffers(
                    &vk::CommandBufferAllocateInfo::default()
                        .command_pool(command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(1),
                )
                .ok()?[0]
        };
        unsafe {
            device
                .begin_command_buffer(
                    command_buffer,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .ok()?;
            let color = vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            };
            let barriers: Vec<_> = slots
                .iter()
                .map(|slot| {
                    vk::ImageMemoryBarrier::default()
                        .src_access_mask(vk::AccessFlags::empty())
                        .dst_access_mask(
                            vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE,
                        )
                        .old_layout(vk::ImageLayout::UNDEFINED)
                        .new_layout(vk::ImageLayout::GENERAL)
                        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                        .image(slot.image)
                        .subresource_range(color)
                })
                .collect();
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barriers,
            );
            device.end_command_buffer(command_buffer).ok()?;
            let command_buffers = [command_buffer];
            let submit = vk::SubmitInfo::default().command_buffers(&command_buffers);
            device
                .queue_submit(graphics_queue, &[submit], vk::Fence::null())
                .ok()?;
            device.queue_wait_idle(graphics_queue).ok()?;
            device.free_command_buffers(command_pool, &[command_buffer]);
            device.destroy_command_pool(command_pool, None);
        }

        Some(Self {
            width,
            height,
            frame_size,
            slots,
        })
    }

    pub fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub fn frame_size(&self) -> usize {
        self.frame_size as usize
    }

    pub fn image_raw(&self, slot: usize) -> u64 {
        self.slots[slot].image.as_raw()
    }

    pub fn buffer_raw(&self, slot: usize) -> u64 {
        self.slots[slot].buffer.as_raw()
    }

    /// Mapped staging addresses, for the writer thread.  They stay valid while
    /// this `Nv12Readback` is alive.
    pub fn writer_ptrs(&self) -> Vec<usize> {
        self.slots
            .iter()
            .map(|slot| slot.mapped as usize)
            .collect()
    }

    #[allow(dead_code)]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[allow(dead_code)]
    pub fn height(&self) -> u32 {
        self.height
    }
}

/// Put the current miniquad Vulkan context into offscreen export mode.
pub fn enable_offscreen_video() -> bool {
    let Some(context) = macroquad::window::miniquad_context().vulkan_context() else {
        return false;
    };
    context.enable_offscreen_video();
    true
}
