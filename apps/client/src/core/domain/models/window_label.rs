#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowLabel {
    Main,
}

impl AsRef<str> for WindowLabel {
    fn as_ref(&self) -> &str {
        match self {
            WindowLabel::Main => "main",
        }
    }
}
