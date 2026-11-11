/// The extension of the image format `head` (the first bytes of a file) starts, if taken.
pub fn sniff_image_extension(head: &[u8]) -> Option<&'static str> {
    if head.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if head.starts_with(b"\xff\xd8\xff") {
        Some("jpg")
    } else if head.starts_with(b"GIF87a") || head.starts_with(b"GIF89a") {
        Some("gif")
    } else if head.len() >= 12 && &head[..4] == b"RIFF" && &head[8..12] == b"WEBP" {
        Some("webp")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_tell_formats_by_their_first_bytes() {
        assert_eq!(sniff_image_extension(b"\x89PNG\r\n\x1a\nrest"), Some("png"));
        assert_eq!(sniff_image_extension(b"\xff\xd8\xff\xe0"), Some("jpg"));
        assert_eq!(sniff_image_extension(b"GIF89a.."), Some("gif"));
        assert_eq!(sniff_image_extension(b"RIFF\0\0\0\0WEBPVP8 "), Some("webp"));
    }

    #[test]
    fn should_not_take_other_content_for_an_image() {
        assert_eq!(sniff_image_extension(b"<html><script>"), None);
        assert_eq!(sniff_image_extension(b"RIFF\0\0\0\0WAVE"), None);
        assert_eq!(sniff_image_extension(b""), None);
    }
}
