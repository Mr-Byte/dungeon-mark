use std::path::PathBuf;

use crate::{config::Config, error::Result, render::Renderer, transform::Transformer};

mod entry;
mod journal;
mod toc;

pub use entry::*;
pub use journal::*;
pub use toc::*;

pub struct DMJournal {
    /// The root directory of the journal.
    pub root: PathBuf,
    /// Build configuration for the journal.
    pub config: Config,
    /// An in-memory representation of the journal.
    pub journal: Journal,

    /// Transformers applied to the entirety of a journal.
    _transformers: Vec<Box<dyn Transformer>>,
    /// Renderers used to output the contents of a journal in various formats.
    _renderers: Vec<Box<dyn Renderer>>,
}

impl DMJournal {
    pub fn load(root: impl Into<PathBuf>) -> Result<DMJournal> {
        let root = root.into();
        let config_location = root.join("journal.toml");

        let config = if config_location.exists() {
            Config::load(config_location)?
        } else {
            Config::default()
        };

        DMJournal::load_with_config(root, config)
    }

    pub fn load_with_config(root: impl Into<PathBuf>, config: Config) -> Result<DMJournal> {
        let root = root.into();
        // TODO: Load and configure preprocessors from config.
        let preprocessors = Vec::new();

        let journal = Journal::load(&root, config.clone(), preprocessors)?;
        let journal = DMJournal {
            root,
            config,
            journal,
            _transformers: Vec::new(),
            _renderers: Vec::new(),
        };

        Ok(journal)
    }
}
