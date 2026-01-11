#![allow(dead_code, unused)]
use anyhow::{anyhow, Error, Result};
use ash::khr::ray_tracing_pipeline;
use ash::vk::{ExternalMemoryHandleTypeFlags, MemoryGetWin32HandleInfoKHR, HANDLE};
use dashmap::DashMap;
use rayon::prelude::*;
use smallvec::smallvec;
use vulkano::acceleration_structure::AccelerationStructure;
use vulkano::buffer::Subbuffer;
use vulkano::pipeline::PipelineLayout;
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::ray_tracing::{RayTracingPipeline, RayTracingPipelineCreateInfo};
use std::sync::{Arc, Mutex, MutexGuard, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract};
use vulkano::command_buffer::{ClearColorImageInfo, CommandBufferSubmitInfo, RenderingAttachmentInfo, RenderingInfo, SemaphoreSubmitInfo, SubmitInfo};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, DeviceFeatures, Queue, QueueCreateInfo, QueueFlags};
use vulkano::format::{ClearColorValue, Format};
use vulkano::image::sys::RawImage;
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{MemoryAllocator, StandardMemoryAllocator};
use vulkano::memory::{DedicatedAllocation, DeviceMemory, ExternalMemoryHandleTypes, MemoryAllocateInfo, MemoryPropertyFlags, ResourceMemory};
use vulkano::sync::fence::{Fence, FenceCreateInfo};
use vulkano::sync::semaphore::{ExternalSemaphoreHandleType, ExternalSemaphoreHandleTypes, Semaphore, SemaphoreCreateInfo};
use vulkano::{NonExhaustive, Version, VulkanLibrary, VulkanObject};
use windows_sys::Win32::Foundation::CloseHandle;

use crate::geometry::{GeometryType, GeometryManager};

#[derive(Clone)]
pub struct VkBackend {
    raw: Raw,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>,
    size: Arc<RwLock<(u32, u32)>>,
    memory_type_index: u32,
    render_target: RenderTargetWrapper,
    semaphore: SharedSemaphore,
    state: Arc<RenderState>,
    geometry_manager: Arc<GeometryManager>
}

