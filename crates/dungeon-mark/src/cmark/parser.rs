use pulldown_cmark::{Event, OffsetIter, Options, Parser};

use std::{fmt::Display, iter::Peekable};

pub struct CMarkParser<'a> {
    source: &'a str,
    events: Peekable<OffsetIter<'a, 'a>>,
    offset: usize,
}

impl<'a> CMarkParser<'a> {
    pub fn new(source: &str) -> CMarkParser<'_> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);

        let events = Parser::new(source).into_offset_iter().peekable();

        CMarkParser {
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

    /// Peek the next event in the stream without consuming it.
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

    /// Iterates over the stream, returning any events where `delimeter` returns `false`.
    /// Once `delimeter` returns `true` the iterator ends, but the matched event is not consumed.
    pub fn iter_until(
        &mut self,
        delimeter: impl Fn(&Event<'a>) -> bool + 'a,
    ) -> impl Iterator<Item = Event<'a>> + '_ {
        std::iter::from_fn(move || match self.peek_event() {
            Some(event) if delimeter(event) => None,
            Some(_) => self.next_event(),
            None => None,
        })
    }

    /// Iterates over the stream, returning any events where `delimeter` returns `false`.
    /// Once `delimeter` returns `true` the iterator ends, but the matched event is consumed, but not included.
    pub fn iter_until_and_consume(
        &mut self,
        delimeter: impl Fn(&Event<'a>) -> bool + 'a,
    ) -> impl Iterator<Item = Event<'a>> + '_ {
        std::iter::from_fn(move || match self.next_event() {
            Some(event) if delimeter(&event) => None,
            None => None,
            event => event,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

impl Display for Position {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "line: {}, column: {}", self.line, self.column)
    }
}
