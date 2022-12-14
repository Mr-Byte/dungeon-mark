use crate::error::Result;

use serde::{Deserialize, Serialize};

pub trait Preprocessor {
    fn name(&self) -> &str;

    fn run(&self, ctx: &PreprocessorContext) -> Result<()>;

    // TODO: Do I need to add a "supports renderer" method?
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PreprocessorContext;