impl VkBackend {
    pub fn new() -> Result<VkBackend> {
        let library = VulkanLibrary::new().map_err(|e| anyhow!("backend.rs:error in loading VulkanLibrary: {:?}", e))?;
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                application_name: Some("nexus-native".to_string()),
                flags: InstanceCreateFlags::ENUMERATE_PORTABILITY,
                ..Default::default()
            },
        )?;
        let physical_devices = instance.enumerate_physical_devices()?;
        let mut score = 0u32;
        let mut device = None;
        let mut dp = None;
        for physical_device in physical_devices {
            let s = check_physical_device(physical_device.clone());
            if s.graphics && s.score > score {
                score = s.score;
                device = Some(physical_device);
                dp = Some(s);
            }
        }
        if device.is_none() {
            return Err(anyhow!("no suitable device found"));
        }
        let physical_device = device.unwrap();
        let device_properties = dp.unwrap();
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_queue_family_index, queue_family_properties)| {
                queue_family_properties
                    .queue_flags
                    .contains(QueueFlags::GRAPHICS)
            })
            .ok_or(anyhow!("no suitable queue family"))? as u32;
        let (device, mut queues) = Device::new(
            physical_device.clone(),
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: DeviceExtensions {
                    khr_external_memory: true,
                    #[cfg(windows)] khr_external_memory_win32: true,
                    #[cfg(windows)] khr_external_semaphore_win32: true,
                    #[cfg(unix)] khr_external_memory_fd: true,
                    #[cfg(unix)] khr_external_semaphore_fd: true,
                    khr_acceleration_structure: device_properties.ray_trace,
                    khr_ray_tracing_pipeline: device_properties.ray_trace,
                    khr_ray_query: device_properties.ray_trace,
                    khr_deferred_host_operations: device_properties.ray_trace,
                    ..Default::default()
                },
                enabled_features: DeviceFeatures {
                    dynamic_rendering: true,
                    ray_tracing_pipeline: device_properties.ray_trace,
                    ray_query: device_properties.ray_trace,
                    acceleration_structure: device_properties.ray_trace,
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;
        let memory_type_index = physical_device.memory_properties().memory_types.iter().position(|i| i.property_flags.contains(MemoryPropertyFlags::DEVICE_LOCAL)).ok_or(anyhow!("no suitable device memory type"))? as u32;
        let queue = queues.next().unwrap();
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let command_buffer_allocator = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(), StandardCommandBufferAllocatorCreateInfo::default())
        );
        let raw = Raw::load(instance, device.clone()).map_err(|e| anyhow!("backend.rs:error in loading ash objects: {:?}", e))?;
        let gl_ready = Semaphore::new(
            device.clone(),
            SemaphoreCreateInfo {
                #[cfg(windows)] export_handle_types: ExternalSemaphoreHandleTypes::OPAQUE_WIN32,
                #[cfg(unix)] export_handle_types: ExternalSemaphoreHandleTypes::OPAQUE_FD,
                ..Default::default()
            }
        ).map_err(|err| anyhow!("backend.rs:error in creating semaphore gl_ready: {:?}", err))?;
        let gl_complete = Semaphore::new(
            device.clone(),
            SemaphoreCreateInfo {
                #[cfg(windows)] export_handle_types: ExternalSemaphoreHandleTypes::OPAQUE_WIN32,
                #[cfg(unix)] export_handle_types: ExternalSemaphoreHandleTypes::OPAQUE_FD,
                ..Default::default()
            }
        ).map_err(|err| anyhow!("backend.rs:error in creating semaphore gl_complete: {:?}", err))?;
        /*let ray_tracing_pipeline = RayTracingPipeline::new(
            device.clone(),
            None,
            {
                let mut info = RayTracingPipelineCreateInfo::layout(PipelineLayout::new(
                    device.clone(),
                    PipelineLayoutCreateInfo::default(),
                ).map_err(|err| anyhow!("backend.rs:error in creating ray tracing pipeline layout: {:?}", err))?);
                info.max_pipeline_ray_recursion_depth = 8;
                info
            },
            ).map_err(|err| anyhow!("backend.rs:error in creating ray tracing pipeline: {:?}", err))?;*/
        Ok(VkBackend {
            raw,
            queue,
            memory_allocator: memory_allocator.clone(),
            command_buffer_allocator,
            size: Arc::new(RwLock::new((0, 0))),
            memory_type_index,
            render_target: RenderTargetWrapper::default(),
            semaphore: SharedSemaphore {
                handle_gl_ready: gl_ready.export_win32_handle(ExternalSemaphoreHandleType::OpaqueWin32).map_err(|err| anyhow!("backend.rs:error in exporting semaphore gl_ready: {:?}", err))?,
                gl_ready: Arc::new(gl_ready),
                handle_gl_complete: gl_complete.export_win32_handle(ExternalSemaphoreHandleType::OpaqueWin32).map_err(|err| anyhow!("backend.rs:error in exporting semaphore gl_complete: {:?}", err))?,
                gl_complete: Arc::new(gl_complete),
            },
            state: Arc::new(RenderState::default()),
            geometry_manager: Arc::new(GeometryManager::new(memory_allocator))
        })
    }

    pub fn start_rendering_thread(self: Arc<Self>) {
        if self.state.closed() {
            return
        }
        self.state.set_available(true);
        let this = self.clone();
        thread::spawn(move || {
            while(this.state.available()) {
                    if let Err(e) = this.render_and_wait() {
                        this.state.set_error(e);
                        break;
                    };
            }
        });
    }

    pub fn end_rendering_thread(&self) {
        self.state().set_available(false);
    }
    
    #[inline]
    fn render_and_wait(&self) -> Result<()> {
        Ok(self.render()?.wait(None)?)
    }

    pub fn render(&self) -> Result<Arc<Fence>> {
        let target = self.render_target.get_value()?;
        let mut builder = AutoCommandBufferBuilder::primary(
            self.command_buffer_allocator.clone(),
            self.queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;
        builder
            .clear_color_image(ClearColorImageInfo {
                clear_value: ClearColorValue::Float([0.0, 0.0, 1.0, 0.0]),
                ..ClearColorImageInfo::image(target.image.clone())
            })?
            /*.begin_rendering(RenderingInfo {
                color_attachments: vec![Some(RenderingAttachmentInfo::image_view(target.image_view))],
                ..Default::default()
            })?
            .end_rendering()?*/;
        let command_buffer = builder.build()?;
        let fence = Arc::new(Fence::from_pool(self.device())?);
        self.queue.with(|mut queue| -> Result<()> {
            unsafe { Ok(queue.submit(
                &[SubmitInfo {
                    wait_semaphores: vec![SemaphoreSubmitInfo::new(self.semaphore.gl_complete.clone())],
                    command_buffers: vec![CommandBufferSubmitInfo::new(command_buffer)],
                    signal_semaphores: vec![SemaphoreSubmitInfo::new(self.semaphore.gl_ready.clone())],
                    ..Default::default()
                }],
                Some(&fence)
            )?) }
        })?;
        Ok(fence)
    }

    pub fn build_acceleration_structure(&self, buffer: Vec<u8>, geometry_type: GeometryType) -> u64 {
        self.geometry_manager.add_temporary_geometry(buffer, geometry_type)
    }

    #[inline]
    pub fn resize(&self, size: (u32, u32)) {
        let mut guard = self.size.write().unwrap();
        guard.0 = size.0;
        guard.1 = size.1;
    }

    pub fn update(&self) -> Result<()> {
        let size = self.size.read()
            .map_err(|err| anyhow!("backend.rs:error in creating RawImage: {:?}", err) )?;
        let create_info = ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [size.0, size.1, 1],
            usage: ImageUsage::COLOR_ATTACHMENT | ImageUsage::SAMPLED | ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
            external_memory_handle_types: ExternalMemoryHandleTypes::OPAQUE_WIN32,
            ..Default::default()
        };
        let raw_image = RawImage::new(self.device(), create_info)
            .map_err(|err| anyhow!("backend.rs:error in creating RawImage: {:?}", err) )?;
        let size = raw_image.memory_requirements()[0].layout.size();
        let memory = self.memory_allocator.allocate_dedicated(
            self.memory_type_index,
            size,
            Some(DedicatedAllocation::Image(&raw_image)),
            #[cfg(windows)] ExternalMemoryHandleTypes::OPAQUE_WIN32,
            #[cfg(unix)] ExternalMemoryHandleTypes::OPAQUE_FD
        ).map_err(|err| anyhow!("backend.rs:error in allocating device memory: {:?}", err) )?;
        let device_memory = memory.device_memory.clone();
        let allocation = unsafe { 
            ResourceMemory::from_allocation(self.memory_allocator.clone(), memory)
        };
        let image = Arc::new(raw_image
            .bind_memory([allocation])
            .map_err(|(err,_,_)| anyhow!("backend.rs:error in binding memory and image: {:?}", err) )?);
        self.render_target.destroy();
        let vk_memory = device_memory.handle();
        let info = MemoryGetWin32HandleInfoKHR {
            memory: vk_memory,
            handle_type: ExternalMemoryHandleTypeFlags::OPAQUE_WIN32,
            ..Default::default()
        };
        let handle = unsafe {
            self.raw.ext_device.get_memory_win32_handle(&info)
                .map_err(|err| anyhow!("backend.rs:error in getting external memory handle: {:?}", err))?
        };
        let view = ImageView::new_default(image.clone()).map_err(|err| anyhow!("backend.rs:error in creating image view: {:?}", err) )?;
        self.render_target.reset(device_memory, image, handle, view, size).map_err(|err| anyhow!("backend.rs:error in resetting RenderTarget: {:?}", err) )?;
        Ok(())
    }
    
    #[inline]
    pub fn target(&self) -> RenderTargetWrapper {
        self.render_target.clone()
    }

    #[inline]
    pub fn semaphore(&self) -> SharedSemaphore {
        self.semaphore.clone()
    }

    #[inline]
    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
    }

    #[inline]
    pub fn memory_allocator(&self) -> Arc<StandardMemoryAllocator> {
        self.memory_allocator.clone()
    }

    #[inline]
    pub fn command_buffer_allocator(&self) -> Arc<StandardCommandBufferAllocator> {
        self.command_buffer_allocator.clone()
    }

    #[inline]
    pub fn device(&self) -> Arc<Device> {
        self.queue.device().clone()
    }

    #[inline]
    pub fn physical_device(&self) -> Arc<PhysicalDevice> {
        self.device().physical_device().clone()
    }

    #[inline]
    pub fn instance(&self) -> Arc<Instance> {
        self.device().instance().clone()
    }
    
    #[inline]
    pub(crate) fn state(&self) -> Arc<RenderState> {
        self.state.clone()
    }
}

