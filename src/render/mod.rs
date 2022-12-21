use serde::{Deserialize, Serialize};

use crate::error::Result;

pub trait Renderer {
    fn name(&self) -> &str;

    fn render(&self, ctx: &RenderContext) -> Result<()>;
}

#[non_exhaustive]
#[derive(Debug, Serialize, Deserialize)]
pub struct RenderContext;
