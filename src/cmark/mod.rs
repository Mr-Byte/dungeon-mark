//! Useful utilities for parsing and working with CommonMark files.

mod parser;

pub use parser::*;

use pulldown_cmark::Event;
use pulldown_cmark_to_cmark::{cmark_with_options, Options};
use std::borrow::Borrow;

use crate::error::Result;

pub trait EventIteratorExt {
    /// Consume an event collection and return a stringified representation.
    fn stringify(self) -> Result<String>;
}

impl<'a, I, E> EventIteratorExt for I
where
    I: Iterator<Item = E>,
    E: Borrow<Event<'a>>,
{
    fn stringify(self) -> Result<String> {
        // TODO: Is there a safe default buffer capacity? Does it matter?
        let mut buffer = String::new();
        let options = Options {
            code_block_token_count: 3,
            ..Default::default()
        };

        cmark_with_options(self, &mut buffer, options)?;

        Ok(buffer)
    }
}
