use tokio::task::spawn_blocking;

pub async fn sha1_async<T>(input: T) -> String
where
    T: AsRef<[u8]> + Send + 'static,
{
    spawn_blocking(move || sha1_smol::Sha1::from(input.as_ref()).hexdigest())
        .await
        .expect("sha1_async: task panicked")
}
