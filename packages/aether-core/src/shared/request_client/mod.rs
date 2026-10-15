pub(crate) mod domain;
pub mod infra;

pub use domain::{
    BytesProgress, FileDownload, Method, Request, RequestClient, RequestClientExt, RequestError,
    SavedDownload,
};
