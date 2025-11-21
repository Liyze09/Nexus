mod backend;

use crate::backend::VkBackend;
use jni::objects::JClass;
use jni::sys::{jint, jlong};
use jni::JNIEnv;
use windows_sys::Win32::Foundation::CloseHandle;

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
    Box::into_raw(Box::from(context)) as usize as u64 as i64
}

#[allow(non_snake_case)]
#[unsafe(no_mangle)]
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_render<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
) -> jlong {
    let renderer = unsafe { load_context(ctx) };
    match render_and_export(renderer) {
        Ok(handle) => handle as i64 as jlong,
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
extern "system" fn Java_io_github_liyze09_nexus_NexusClientMain_cleanup<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx:jlong,
) {
    let renderer = unsafe { load_context(ctx) };
    renderer.destory_export_memory()
}

fn render_and_export(vk_backend: VkBackend) -> anyhow::Result<isize> {
    let (_, mem) = vk_backend.render()?;
    vk_backend.export(mem)
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
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    ctx: jlong,
    width: jint,
    height: jint,
) {
    let renderer = unsafe { load_context(ctx) };
    renderer.resize((width as u32, height as u32));
}

unsafe fn load_context(addr: i64) -> VkBackend {
    unsafe {
        let ptr = addr as *mut VkBackend;
        (*ptr).clone()
    }
}
