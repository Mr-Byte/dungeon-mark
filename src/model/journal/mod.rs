mod entry;

pub use entry::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum JournalItem {
    Entry(JournalEntry),
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Journal {
    pub items: Vec<JournalItem>,
}

// impl Journal {
//     pub fn load(
//         root: impl Into<PathBuf>,
//         config: crate::config::Config,
//         preprocessors: Vec<Box<dyn Preprocessor>>,
//     ) -> Result<Journal> {
//         let root = root.into();
//         let source_path = root.join(&config.journal.source);
//         let toc = TableOfContents::load(&source_path)?;
//         let ctx = PreprocessorContext { root, config };
//         let items = load_journal_items(source_path, &toc.items, &ctx, &preprocessors)?;
//         let journal = Self { items };

//         Ok(journal)
//     }
// }

// fn load_journal_items(
//     source_path: impl Into<PathBuf>,
//     items: &[TOCItem],
//     ctx: &PreprocessorContext,
//     preprocessors: &[Box<dyn Preprocessor>],
// ) -> Result<Vec<JournalItem>> {
//     let mut results = Vec::new();
//     let source_path = source_path.into();

//     for item in items {
//         match item {
//             TOCItem::Link(Link {
//                 name,
//                 location,
//                 nested_items,
//             }) => {
//                 if let Some(location) = location {
//                     let document = Document::load(name.clone(), &source_path, &location)?;
//                     let document = preprocessors.iter().run(ctx, document)?;
//                     let entry = JournalEntry::from_document(document)?;

//                     results.push(JournalItem::Entry(entry));

//                     let nested_items =
//                         load_journal_items(&source_path, &nested_items, ctx, preprocessors)?;
//                     results.extend(nested_items);
//                 }
//             }
//             TOCItem::SectionTitle(_) => (),
//             TOCItem::Separator => (),
//         }
//     }

//     Ok(results)
// }
