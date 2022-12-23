use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{config::Config, error::Result, model::journal::Journal};

pub trait Renderer {
    fn name(&self) -> &str;

    fn render(&self, ctx: &RenderContext, journal: &Journal) -> Result<()>;
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub struct RenderContext {
    pub root: PathBuf,
    pub config: Config,
}

impl RenderContext {
    pub fn new(root: PathBuf, config: Config) -> Self {
        Self { root, config }
    }
}
