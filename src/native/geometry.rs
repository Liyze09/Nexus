use std::sync::{Arc, Mutex, atomic::{AtomicU64, Ordering}};

use anyhow::{Ok, Result};
use vulkano::{acceleration_structure::{AccelerationStructureBuildRangeInfo, AccelerationStructureBuildType, AccelerationStructureGeometryAabbsData}, format::Format};
use dashmap::DashMap;
use vulkano::{acceleration_structure::{AccelerationStructure, AccelerationStructureBuildGeometryInfo, AccelerationStructureCreateInfo, AccelerationStructureGeometries, AccelerationStructureGeometryTrianglesData, AccelerationStructureType}, buffer::{BufferUsage, Subbuffer, allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo}}, command_buffer::{self, CopyBufferInfo}, memory::allocator::{MemoryTypeFilter, StandardMemoryAllocator}, sync::{self, GpuFuture}};
use smallvec::smallvec;

use crate::backend::VkBackend;


pub struct GeometryManager {
    incremental_id: AtomicU64,
    blas: DashMap<u64, Arc<AccelerationStructure>>,
    temp_geometry: Mutex<Vec<Arc<(u64, Vec<u8>, GeometryType)>>>,
    host_subbuffer_alloc: Mutex<SubbufferAllocator>,
    device_subbuffer_alloc: Mutex<SubbufferAllocator>,
}

impl GeometryManager {
    pub fn new(allocator: Arc<StandardMemoryAllocator>) -> Self {
        Self {
            incremental_id: AtomicU64::new(0),
            blas: DashMap::new(),
            temp_geometry: Mutex::new(Vec::new()),
            host_subbuffer_alloc: Mutex::new(SubbufferAllocator::new(allocator.clone(),
        SubbufferAllocatorCreateInfo {
            buffer_usage: BufferUsage::TRANSFER_SRC,
            memory_type_filter: MemoryTypeFilter::PREFER_HOST,
            ..Default::default()
        })),
            device_subbuffer_alloc: Mutex::new(SubbufferAllocator::new(allocator.clone(),
        SubbufferAllocatorCreateInfo {
            buffer_usage: BufferUsage::TRANSFER_DST | BufferUsage::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY | BufferUsage::ACCELERATION_STRUCTURE_STORAGE | BufferUsage::SHADER_DEVICE_ADDRESS | BufferUsage::STORAGE_BUFFER,
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
            ..Default::default()
        }))
        }
    }
    
    pub fn add_temporary_geometry(&self, buffer: Vec<u8>, geometry_type: GeometryType) -> u64 {
        let id = self.incremental_id.fetch_add(1, Ordering::AcqRel);
        self.temp_geometry.lock().unwrap().push(Arc::new((id, buffer, geometry_type)));
        id
    }

    pub fn build(&self, backend: &VkBackend) -> Result<impl GpuFuture> {
        let temp_geometry = self.temp_geometry.lock().unwrap();
        let mut buffers = Vec::with_capacity(temp_geometry.len());
        let device_subbuffer_alloc = self.device_subbuffer_alloc.lock().unwrap();
        let mut command_buffer_builder_copy = command_buffer::AutoCommandBufferBuilder::primary(
                backend.command_buffer_allocator(),
                backend.queue().transfer.queue_family_index(),
                command_buffer::CommandBufferUsage::OneTimeSubmit,
        )?;
        {
            let host_subbuffer_alloc = self.host_subbuffer_alloc.lock().unwrap();
            
            
            for entry in temp_geometry.iter() {
                let (id, buffer, geometry_type) = (**entry).clone();
                let host_buffer = host_subbuffer_alloc.allocate_slice(buffer.len() as vulkano::DeviceSize)?;
                {
                    let mut host_content = host_buffer.write()?;
                    host_content.copy_from_slice(&buffer);
                }
                let device_buffer: Subbuffer<[u8]> = device_subbuffer_alloc.allocate_slice(buffer.len() as vulkano::DeviceSize)?;
                command_buffer_builder_copy.copy_buffer(CopyBufferInfo::buffers(host_buffer, device_buffer.clone()))?;
                buffers.push((id, device_buffer, geometry_type));
            }
            drop(temp_geometry);
        }
        let mut command_buffer_builder_build = command_buffer::AutoCommandBufferBuilder::primary(
            backend.command_buffer_allocator(),
            backend.queue().compute.queue_family_index(),
            command_buffer::CommandBufferUsage::OneTimeSubmit,
        )?;
        let mut build_infos = Vec::new();
        let mut scratch_buffer_max_size = 0;
        for (id, device_buffer, geometry_type) in buffers {
            let mut info = AccelerationStructureCreateInfo::new(
                    device_buffer.clone()
            );
            info.ty = AccelerationStructureType::BottomLevel;
            let blas = unsafe { AccelerationStructure::new(
                backend.device(),
                info,
            )?};
            self.blas.insert(id, blas.clone());
            let primitive_count;
            let first_vertex = 0;
            let transform_offset = 0;
            let geometry = match geometry_type {
                GeometryType::Triangles => {
                    AccelerationStructureGeometries::Triangles(vec![
                        {
                            let mut data = AccelerationStructureGeometryTrianglesData::new(Format::R32G32B32_SFLOAT);
                            data.vertex_data = Some(device_buffer.clone());
                            data.vertex_stride = 12;
                            data.max_vertex = (device_buffer.size() as u32 / data.vertex_stride) - 1;
                            primitive_count = device_buffer.size() as u32 / (3 * data.vertex_stride);
                            // TODO: Index and transform data
                            data
                        }])
                },
                GeometryType::AABBs => {
                    AccelerationStructureGeometries::Aabbs(vec![
                        {
                            let mut data = AccelerationStructureGeometryAabbsData::default();
                            data.data = Some(device_buffer.clone());
                            data.stride = 24;
                            primitive_count = device_buffer.size() as u32 / data.stride;
                            data
                        }])
                },
            };
            let mut info = AccelerationStructureBuildGeometryInfo::new(
                        geometry
                    );
            info.dst_acceleration_structure = Some(blas.clone());
            info.scratch_data = None;
            let scratch_size = backend.device().acceleration_structure_build_sizes(AccelerationStructureBuildType::Device, &info, &[primitive_count])?.build_scratch_size;
            scratch_buffer_max_size = scratch_buffer_max_size.max(scratch_size);
            build_infos.push((info, smallvec![AccelerationStructureBuildRangeInfo {
                primitive_count,
                primitive_offset: 0,
                first_vertex,
                transform_offset,
            }]));
            
        }
        for (mut info, ranges) in build_infos {
            info.scratch_data = Some(device_subbuffer_alloc.allocate_slice(scratch_buffer_max_size)?);
            unsafe {
                command_buffer_builder_build.build_acceleration_structure(
                    info, 
                    ranges
                )?;
            }
        }
        let command_buffer_copy = command_buffer_builder_copy.build()?;
        let command_buffer_build = command_buffer_builder_build.build()?;
        let future = sync::now(backend.device())
            .then_execute(backend.queue().transfer.clone(), command_buffer_copy)?
            .then_signal_semaphore_and_flush()?
            .then_execute(backend.queue().compute.clone(), command_buffer_build)?
            .then_signal_fence_and_flush()?;
        Ok(future)
    }
}

#[derive(Clone)]
pub enum GeometryType {
    Triangles,
    AABBs,
}
