use serde::{Deserialize, Serialize};

use crate::{error::Result, model::journal::Journal};

pub trait Renderer {
    fn name(&self) -> &str;

    fn render(&self, ctx: &RenderContext, journal: &Journal) -> Result<()>;
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub struct RenderContext;
