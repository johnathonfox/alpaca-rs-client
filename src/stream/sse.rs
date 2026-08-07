//! Minimal Server-Sent Events (SSE) frame parser.
//!
//! The events endpoints (`/v2beta1/events/activities`,
//! `/v1beta1/events/corporate-actions`) speak `text/event-stream` rather than
//! WebSocket, so [`EventStream`](super::events::EventStream) feeds the raw
//! response bytes through [`SseParser`] and decodes the resulting frames.
//!
//! The parser implements the dispatch rules of the [WHATWG event stream
//! specification](https://html.spec.whatwg.org/multipage/server-sent-events.html):
//! `data:` values accumulate and are joined with newlines, `event:` names the
//! frame, `id:` updates the last event id (which survives frames that carry no
//! data and is replayed via the `Last-Event-Id` header on reconnect), lines
//! starting with `:` are comments (Alpaca sends `:heartbeat` and
//! `: internal server error`), and a blank line dispatches the frame.
//!
//! Lines may end in `\n` or `\r\n`; lone `\r` terminators, which no Alpaca
//! endpoint emits, are treated as part of the line.

/// A dispatched SSE frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseFrame {
    /// Value of the `event:` field, if the frame carried one.
    pub event: Option<String>,
    /// The accumulated `data:` payload, with multiple `data:` lines joined by
    /// newlines. Never empty: frames without data are not dispatched.
    pub data: String,
    /// The last event id known when this frame was dispatched. It is sticky —
    /// a frame without its own `id:` inherits the previous one, matching the
    /// specification's "last event ID buffer".
    pub id: Option<String>,
}

/// Incremental parser turning a byte stream into [`SseFrame`]s.
///
/// Feed arbitrary chunks with [`push`](Self::push) — they may split lines or
/// even multi-byte UTF-8 characters — then drain whole frames with
/// [`next_frame`](Self::next_frame).
#[derive(Debug, Default)]
pub struct SseParser {
    /// Bytes received but not yet consumed as complete lines.
    buffer: Vec<u8>,
    /// Value of the pending frame's `event:` field.
    event: Option<String>,
    /// Accumulated `data:` values of the pending frame.
    data: String,
    /// Whether the pending frame saw at least one `data:` line.
    has_data: bool,
    /// The sticky last event id, updated by every `id:` line.
    last_event_id: Option<String>,
}

impl SseParser {
    /// Creates an empty parser.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a chunk of response bytes.
    pub fn push(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// Bytes received but not yet consumed as a complete line.
    ///
    /// This only grows while a line is incomplete, so it is normally near
    /// zero. A peer that never emits a newline would otherwise grow it without
    /// bound, so the transport driving this parser caps it — see
    /// [`EventStream`](super::events::EventStream).
    pub fn buffered_len(&self) -> usize {
        self.buffer.len()
    }

    /// The last event id seen so far, used as the `Last-Event-Id` reconnect
    /// header. Updated by `id:` lines even when the surrounding frame carried
    /// no data.
    pub fn last_event_id(&self) -> Option<&str> {
        self.last_event_id.as_deref()
    }

    /// Returns the next complete frame, or `None` when the buffered bytes do
    /// not yet contain one.
    pub fn next_frame(&mut self) -> Option<SseFrame> {
        while let Some(line) = self.take_line() {
            if line.is_empty() {
                if let Some(frame) = self.dispatch() {
                    return Some(frame);
                }
            } else {
                self.handle_line(&line);
            }
        }
        None
    }

    /// Removes and returns the next `\n`- or `\r\n`-terminated line, or `None`
    /// if the buffer holds no complete line yet.
    fn take_line(&mut self) -> Option<String> {
        let index = self.buffer.iter().position(|byte| *byte == b'\n')?;
        let mut line: Vec<u8> = self.buffer.drain(..=index).collect();
        line.pop();
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        // A complete line always holds whole UTF-8 characters, so the lossy
        // conversion only kicks in for genuinely malformed input.
        Some(String::from_utf8_lossy(&line).into_owned())
    }

    /// Applies one non-blank line to the pending frame.
    fn handle_line(&mut self, line: &str) {
        if line.starts_with(':') {
            return;
        }
        let (field, value) = match line.split_once(':') {
            Some((field, value)) => (field, value.strip_prefix(' ').unwrap_or(value)),
            // A line without a colon is a field name with an empty value.
            None => (line, ""),
        };
        match field {
            "event" => self.event = Some(value.to_string()),
            "data" => {
                if self.has_data {
                    self.data.push('\n');
                }
                self.data.push_str(value);
                self.has_data = true;
            }
            // The specification ignores ids containing a NUL character.
            "id" if !value.contains('\0') => self.last_event_id = Some(value.to_string()),
            // `retry:` and unknown fields are ignored; reconnection backoff is
            // configured through `ReconnectOptions` instead.
            _ => {}
        }
    }

    /// Dispatches the pending frame on a blank line. Frames without any
    /// `data:` line are discarded (they only serve to update the last event
    /// id), per the specification.
    fn dispatch(&mut self) -> Option<SseFrame> {
        let event = self.event.take();
        if !self.has_data {
            self.data.clear();
            return None;
        }
        self.has_data = false;
        Some(SseFrame {
            event,
            data: std::mem::take(&mut self.data),
            id: self.last_event_id.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Feeds a whole stream at once and collects every frame it dispatches.
    fn frames(stream: &str) -> Vec<SseFrame> {
        let mut parser = SseParser::new();
        parser.push(stream.as_bytes());
        std::iter::from_fn(|| parser.next_frame()).collect()
    }

    #[test]
    fn parses_a_single_data_frame() {
        let parsed = frames("data: {\"a\":1}\n\n");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].data, "{\"a\":1}");
        assert_eq!(parsed[0].event, None);
        assert_eq!(parsed[0].id, None);
    }

    #[test]
    fn joins_multi_line_data_with_newlines() {
        let parsed = frames("data: line one\ndata: line two\ndata:\n\n");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].data, "line one\nline two\n");
    }

