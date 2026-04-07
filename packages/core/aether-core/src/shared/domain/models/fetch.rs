use tokio::sync::Semaphore;

#[derive(Debug)]
pub struct FetchSemaphore(pub Semaphore);
