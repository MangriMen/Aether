use std::future::Future;

/// Runs the async work of a host function to completion.
///
/// Host functions run inside a plugin call on tokio's blocking pool (`PluginHandle`), so
/// blocking the thread holds up no other task. `block_in_place` would panic on a
/// current-thread runtime.
pub(super) fn block_on<F: Future>(future: F) -> F::Output {
    tokio::runtime::Handle::current().block_on(future)
}

#[cfg(test)]
mod tests {
    use std::{
        sync::{Arc, Mutex, OnceLock},
        time::Duration,
    };

    use extism::{Function, Manifest, PluginBuilder, UserData, Wasm};

    use super::block_on;
    use crate::features::instance::PluginId;
    use crate::features::plugins::{
        PluginCallError, PluginError, PluginHandle, infra::extism::models::ExtismPluginInstance,
    };

    /// Exports `run`, which calls the host function `wait` once.
    const PLUGIN: &str = r#"(module
        (import "extism:host/user" "wait" (func $wait (result i64)))
        (func (export "run") (result i32)
            (drop (call $wait))
            (i32.const 0)))"#;

    /// A host function whose async work needs the runtime's timer.
    fn waiting_plugin() -> PluginHandle {
        let wait = Function::new(
            "wait",
            [],
            [extism::ValType::I64],
            UserData::new(()),
            |_, _, outputs, _| {
                // Stands for a host function awaiting a timer, which needs the runtime's time driver.
                block_on(tokio::time::sleep(Duration::from_millis(10)));
                outputs[0] = extism::Val::I64(0);
                Ok(())
            },
        );
        let manifest = Manifest::new([Wasm::data(PLUGIN.as_bytes().to_vec())]);
        let plugin = PluginBuilder::new(manifest)
            .with_functions([wait])
            .build()
            .expect("the test plugin should load");

        PluginHandle::new(ExtismPluginInstance::new(
            plugin,
            PluginId::from_static("test"),
        ))
    }

    #[tokio::test]
    async fn should_complete_host_work_when_running_on_a_current_thread_runtime() {
        let plugin = waiting_plugin();

        let result = plugin
            .run("run", |instance| {
                instance.call_bytes("run", &[]).map(<[u8]>::len)
            })
            .await
            .flatten();

        assert!(result.is_ok(), "the call should succeed: {result:?}");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn should_complete_host_work_when_running_on_a_multi_thread_runtime() {
        let plugin = waiting_plugin();

        let result = plugin
            .run("run", |instance| {
                instance.call_bytes("run", &[]).map(<[u8]>::len)
            })
            .await
            .flatten();

        assert!(result.is_ok(), "the call should succeed: {result:?}");
    }

    /// Exports `run`, whose host function `call_self` calls back into the same plugin, and
    /// `noop`, which that call back asks for.
    const SELF_CALLING_PLUGIN: &str = r#"(module
        (import "extism:host/user" "call_self" (func $call_self (result i64)))
        (func (export "run") (result i32)
            (drop (call $call_self))
            (i32.const 0))
        (func (export "noop") (result i32)
            (i32.const 0)))"#;

    /// A plugin calling itself back through a host function, the way a host function that
    /// reaches the plugin's own provider would. Without the refusal the call back waits for
    /// the call it is part of, forever.
    #[test]
    fn should_return_an_error_not_deadlock_when_a_host_function_calls_its_own_plugin() {
        let handle = Arc::new(OnceLock::<PluginHandle>::new());
        let call_back = Arc::new(Mutex::new(None));

        let call_self = Function::new(
            "call_self",
            [],
            [extism::ValType::I64],
            UserData::new(()),
            {
                let handle = Arc::clone(&handle);
                let call_back = Arc::clone(&call_back);
                move |_, _, outputs, _| {
                    let plugin = handle.get().expect("the handle is set before the call");
                    let result = block_on(plugin.run("noop", |instance| {
                        instance.call_bytes("noop", &[]).map(|_| ())
                    }))
                    .flatten();
                    *call_back.lock().unwrap() = Some(result);
                    outputs[0] = extism::Val::I64(0);
                    Ok(())
                }
            },
        );
        let manifest = Manifest::new([Wasm::data(SELF_CALLING_PLUGIN.as_bytes().to_vec())]);
        let plugin = PluginBuilder::new(manifest)
            .with_functions([call_self])
            .build()
            .expect("the test plugin should load");
        let plugin = PluginHandle::new(ExtismPluginInstance::new(
            plugin,
            PluginId::from_static("test"),
        ));
        assert!(handle.set(plugin.clone()).is_ok());

        // A runtime let go rather than waited for: a deadlocked call would hang its drop.
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let outer = runtime.block_on(async {
            tokio::time::timeout(
                Duration::from_secs(5),
                plugin.run("run", |instance| {
                    instance.call_bytes("run", &[]).map(|_| ())
                }),
            )
            .await
        });
        runtime.shutdown_background();

        let outer = outer.expect("the call back should not deadlock").flatten();
        assert!(outer.is_ok(), "the outer call goes on: {outer:?}");
        let call_back = call_back.lock().unwrap().take();
        assert!(
            matches!(
                call_back,
                Some(Err(PluginError::FunctionCallFailed {
                    error: PluginCallError::Reentrant,
                    ..
                }))
            ),
            "the call back should be refused: {call_back:?}"
        );
    }
}
