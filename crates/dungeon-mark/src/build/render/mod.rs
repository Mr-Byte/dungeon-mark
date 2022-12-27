mod command;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{config::Config, error::Result, model::journal::Journal};

pub use command::*;

pub trait Renderer {
    fn name(&self) -> &str;

    fn render(&self, ctx: RenderContext) -> Result<()>;
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub struct RenderContext {
    /// The root directory of the journal.toml file.
    pub root: PathBuf,
    /// The directory where the renderer **must** put its output.
    /// This directory is not guaranteed to be empty nor to exist.
    pub destination: PathBuf,
    /// The configuration of the book.
    pub config: Config,
    /// The journal itself.
    pub journal: Journal,
}

impl RenderContext {
    pub fn new(root: PathBuf, destination: PathBuf, config: Config, journal: Journal) -> Self {
        Self {
            root,
            destination,
            config,
            journal,
        }
    }
}
