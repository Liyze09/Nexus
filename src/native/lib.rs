pub mod backend;
pub mod shader;
pub mod geometry;

use std::sync::Arc;
use crate::backend::VkBackend;
use jni::objects::JClass;
use jni::sys::{jint, jlong};
use jni::JNIEnv;

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
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_refresh<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
) -> jlong {
    let renderer = unsafe { load_context(ctx) };
    renderer.state().handle_error(
        |e| {
            env.throw_new(
                String::from("io/github/liyze09/nexus/exception/VulkanException"),
                e.to_string(),
            )
                .unwrap();
        }
    );
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
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_startRendering<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong
) {

    let renderer = unsafe { load_context(ctx) };
    renderer.start_rendering_thread();
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_endRendering<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong
) {

    let renderer = unsafe { load_context(ctx) };
    renderer.end_rendering_thread();
}

unsafe fn load_context(addr: i64) -> Arc<VkBackend> {
    unsafe {
        let ptr = addr as *mut Arc<VkBackend>;
        (*ptr).clone()
    }
}
