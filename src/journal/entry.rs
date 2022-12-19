use anyhow::Context;
use pulldown_cmark::{Event, HeadingLevel, Tag};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::Read, path::PathBuf};

use crate::{
    cmark::{CMarkParser, EventIteratorExt as _},
    error::Result,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum SectionLevel {
    H1 = 1,
    H2,
    H3,
    H4,
    H5,
    H6,
}

impl From<HeadingLevel> for SectionLevel {
    fn from(value: HeadingLevel) -> Self {
        match value {
            HeadingLevel::H1 => SectionLevel::H1,
            HeadingLevel::H2 => SectionLevel::H2,
            HeadingLevel::H3 => SectionLevel::H3,
            HeadingLevel::H4 => SectionLevel::H4,
            HeadingLevel::H5 => SectionLevel::H5,
            HeadingLevel::H6 => SectionLevel::H6,
        }
    }
}

/// A `Section` represents all text following a heading in a `JournalEntry`.
/// Any headings that have a lower-level than the `Section` that follow the section
/// will be nested inside this section. Any `Section` with the same level as the
/// current section will be a sibling section in the parent `Section` or `JournalEntry`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Section {
    /// The title of the section as provided by the heading.
    pub title: String,
    /// The heading level of the section ranging from H1 to H6.
    pub level: SectionLevel,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalEntry {
    // The name of the journal entry.
    pub name: String,
    /// An optional top level journal entry body, which makes up any elementsthat preceed the first
    /// heading in the journal entry.
    pub body: Option<String>,
    /// The sections (delineated by Markdown headings) of the journal entry.
    pub sections: Vec<Section>,
    /// The location of this journal entry relative to the `JOURNAL.md` file.
    pub entry_path: Option<PathBuf>,
}

impl JournalEntry {
    /// Load a journal entry relative to the `source_path`.
    pub fn load(
        name: String,
        source_path: impl Into<PathBuf>,
        entry_path: impl Into<PathBuf>,
    ) -> Result<Self> {
        let mut buffer = String::new();
        let root_path = source_path.into();
        let source_path = entry_path.into();
        let file_path = root_path.join(&source_path);

        File::open(&file_path)
            .with_context(|| format!("Failed to open journal entry: {}", file_path.display()))?
            .read_to_string(&mut buffer)
            .with_context(|| format!("Failed to read journal entry: {}", file_path.display()))?;

        let (body, sections) = JournalEntryParser::new(&buffer)
            .parse()
            .with_context(|| format!("Unable to parse journal entry: {}", file_path.display()))?;

        let entry = Self {
            name,
            entry_path: Some(source_path),
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
        let sections = self.parse_sections()?;

        Ok((body, sections))
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

    fn parse_sections(&mut self) -> Result<Vec<Section>> {
        let mut sections = Vec::new();

        loop {
            match self.parser.next_event() {
                Some(Event::Start(Tag::Heading(heading_level, ..))) => {
                    let section = self.parse_section(heading_level)?;
                    sections.push(section)
                }
                Some(_) => (), // TODO: Ignore for now!
                None => break,
            }
        }

        Ok(sections)
    }

    fn parse_section(&mut self, level: HeadingLevel) -> Result<Section> {
        let title = self
            .parser
            .iter_until_and_consume(|event| {
                matches! {
                    event,
                    Event::End(Tag::Heading(..))
                }
            })
            .stringify()?;

        let body = self
            .parser
            .iter_until(|event| {
                matches! {
                    event,
                    Event::Start(Tag::Heading(..))
                }
            })
            .stringify()?;

        let mut sections = Vec::new();

        loop {
            match self.parser.peek_event() {
                Some(Event::Start(Tag::Heading(heading_level, ..))) if *heading_level > level => {
                    let heading_level = *heading_level;
                    self.parser.next_event();
                    sections.push(self.parse_section(heading_level)?);
                }
                Some(_) => break,
                None => break,
            }
        }

        Ok(Section {
            title,
            level: level.into(),
            body,
            metadata: HashMap::new(),
            sections,
        })
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

    #[test]
    fn parses_top_level_sections() {
        let input = "# First Top Level
# Second Top Level";
        let (_, sections) = JournalEntryParser::new(input)
            .parse()
            .expect("unable to parse input");

        let expected = vec![
            Section {
                title: String::from("First Top Level"),
                level: SectionLevel::H1,
                body: String::from(""),
                metadata: HashMap::new(),
                sections: Vec::new(),
            },
            Section {
                title: String::from("Second Top Level"),
                level: SectionLevel::H1,
                body: String::from(""),
                metadata: HashMap::new(),
                sections: Vec::new(),
            },
        ];

        assert_eq!(sections, expected)
    }

    #[test]
    fn parses_top_level_sections_where_sections_have_reverse_ordering() {
        let input = "### First Top Level
## Second Top Level
# Third Top Level";
        let (_, sections) = JournalEntryParser::new(input)
            .parse()
            .expect("unable to parse input");

        let expected = vec![
            Section {
                title: String::from("First Top Level"),
                level: SectionLevel::H3,
                body: String::from(""),
                metadata: HashMap::new(),
                sections: Vec::new(),
            },
            Section {
                title: String::from("Second Top Level"),
                level: SectionLevel::H2,
                body: String::from(""),
                metadata: HashMap::new(),
                sections: Vec::new(),
            },
            Section {
                title: String::from("Third Top Level"),
                level: SectionLevel::H1,
                body: String::from(""),
                metadata: HashMap::new(),
                sections: Vec::new(),
            },
        ];

        assert_eq!(sections, expected)
    }

    #[test]
    fn parses_top_level_sections_where_sections_have_h2_level() {
        let input = "## First Top Level
## Second Top Level
## Third Top Level";
        let (_, sections) = JournalEntryParser::new(input)
            .parse()
            .expect("unable to parse input");

        let expected = vec![
            Section {
                title: String::from("First Top Level"),
                level: SectionLevel::H2,
                body: String::from(""),
                metadata: HashMap::new(),
                sections: Vec::new(),
            },
            Section {
                title: String::from("Second Top Level"),
                level: SectionLevel::H2,
                body: String::from(""),
                metadata: HashMap::new(),
                sections: Vec::new(),
            },
            Section {
                title: String::from("Third Top Level"),
                level: SectionLevel::H2,
                body: String::from(""),
                metadata: HashMap::new(),
                sections: Vec::new(),
            },
        ];

        assert_eq!(sections, expected)
    }

    #[test]
    fn parses_top_level_sections_with_nested_sections() {
        let input = "# First Top Level
Test
## First Nested
Test
### Inner Nested
Test
## Second Nested
Test
# Second Top Level
Test";
        let (_, sections) = JournalEntryParser::new(input)
            .parse()
            .expect("unable to parse input");

        let expected = vec![
            Section {
                title: String::from("First Top Level"),
                level: SectionLevel::H1,
                body: String::from("Test"),
                metadata: HashMap::new(),
                sections: vec![
                    Section {
                        title: String::from("First Nested"),
                        level: SectionLevel::H2,
                        body: String::from("Test"),
                        metadata: HashMap::new(),
                        sections: vec![Section {
                            title: String::from("Inner Nested"),
                            level: SectionLevel::H3,
                            body: String::from("Test"),
                            metadata: HashMap::new(),
                            sections: Vec::new(),
                        }],
                    },
                    Section {
                        title: String::from("Second Nested"),
                        level: SectionLevel::H2,
                        body: String::from("Test"),
                        metadata: HashMap::new(),
                        sections: Vec::new(),
                    },
                ],
            },
            Section {
                title: String::from("Second Top Level"),
                level: SectionLevel::H1,
                body: String::from("Test"),
                metadata: HashMap::new(),
                sections: Vec::new(),
            },
        ];

        assert_eq!(sections, expected)
    }
}