fn check_physical_device(physical_device: Arc<PhysicalDevice>) -> DeviceProperties {
    let mut score = 0u32;
    let mut graphics = true;
    let mut ray_trace = false;
    if physical_device.api_version() >= Version::V1_4 {
        score += 1;
    }
    if physical_device.supported_features().ray_tracing_pipeline
    || !physical_device.supported_extensions().khr_deferred_host_operations
    {
        score += 1;
        ray_trace = true;
    }
    if physical_device.api_version() < Version::V1_3
    || !physical_device.supported_extensions().khr_external_memory
    || !physical_device.supported_features().dynamic_rendering
    {
        graphics = false;
    }
    DeviceProperties {
        graphics,
        ray_trace,
        score,
    }
}

impl Drop for VkBackend {
    fn drop(&mut self) {
        self.state.close();
        self.render_target.destroy();
    }
}

#[derive(Clone)]
struct Raw {
    vk_instance: ash::Instance,
    ext_device: ash::khr::external_memory_win32::Device,
    vk_device: ash::Device,
}

impl Raw {
    fn load(instance: Arc<Instance>, device: Arc<Device>) -> Result<Self> {
        let vk_raw_device = device.handle();
        let vk_raw_instance = instance.handle();
        let fns = instance.fns();
        let vk_instance = ash::Instance::from_parts_1_3(
            vk_raw_instance,
            fns.v1_0.clone(),
            fns.v1_1.clone(),
            fns.v1_3.clone(),
        );
        let fns = device.fns();
        let vk_device = ash::Device::from_parts_1_3(
            vk_raw_device,
            fns.v1_0.clone(),
            fns.v1_1.clone(),
            fns.v1_2.clone(),
            fns.v1_3.clone(),
        );

        let ext_device = ash::khr::external_memory_win32::Device::new(&vk_instance, &vk_device);
        Ok(Raw {
            vk_device,
            vk_instance,
            ext_device,
        })
    }
}

