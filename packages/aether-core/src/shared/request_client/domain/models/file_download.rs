/// What [`RequestClient::download_to_file_conditional`] did.
///
/// [`RequestClient::download_to_file_conditional`]: crate::shared::request_client::RequestClient::download_to_file_conditional
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileDownload {
    /// The server answered `304 Not Modified` to the request's own validators: nothing was
    /// written.
    NotModified,
    /// The body is saved, as the response described it.
    Saved(SavedDownload),
}

/// A body saved to a file, with what the response said about it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SavedDownload {
    pub size: u64,
    /// The `Content-Type`, as sent: the server's claim, not checked against the body.
    pub content_type: Option<String>,
    /// The `ETag`, to ask with `If-None-Match` whether the body changed.
    pub etag: Option<String>,
    /// The `Last-Modified`, to ask with `If-Modified-Since` whether the body changed.
    pub last_modified: Option<String>,
}
