use std::collections::HashMap;

use pulldown_cmark::{CodeBlockKind, Event, Tag};

use super::Transformer;

use crate::{
    cmark::{self, EventIteratorExt},
    error::Result,
    journal::{Journal, JournalItem, Section, SectionMetadata},
};

pub struct MetadataPreprocessor;

impl MetadataPreprocessor {
    const NAME: &str = "metadata";
}

impl Transformer for MetadataPreprocessor {
    fn name(&self) -> &str {
        Self::NAME
    }

    fn run(&self, _ctx: &super::TransformerContext, mut journal: Journal) -> Result<Journal> {
        for item in &mut journal.items {
            #[allow(irrefutable_let_patterns)]
            if let JournalItem::Entry(entry) = item {
                entry.try_for_each_mut(extract_metadata)?;
            }
        }

        Ok(journal)
    }
}

fn extract_metadata(section: &mut Section) -> Result<()> {
    let mut body = Vec::new();
    let mut metadata = HashMap::new();
    let mut events = cmark::CMarkParser::new(&section.body);

    loop {
        match events.peek_event() {
            Some(Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(tag))))
                if is_metadata_block(tag) =>
            {
                let (lang, key) = parse_metadata_tag(tag);
                events.next_event();

                let data = events
                    .iter_until_and_consume(|event| {
                        matches! {
                            event,
                            Event::End(Tag::CodeBlock(CodeBlockKind::Fenced(_)))
                        }
                    })
                    .stringify()?;
                let section_meta = SectionMetadata { lang, data };

                metadata.insert(key, section_meta);
                body.push(String::from("\n\n")); // Replace the missing code block with a hard break.
            }
            Some(_) => {
                let text = events
                    .iter_until(|event| {
                        matches! {
                            event,
                            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(tag))) if is_metadata_block(tag)
                        }
                    })
                    .stringify()?;

                body.push(text);
            }
            None => {
                events.next_event();
                break;
            }
        }
    }

    section.body = body.into_iter().collect();
    section.metadata.extend(metadata);

    Ok(())
}

fn is_metadata_block(tag: &str) -> bool {
    let parts: Vec<_> = tag.split(",").map(|part| part.trim()).collect();

    match &parts[..] {
        [_, "metadata", _] => true,
        _ => false,
    }
}

fn parse_metadata_tag(tag: &str) -> (String, String) {
    let parts: Vec<_> = tag.split(",").map(|part| part.trim()).collect();
    let [lang, "metadata", key] = &parts[..] else {
        unreachable!("is_metadata_block invariant was violated")
    };

    (lang.to_string(), key.to_string())
}

#[cfg(test)]
mod test {
    use std::{path::PathBuf, str::FromStr};

    use super::*;
    use crate::{config::Config, journal::JournalEntry, transform::TransformerContext};

    #[test]
    fn extracts_metadata_as_expected() {
        let section_body = r#"Test section
```toml,metadata,test
This is test data
```
Following text"#;

        let original_journal = Journal {
            items: vec![JournalItem::Entry(JournalEntry {
                name: String::from("test"),
                body: None,
                sections: vec![Section {
                    title: String::from("test"),
                    body: String::from(section_body),
                    ..Default::default()
                }],
                entry_path: None,
            })],
        };

        let ctx = TransformerContext {
            root: PathBuf::from_str("test").expect("should parse"),
            config: Config::default(),
        };

        let actual_journal = MetadataPreprocessor
            .run(&ctx, original_journal)
            .expect("journal should be preprocessed");

        let mut metadata = HashMap::new();
        metadata.insert(
            String::from("test"),
            SectionMetadata {
                lang: String::from("toml"),
                data: String::from("This is test data\n"),
            },
        );

        let expected_journal = Journal {
            items: vec![JournalItem::Entry(JournalEntry {
                name: String::from("test"),
                body: None,
                sections: vec![Section {
                    title: String::from("test"),
                    body: String::from("Test section\n\nFollowing text"),
                    metadata,
                    ..Default::default()
                }],
                entry_path: None,
            })],
        };

        assert_eq!(expected_journal, actual_journal);
    }

    #[test]
    fn leaves_code_blocks_not_tagged_as_metdata_alone() {
        let section_body = r#"Test section

```toml
This is test data
```

Following text"#;

        let original_journal = Journal {
            items: vec![JournalItem::Entry(JournalEntry {
                name: String::from("test"),
                body: None,
                sections: vec![Section {
                    title: String::from("test"),
                    body: String::from(section_body),
                    ..Default::default()
                }],
                entry_path: None,
            })],
        };

        let ctx = TransformerContext {
            root: PathBuf::from_str("test").expect("should parse"),
            config: Config::default(),
        };

        let actual_journal = MetadataPreprocessor
            .run(&ctx, original_journal)
            .expect("journal should be preprocessed");

        let expected_journal = Journal {
            items: vec![JournalItem::Entry(JournalEntry {
                name: String::from("test"),
                body: None,
                sections: vec![Section {
                    title: String::from("test"),
                    body: String::from(section_body),
                    ..Default::default()
                }],
                entry_path: None,
            })],
        };

        assert_eq!(expected_journal, actual_journal);
    }
}
