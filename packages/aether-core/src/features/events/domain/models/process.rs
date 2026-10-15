use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ProcessEvent {
    pub instance_id: String,
    pub process_id: Uuid,
    pub event: ProcessEventType,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum ProcessEventType {
    Launched,
    Finished,
}
