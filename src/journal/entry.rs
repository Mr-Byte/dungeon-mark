use anyhow::Context;
use pulldown_cmark::{Event, Tag};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use crate::{
    cmark::{CMarkParser, EventIteratorExt as _},
    config::Config,
    error::Result,
};

/// A `Section` represents all text following a heading in a `JournalEntry`.
/// Any headings that have a lower-level than the `Section` that follow the section
/// will be nested inside this section. Any `Section` with the same level as the
/// current section will be a sibling section in the parent `Section` or `JournalEntry`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    /// The title of the section as provided by the heading.
    pub title: String,
    /// All text that follows this section, excluding the text of any child sections
    /// or sibling sections.
    pub body: String,
    /// Metadata associated with a section.
    pub metadata: HashMap<String, String>,
    /// Any child sections that are nested below the current section.
    pub sections: Vec<Section>,
}

/// A `JournalEntry` is an in-memory representation of a single Markdown file on disk.
/// It is organized into sections based on headings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    /// The location of the document relative to the "source root" config option.
    pub source_path: PathBuf,
    /// An optional top level journal entry body, which makes up any elements the preceed the first heading in the document.
    pub body: Option<String>,
    /// The sections (delineated by Markdown headings) of the document.
    pub sections: Vec<Section>,
}

impl JournalEntry {
    /// Load a journal entry with the given path relative to the config's source root.
    pub fn load(path: PathBuf, config: &Config) -> Result<Self> {
        let mut buffer = String::new();
        let entry_path = config.journal.source.join(&path);

        File::open(&entry_path)
            .with_context(|| format!("failed to open journal entry: {}", entry_path.display()))?
            .read_to_string(&mut buffer)
            .with_context(|| format!("failed to read journal entry: {}", entry_path.display()))?;

        let (body, sections) = JournalEntryParser::new(&buffer)
            .parse()
            .with_context(|| format!("unable to parse journal entry: {}", entry_path.display()))?;

        let entry = Self {
            source_path: path,
            body,
            sections,
        };

        Ok(entry)
    }
}

struct JournalEntryParser<'a> {
    parser: CMarkParser<'a>,
}

impl<'a> JournalEntryParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            parser: CMarkParser::new(source),
        }
    }

    fn parse(mut self) -> Result<(Option<String>, Vec<Section>)> {
        let body = self.parse_body()?;

        Ok((body, Vec::new()))
    }

    fn parse_body(&mut self) -> Result<Option<String>> {
        let mut events = Vec::new();

        loop {
            match self.parser.peek_event() {
                Some(Event::Start(Tag::Heading(..))) => break,
                Some(_) => {
                    let event = self
                        .parser
                        .next_event()
                        .expect("event was empty, when it shouldn't have been");
                    events.push(event);
                }
                None => {
                    self.parser.next_event();
                    break;
                }
            }
        }

        let body = events
            .iter()
            .stringify()
            .with_context(|| "failed to stringify journal entry body")?;
        let body = if body.is_empty() { None } else { Some(body) };

        Ok(body)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parses_top_level_body() {
        let input = "Top level body.\nWith multiple lines.\n\nIncluding heard breaks.";
        let (body, _) = JournalEntryParser::new(input)
            .parse()
            .expect("unable to parse input");

        let expected = Some(String::from(input));

        assert_eq!(body, expected)
    }
}
