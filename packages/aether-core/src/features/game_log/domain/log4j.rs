use chrono::{DateTime, Local};

const EVENT_OPEN: &str = "<log4j:Event ";
const EVENT_CLOSE: &str = "</log4j:Event>";
/// An event larger than this is written out as the raw lines it arrived in.
const MAX_EVENT_BYTES: usize = 1024 * 1024;

/// Turns the game's log4j XML console output into plain log lines. The vanilla client config
/// prints every entry as a multi-line `<log4j:Event>`; anything else (JVM messages, output of
/// versions that predate log4j) passes through unchanged.
#[derive(Default)]
pub struct Log4jNormalizer {
    pending: Vec<String>,
    pending_bytes: usize,
}

impl Log4jNormalizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_line(&mut self, line: &str) -> Vec<String> {
        if self.pending.is_empty() && !line.starts_with(EVENT_OPEN) {
            return vec![line.to_string()];
        }

        self.pending_bytes += line.len();
        self.pending.push(line.to_string());

        if line.contains(EVENT_CLOSE) {
            let raw = std::mem::take(&mut self.pending);
            self.pending_bytes = 0;
            return render_event(&raw.join("\n")).unwrap_or(raw);
        }
        if self.pending_bytes > MAX_EVENT_BYTES {
            return self.finish();
        }
        Vec::new()
    }

    /// Emits an event that never got its closing tag as the raw lines it arrived in.
    pub fn finish(&mut self) -> Vec<String> {
        self.pending_bytes = 0;
        std::mem::take(&mut self.pending)
    }
}

fn render_event(raw: &str) -> Option<Vec<String>> {
    let open_end = raw.find('>')?;
    let open_tag = &raw[..open_end];

    let level = attribute(open_tag, "level").unwrap_or_else(|| "INFO".to_string());
    let thread = attribute(open_tag, "thread").unwrap_or_else(|| "main".to_string());
    let time = attribute(open_tag, "timestamp")
        .and_then(|ts| ts.parse::<i64>().ok())
        .and_then(DateTime::from_timestamp_millis)
        .map(|utc| utc.with_timezone(&Local).format("%H:%M:%S").to_string())
        .unwrap_or_default();

    let message = element_text(raw, "log4j:Message").unwrap_or_default();
    let mut lines = Vec::new();
    for (index, line) in message.lines().enumerate() {
        if index == 0 {
            lines.push(format!("[{time}] [{thread}/{level}]: {line}"));
        } else {
            lines.push(line.to_string());
        }
    }
    if lines.is_empty() {
        lines.push(format!("[{time}] [{thread}/{level}]: "));
    }
    if let Some(throwable) = element_text(raw, "log4j:Throwable") {
        lines.extend(throwable.lines().map(str::to_string));
    }
    Some(lines)
}

fn attribute(tag: &str, name: &str) -> Option<String> {
    let needle = format!(" {name}=\"");
    let start = tag.find(&needle)? + needle.len();
    let end = tag[start..].find('"')? + start;
    Some(decode_entities(&tag[start..end]))
}

/// The content of the first `<name>` element: its CDATA sections joined (a `]]>` inside the
/// text is split across two sections), or the entity-decoded text when it has none.
fn element_text(raw: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let start = raw.find(&open)? + open.len();
    let end = raw[start..].find(&close)? + start;
    let body = &raw[start..end];

    if !body.contains("<![CDATA[") {
        return Some(decode_entities(body.trim_matches(['\r', '\n'])));
    }

    let mut text = String::new();
    let mut rest = body;
    while let Some(section_start) = rest.find("<![CDATA[") {
        let content = &rest[section_start + "<![CDATA[".len()..];
        let Some(section_end) = content.find("]]>") else {
            text.push_str(content);
            return Some(text);
        };
        text.push_str(&content[..section_end]);
        rest = &content[section_end + 3..];
    }
    Some(text)
}

fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed(lines: &[&str]) -> Vec<String> {
        let mut normalizer = Log4jNormalizer::new();
        let mut out: Vec<String> = lines
            .iter()
            .flat_map(|line| normalizer.push_line(line))
            .collect();
        out.extend(normalizer.finish());
        out
    }

    fn event(level: &str, thread: &str, body: &str) -> Vec<String> {
        vec![
            format!(
                "<log4j:Event logger=\"x\" timestamp=\"1700000000000\" level=\"{level}\" thread=\"{thread}\">"
            ),
            format!("<log4j:Message><![CDATA[{body}]]></log4j:Message>"),
            "</log4j:Event>".to_string(),
        ]
    }

    fn feed_owned(lines: &[String]) -> Vec<String> {
        feed(&lines.iter().map(String::as_str).collect::<Vec<_>>())
    }

    #[test]
    fn should_render_an_event_as_a_plain_line_when_it_spans_lines() {
        let out = feed_owned(&event("WARN", "Render thread", "Hello"));

        assert_eq!(out.len(), 1);
        assert!(
            out[0].ends_with("[Render thread/WARN]: Hello"),
            "{}",
            out[0]
        );
    }

    #[test]
    fn should_pass_plain_lines_through_when_they_are_not_events() {
        assert_eq!(
            feed(&["Error: Could not create the JVM", ""]),
            ["Error: Could not create the JVM", ""]
        );
    }

    #[test]
    fn should_join_cdata_sections_when_the_text_contains_a_cdata_end() {
        let out = feed_owned(&event("INFO", "main", "a]]]]><![CDATA[>b"));

        assert!(out[0].ends_with("]: a]]>b"), "{}", out[0]);
    }

    #[test]
    fn should_keep_message_and_throwable_lines_when_the_event_has_both() {
        let out = feed(&[
            "<log4j:Event logger=\"x\" timestamp=\"1700000000000\" level=\"ERROR\" thread=\"main\">",
            "<log4j:Message><![CDATA[first",
            "second]]></log4j:Message>",
            "<log4j:Throwable><![CDATA[java.lang.Error: boom",
            "\tat Foo.bar(Foo.java:1)",
            "]]></log4j:Throwable>",
            "</log4j:Event>",
        ]);

        assert!(out[0].ends_with("[main/ERROR]: first"));
        assert_eq!(out[1], "second");
        assert_eq!(out[2], "java.lang.Error: boom");
        assert_eq!(out[3], "\tat Foo.bar(Foo.java:1)");
    }

    #[test]
    fn should_decode_entities_when_attributes_are_escaped() {
        let out = feed(&[
            "<log4j:Event logger=\"x\" timestamp=\"1700000000000\" level=\"INFO\" thread=\"a&amp;b\"><log4j:Message><![CDATA[m]]></log4j:Message></log4j:Event>",
        ]);

        assert!(out[0].contains("[a&b/INFO]"), "{}", out[0]);
    }

    #[test]
    fn should_emit_raw_lines_when_an_event_never_closes() {
        let out = feed(&[
            "<log4j:Event logger=\"x\" level=\"INFO\" thread=\"main\">",
            "<log4j:Message><![CDATA[cut",
        ]);

        assert_eq!(out.len(), 2);
        assert!(out[0].starts_with("<log4j:Event"));
    }

    #[test]
    fn should_pass_plain_text_through_when_it_follows_an_event() {
        let mut lines = event("INFO", "main", "one");
        lines.push("plain".to_string());

        let out = feed_owned(&lines);

        assert_eq!(out.last().map(String::as_str), Some("plain"));
    }
}
