use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{config::Config, error::Result, model::journal::Journal};

/// A preprocessor will take a journal with unparsed entries (all contents are in the body, no sections)
/// and transforms that journal prior to running it through the parsing stage.
pub trait Preprocessor {
    fn name(&self) -> &str;

    fn run(&self, ctx: &PreprocessorContext, journal: Journal) -> Result<Journal>;
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreprocessorContext {
    /// Absolute path to the root of the journal (where journal.toml lives).
    pub root: PathBuf,

    /// Configuration for the journal from the journal.toml file.
    pub config: Config,
}

impl PreprocessorContext {
    pub(crate) fn new(root: PathBuf, config: Config) -> Self {
        Self { root, config }
    }
}
