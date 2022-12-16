use pulldown_cmark::{Event, OffsetIter, Parser};

use std::iter::Peekable;

pub struct MarkdownParser<'a> {
    source: &'a str,
    events: Peekable<OffsetIter<'a, 'a>>,
    offset: usize,
}

impl<'a> MarkdownParser<'a> {
    pub fn new(source: &str) -> MarkdownParser<'_> {
        let events = Parser::new(source).into_offset_iter().peekable();

        MarkdownParser {
            source,
            events,
            offset: 0,
        }
    }

    /// Provides the line and column of the last emitted event.
    pub fn position(&self) -> Position {
        let previous = self.source[..self.offset].as_bytes();
        let line = memchr::Memchr::new(b'\n', previous).count() + 1;
        let start_of_line = memchr::memrchr(b'\n', previous).unwrap_or(0);
        let column = self.source[start_of_line..self.offset].chars().count();

        Position { line, column }
    }

    /// Peek the next event in the stream without consume it.
    pub fn peek_event(&mut self) -> Option<&Event<'a>> {
        self.events.peek().map(|(event, _)| event)
    }

    /// Consume the next event in stream.
    pub fn next_event(&mut self) -> Option<Event<'a>> {
        self.events.next().map(|(event, range)| {
            self.offset = range.start;
            event
        })
    }

    /// Consumes all events up to and including the delimeter and returns all events before the matched delimeter.
    pub fn collect_until(&mut self, delimeter: impl Fn(&Event<'a>) -> bool) -> Vec<Event<'a>> {
        let mut events = Vec::new();

        loop {
            match self.next_event() {
                Some(event) if delimeter(&event) => break,
                Some(other) => events.push(other),
                None => break,
            }
        }

        events
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}
