use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::journal::{JournalEntry, TableOfContents};

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub table_of_contents: TableOfContents,
    pub documents: Vec<JournalEntry>,
}

impl Journal {
    pub(crate) fn load(
        _root: impl Into<PathBuf>,
        _config: crate::config::Config,
    ) -> Result<Journal> {
        todo!()
    }
}
