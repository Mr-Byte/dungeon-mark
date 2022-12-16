use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::compendium::{Document, Journal};
use crate::error::Result;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compendium {
    pub journal: Journal,
    pub documents: Vec<Document>,
}

impl Compendium {
    pub(crate) fn load(
        _root: impl Into<PathBuf>,
        _config: crate::config::Config,
    ) -> Result<Compendium> {
        todo!()
    }
}
