use crate::model::journal::Journal;

use super::Preprocessor;

/// A preprocessor that will look for directives in the form of `{{#...}}` in journal entry bodies and
/// perform transforms to replace those directives.
/// - `{{#title ...}}` Replace the title of the document with another title.
/// - `{{#include ...}}` Include an arbitrary file from disk, relative to the location of the journal entry.
pub struct DirectivePreprocessor;

impl DirectivePreprocessor {
    pub(crate) fn new() -> Self {
        Self
    }
}

impl Preprocessor for DirectivePreprocessor {
    fn name(&self) -> &str {
        "directive"
    }

    fn run(&self, _ctx: &super::PreprocessorContext, journal: Journal) -> anyhow::Result<Journal> {
        Ok(journal)
    }
}
