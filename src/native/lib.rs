pub mod backend;
pub mod shader;
pub mod geometry;
pub mod texture;

use std::sync::Arc;
use crate::backend::VkBackend;
use ash::vk::HANDLE;
use jni::objects::JClass;
use jni::sys::{jint, jlong};
use jni::JNIEnv;
use vulkano::image::ImageUsage;

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
extern "system" fn  Java_io_github_liyze09_nexus_NexusClientMain_getGLReady<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
) -> jlong {
    let renderer = unsafe { load_context(ctx) };
    renderer.semaphore().handle_gl_ready as i64 as jlong
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn  Java_io_github_liyze09_nexus_NexusClientMain_getGLComplete<'local>(
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
        Ok(_) => {},
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
        Ok(_) => {},
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
        ImageUsage::SAMPLED | ImageUsage::TRANSFER_DST | ImageUsage::TRANSFER_SRC | ImageUsage::COLOR_ATTACHMENT | ImageUsage::STORAGE | ImageUsage::INPUT_ATTACHMENT,
        (width as u32, height as u32),
        mip_levels as u32) {
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

unsafe fn load_context(addr: i64) -> Arc<VkBackend> {
    unsafe {
        let ptr = addr as *mut Arc<VkBackend>;
        (*ptr).clone()
    }
}