#[derive(Default)]
pub struct RenderState {
    rendering: AtomicBool,
    closed: AtomicBool,
    error: Mutex<Option<Error>>,
}

impl RenderState {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    #[inline]
    fn close(&self) {
        self.set_available(false);
        self.closed.store(true, Ordering::SeqCst);
    }
    
    #[inline]
    pub fn available(&self) -> bool {
        self.rendering.load(Ordering::SeqCst)
    }
    
    #[inline]
    pub fn set_available(&self, available: bool) {
        self.rendering.store(available, Ordering::SeqCst);
    }
    
    #[inline]
    pub fn handle_error(&self, handler: impl FnOnce(&Error)) {
        if let Some(e) = &self.error.lock().unwrap().take() {
            handler(e)
        }
    }
    
    #[inline]
    pub fn set_error(&self, error: Error) {
        self.error.lock().unwrap().replace(error);
    }
}

#[derive(Clone, Default)]
pub struct RenderTargetWrapper(Arc<RwLock<Option<RenderTarget>>>);
#[derive(Clone)]
pub struct RenderTarget {
    pub memory: Arc<DeviceMemory>,
    pub size: u64,
    pub handle: HANDLE,
    pub image: Arc<Image>,
    pub image_view: Arc<ImageView>
}

#[derive(Clone)]
pub struct SharedSemaphore {
    pub gl_ready: Arc<Semaphore>,
    pub gl_complete: Arc<Semaphore>,
    pub handle_gl_ready: HANDLE,
    pub handle_gl_complete: HANDLE
}

impl RenderTargetWrapper {
    pub fn destroy(&self) -> Result<()> {
        let mut value = self.0.write().map_err(|err| anyhow!("{}", err))?;
        if let Some(mem) = &*value {
            unsafe { CloseHandle(mem.handle as _); }
        }
        *value = None;
        Ok(())
    }
    pub fn get_value(&self) -> Result<RenderTarget> {
        self.0.read().map_err(|err| anyhow!("{}", err))?.clone().ok_or(anyhow!("backend.rs:render target not found"))
    }

    pub fn reset(&self, memory: Arc<DeviceMemory>, image: Arc<Image>, handle: HANDLE, image_view: Arc<ImageView>, size: u64) -> Result<()> {
        let mut value = self.0.write().map_err(|err| anyhow!("{}", err))?;
        *value = Some(RenderTarget{memory, handle, image, image_view, size});
        Ok(())
    }
}

pub struct DeviceProperties {
    graphics: bool,
    ray_trace: bool,
    score: u32,
}


