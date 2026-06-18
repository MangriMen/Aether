use crate::features::events::{
    InstanceEvent, PluginEvent, ProcessEvent, ProgressEvent, WarningEvent,
};

#[derive(Debug, Clone)]
pub enum Event {
    Instance(InstanceEvent),
    Process(ProcessEvent),
    Progress(ProgressEvent),
    Plugin(PluginEvent),
    Warning(WarningEvent),
}

impl From<InstanceEvent> for Event {
    fn from(value: InstanceEvent) -> Self {
        Self::Instance(value)
    }
}

impl TryFrom<Event> for InstanceEvent {
    type Error = ();

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        match value {
            Event::Instance(p) => Ok(p),
            _ => Err(()),
        }
    }
}

impl From<PluginEvent> for Event {
    fn from(value: PluginEvent) -> Self {
        Self::Plugin(value)
    }
}

impl TryFrom<Event> for PluginEvent {
    type Error = ();

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        match value {
            Event::Plugin(p) => Ok(p),
            _ => Err(()),
        }
    }
}

impl From<ProcessEvent> for Event {
    fn from(value: ProcessEvent) -> Self {
        Self::Process(value)
    }
}

impl TryFrom<Event> for ProcessEvent {
    type Error = ();

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        match value {
            Event::Process(p) => Ok(p),
            _ => Err(()),
        }
    }
}

impl From<WarningEvent> for Event {
    fn from(value: WarningEvent) -> Self {
        Self::Warning(value)
    }
}

impl TryFrom<Event> for WarningEvent {
    type Error = ();

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        match value {
            Event::Warning(p) => Ok(p),
            _ => Err(()),
        }
    }
}

impl From<ProgressEvent> for Event {
    fn from(value: ProgressEvent) -> Self {
        Self::Progress(value)
    }
}

impl TryFrom<Event> for ProgressEvent {
    type Error = ();

    fn try_from(value: Event) -> Result<Self, Self::Error> {
        match value {
            Event::Progress(p) => Ok(p),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::events::{InstanceEventType, ProcessEventType, ProgressEventType};
    use uuid::Uuid;

    fn make_instance_event() -> InstanceEvent {
        InstanceEvent {
            event: InstanceEventType::Created,
            instance_id: "test-id".into(),
        }
    }

    fn make_plugin_event() -> PluginEvent {
        PluginEvent::Add {
            plugin_id: "test-plugin".into(),
        }
    }

    fn make_process_event() -> ProcessEvent {
        ProcessEvent {
            instance_id: "test-id".into(),
            process_id: Uuid::new_v4(),
            event: ProcessEventType::Launched,
            message: "started".into(),
        }
    }

    fn make_progress_event() -> ProgressEvent {
        ProgressEvent {
            progress_bar_id: Uuid::new_v4(),
            fraction: Some(0.5),
            message: "downloading".into(),
            event: ProgressEventType::PackFileDownload {
                instance_path: "/path".into(),
                pack_name: "pack".into(),
                icon: None,
                pack_version: "1.0".into(),
            },
        }
    }

    fn make_warning_event() -> WarningEvent {
        WarningEvent {
            message: "warning msg".into(),
        }
    }

    #[test]
    fn should_roundtrip_instance_event() {
        let original = make_instance_event();
        let event: Event = original.clone().into();
        let recovered: InstanceEvent = event.try_into().unwrap();
        assert_eq!(recovered.instance_id, original.instance_id);
    }

    #[test]
    fn should_roundtrip_plugin_event() {
        let original = make_plugin_event();
        let event: Event = original.clone().into();
        let recovered: PluginEvent = event.try_into().unwrap();
        assert!(matches!(recovered, PluginEvent::Add { .. }));
    }

    #[test]
    fn should_roundtrip_process_event() {
        let original = make_process_event();
        let event: Event = original.clone().into();
        let recovered: ProcessEvent = event.try_into().unwrap();
        assert_eq!(recovered.instance_id, original.instance_id);
    }

    #[test]
    fn should_roundtrip_progress_event() {
        let original = make_progress_event();
        let event: Event = original.clone().into();
        let recovered: ProgressEvent = event.try_into().unwrap();
        assert_eq!(recovered.fraction, original.fraction);
    }

    #[test]
    fn should_roundtrip_warning_event() {
        let original = make_warning_event();
        let event: Event = original.clone().into();
        let recovered: WarningEvent = event.try_into().unwrap();
        assert_eq!(recovered.message, original.message);
    }

    #[test]
    fn should_fail_try_from_wrong_variant() {
        let event: Event = make_instance_event().into();
        assert!(PluginEvent::try_from(event).is_err());
    }
}
