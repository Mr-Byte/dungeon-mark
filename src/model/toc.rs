use anyhow::{anyhow, bail, Context};
use pulldown_cmark::{Event, HeadingLevel, Tag};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    cmark::{CMarkParser, EventIteratorExt},
    error::{Error, Result},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOfContents {
    /// An optional title for the TOC.
    pub title: Option<String>,
    /// All items making up the TOC.
    pub items: Vec<TOCItem>,
}

impl TableOfContents {
    /// Load the table of contents from JOURNAL.md relative to the provided path.
    pub fn load(source_path: impl AsRef<Path>) -> Result<Self> {
        let journal_path = source_path.as_ref().join("JOURNAL.md");
        let source = fs::read_to_string(&journal_path)
            .with_context(|| format!("Failed to open {}", journal_path.display()))?;

        let (title, items) = TOCParser::new(&source)
            .parse()
            .with_context(|| format!("Failed to parse {}", journal_path.display()))?;

        Ok(Self { title, items })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Link {
    /// The name of the section this link points to.
    pub name: String,
    /// An optional path (relative to the config's source root) pointed to by the link.
    pub location: Option<PathBuf>,
    /// Any table of content items nested below this link.
    pub nested_items: Vec<TOCItem>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionTitle {
    /// The title for a section of the table of content provided by an H1 level heading.
    pub title: String,
}

/// A table of contents item which is either a link, a separator, or a section title.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TOCItem {
    /// A link to a journal entry, including nested entries.
    Link(Link),
    /// Section title for a portion of the table of contents.
    SectionTitle(SectionTitle),
    /// A separator between unnamed sections.
    Separator,
}

impl TOCItem {
    pub fn maybe_link_mut(&mut self) -> Option<&mut Link> {
        match self {
            TOCItem::Link(ref mut link) => Some(link),
            _ => None,
        }
    }

    pub fn maybe_link(&self) -> Option<&Link> {
        match self {
            TOCItem::Link(ref link) => Some(link),
            _ => None,
        }
    }

    pub fn maybe_section_title_mut(&mut self) -> Option<&mut SectionTitle> {
        match self {
            TOCItem::SectionTitle(ref mut title) => Some(title),
            _ => None,
        }
    }

    pub fn maybe_section_title(&self) -> Option<&SectionTitle> {
        match self {
            TOCItem::SectionTitle(ref title) => Some(title),
            _ => None,
        }
    }

    pub fn is_separator(&self) -> bool {
        matches! { self, TOCItem::Separator }
    }
}

struct TOCParser<'a> {
    parser: CMarkParser<'a>,
}

impl<'a> TOCParser<'a> {
    fn new(source: &'a str) -> Self {
        let parser = CMarkParser::new(source);

        Self { parser }
    }

    fn parse(mut self) -> Result<(Option<String>, Vec<TOCItem>)> {
        let title = self.parse_title()?;
        let items = self.parse_toc()?;

        Ok((title, items))
    }

    fn parse_title(&mut self) -> Result<Option<String>> {
        loop {
            let event = self.parser.peek_event();
            match event {
                Some(Event::Start(Tag::Heading(HeadingLevel::H1, ..))) => {
                    // NOTE: Skip the start tag that was peeked.
                    self.parser.next_event();
                    let heading = self
                        .parser
                        .iter_until_and_consume(|event| {
                            matches!(event, Event::End(Tag::Heading(HeadingLevel::H1, ..)))
                        })
                        .stringify()?;

                    return Ok(Some(heading));
                }
                Some(Event::Html(_)) => {
                    self.parser.next_event(); // Skip HTML, such as comments.
                }
                _ => return Ok(None),
            }
        }
    }

    fn parse_toc(&mut self) -> Result<Vec<TOCItem>> {
        let mut toc_items = Vec::new();

        loop {
            let title = match self.parser.peek_event() {
                Some(Event::Start(Tag::Heading(HeadingLevel::H1, ..))) => {
                    self.parser.next_event();
                    let heading = self
                        .parser
                        .iter_until_and_consume(|event| {
                            matches! {
                                event,
                                Event::End(Tag::Heading(HeadingLevel::H1, .. ))
                            }
                        })
                        .stringify()?;

                    Some(heading)
                }
                Some(_) => None,
                None => break, // End of input, end parsing.
            };

            if let Some(title) = title {
                toc_items.push(TOCItem::SectionTitle(SectionTitle { title }));
            }

            let items = self
                .parse_toc_items()
                .with_context(|| "There was an error parsing TOC entries")?;

            toc_items.extend(items);
        }

        Ok(toc_items)
    }

