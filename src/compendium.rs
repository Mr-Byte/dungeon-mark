use std::path::PathBuf;

use crate::{config::Config, error::Result, preprocessor::Preprocessor, renderer::Renderer};

mod compendium;
mod document;
mod journal;

pub use compendium::*;
pub use document::*;
pub use journal::*;

pub struct DMCompendium {
    /// The root directory of the compendium.
    pub root: PathBuf,
    /// Build configuration for the compendium.
    pub config: Config,
    /// An in-memory representation of the compendium.
    pub compendium: Compendium,

    /// Preprocessors applied to the entirety of a compendium.
    preprocessors: Vec<Box<dyn Preprocessor>>,
    /// Renderers used to output the contents of a compendium in various formats.
    renderers: Vec<Box<dyn Renderer>>,
}

impl DMCompendium {
    pub fn load(root: impl Into<PathBuf>) -> Result<DMCompendium> {
        let root = root.into();
        let config_location = root.join("compendium.toml");

        let config = if config_location.exists() {
            Config::load(config_location)?
        } else {
            Config::default()
        };

        DMCompendium::load_with_config(root, config)
    }

    pub fn load_with_config(root: impl Into<PathBuf>, config: Config) -> Result<DMCompendium> {
        let root = root.into();
        let compendium = Compendium::load(&root, config.clone())?;

        let compendium = DMCompendium {
            root,
            config,
            compendium,
            preprocessors: Vec::new(),
            renderers: Vec::new(),
        };

        Ok(compendium)
    }
}
