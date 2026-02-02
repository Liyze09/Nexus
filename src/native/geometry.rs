use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

use anyhow::{Ok, Result};
use dashmap::DashMap;
use meshopt::{optimize_overdraw_in_place, optimize_vertex_cache_in_place, VertexDataAdapter};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use smallvec::smallvec;
use vulkano::{
    acceleration_structure::{
        AccelerationStructure, AccelerationStructureBuildGeometryInfo,
        AccelerationStructureCreateInfo, AccelerationStructureGeometries,
        AccelerationStructureGeometryTrianglesData, AccelerationStructureType,
    },
    buffer::{
        allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo},
        BufferUsage, Subbuffer,
    },
    command_buffer::{self},
    memory::allocator::{MemoryTypeFilter, StandardMemoryAllocator},
    sync::{self, GpuFuture},
};
use vulkano::{
    acceleration_structure::{
        AccelerationStructureBuildRangeInfo, AccelerationStructureBuildType,
        AccelerationStructureGeometryAabbsData,
    },
    buffer::IndexBuffer,
    format::Format,
};

use crate::backend::VkBackend;

pub struct GeometryManager {
    incremental_id: AtomicU64,
    blas: DashMap<u64, BuiltGeometry>,
    temp_geometry: Mutex<Vec<(u64, GeometryData)>>,
    device_subbuffer_alloc: Mutex<SubbufferAllocator>,
}