    fn parse_toc_items(&mut self) -> Result<Vec<TOCItem>> {
        let mut items = Vec::new();

        loop {
            match self.parser.peek_event() {
                Some(Event::Start(Tag::Heading(HeadingLevel::H1, ..))) => break, // A new section is being started.
                Some(Event::Start(Tag::Item)) => {
                    self.parser.next_event();

                    let item = self.parse_toc_item()?;
                    items.push(item);
                }
                Some(Event::Start(Tag::List(..))) => {
                    self.parser.next_event();

                    if items.is_empty() {
                        continue;
                    }

                    if let Some(last_item) = items.last_mut().and_then(TOCItem::maybe_link_mut) {
                        last_item.nested_items = self.parse_toc_items()?;
                    }
                }
                Some(Event::End(Tag::List(..))) => {
                    self.parser.next_event();
                    break;
                }
                Some(Event::Start(other_tag)) => {
                    let other_tag = other_tag.clone();

                    while let Some(event) = self.parser.next_event() {
                        if event == Event::End(other_tag.clone()) {
                            break;
                        }
                    }
                }
                Some(Event::Rule) => {
                    self.parser.next_event();
                    items.push(TOCItem::Separator)
                }
                Some(_) => {
                    self.parser.next_event();
                }
                None => break,
            }
        }

        Ok(items)
    }

    fn parse_toc_item(&mut self) -> Result<TOCItem> {
        loop {
            match self.parser.next_event() {
                Some(Event::Start(Tag::Paragraph)) => continue,
                Some(Event::Start(Tag::Link(_, href, _))) => {
                    let link = self.parse_link(href.to_string())?;

                    return Ok(TOCItem::Link(link));
                }
                _ => {
                    bail!(
                        self.parse_error("Items in the table of contents must only contain links.")
                    )
                }
            }
        }
    }

    fn parse_link(&mut self, href: String) -> Result<Link> {
        let href = href.replace("%20", " ");
        let name: String = self
            .parser
            .iter_until_and_consume(|event| matches! {event, Event::End(Tag::Link(..))})
            .map(|event| match event {
                Event::SoftBreak => Event::Text(" ".into()),
                other => other,
            })
            .stringify()?;

        let location = if href.is_empty() {
            None
        } else {
            Some(PathBuf::from(href))
        };

        let link = Link {
            name,
            location,
            nested_items: Vec::new(),
        };

        Ok(link)
    }

