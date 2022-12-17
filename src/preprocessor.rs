use serde::{Deserialize, Serialize};

use crate::{error::Result, journal::Journal};

pub trait Preprocessor {
    fn name(&self) -> &str;

    fn run(&self, ctx: &PreprocessorContext, journal: Journal) -> Result<Journal>;

    // TODO: Do I need to add a "supports renderer" method?
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub struct PreprocessorContext;
