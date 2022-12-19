use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::journal::JournalEntry;

use super::{Link, TOCItem, TableOfContents};

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub items: Vec<JournalItem>,
}

impl Journal {
    pub fn load(root_path: impl Into<PathBuf>, config: crate::config::Config) -> Result<Journal> {
        let source_path = root_path.into().join(config.journal.source);
        let toc = TableOfContents::load(&source_path)?;
        let items = load_journal_items(source_path, &toc.items)?;
        let journal = Self { items };

        Ok(journal)
    }
}

fn load_journal_items(
    source_path: impl Into<PathBuf>,
    items: &[TOCItem],
) -> Result<Vec<JournalItem>> {
    let mut results = Vec::new();
    let source_path = source_path.into();

    for item in items {
        match item {
            TOCItem::Link(Link {
                name,
                location,
                nested_items,
            }) => {
                if let Some(location) = location {
                    let entry = JournalEntry::load(name.clone(), &source_path, &location)?;
                    results.push(JournalItem::Entry(entry));

                    let nested_items = load_journal_items(&source_path, &nested_items)?;
                    results.extend(nested_items);
                }
            }
            TOCItem::SectionTitle(_) => (),
            TOCItem::Separator => (),
        }
    }

    Ok(results)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JournalItem {
    Entry(JournalEntry),
}
