use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use memchr::memmem::Finder;

use super::{Preprocessor, PreprocessorContext};
use crate::error::Result;
use crate::model::journal::{Journal, JournalEntry, JournalItem};

const OPEN_SEQUENCE: &str = "{{#";
const CLOSE_SEQUENCE: &str = "}}";

/// A preprocessor that will look for directives in the form of `{{#...}}` in journal entry bodies and
/// perform transforms to replace those directives.
/// - `{{#title ...}}` Replace the title of the document with another title.
/// - `{{#include ...}}` Include an arbitrary file from disk, relative to the location of the journal entry.
pub struct DirectivePreprocessor {
    open_finder: Finder<'static>,
    close_finder: Finder<'static>,
}

impl DirectivePreprocessor {
    pub(crate) fn new() -> Self {
        Self {
            open_finder: Finder::new(OPEN_SEQUENCE),
            close_finder: Finder::new(CLOSE_SEQUENCE),
        }
    }
}

impl Preprocessor for DirectivePreprocessor {
    fn name(&self) -> &str {
        "directive"
    }

    fn run(&self, ctx: &PreprocessorContext, mut journal: Journal) -> Result<Journal> {
        for item in &mut journal.items {
            let JournalItem::Entry(ref mut entry) = item else {
                continue;
            };

            self.preprocess_entry(ctx, entry)?;
        }

        Ok(journal)
    }
}

impl DirectivePreprocessor {
    fn preprocess_entry(&self, ctx: &PreprocessorContext, entry: &mut JournalEntry) -> Result<()> {
        let Some(ref body) = entry.body else {
            return Ok(());
        };

        let mut input = &body.clone()[..];
        let mut processed_body = Vec::new();

        while let Some(start) = self.open_finder.find(input.as_bytes()) {
            let Some(end) = self.close_finder.find(input.as_bytes()) else {
                anyhow::bail!("Cannot find matching closing brace pair")
            };

            let end = end + CLOSE_SEQUENCE.len();

            if start >= end {
                anyhow::bail!("Closing brace pair found before opening brace pair")
            }

            let directive = &input[start..end];
            let replacement = preprocess_directive(ctx, entry, directive)?;

            processed_body.push(String::from(&input[..start]));
            processed_body.push(replacement);
            input = &input[end..];
        }

        // let mut entry = entry.clone();
        entry.body = Some(processed_body.join(""));

        Ok(())
    }
}

fn preprocess_directive(
    ctx: &PreprocessorContext,
    entry: &mut JournalEntry,
    directive: &str,
) -> Result<String> {
    let Some(parsed_directive) = directive
        .strip_prefix(OPEN_SEQUENCE) else {
            anyhow::bail!("Directive must start with {{#")
        };

    let Some(parsed_directive) = parsed_directive
        .strip_suffix(CLOSE_SEQUENCE) else {
            anyhow::bail!("Directive must end with }}")
        };

    // Directive was a title replacement.
    if let Some(title) = parsed_directive.strip_prefix("title") {
        entry.title = String::from(title.trim());
        return Ok(String::from(""));
    }

    if let Some(path) = parsed_directive.strip_prefix("include") {
        let Some(ref entry_path) = entry.path else {
            anyhow::bail!("The given journal entry has no file path and cannot have #include directives");
        };

        let path = PathBuf::from(path.trim());
        let mut include_path = ctx.root.join(&ctx.config.journal.source).join(entry_path);
        include_path.pop();
        include_path.push(path);

        return fs::read_to_string(&include_path)
            .with_context(|| format!("failed to open file: {}", include_path.display()));
    }

    // Unmatched directive, leave it be.
    return Ok(String::from(directive));
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use crate::{build::preprocess::PreprocessorContext, config::Config};

    fn new_journal(input: &str) -> Journal {
        Journal {
            title: None,
            items: vec![JournalItem::Entry(JournalEntry {
                title: String::from("Test"),
                body: Some(String::from(input)),
                sections: Vec::new(),
                path: None,
            })],
        }
    }

    #[test]
    fn succeeds_with_balanced_braces() {
        let body = "{{#title test}} {{#title test}}";
        let journal = new_journal(body);
        let preprocessor = DirectivePreprocessor::new();
        let ctx = PreprocessorContext::new(PathBuf::from("test"), Config::default());

        preprocessor
            .run(&ctx, journal)
            .expect("failed to unwrap balanced braces");
    }

    #[test]
    fn updates_title_with_directive() {
        let body = "{{#title Test Title}}";
        let journal = new_journal(body);
        let preprocessor = DirectivePreprocessor::new();
        let ctx = PreprocessorContext::new(PathBuf::from("test"), Config::default());
        let journal = preprocessor
            .run(&ctx, journal)
            .expect("failed to unwrap balanced braces");

        let JournalItem::Entry(ref entry) = journal.items[0] else {
            panic!("first item was not an entry")
        };

        assert_eq!("Test Title", entry.title)
    }

    #[test]
    #[should_panic]
    fn fails_with_unbalanced_braces() {
        let body = "}}test{{#";
        let journal = new_journal(body);
        let preprocessor = DirectivePreprocessor::new();
        let ctx = PreprocessorContext::new(PathBuf::from("test"), Config::default());

        preprocessor.run(&ctx, journal).unwrap();
    }

    #[test]
    #[should_panic]
    fn fails_with_no_directive_closure() {
        let body = "{{#include";
        let journal = new_journal(body);
        let preprocessor = DirectivePreprocessor::new();
        let ctx = PreprocessorContext::new(PathBuf::from("test"), Config::default());

        preprocessor.run(&ctx, journal).unwrap();
    }
}
