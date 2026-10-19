use crate::features::{
    events::infra::{CacheEventDto, ProgressEventDto, WarningEventDto},
    instance::infra::InstanceEventDto,
    plugins::infra::PluginEventDto,
    process::infra::ProcessEventDto,
};

/// An event DTO that carries its place in the order the backend sent events
/// ([`TauriEventEmitter`](super::TauriEventEmitter)).
pub trait WithSeq: Sized {
    fn with_seq(self, seq: u32) -> Self;
}

macro_rules! impl_with_seq {
    ($($dto:ty),+) => {
        $(impl WithSeq for $dto {
            fn with_seq(mut self, seq: u32) -> Self {
                self.seq = seq;
                self
            }
        })+
    };
}

impl_with_seq!(
    CacheEventDto,
    InstanceEventDto,
    PluginEventDto,
    ProcessEventDto,
    ProgressEventDto,
    WarningEventDto
);
