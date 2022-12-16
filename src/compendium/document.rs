use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

/// A `Document` is an in-memory representation of a single Markdown file on disk.
/// It is organized into sections based on headings. A documen's contents can be
/// transformed by a `DocumentProcessor` during the document processing phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// The location of the document relative to the "source root" config option.
    pub path: PathBuf,
    /// The sections (delineated by Markdown headings) of the document.
    pub sections: Vec<Section>,
}

/// A `Section` represents all text following a heading in a `Document`.
/// Any headings that have a lower-level than the `Section` that follow the section
/// will be nested inside this section. Any `Section` with the same level as the
/// current section will be a sibling section in the parent `Section` or `Document`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    /// The name of the section as provided by the heading.
    pub name: String,
    /// All text that follows this section, excluding the text of any child sections
    /// or sibling sections.
    pub body: String,
    /// Metadata associated with a section (usually extracted by a metadata document processor stage).
    pub metadata: HashMap<String, String>,
    /// Any child sections that are nested below the current section.
    pub sections: Vec<Section>,
}
