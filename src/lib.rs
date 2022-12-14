/// All documents in a book go through:
///     Preprocessor -> Document Tree Generator -> Section Processor -> Renderer
mod config;
mod document;
mod preprocessor;
mod renderer;

pub mod error {
    pub use anyhow::Result;
}
