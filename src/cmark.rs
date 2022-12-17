//! Useful utilities for parsing and working with CommonMark files.

mod parser;

pub use parser::*;

use pulldown_cmark::Event;
use pulldown_cmark_to_cmark::cmark;

use crate::error::Result;

pub trait EventCollectionExt {
    /// Consume an event collection and return a stringified representation.
    fn stringify(self) -> Result<String>;
}

impl<'a, T> EventCollectionExt for T
where
    T: IntoIterator<Item = Event<'a>>,
{
    fn stringify(self) -> Result<String> {
        // TODO: Is there a safe default buffer capacity? Does it matter?
        let mut buffer = String::new();
        cmark(self.into_iter(), &mut buffer)?;

        Ok(buffer)
    }
}
