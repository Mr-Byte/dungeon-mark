#![deny(rust_2018_idioms)]
#![deny(clippy::all)]
#![allow(clippy::module_inception)]

pub mod cmark;
pub mod config;
pub mod document;
pub mod journal;
pub mod preprocess;
pub mod render;
pub mod transform;

pub mod error {
    pub use anyhow::{Error, Result};
}
