use pulldown_cmark::{Event, HeadingLevel, Tag};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

use crate::{
    cmark::EventCollectionExt,
    error::{Error, Result},
    parser::MarkdownParser,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub title: Option<String>,
    pub entries: Vec<JournalEntry>,
}

impl FromStr for Journal {
    type Err = Error;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        JournalParser::new(source).parse()
    }
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
    parser: MarkdownParser<'a>,
}

impl<'a> JournalParser<'a> {
    fn new(source: &str) -> JournalParser<'_> {
        let parser = MarkdownParser::new(source);

        JournalParser { parser }
    }

    fn parse(mut self) -> Result<Journal> {
        let title = self.parse_title()?;

        Ok(Journal {
            title,
            entries: Vec::new(),
        })
    }

    fn parse_title<'b>(&'b mut self) -> Result<Option<String>> {
        loop {
            let event = self.parser.peek_event();
            match event {
                Some(Event::Start(Tag::Heading(HeadingLevel::H1, ..))) => {
                    // NOTE: Skip the start tag that was peeked.
                    self.parser.next_event();
                    let events = self.parser.collect_until(|event| {
                        matches!(event, Event::End(Tag::Heading(HeadingLevel::H1, ..)))
                    });

                    return Ok(Some(events.stringify()?));
                }
                Some(Event::Html(_)) => {
                    self.parser.next_event(); // Skip HTML, such as comments.
                }
                _ => return Ok(None),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_title() {
        let input = "# Journal Title";
        let journal: Journal = input.parse().expect("journal failed to parse");

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
        let journal: Journal = input.parse().expect("journal failed to parse");

        assert_eq!(
            "Actual Title",
            journal.title.expect("journal title was empty")
        )
    }
}
