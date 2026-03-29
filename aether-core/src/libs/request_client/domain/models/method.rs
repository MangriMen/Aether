#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    Options,
    Get,
    Post,
    Put,
    Delete,
    Head,
    Trace,
    Connect,
    Patch,
}
