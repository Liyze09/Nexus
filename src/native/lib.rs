pub mod backend;
pub mod geometry;
pub mod shader;
pub mod texture;

use crate::backend::VkBackend;
use ash::vk::HANDLE;
use jni::objects::ReleaseMode;
use jni::objects::{JClass, JFloatArray, JIntArray, JObjectArray, JString};
use jni::sys::{jint, jlong};
use jni::JNIEnv;
use mimalloc::MiMalloc;
use std::collections::HashMap;
use std::sync::Arc;
use vulkano::image::ImageUsage;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_initNative<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jlong {
    let context = match VkBackend::new() {
        Ok(context) => context,
        Err(err) => {
            env.throw_new(
                String::from("io/github/liyze09/nexus/exception/VulkanException"),
                err.to_string(),
            )
            .unwrap();
            return -1;
        }
    };
    Box::into_raw(Box::from(Arc::new(context))) as usize as u64 as i64
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_getTextureSize<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
) -> jlong {
    let renderer = unsafe { load_context(ctx) };
    match renderer.target().get_value() {
        Ok(target) => target.size as i64 as jlong,
        Err(err) => {
            env.throw_new(
                String::from("io/github/liyze09/nexus/exception/VulkanException"),
                err.to_string(),
            )
            .unwrap();
            -1
        }
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_getGLReady<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
) -> jlong {
    let renderer = unsafe { load_context(ctx) };
    renderer.semaphore().handle_gl_ready as i64 as jlong
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_getGLComplete<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
) -> jlong {
    let renderer = unsafe { load_context(ctx) };
    renderer.semaphore().handle_gl_complete as i64 as jlong
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_close<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
) {
    unsafe {
        drop(Box::from_raw(ctx as *mut VkBackend));
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_resize<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
    width: jint,
    height: jint,
) {
    let renderer = unsafe { load_context(ctx) };
    renderer.resize((width as u32, height as u32));
    match renderer.update() {
        Ok(_) => {}
        Err(err) => {
            env.throw_new(
                String::from("io/github/liyze09/nexus/exception/VulkanException"),
                err.to_string(),
            )
            .unwrap();
        }
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_render<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
) {
    let renderer = unsafe { load_context(ctx) };
    match renderer.render() {
        Ok(_) => {}
        Err(err) => {
            env.throw_new(
                String::from("io/github/liyze09/nexus/exception/VulkanException"),
                err.to_string(),
            )
            .unwrap();
        }
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_getGLTexture<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
) -> jlong {
    let renderer = unsafe { load_context(ctx) };
    match renderer.target().get_value() {
        Ok(target) => target.handle as i64 as jlong,
        Err(err) => {
            env.throw_new(
                String::from("io/github/liyze09/nexus/exception/VulkanException"),
                err.to_string(),
            )
            .unwrap();
            -1
        }
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_acquireVulkanTexture<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
    width: jint,
    height: jint,
    mip_levels: jint,
) -> jlong {
    let renderer = unsafe { load_context(ctx) };
    match renderer.create_external_texture(
        ImageUsage::SAMPLED
            | ImageUsage::TRANSFER_DST
            | ImageUsage::TRANSFER_SRC
            | ImageUsage::COLOR_ATTACHMENT
            | ImageUsage::STORAGE
            | ImageUsage::INPUT_ATTACHMENT,
        (width as u32, height as u32),
        mip_levels as u32,
    ) {
        Ok(target) => target.3 as i64 as jlong,
        Err(err) => {
            env.throw_new(
                String::from("io/github/liyze09/nexus/exception/VulkanException"),
                err.to_string(),
            )
            .unwrap();
            -1
        }
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_getVulkanTextureSize<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
    handle: jlong,
) -> jlong {
    let renderer = unsafe { load_context(ctx) };
    match renderer.get_texture_size_by_handle(handle as HANDLE) {
        Ok(size) => size as i64 as jlong,
        Err(err) => {
            env.throw_new(
                String::from("io/github/liyze09/nexus/exception/VulkanException"),
                err.to_string(),
            )
            .unwrap();
            -1
        }
    }
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_syncAtlas<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
    texture_handle: jlong,
    atlas_name: JString<'local>,
    sprite_names: JObjectArray<'local>,
    sprite_x: JIntArray<'local>,
    sprite_y: JIntArray<'local>,
    sprite_width: JIntArray<'local>,
    sprite_height: JIntArray<'local>,
    sprite_u0: JFloatArray<'local>,
    sprite_v0: JFloatArray<'local>,
    sprite_u1: JFloatArray<'local>,
    sprite_v1: JFloatArray<'local>,
) {
    let renderer = unsafe { load_context(ctx) };

    // Helper function to throw exception and return from function
    fn throw_and_return<'a>(env: &mut JNIEnv<'a>, msg: &str) {
        env.throw_new(
            String::from("java/lang/IllegalArgumentException"),
            msg.to_string(),
        )
        .unwrap();
    }

    // Convert Java string to Rust string
    let atlas_name_str = match env.get_string(&atlas_name) {
        Ok(jstr) => jstr.into(),
        Err(err) => {
            throw_and_return(&mut env, &format!("Failed to get atlas name: {}", err));
            return;
        }
    };

    // Get sprite names array length
    let len = match env.get_array_length(&sprite_names) {
        Ok(len) => len,
        Err(err) => {
            throw_and_return(
                &mut env,
                &format!("Failed to get sprite names length: {}", err),
            );
            return;
        }
    };

    // Check all arrays have same length
    let check_len = |env: &mut JNIEnv<'_>, array: &JIntArray, name: &str| -> bool {
        match env.get_array_length(array) {
            Ok(array_len) if array_len == len => true,
            Ok(array_len) => {
                throw_and_return(
                    env,
                    &format!("Array {} has length {}, expected {}", name, array_len, len),
                );
                false
            }
            Err(err) => {
                throw_and_return(
                    env,
                    &format!("Failed to get array {} length: {}", name, err),
                );
                false
            }
        }
    };

    let check_float_len = |env: &mut JNIEnv<'_>, array: &JFloatArray, name: &str| -> bool {
        match env.get_array_length(array) {
            Ok(array_len) if array_len == len => true,
            Ok(array_len) => {
                throw_and_return(
                    env,
                    &format!("Array {} has length {}, expected {}", name, array_len, len),
                );
                false
            }
            Err(err) => {
                throw_and_return(
                    env,
                    &format!("Failed to get array {} length: {}", name, err),
                );
                false
            }
        }
    };

    if !check_len(&mut env, &sprite_x, "sprite_x")
        || !check_len(&mut env, &sprite_y, "sprite_y")
        || !check_len(&mut env, &sprite_width, "sprite_width")
        || !check_len(&mut env, &sprite_height, "sprite_height")
        || !check_float_len(&mut env, &sprite_u0, "sprite_u0")
        || !check_float_len(&mut env, &sprite_v0, "sprite_v0")
        || !check_float_len(&mut env, &sprite_u1, "sprite_u1")
        || !check_float_len(&mut env, &sprite_v1, "sprite_v1")
    {
        return;
    }

    // Get sprite names (object array)
    let mut sprite_names_vec: Vec<String> = Vec::with_capacity(len as usize);
    for i in 0..len {
        match env.get_object_array_element(&sprite_names, i) {
            Ok(jstr_obj) => match env.get_string(&jstr_obj.into()) {
                Ok(rust_str) => sprite_names_vec.push(rust_str.into()),
                Err(err) => {
                    throw_and_return(
                        &mut env,
                        &format!("Failed to get sprite name at index {}: {}", i, err),
                    );
                    return;
                }
            },
            Err(err) => {
                throw_and_return(
                    &mut env,
                    &format!("Failed to get sprite name object at index {}: {}", i, err),
                );
                return;
            }
        }
    }

    // Helper functions to get primitive array data
    let get_int_array = |env: &mut JNIEnv, array: &JIntArray, name: &str| -> Option<Vec<i32>> {
        // Use ReleaseMode::NoCopyBack since we only read the data
        match unsafe { env.get_array_elements(array, ReleaseMode::NoCopyBack) } {
            Ok(elements) => {
                let vec: Vec<i32> = elements.iter().copied().collect();
                Some(vec)
            }
            Err(err) => {
                throw_and_return(
                    env,
                    &format!("Failed to get {} array elements: {}", name, err),
                );
                None
            }
        }
    };

    let get_float_array = |env: &mut JNIEnv, array: &JFloatArray, name: &str| -> Option<Vec<f32>> {
        // Use ReleaseMode::NoCopyBack since we only read the data
        match unsafe { env.get_array_elements(array, ReleaseMode::NoCopyBack) } {
            Ok(elements) => {
                let vec: Vec<f32> = elements.iter().copied().collect();
                Some(vec)
            }
            Err(err) => {
                throw_and_return(
                    env,
                    &format!("Failed to get {} array elements: {}", name, err),
                );
                None
            }
        }
    };

    let sprite_x_vec = match get_int_array(&mut env, &sprite_x, "sprite_x") {
        Some(vec) => vec,
        None => return,
    };
    let sprite_y_vec = match get_int_array(&mut env, &sprite_y, "sprite_y") {
        Some(vec) => vec,
        None => return,
    };
    let sprite_width_vec = match get_int_array(&mut env, &sprite_width, "sprite_width") {
        Some(vec) => vec,
        None => return,
    };
    let sprite_height_vec = match get_int_array(&mut env, &sprite_height, "sprite_height") {
        Some(vec) => vec,
        None => return,
    };
    let sprite_u0_vec = match get_float_array(&mut env, &sprite_u0, "sprite_u0") {
        Some(vec) => vec,
        None => return,
    };
    let sprite_v0_vec = match get_float_array(&mut env, &sprite_v0, "sprite_v0") {
        Some(vec) => vec,
        None => return,
    };
    let sprite_u1_vec = match get_float_array(&mut env, &sprite_u1, "sprite_u1") {
        Some(vec) => vec,
        None => return,
    };
    let sprite_v1_vec = match get_float_array(&mut env, &sprite_v1, "sprite_v1") {
        Some(vec) => vec,
        None => return,
    };

    // Build sprite map
    let mut sprite_map = HashMap::new();
    for i in 0..len as usize {
        let sprite_info = crate::texture::SpriteInfo {
            name: sprite_names_vec[i].clone(),
            x: sprite_x_vec[i] as u32,
            y: sprite_y_vec[i] as u32,
            width: sprite_width_vec[i] as u32,
            height: sprite_height_vec[i] as u32,
            u0: sprite_u0_vec[i],
            v0: sprite_v0_vec[i],
            u1: sprite_u1_vec[i],
            v1: sprite_v1_vec[i],
        };
        sprite_map.insert(sprite_names_vec[i].clone(), sprite_info);
    }

    // Sync atlas with renderer
    renderer.sync_atlas(
        match renderer.get_exported_image_by_handle(texture_handle as HANDLE) {
            Ok(exported_image) => exported_image.image,
            Err(err) => {
                throw_and_return(
                    &mut env,
                    &format!("Failed to get texture by handle: {}", err),
                );
                return;
            }
        },
        atlas_name_str,
        sprite_map,
    );
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_uploadChunkMesh<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
    chunk_x: jint,
    chunk_z: jint,
    vertices: JFloatArray<'local>,
    indices: JIntArray<'local>,
) {
    let renderer = unsafe { load_context(ctx) };

    // Helper function to throw exception and return from function
    fn throw_and_return<'a>(env: &mut JNIEnv<'a>, msg: &str) {
        env.throw_new(
            String::from("java/lang/IllegalArgumentException"),
            msg.to_string(),
        )
        .unwrap();
    }

    // Get vertices array length
    let _vertices_len = match env.get_array_length(&vertices) {
        Ok(len) => len,
        Err(err) => {
            throw_and_return(
                &mut env,
                &format!("Failed to get vertices array length: {}", err),
            );
            return;
        }
    };

    // Get indices array length
    let _indices_len = match env.get_array_length(&indices) {
        Ok(len) => len,
        Err(err) => {
            throw_and_return(
                &mut env,
                &format!("Failed to get indices array length: {}", err),
            );
            return;
        }
    };

    // Helper function to get float array data
    let get_float_array = |env: &mut JNIEnv, array: &JFloatArray, name: &str| -> Option<Vec<f32>> {
        // Use ReleaseMode::NoCopyBack since we only read the data
        match unsafe { env.get_array_elements(array, ReleaseMode::NoCopyBack) } {
            Ok(elements) => {
                let vec: Vec<f32> = elements.iter().map(|&x| x).collect();
                Some(vec)
            }
            Err(err) => {
                throw_and_return(
                    env,
                    &format!("Failed to get {} array elements: {}", name, err),
                );
                None
            }
        }
    };

    // Helper function to get int array data
    let get_int_array = |env: &mut JNIEnv, array: &JIntArray, name: &str| -> Option<Vec<i32>> {
        // Use ReleaseMode::NoCopyBack since we only read the data
        match unsafe { env.get_array_elements(array, ReleaseMode::NoCopyBack) } {
            Ok(elements) => {
                let vec: Vec<i32> = elements.iter().map(|&x| x).collect();
                Some(vec)
            }
            Err(err) => {
                throw_and_return(
                    env,
                    &format!("Failed to get {} array elements: {}", name, err),
                );
                None
            }
        }
    };

    let vertices_vec = match get_float_array(&mut env, &vertices, "vertices") {
        Some(vec) => vec,
        None => return,
    };

    let indices_vec = match get_int_array(&mut env, &indices, "indices") {
        Some(vec) => vec,
        None => return,
    };

    // Convert vertices to bytes (f32 to u8)
    let vertices_bytes: Vec<u8> = vertices_vec
        .iter()
        .flat_map(|&f| f.to_ne_bytes().to_vec())
        .collect();

    let index_data: Vec<u32> = indices_vec.iter().map(|&i| i as u32).collect();

    // Create geometry data
    let geometry_data = crate::geometry::GeometryData::from_triangles(vertices_bytes, index_data);

    // Add to geometry manager for processing
    renderer.upload_chunk_mesh(chunk_x, chunk_z, geometry_data);
}

unsafe fn load_context(addr: i64) -> Arc<VkBackend> {
    unsafe {
        let ptr = addr as *mut Arc<VkBackend>;
        (*ptr).clone()
    }
}
