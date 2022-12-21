use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, path::PathBuf};

use crate::{config::Config, document::Document, error::Result};

/// A preprocessor takes an unparsed CommonMark file and applies transforms to the document
/// prior to it being fed through the journal parsing stage.
pub trait Preprocessor {
    fn name(&self) -> &str;

    fn run(&self, ctx: &PreprocessorContext, document: Document) -> Result<Document>;
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessorContext {
    /// Absolute path to the root of the journal (where journal.toml lives).
    pub root: PathBuf,

    /// Configuration for the journal from the journal.toml file.
    pub config: Config,
}

pub(crate) trait PreprocessorExt {
    fn run(self, ctx: &PreprocessorContext, document: Document) -> Result<Document>;
}

impl<I, P> PreprocessorExt for I
where
    I: Iterator<Item = P>,
    P: Borrow<Box<dyn Preprocessor>>,
{
    fn run(mut self, ctx: &PreprocessorContext, document: Document) -> Result<Document> {
        self.try_fold(document, |document, preprocessor| {
            preprocessor.borrow().run(ctx, document)
        })
    }
}
