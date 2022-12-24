use anyhow::Context;
use pulldown_cmark::{Event, HeadingLevel, Tag};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    cmark::{CMarkParser, EventIteratorExt as _},
    error::Result,
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum SectionLevel {
    #[default]
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
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Section {
    /// The title of the section as provided by the heading.
    pub title: String,
    /// The heading level of the section ranging from H1 to H6.
    pub level: SectionLevel,
    /// All text that follows this section, excluding the text of any child sections
    /// or sibling sections.
    pub body: String,
    /// Metadata associated with a section.
    pub metadata: HashMap<String, SectionMetadata>,
    /// Any child sections that are nested below the current section.
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionMetadata {
    pub lang: String,
    pub data: String,
}

/// A `JournalEntry` is an in-memory representation of a single Markdown file on disk.
/// It is organized into sections based on headings.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JournalEntry {
    // The title of the journal entry.
    pub title: String,
    /// An optional top level journal entry body, which makes up any elements that preceed the first
    /// heading in the journal entry.
    pub body: Option<String>,
    /// The sections (delineated by Markdown headings) of the journal entry.
    pub sections: Vec<Section>,
    /// The location of this journal entry relative to the `JOURNAL.md` file.
    pub path: Option<PathBuf>,
    /// The nesting level of the journal entry (up to H6).
    pub level: u8,
}

impl JournalEntry {
    pub fn load(
        title: String,
        source_path: impl Into<PathBuf>,
        path: impl Into<PathBuf>,
        level: u8,
    ) -> Result<JournalEntry> {
        let source_path = source_path.into();
        let path = path.into();
        let file_path = source_path.join(&path);
        let body = fs::read_to_string(&file_path)
            .with_context(|| format!("Failed to open journal entry: {}", file_path.display()))?;

        let document = Self {
            title,
            path: Some(path),
            body: Some(body),
            sections: Vec::new(),
            level: level.into(),
        };

        Ok(document)
    }

    pub fn parse(mut self) -> Result<JournalEntry> {
        let Some(body) = self.body else {
            return Ok(self);
        };

        let parser = JournalEntryParser::new(&body);
        let (body, sections) = parser.parse()?;
        self.sections.extend(sections);

        Ok(Self { body, ..self })
    }

    /// Iterate over a flattened representation of all sections in a journal entry, providing a mutable reference
    /// to each entry.
    pub fn for_each_mut<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut Section),
    {
        for_each_mut(&mut func, &mut self.sections)
    }

    /// Iterate over a flattened representation of all sections in a journal entry, providing a mutable reference
    /// to each entry. Stops iterating on the first closure to return an error.
    pub fn try_for_each_mut<F>(&mut self, mut func: F) -> Result<()>
    where
        F: FnMut(&mut Section) -> Result<()>,
    {
        try_for_each_mut(&mut func, &mut self.sections)
    }
}

fn for_each_mut<'a, I, F>(func: &mut F, sections: I)
where
    I: IntoIterator<Item = &'a mut Section>,
    F: FnMut(&mut Section),
{
    for section in sections {
        for_each_mut(func, &mut section.sections);

        func(section);
    }
}

fn try_for_each_mut<'a, I, F>(func: &mut F, sections: I) -> Result<()>
where
    I: IntoIterator<Item = &'a mut Section>,
    F: FnMut(&mut Section) -> Result<()>,
{
    for section in sections {
        try_for_each_mut(func, &mut section.sections)?;

        func(section)?;
    }

    Ok(())
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
        let entry = JournalEntry {
            body: Some(String::from(input)),
            ..Default::default()
        };
        let entry = entry.parse().expect("should parse");

        let expected = Some(String::from(input));

        assert_eq!(expected, entry.body);
    }

    #[test]
    fn parses_top_level_sections() {
        let input = "# First Top Level
# Second Top Level";
        let entry = JournalEntry {
            body: Some(String::from(input)),
            ..Default::default()
        };
        let entry = entry.parse().expect("should parse");

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

        assert_eq!(expected, entry.sections);
    }

    #[test]
    fn parses_top_level_sections_where_sections_have_reverse_ordering() {
        let input = "### First Top Level
## Second Top Level
# Third Top Level";
        let entry = JournalEntry {
            body: Some(String::from(input)),
            ..Default::default()
        };
        let entry = entry.parse().expect("should parse");

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

        assert_eq!(expected, entry.sections);
    }

    #[test]
    fn parses_top_level_sections_where_sections_have_h2_level() {
        let input = "## First Top Level
## Second Top Level
## Third Top Level";
        let entry = JournalEntry {
            body: Some(String::from(input)),
            ..Default::default()
        };
        let entry = entry.parse().expect("should parse");

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

        assert_eq!(expected, entry.sections);
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
        let entry = JournalEntry {
            body: Some(String::from(input)),
            ..Default::default()
        };
        let entry = entry.parse().expect("should parse");

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

        assert_eq!(expected, entry.sections);
    }
}
