use pulldown_cmark::{Event, HeadingLevel, Tag};
use serde::{Deserialize, Serialize};
use std::{iter::Peekable, path::PathBuf};

use crate::error::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub title: Option<String>,
    pub entries: Vec<JournalEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub name: String,
    pub location: Option<PathBuf>,
    pub nested_entries: Vec<JournalEntry>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JournalEntry {
    Link(Link),
}

impl From<Link> for JournalEntry {
    fn from(link: Link) -> Self {
        JournalEntry::Link(link)
    }
}

struct JournalParser<'a> {
    source: &'a str,
    events: Peekable<pulldown_cmark::OffsetIter<'a, 'a>>,
    offset: usize,
}

impl<'a> JournalParser<'a> {
    fn new(source: &str) -> JournalParser<'_> {
        let events = pulldown_cmark::Parser::new(source)
            .into_offset_iter()
            .peekable();

        JournalParser {
            source,
            events,
            offset: 0,
        }
    }

    fn position(&self) -> Position {
        let previous = self.source[..self.offset].as_bytes();
        let line = memchr::Memchr::new(b'\n', previous).count() + 1;
        let start_of_line = memchr::memrchr(b'\n', previous).unwrap_or(0);
        let column = self.source[start_of_line..self.offset].chars().count();

        Position { line, column }
    }

    fn parse(mut self) -> Result<Journal> {
        let title = self.parse_title();

        Ok(Journal {
            title,
            entries: Vec::new(),
        })
    }

    fn parse_title(&mut self) -> Option<String> {
        loop {
            let event = self.peek_event();
            match event {
                Some(Event::Start(Tag::Heading(HeadingLevel::H1, ..))) => {
                    // NOTE: Skip the start tag that was peeked.
                    self.next_event();
                    let mut events = Vec::new();

                    loop {
                        match self.next_event() {
                            Some(Event::End(Tag::Heading(HeadingLevel::H1, ..))) => break,
                            Some(other) => events.push(other),
                            None => break,
                        }
                    }

                    let title = convert_events_to_string(events);

                    return Some(title);
                }
                Some(Event::Html(_)) => {
                    self.next_event(); // Skip HTML, such as comments.
                }
                _ => return None,
            }
        }
    }

    fn peek_event(&mut self) -> Option<&Event<'a>> {
        self.events.peek().map(|(event, _)| event)
    }

    fn next_event(&mut self) -> Option<Event<'a>> {
        self.events.next().map(|(event, range)| {
            self.offset = range.start;
            event
        })
    }
}

fn convert_events_to_string(events: Vec<Event<'_>>) -> String {
    events
        .into_iter()
        .filter_map(|event| match event {
            Event::Text(text) | Event::Code(text) => Some(text.into_string()),
            Event::SoftBreak => Some(String::from(" ")),
            _ => None,
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_title() {
        let input = "# Journal Title";
        let journal = JournalParser::new(input)
            .parse()
            .expect("journal did not parse");

        assert_eq!(
            "Journal Title",
            journal.title.expect("journal title was empty")
        )
    }

    #[test]
    fn skips_comments_and_parses_title() {
        let input = r"<!-- # Journal Title -->
# Actual Title
";
        let journal = JournalParser::new(input)
            .parse()
            .expect("journal did not parse");

        assert_eq!(
            "Actual Title",
            journal.title.expect("journal title was empty")
        )
    }
}
