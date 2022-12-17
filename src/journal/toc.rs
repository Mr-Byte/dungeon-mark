use anyhow::{anyhow, bail};
use pulldown_cmark::{Event, HeadingLevel, Tag};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, path::PathBuf, str::FromStr};

use crate::{
    cmark::{CMarkParser, EventCollectionExt},
    error::{Error, Result},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOfContents {
    pub title: Option<String>,
    pub items: Vec<TOCItem>,
}

impl FromStr for TableOfContents {
    type Err = Error;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        TOCParser::new(source).parse()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub name: String,
    pub location: Option<PathBuf>,
    pub nested_items: Vec<TOCItem>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TOCItem {
    Link(Link),
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
}

struct TOCParser<'a> {
    parser: CMarkParser<'a>,
}

impl<'a> TOCParser<'a> {
    fn new(source: &str) -> TOCParser<'_> {
        let parser = CMarkParser::new(source);

        TOCParser { parser }
    }

    fn parse(mut self) -> Result<TableOfContents> {
        let title = self.parse_title()?;
        let items = self.parse_toc_items()?;

        Ok(TableOfContents { title, items })
    }

    fn parse_title(&mut self) -> Result<Option<String>> {
        loop {
            let event = self.parser.peek();
            match event {
                Some(Event::Start(Tag::Heading(HeadingLevel::H1, ..))) => {
                    // NOTE: Skip the start tag that was peeked.
                    self.parser.next();
                    let events = self.parser.consume_until(|event| {
                        matches!(event, Event::End(Tag::Heading(HeadingLevel::H1, ..)))
                    });

                    return Ok(Some(events.stringify()?));
                }
                Some(Event::Html(_)) => {
                    self.parser.next(); // Skip HTML, such as comments.
                }
                _ => return Ok(None),
            }
        }
    }

    fn parse_toc_items(&mut self) -> Result<Vec<TOCItem>> {
        let mut items = Vec::new();

        loop {
            match self.parser.next() {
                Some(Event::Start(Tag::Item)) => {
                    let item = self.parse_toc_item()?;
                    items.push(item);
                }
                Some(Event::Start(Tag::List(..))) => {
                    if items.is_empty() {
                        continue;
                    }

                    if let Some(last_item) = items.last_mut().and_then(TOCItem::maybe_link_mut) {
                        last_item.nested_items = self.parse_toc_items()?;
                    }
                }
                Some(Event::End(Tag::List(..))) => break,
                Some(_) => {}
                None => break,
            }
        }

        Ok(items)
    }

    fn parse_toc_item(&mut self) -> Result<TOCItem> {
        loop {
            match self.parser.next() {
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
        let name = self
            .parser
            .consume_until(|event| matches! {event, Event::End(Tag::Link(..))})
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

    #[test]
    fn parses_title() {
        let input = "# Journal Title";
        let toc: TableOfContents = input.parse().expect("toc failed to parse");

        assert_eq!("Journal Title", toc.title.expect("toc title was empty"))
    }

    #[test]
    fn skips_comments_and_parses_title() {
        let input = r"<!-- # Journal Title -->
# Actual Title
";
        let toc: TableOfContents = input.parse().expect("toc failed to parse");

        assert_eq!("Actual Title", toc.title.expect("toc title was empty"))
    }

    #[test]
    fn lists_all_top_level_links() {
        let input = r#"
* [Entry 1](entry1.md)
* [Entry 2](entry2.md)
"#;
        let toc: TableOfContents = input.parse().expect("toc failed to parse");

        assert_eq!(2, toc.items.len());
        assert!(matches! { &toc.items[0], TOCItem::Link(Link { name, .. }) if name == "Entry 1" });
        assert!(
            matches! { &toc.items[0], TOCItem::Link(Link { location: Some(location), .. }) if location.ends_with("entry1.md") }
        );
        assert!(matches! { &toc.items[1], TOCItem::Link(Link { name, .. }) if name == "Entry 2" });
        assert!(
            matches! { &toc.items[1], TOCItem::Link(Link { location: Some(location), .. }) if location.ends_with("entry2.md") }
        );
    }

    #[test]
    fn lists_links_with_nested_links() {
        let input = r#"
* [Entry 1](entry1.md)
  1. [Entry 2](entry2.md)
"#;
        let toc: TableOfContents = input.parse().expect("toc failed to parse");

        assert_eq!(1, toc.items.len());
        assert!(matches! { &toc.items[0], TOCItem::Link(Link { name, .. }) if name == "Entry 1" });
        assert!(
            matches! { &toc.items[0], TOCItem::Link(Link { location: Some(location), .. }) if location.ends_with("entry1.md") }
        );

        let link = toc.items[0]
            .maybe_link()
            .expect("the item should be a link");

        assert!(
            matches! { &link.nested_items[0], TOCItem::Link(Link { name, .. }) if name == "Entry 2" }
        );
        assert!(
            matches! { &link.nested_items[0], TOCItem::Link(Link { location: Some(location), .. }) if location.ends_with("entry2.md") }
        );
    }
}
