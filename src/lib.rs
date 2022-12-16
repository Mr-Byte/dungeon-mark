//! All documents in a book go through:
//!     Preprocessor -> Document Tree Generator -> Section Processor -> Renderer
//!

#![deny(rust_2018_idioms)]

mod cmark;
mod parser;

pub mod compendium;
pub mod config;
pub mod preprocessor;
pub mod renderer;

pub mod error {
    pub use anyhow::{Error, Result};
}
