use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::Read, path::PathBuf};

use crate::error::Result;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// The path of the document relative to the location of `JOURNAL.md`.
    pub path: PathBuf,
    /// The contents of the document as stored on disk.
    pub content: String,
    /// The title of the document as specified by the `JOURNAL.md` file.
    pub title: String,
}

impl Document {
    pub fn load(
        title: String,
        source_path: impl Into<PathBuf>,
        path: impl Into<PathBuf>,
    ) -> Result<Document> {
        let mut content = String::new();
        let source_path = source_path.into();
        let path = path.into();
        let file_path = source_path.join(&path);

        File::open(&file_path)
            .with_context(|| format!("Failed to open journal entry: {}", file_path.display()))?
            .read_to_string(&mut content)
            .with_context(|| format!("Failed to read journal entry: {}", file_path.display()))?;

        let document = Document {
            path,
            title,
            content,
        };

        Ok(document)
    }
}
