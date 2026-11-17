/// A plugin wrote a value of an enum that its API version does not have: `unknown`, which is what
/// a reader makes of a name it cannot place. Only a launcher newer than the plugin sends it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the plugin sent an unknown {kind}, the known ones are {}", expected.join(", "))]
pub struct UnknownValue {
    /// What the value is of: `mod loader`, `content type`.
    pub kind: &'static str,
    /// The values the API has, as they are written on the wire.
    pub expected: &'static [&'static str],
}
