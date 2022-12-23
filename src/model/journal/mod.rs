mod entry;

pub use entry::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChapterTitle {
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JournalItem {
    Entry(JournalEntry),
    ChapterTitle(ChapterTitle),
    Separator,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Journal {
    pub title: Option<String>,
    pub items: Vec<JournalItem>,
}
