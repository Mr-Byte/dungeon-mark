use dungeon_mark::{
    build::render::{RenderContext, Renderer},
    config::Config,
    error::Result,
    model::journal::Journal,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Default)]
pub struct TestRenderer(Rc<RefCell<Option<Journal>>>, Rc<RefCell<Option<Config>>>);

impl TestRenderer {
    #[allow(dead_code)] // Avoid a false positive on the dead code analysis.
    pub fn journal(&self) -> Journal {
        self.0.borrow_mut().take().expect("result was not set")
    }

    #[allow(dead_code)] // Avoid a false positive on the dead code analysis.
    pub fn config(&self) -> Config {
        self.1.borrow_mut().take().expect("result was not set")
    }
}

impl Renderer for TestRenderer {
    fn name(&self) -> &str {
        "test_renderer"
    }

    fn render(&self, ctx: RenderContext) -> Result<()> {
        *self.0.borrow_mut() = Some(ctx.journal.clone());
        *self.1.borrow_mut() = Some(ctx.config.clone());

        Ok(())
    }
}
