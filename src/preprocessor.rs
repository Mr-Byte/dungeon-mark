use serde::{Deserialize, Serialize};

use crate::{compendium::Compendium, error::Result};

pub trait Preprocessor {
    fn name(&self) -> &str;

    fn run(&self, ctx: &PreprocessorContext, compendium: Compendium) -> Result<Compendium>;

    // TODO: Do I need to add a "supports renderer" method?
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub struct PreprocessorContext;
