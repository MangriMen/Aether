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

/// Whether `head` (the first bytes of a file) starts an SVG document: an XML text whose root is
/// `svg`, possibly after a BOM, a declaration, comments and a doctype.
pub fn looks_like_svg(head: &[u8]) -> bool {
    let text = String::from_utf8_lossy(head);
    let mut rest = text.trim_start_matches('\u{feff}').trim_start();
    loop {
        if let Some(after) = rest.strip_prefix("<?") {
            let Some((_, tail)) = after.split_once("?>") else {
                return false;
            };
            rest = tail.trim_start();
        } else if let Some(after) = rest.strip_prefix("<!--") {
            let Some((_, tail)) = after.split_once("-->") else {
                return false;
            };
            rest = tail.trim_start();
        } else if rest
            .get(..9)
            .is_some_and(|p| p.eq_ignore_ascii_case("<!doctype"))
        {
            let Some((_, tail)) = rest.split_once('>') else {
                return false;
            };
            rest = tail.trim_start();
        } else {
            break;
        }
    }
    rest.strip_prefix("<svg")
        .and_then(|tail| tail.chars().next())
        .is_some_and(|c| c.is_whitespace() || c == '>' || c == '/')
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

    #[test]
    fn should_take_an_svg_for_one_after_its_prologue() {
        assert!(looks_like_svg(b"<svg xmlns='http://www.w3.org/2000/svg'/>"));
        assert!(looks_like_svg(
            b"\xef\xbb\xbf  <?xml version='1.0'?>\n<!-- a -->\n<!DOCTYPE svg>\n<svg>"
        ));
        assert!(!looks_like_svg(b"<html><svg>"));
        assert!(!looks_like_svg(b"<svgfoo>"));
        assert!(!looks_like_svg(b""));
    }
}
