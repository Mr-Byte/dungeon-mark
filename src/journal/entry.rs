use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, str::FromStr};

use crate::{
    cmark::CMarkParser,
    error::{Error, Result},
};

/// A `Section` represents all text following a heading in a `JournalEntry`.
/// Any headings that have a lower-level than the `Section` that follow the section
/// will be nested inside this section. Any `Section` with the same level as the
/// current section will be a sibling section in the parent `Section` or `JournalEntry`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    /// The name of the section as provided by the heading.
    pub name: String,
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
    pub path: PathBuf,
    /// The sections (delineated by Markdown headings) of the document.
    pub sections: Vec<Section>,
}

impl FromStr for JournalEntry {
    type Err = Error;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        JournalEntryParser::new(source).parse()
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

    fn parse(mut self) -> Result<JournalEntry> {
        todo!()
    }
}