    #[test]
    fn strips_only_one_leading_space_from_values() {
        let parsed = frames("data:  padded\n\ndata:tight\n\n");
        assert_eq!(parsed[0].data, " padded");
        assert_eq!(parsed[1].data, "tight");
    }

    #[test]
    fn reads_event_names_and_handles_colonless_lines() {
        let parsed = frames("event: update\ndata\ndata: body\n\n");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].event.as_deref(), Some("update"));
        assert_eq!(parsed[0].data, "\nbody");
    }

    #[test]
    fn ignores_comments_and_unknown_fields() {
        let parsed = frames(": heartbeat\nretry: 5000\nnope: value\ndata: kept\n\n");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].data, "kept");
    }

    #[test]
    fn accepts_crlf_line_endings() {
        let parsed = frames("id: 1\r\ndata: crlf\r\n\r\n");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].data, "crlf");
        assert_eq!(parsed[0].id.as_deref(), Some("1"));
    }

    #[test]
    fn does_not_dispatch_frames_without_data() {
        // An id-only frame updates the last event id but yields no frame.
        let mut parser = SseParser::new();
        parser.push(b"id: 01ABC\n\n");
        assert!(parser.next_frame().is_none());
        assert_eq!(parser.last_event_id(), Some("01ABC"));
    }

    #[test]
    fn last_event_id_is_sticky_across_frames() {
        let parsed = frames("id: 01A\ndata: first\n\ndata: second\n\nid: 01B\ndata: third\n\n");
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].id.as_deref(), Some("01A"));
        // No `id:` of its own — inherits the previous one.
        assert_eq!(parsed[1].id.as_deref(), Some("01A"));
        assert_eq!(parsed[2].id.as_deref(), Some("01B"));
    }

    #[test]
    fn ids_containing_nul_are_ignored() {
        let mut parser = SseParser::new();
        parser.push("id: 01A\ndata: x\n\nid: bad\0id\ndata: y\n\n".as_bytes());
        let first = parser.next_frame().expect("first frame");
        let second = parser.next_frame().expect("second frame");
        assert_eq!(first.id.as_deref(), Some("01A"));
        assert_eq!(second.id.as_deref(), Some("01A"));
    }

    #[test]
    fn reassembles_frames_split_across_chunks() {
        let mut parser = SseParser::new();
        // Split mid-field-name, mid-value and mid-terminator.
        for chunk in ["da", "ta: he", "llo\n", "\ndata: wor", "ld\n\n"] {
            parser.push(chunk.as_bytes());
        }
        let first = parser.next_frame().expect("first frame");
        let second = parser.next_frame().expect("second frame");
        assert_eq!(first.data, "hello");
        assert_eq!(second.data, "world");
        assert!(parser.next_frame().is_none());
    }

    #[test]
    fn reassembles_multibyte_characters_split_across_chunks() {
        let mut parser = SseParser::new();
        let payload = "data: caf\u{e9}\n\n".as_bytes();
        // `é` is two bytes; cut the chunk boundary between them.
        let split = payload.len() - 4;
        parser.push(&payload[..split]);
        assert!(parser.next_frame().is_none());
        parser.push(&payload[split..]);
        let frame = parser.next_frame().expect("frame");
        assert_eq!(frame.data, "caf\u{e9}");
    }

    #[test]
    fn incomplete_trailing_frame_is_withheld() {
        let mut parser = SseParser::new();
        parser.push(b"data: pending\n");
        assert!(parser.next_frame().is_none());
        parser.push(b"\n");
        assert_eq!(parser.next_frame().map(|f| f.data), Some("pending".into()));
    }
}
