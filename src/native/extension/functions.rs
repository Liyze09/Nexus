use wasmtime::component::{LinkerInstance, TypedFunc, WasmStr};

use crate::extension::wasm::ExtensionContext;

pub(crate) fn register_host_functions<'a>(ark:&mut LinkerInstance<'a, ExtensionContext>) -> anyhow::Result<()> {
            ark.func_wrap("register",  |store, params: (String, String) | {
                let id = store.data().package.manifest.id.clone();
                let registry = store.data().public_registry.clone();
                let instance = store.data().instance.ok_or(wasmtime::Error::msg("Instance not found"))?;
                let fun: TypedFunc<(), ()>= instance.get_typed_func(store, params.1)?;
                registry.lock().unwrap().insert(params.0, (fun, id));
                Ok(())
            })?;

            ark.func_wrap("info", |store, params: (WasmStr,) | {
                log::info!("[ark-ext-{}] {}", &store.data().package.manifest.id,  params.0.to_str(&store)?);
                Ok(())
            })?;

            ark.func_wrap("warn", |store, params: (WasmStr,) | {
                log::warn!("[ark-ext-{}] {}", &store.data().package.manifest.id,  params.0.to_str(&store)?);
                Ok(())
            })?;

            ark.func_wrap("error", |store, params: (WasmStr,) | {
                log::error!("[ark-ext-{}] {}", &store.data().package.manifest.id,  params.0.to_str(&store)?);
                Ok(())
            })?;

            ark.func_wrap("debug", |store, params: (WasmStr,) | {
                log::debug!("[ark-ext-{}] {}", &store.data().package.manifest.id,  params.0.to_str(&store)?);
                Ok(())
            })?;

            ark.func_wrap("trace", |store, params: (WasmStr,) | {
                log::trace!("[ark-ext-{}] {}", &store.data().package.manifest.id,  params.0.to_str(&store)?);
                Ok(())
            })?;
            Ok(())
}
