use std::path::PathBuf;

use crate::{config::Config, error::Result, model::toc::TableOfContents};

use self::{preprocess::Preprocessor, render::Renderer, transform::Transformer};

pub mod preprocess;
pub mod render;
pub mod transform;

// - When new JournaBuilder is created
//    - Load config
//    - Configure preprocessors, transformers, and renderers
//    - Load table of contents
// - When build is called
//    - Go through table of contents
//        - Load any documents and pre-process them
//        - Convert documents to JournalEntry
//        - Append converted TOC items and journal entries into JournalItem and append to journal
//    - Run all transformers on the produced journal
//    - Run all renderers on the transformed journal

// How to integration test this all??
// Do I have a special integration test renderer that just stores the journal passed in, for reference? 🤔

pub struct JournalBuilder {
    config: Config,
    table_of_contents: TableOfContents,
    preprocessors: Vec<Box<dyn Preprocessor>>,
    transformers: Vec<Box<dyn Transformer>>,
    renderers: Vec<Box<dyn Renderer>>,
}

impl JournalBuilder {
    pub fn load(root: impl Into<PathBuf>) -> Result<Self> {
        todo!()
    }

    pub fn build(self) -> Result<()> {
        todo!()
    }
}
