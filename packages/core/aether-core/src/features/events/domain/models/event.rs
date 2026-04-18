use serde::{Deserialize, Serialize};

use crate::features::events::{
    InstanceEvent, PluginEvent, ProcessEvent, ProgressEvent, WarningEvent,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
