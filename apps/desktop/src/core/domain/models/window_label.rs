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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_label_main_returns_main_str() {
        assert_eq!(WindowLabel::Main.as_ref(), "main");
    }
}