impl GeometryManager {
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>) -> Self {
        Self {
            incremental_id: AtomicU64::new(0),
            blas: DashMap::new(),
            temp_geometry: Mutex::new(Vec::new()),
            device_subbuffer_alloc: Mutex::new(SubbufferAllocator::new(
                memory_allocator.clone(),
                SubbufferAllocatorCreateInfo {
                    buffer_usage: BufferUsage::TRANSFER_DST
                        | BufferUsage::ACCELERATION_STRUCTURE_BUILD_INPUT_READ_ONLY
                        | BufferUsage::ACCELERATION_STRUCTURE_STORAGE
                        | BufferUsage::SHADER_DEVICE_ADDRESS
                        | BufferUsage::STORAGE_BUFFER,
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
            )),
        }
    }

    pub fn add_temporary_geometry(&self, geometry_data: GeometryData) -> u64 {
        let id = self.incremental_id.fetch_add(1, Ordering::AcqRel);
        self.temp_geometry.lock().unwrap().push((id, geometry_data));
        id
    }

    pub fn build_all(&self, backend: &VkBackend) -> Result<Box<dyn GpuFuture>> {
        let mut temp = self.temp_geometry.lock().unwrap();
        let temp_geometry = std::mem::take(&mut *temp)
            .into_par_iter()
            .map(|(id, geometry_data)| (id, self.optimize_triangle_geometry(geometry_data)))
            .collect::<Vec<_>>();
        drop(temp);
        let mut buffers = Vec::with_capacity(temp_geometry.len());
        let device_subbuffer_alloc = self.device_subbuffer_alloc.lock().unwrap();
        let mut command_buffer_builder_copy = command_buffer::AutoCommandBufferBuilder::primary(
            backend.command_buffer_allocator().clone(),
            backend.queue().transfer.queue_family_index(),
            command_buffer::CommandBufferUsage::OneTimeSubmit,
        )?;
        {
            for entry in temp_geometry.into_iter() {
                let (id, geometry_data) = entry;

                let (buffer, index_buffer) = (
                    geometry_data.buffer.clone(),
                    geometry_data.index_buffer.clone(),
                );
                let device_buffer: Subbuffer<[u8]> =
                    device_subbuffer_alloc.allocate_slice(buffer.len() as vulkano::DeviceSize)?;
                let device_index_buffer: Option<IndexBuffer> = match &*index_buffer {
                    Some(index_buffer) => {
                        let device_index_buffer: Subbuffer<[u32]> = device_subbuffer_alloc
                            .allocate_slice(index_buffer.len() as vulkano::DeviceSize)?;
                        backend.transfer_data(
                            index_buffer,
                            device_index_buffer.clone(),
                            &mut command_buffer_builder_copy,
                        )?;
                        Some(IndexBuffer::U32(device_index_buffer))
                    }
                    None => None,
                };
                backend.transfer_data(
                    &buffer,
                    device_buffer.clone(),
                    &mut command_buffer_builder_copy,
                )?;
                buffers.push((id, (device_buffer, device_index_buffer), geometry_data));
            }
        }

        if !backend.device_properties().ray_trace {
            for (id, device_buffer, geometry_data) in buffers {
                self.blas.insert(
                    id,
                    BuiltGeometry {
                        acceleration_structure: None,
                        geometry_type: geometry_data.geometry_type,
                        triangle_vertex: Some(device_buffer.0.clone()),
                        triangle_index: device_buffer.1.clone(),
                    },
                );
            }
            let future = sync::now(backend.device().clone())
                .then_execute(
                    backend.queue().transfer.clone(),
                    command_buffer_builder_copy.build()?,
                )?
                .then_signal_fence_and_flush()?;
            Ok(Box::new(future))
        } else {
            let mut command_buffer_builder_build =
                command_buffer::AutoCommandBufferBuilder::primary(
                    backend.command_buffer_allocator().clone(),
                    backend.queue().compute.queue_family_index(),
                    command_buffer::CommandBufferUsage::OneTimeSubmit,
                )?;
            let mut build_infos = Vec::new();
            let mut scratch_buffer_max_size = 0;
            for (id, device_buffer, geometry_data) in buffers {
                let primitive_count;
                let first_vertex = 0;
                let transform_offset = 0;
                let geometry = match geometry_data.geometry_type {
                    GeometryType::Triangles => {
                        AccelerationStructureGeometries::Triangles(vec![{
                            let mut data = AccelerationStructureGeometryTrianglesData::new(
                                Format::R32G32B32_SFLOAT,
                            );
                            data.vertex_data = Some(device_buffer.0.clone());
                            data.vertex_stride = 12;
                            data.max_vertex =
                                (device_buffer.0.size() as u32 / data.vertex_stride) - 1;
                            if let Some(index_buffer) = &device_buffer.1 {
                                data.index_data = Some(index_buffer.clone());
                            }
                            primitive_count =
                                device_buffer.0.size() as u32 / (3 * data.vertex_stride);
                            // TODO: transform data
                            data
                        }])
                    }
                    GeometryType::AABBs => AccelerationStructureGeometries::Aabbs(vec![{
                        let data = AccelerationStructureGeometryAabbsData {
                            data: Some(device_buffer.0.clone()),
                            stride: 24,
                            ..Default::default()
                        };
                        primitive_count = device_buffer.0.size() as u32 / data.stride;
                        data
                    }]),
                };
                let mut geometry_info = AccelerationStructureBuildGeometryInfo::new(geometry);
                geometry_info.scratch_data = None;
                let build_size = backend.device().acceleration_structure_build_sizes(
                    AccelerationStructureBuildType::Device,
                    &geometry_info,
                    &[primitive_count],
                )?;
                let scratch_size = build_size.build_scratch_size;
                scratch_buffer_max_size = scratch_buffer_max_size.max(scratch_size);
                let blas = {
                    let mut info = AccelerationStructureCreateInfo::new(
                        device_subbuffer_alloc
                            .allocate_slice(build_size.acceleration_structure_size)?,
                    );
                    info.ty = AccelerationStructureType::BottomLevel;
                    unsafe { AccelerationStructure::new(backend.device().clone(), info)? }
                };
                geometry_info.dst_acceleration_structure = Some(blas.clone());
                self.blas.insert(
                    id,
                    BuiltGeometry {
                        acceleration_structure: Some(blas.clone()),
                        geometry_type: geometry_data.geometry_type,
                        triangle_vertex: Some(device_buffer.0.clone()),
                        triangle_index: device_buffer.1.clone(),
                    },
                );
                build_infos.push((
                    geometry_info,
                    smallvec![AccelerationStructureBuildRangeInfo {
                        primitive_count,
                        primitive_offset: 0,
                        first_vertex,
                        transform_offset,
                    }],
                ));
            }
            let scratch_buffer = device_subbuffer_alloc.allocate_slice(scratch_buffer_max_size)?;
            for (mut info, ranges) in build_infos {
                info.scratch_data = Some(scratch_buffer.clone());
                unsafe {
                    command_buffer_builder_build.build_acceleration_structure(info, ranges)?;
                }
            }
            let command_buffer_copy = command_buffer_builder_copy.build()?;
            let command_buffer_build = command_buffer_builder_build.build()?;
            let future = sync::now(backend.device().clone())
                .then_execute(backend.queue().transfer.clone(), command_buffer_copy)?
                .then_signal_semaphore_and_flush()?
                .then_execute(backend.queue().compute.clone(), command_buffer_build)?
                .then_signal_fence_and_flush()?;
            Ok(Box::new(future))
        }
    }

    /// Optimize triangle geometry using meshopt library
    pub fn optimize_triangle_geometry(&self, geometry_data: GeometryData) -> GeometryData {
        if geometry_data.geometry_type != GeometryType::Triangles {
            return geometry_data;
        }
        // Convert indices to u32 indices for optimization
        let mut index_buffer = match &*geometry_data.index_buffer {
            Some(index_data) => index_data.clone(),
            None => return geometry_data,
        };

        let vertex_count = geometry_data.buffer.len() / 12;
        optimize_vertex_cache_in_place(index_buffer.as_mut_slice(), vertex_count);
        let vertex_data_adapter =
            match VertexDataAdapter::new(geometry_data.buffer.as_slice(), 12, 0) {
                std::result::Result::Ok(adapter) => adapter,
                Err(_) => return geometry_data,
            };
        optimize_overdraw_in_place(&mut index_buffer, &vertex_data_adapter, 1.05);
        GeometryData::new(
            GeometryType::Triangles,
            geometry_data.buffer,
            Arc::new(Some(index_buffer)),
        )
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum GeometryType {
    Triangles,
    AABBs,
}

#[derive(Clone)]
pub struct GeometryData {
    pub geometry_type: GeometryType,
    pub buffer: Arc<Vec<u8>>,
    pub index_buffer: Arc<Option<Vec<u32>>>,
}

impl GeometryData {
    #[inline]
    pub fn new(
        geometry_type: GeometryType,
        buffer: Arc<Vec<u8>>,
        index_buffer: Arc<Option<Vec<u32>>>,
    ) -> Self {
        Self {
            geometry_type,
            buffer,
            index_buffer,
        }
    }

    pub fn from_triangles(buffer: Vec<u8>, index_buffer: Vec<u32>) -> Self {
        Self::new(
            GeometryType::Triangles,
            Arc::new(buffer),
            Arc::new(Some(index_buffer)),
        )
    }

    pub fn from_aabbs(buffer: Vec<u8>) -> Self {
        Self::new(GeometryType::AABBs, Arc::new(buffer), Arc::new(None))
    }
}

#[derive(Clone)]
pub struct BuiltGeometry {
    pub acceleration_structure: Option<Arc<AccelerationStructure>>,
    pub geometry_type: GeometryType,
    pub triangle_vertex: Option<Subbuffer<[u8]>>,
    pub triangle_index: Option<IndexBuffer>,
}