    fn parse_error(&self, message: impl Display) -> Error {
        let position = self.parser.position();

        anyhow!(
            "failed to parse JOURNAL.md line: {}, column: {}: {}",
            position.line,
            position.column,
            message
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // Convenience function to parse out TOC.
    fn parse(source: &str) -> (Option<String>, Vec<TOCItem>) {
        TOCParser::new(source).parse().expect("TOC failed to parse")
    }

    #[test]
    fn parses_title() {
        let input = "# Journal Title";
        let (title, _) = parse(input);

        assert_eq!("Journal Title", title.expect("toc title was empty"))
    }

    #[test]
    fn skips_comments_and_parses_title() {
        let input = r"<!-- # Journal Title -->
# Actual Title
";
        let (title, _) = parse(input);

        assert_eq!("Actual Title", title.expect("toc title was empty"))
    }

    #[test]
    fn lists_all_top_level_links() {
        let input = r#"
* [Entry 1](entry1.md)
* [Entry 2](entry2.md)
"#;
        let (_, items) = parse(input);
        let expected = vec![
            TOCItem::Link(Link {
                name: String::from("Entry 1"),
                location: Some(PathBuf::from("entry1.md")),
                nested_items: Vec::new(),
            }),
            TOCItem::Link(Link {
                name: String::from("Entry 2"),
                location: Some(PathBuf::from("entry2.md")),
                nested_items: Vec::new(),
            }),
        ];

        assert_eq!(items, expected);
    }

    #[test]
    fn lists_all_top_level_links_separated_by_comments() {
        let input = r#"
* [Entry 1](entry1.md)
<!-- comment -->
* [Entry 2](entry2.md)
"#;
        let (_, items) = parse(input);
        let expected = vec![
            TOCItem::Link(Link {
                name: String::from("Entry 1"),
                location: Some(PathBuf::from("entry1.md")),
                nested_items: Vec::new(),
            }),
            TOCItem::Link(Link {
                name: String::from("Entry 2"),
                location: Some(PathBuf::from("entry2.md")),
                nested_items: Vec::new(),
            }),
        ];

        assert_eq!(items, expected);
    }

    #[test]
    fn lists_all_top_level_links_separated_by_separator() {
        let input = r#"
* [Entry 1](entry1.md)
---
* [Entry 2](entry2.md)
"#;

        let (_, items) = parse(input);
        let expected = vec![
            TOCItem::Link(Link {
                name: String::from("Entry 1"),
                location: Some(PathBuf::from("entry1.md")),
                nested_items: Vec::new(),
            }),
            TOCItem::Separator,
            TOCItem::Link(Link {
                name: String::from("Entry 2"),
                location: Some(PathBuf::from("entry2.md")),
                nested_items: Vec::new(),
            }),
        ];

        assert_eq!(items, expected);
    }

    #[test]
    fn lists_all_top_level_links_separated_by_heading() {
        let input = r#"
* [Entry 1](entry1.md)
# Next Section
* [Entry 2](entry2.md)
"#;

        let (_, items) = parse(input);
        let expected = vec![
            TOCItem::Link(Link {
                name: String::from("Entry 1"),
                location: Some(PathBuf::from("entry1.md")),
                nested_items: Vec::new(),
            }),
            TOCItem::SectionTitle(SectionTitle {
                title: String::from("Next Section"),
            }),
            TOCItem::Link(Link {
                name: String::from("Entry 2"),
                location: Some(PathBuf::from("entry2.md")),
                nested_items: Vec::new(),
            }),
        ];

        assert_eq!(items, expected);
    }

    #[test]
    fn lists_all_top_level_links_separated_by_second_level_heading() {
        let input = r#"
* [Entry 1](entry1.md)
## Next Section
* [Entry 2](entry2.md)
"#;

        let (_, items) = parse(input);
        let expected = vec![
            TOCItem::Link(Link {
                name: String::from("Entry 1"),
                location: Some(PathBuf::from("entry1.md")),
                nested_items: Vec::new(),
            }),
            TOCItem::Link(Link {
                name: String::from("Entry 2"),
                location: Some(PathBuf::from("entry2.md")),
                nested_items: Vec::new(),
            }),
        ];

        assert_eq!(items, expected);
    }

    #[test]
    fn lists_all_top_level_links_with_nested_links_separated_by_second_level_heading() {
        let input = r#"
* [Entry 1](entry1.md)
  * [Subentry 1](sub_entry1.md)
## Next Section
* [Entry 2](entry2.md)
"#;

        let (_, items) = parse(input);
        let expected = vec![
            TOCItem::Link(Link {
                name: String::from("Entry 1"),
                location: Some(PathBuf::from("entry1.md")),
                nested_items: vec![TOCItem::Link(Link {
                    name: String::from("Subentry 1"),
                    location: Some(PathBuf::from("sub_entry1.md")),
                    nested_items: Vec::new(),
                })],
            }),
            TOCItem::Link(Link {
                name: String::from("Entry 2"),
                location: Some(PathBuf::from("entry2.md")),
                nested_items: Vec::new(),
            }),
        ];

        assert_eq!(items, expected);
    }

    #[test]
    fn lists_all_top_level_links_separated_by_heading_and_paragraph() {
        let input = r#"
* [Entry 1](entry1.md)
# Next Section
This is a paragraph.
* [Entry 2](entry2.md)
"#;

        let (_, items) = parse(input);
        let expected = vec![
            TOCItem::Link(Link {
                name: String::from("Entry 1"),
                location: Some(PathBuf::from("entry1.md")),
                nested_items: Vec::new(),
            }),
            TOCItem::SectionTitle(SectionTitle {
                title: String::from("Next Section"),
            }),
            TOCItem::Link(Link {
                name: String::from("Entry 2"),
                location: Some(PathBuf::from("entry2.md")),
                nested_items: Vec::new(),
            }),
        ];

        assert_eq!(items, expected);
    }

    #[test]
    fn lists_links_with_nested_links() {
        let input = r#"
* [Entry 1](entry1.md)
  1. [Entry 2](entry2.md)
"#;

        let (_, items) = parse(input);
        let expected = vec![TOCItem::Link(Link {
            name: String::from("Entry 1"),
            location: Some(PathBuf::from("entry1.md")),
            nested_items: vec![TOCItem::Link(Link {
                name: String::from("Entry 2"),
                location: Some(PathBuf::from("entry2.md")),
                nested_items: Vec::new(),
            })],
        })];

        assert_eq!(items, expected);
    }

    #[test]
    fn link_titles_with_breaks_are_converted_to_spaces() {
        let input = "* [Entry\n1](entry1.md)";

        let (_, items) = parse(input);
        let expected = vec![TOCItem::Link(Link {
            name: String::from("Entry 1"),
            location: Some(PathBuf::from("entry1.md")),
            nested_items: Vec::new(),
        })];

        assert_eq!(items, expected);
    }
}
