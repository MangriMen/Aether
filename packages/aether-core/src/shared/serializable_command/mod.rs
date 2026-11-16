pub(crate) mod domain;

pub use domain::{
    CommandParseError, HookKind, HookSpawnError, SerializableCommand, parse_command_line,
    parse_wrapper, split_command_line,
};
