#![allow(dead_code, unused)]
use anyhow::{anyhow, Result};
use ash::vk::{DeviceSize, ExternalMemoryHandleTypeFlags, MemoryGetWin32HandleInfoKHR};
use std::sync::{Arc, Mutex, RwLock};
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::format::Format;
use vulkano::image::sys::RawImage;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, AllocationType, MemoryAllocator, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::memory::{DedicatedAllocation, DeviceMemory, ExternalMemoryHandleTypes, MemoryAllocateInfo, MemoryPropertyFlags, ResourceMemory};
use vulkano::{Version, VulkanLibrary, VulkanObject};
use windows_sys::Win32::Foundation::CloseHandle;

#[derive(Clone)]
pub struct VkBackend {
    raw: Raw,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    size: Arc<RwLock<(u32, u32)>>,
    memory_type_index: u32,
    export_memory: Arc<Mutex<ExportMemory>>,
}

#[derive(Clone)]
pub struct ExportMemory(Option<(Arc<DeviceMemory>, isize)>);

impl ExportMemory {
    pub unsafe fn destory(&mut self) {
        if let Some(mem) = &self.0 {
            unsafe { CloseHandle(mem.1 as _); }
        }
        self.0 = None;
    }
}

pub struct DeviceProperties {
    graphics: bool,
    ray_trace: bool,
    score: u32,
}

impl VkBackend {
    pub fn new() -> Result<VkBackend> {
        let library = VulkanLibrary::new()?;
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
        for physical_device in physical_devices {
            let s = check_physical_device(physical_device.clone());
            if s.graphics && s.score > score {
                score = s.score;
                device = Some(physical_device);
            }
        }
        if device.is_none() {
            return Err(anyhow!("no suitable device found"));
        }
        let physical_device = device.unwrap();
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
                    khr_external_memory_win32: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        )?;
        let memory_type_index = physical_device.memory_properties().memory_types.iter().position(|i| i.property_flags.contains(MemoryPropertyFlags::DEVICE_LOCAL)).ok_or(anyhow!("no suitable device memory type"))? as u32;
        let queue = queues.next().unwrap();
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let raw = Raw::load(instance, device)?;
        Ok(VkBackend {
            raw,
            queue,
            memory_allocator,
            size: Arc::new(RwLock::new((0, 0))),
            memory_type_index,
            export_memory: Arc::new(Mutex::new(ExportMemory(None))),
        })
    }

    pub fn render(&self) -> Result<(Arc<Image>, Arc<DeviceMemory>)> {
        let size = self.size.read().unwrap();
        let create_info = ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [size.0, size.1, 1],
            usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC | ImageUsage::STORAGE,
            external_memory_handle_types: ExternalMemoryHandleTypes::OPAQUE_WIN32,
            ..Default::default()
        };
        let raw_image = RawImage::new(self.device().clone(), create_info)
            .map_err(|err| anyhow!("backend.rs:error in creating RawImage: {:?}", err) )?;
        let memory = DeviceMemory::allocate(
            self.device().clone(),
            MemoryAllocateInfo {
                allocation_size: raw_image.memory_requirements()[0].layout.size(),
                memory_type_index: self.memory_type_index,
                dedicated_allocation: Some(DedicatedAllocation::Image(&raw_image)),
                export_handle_types: ExternalMemoryHandleTypes::OPAQUE_WIN32,
                ..Default::default()
            },
        ).map_err(|err| anyhow!("backend.rs:error in allocating device memory: {:?}", err) )?;
        let allocation = ResourceMemory::new_dedicated(memory);
        let memory = allocation.device_memory().clone();
        let image = raw_image
            .bind_memory([allocation])
            .map_err(|(err,_,_)| anyhow!("backend.rs:error in binding memory and image: {:?}", err) )?;
        Ok((Arc::new(image), memory))
    }

    pub fn export(&self, memory: Arc<DeviceMemory>) -> Result<isize> {
        let vk_memory = memory.handle();
        let info = MemoryGetWin32HandleInfoKHR {
            memory: vk_memory,
            handle_type: ExternalMemoryHandleTypeFlags::OPAQUE_WIN32,
            ..Default::default()
        };
        let handle = unsafe {
            self.raw.ext_device.get_memory_win32_handle(&info)
                .map_err(|err| anyhow!("backend.rs:error in getting external memory handle: {:?}", err))?
        };
        self.export_memory.lock().unwrap().0 = Some((memory, handle));
        Ok(handle)
    }

    pub fn resize(&self, size: (u32, u32)) {
        let mut guard = self.size.write().unwrap();
        guard.0 = size.0;
        guard.1 = size.1;
    }

    pub fn destory_export_memory(&self) {
        unsafe { self.export_memory.lock().unwrap().destory() }
    }

    #[inline]
    pub fn queue(&self) -> Arc<Queue> {
        self.queue.clone()
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
}

fn check_physical_device(physical_device: Arc<PhysicalDevice>) -> DeviceProperties {
    let mut score = 0u32;
    let mut graphics = true;
    let mut ray_trace = false;
    if physical_device.api_version() >= Version::V1_4 {
        score += 1;
    }
    if physical_device.supported_features().ray_tracing_pipeline {
        score += 1;
        ray_trace = true;
    }
    if !physical_device.supported_extensions().khr_external_memory {
        graphics = false;
    }
    DeviceProperties {
        graphics,
        ray_trace,
        score,
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
