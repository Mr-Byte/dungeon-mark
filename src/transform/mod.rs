use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{config::Config, error::Result, journal::Journal};

mod metadata;

pub trait Transformer {
    fn name(&self) -> &str;

    fn run(&self, ctx: &TransformerContext, journal: Journal) -> Result<Journal>;

    // TODO: Do I need to add a "supports renderer" method?
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformerContext {
    pub root: PathBuf,

    pub config: Config,
}
