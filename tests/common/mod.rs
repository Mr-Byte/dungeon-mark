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
    pub fn journal(&self) -> Journal {
        self.0.borrow_mut().take().expect("result was not set")
    }

    pub fn config(&self) -> Config {
        self.1.borrow_mut().take().expect("result was not set")
    }
}

impl Renderer for TestRenderer {
    fn name(&self) -> &str {
        "test_renderer"
    }

    fn render(&self, ctx: &RenderContext, journal: &Journal) -> Result<()> {
        *self.0.borrow_mut() = Some(journal.clone());
        *self.1.borrow_mut() = Some(ctx.config.clone());

        Ok(())
    }
}
