use std::future::Future;

pub fn execute_async<F: Future>(future: F) -> F::Output {
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => tokio::task::block_in_place(|| handle.block_on(future)),
        _ => tokio::runtime::Runtime::new()
            .expect("Failed to create runtime")
            .block_on(future),
    }
}
