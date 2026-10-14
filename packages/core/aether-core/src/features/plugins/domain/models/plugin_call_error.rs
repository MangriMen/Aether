/// Why a call into a plugin failed. Each sandbox limit has its own variant so the user is told
/// which limit the plugin hit, not that "the call failed".
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
pub enum PluginCallError {
    #[error("the plugin does not export this function")]
    MissingFunction,

    #[error("the plugin exceeded its time limit")]
    Timeout,

    #[error("the plugin exceeded its instruction limit")]
    OutOfFuel,

    #[error("the plugin exceeded its memory limit")]
    OutOfMemory,

    /// The wasm code trapped. A panic in a Rust plugin aborts into `unreachable`, so this is
    /// how a plugin crash arrives.
    #[error("the plugin crashed: {0}")]
    Trap(String),

    /// The plugin ended the process itself (WASI `proc_exit`) with a non-zero code.
    #[error("the plugin exited with code {0}")]
    NonZeroExit(i32),

    /// The error the plugin itself reported.
    #[error("{0}")]
    Plugin(String),

    /// The call input did not serialize or the plugin's output did not deserialize.
    #[error("the plugin exchanged malformed data: {0}")]
    Codec(String),

    /// The call was made while the plugin was still handling a call that led to it, through its
    /// host functions. Waiting for its turn would wait for itself, so it is refused.
    #[error("the plugin was called again from inside its own call")]
    Reentrant,
}
