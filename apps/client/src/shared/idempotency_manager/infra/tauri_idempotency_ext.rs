use crate::shared::{ActiveRequest, IdempotencyManager, RequestId};

pub trait TauriIdempotencyExt {
    fn lock_cmd(&self, id: RequestId) -> Result<ActiveRequest<'_>, crate::Error>;
}

impl TauriIdempotencyExt for tauri::State<'_, IdempotencyManager> {
    fn lock_cmd(&self, id: RequestId) -> Result<ActiveRequest<'_>, crate::Error> {
        self.lock_request(id).map_err(crate::Error::from)
    }
}
