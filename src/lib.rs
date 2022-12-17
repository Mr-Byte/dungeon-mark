#![deny(rust_2018_idioms)]

pub mod cmark;
pub mod config;
pub mod journal;
pub mod preprocessor;
pub mod renderer;

pub mod error {
    pub use anyhow::{Error, Result};
}
