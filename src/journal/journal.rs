use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::journal::JournalEntry;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub items: Vec<JournalEntry>,
}

impl Journal {
    pub(crate) fn load(
        root_path: impl Into<PathBuf>,
        config: crate::config::Config,
    ) -> Result<Journal> {
        let _source_path = root_path.into().join(config.journal.source);

        // TODO: Load the TOC from disk and extract items from it
        // TODO: Report entries that don't exist.

        let journal = Self { items: Vec::new() };

        Ok(journal)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JournalItem {
    Entry(JournalEntry),
}
