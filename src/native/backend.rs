use anyhow::{anyhow, Result};
use ash::vk::{DeviceSize, ExternalMemoryHandleTypeFlags, MemoryGetWin32HandleInfoKHR};
use std::sync::Arc;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, Queue, QueueCreateInfo, QueueFlags};
use vulkano::format::Format;
use vulkano::image::sys::RawImage;
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateFlags, InstanceCreateInfo};
use vulkano::memory::allocator::{
    AllocationCreateInfo, AllocationType, MemoryAllocator, MemoryTypeFilter,
    StandardMemoryAllocator,
};
use vulkano::memory::{
    DedicatedAllocation, DeviceMemory, ExternalMemoryHandleTypes, MemoryAllocateInfo,
    ResourceMemory,
};
use vulkano::{Version, VulkanLibrary, VulkanObject};

#[derive(Clone)]
pub struct VkBackend {
    raw: Raw,
    queue: Arc<Queue>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    size: (u32, u32),
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
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )?;
        let queue = queues.next().unwrap();
        let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let raw = Raw::load(instance, device)?;
        Ok(VkBackend {
            raw,
            queue,
            memory_allocator,
            size: (0, 0),
        })
    }

    pub fn render(&self) -> Result<(Arc<Image>, Arc<DeviceMemory>)> {
        let create_info = ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_UNORM,
            extent: [self.size.0, self.size.1, 1],
            usage: ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC | ImageUsage::STORAGE,
            external_memory_handle_types: ExternalMemoryHandleTypes::OPAQUE_WIN32,
            ..Default::default()
        };
        let raw_image = RawImage::new(self.device().clone(), create_info)?;
        let memory = self.memory_allocator.allocate(
            raw_image.memory_requirements()[0],
            AllocationType::Unknown,
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                ..Default::default()
            },
            Some(DedicatedAllocation::Image(&raw_image)),
        )?;
        let memory = DeviceMemory::allocate(
            self.device().clone(),
            MemoryAllocateInfo {
                allocation_size: (4 * self.size.0 * self.size.1) as DeviceSize,
                memory_type_index: 0,
                dedicated_allocation: Some(DedicatedAllocation::Image(&raw_image)),
                export_handle_types: ExternalMemoryHandleTypes::OPAQUE_WIN32,
                ..Default::default()
            },
        )?;
        let allocation = ResourceMemory::new_dedicated(memory);
        let memory = allocation.device_memory().clone();
        let image = raw_image
            .bind_memory([allocation])
            .map_err(|(err, _, _)| err)?;
        Ok((Arc::new(image), memory))
    }

    pub fn export(&self, memory: Arc<DeviceMemory>) -> Result<isize> {
        let vk_memory = memory.handle();
        let info = MemoryGetWin32HandleInfoKHR {
            memory: vk_memory,
            handle_type: ExternalMemoryHandleTypeFlags::OPAQUE_WIN32,
            ..Default::default()
        };
        unsafe { Ok(self.raw.ext_device.get_memory_win32_handle(&info)?) }
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
