#![deny(rust_2018_idioms)]
#![deny(clippy::all)]
#![allow(clippy::module_inception)]

pub mod build;
pub mod cmark;
pub mod config;
pub mod model;

pub mod error {
    pub use anyhow::{Error, Result};
}
