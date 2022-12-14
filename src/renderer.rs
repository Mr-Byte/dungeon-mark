use crate::error::Result;

use serde::{Deserialize, Serialize};

pub trait Renderer {
    fn name(&self) -> &str;

    fn render(&self, ctx: &RenderContext) -> Result<()>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenderContext;
