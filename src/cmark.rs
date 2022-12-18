//! Useful utilities for parsing and working with CommonMark files.

mod parser;

use std::borrow::Borrow;

pub use parser::*;

use pulldown_cmark::Event;
use pulldown_cmark_to_cmark::cmark;

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
        cmark(self, &mut buffer)?;

        Ok(buffer)
    }
}
