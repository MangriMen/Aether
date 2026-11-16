pub(crate) mod domain;

pub use domain::{
    CommandParseError, HookKind, HookSpawnError, SerializableCommand, parse_command_line,
    split_command_line,
};
