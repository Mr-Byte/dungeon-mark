pub mod preprocess;
pub mod render;
pub mod transform;

use std::path::{Path, PathBuf};

use self::{
    preprocess::{Preprocessor, PreprocessorContext},
    render::{RenderContext, Renderer},
    transform::{Transformer, TransformerContext},
};
use crate::{
    config::Config,
    error::Result,
    model::{
        journal::{ChapterTitle, Journal, JournalEntry, JournalItem},
        toc::{TOCItem, TableOfContents},
    },
};

pub struct JournalBuilder {
    root: PathBuf,
    config: Config,
    table_of_contents: TableOfContents,
    preprocessors: Vec<Box<dyn Preprocessor>>,
    transformers: Vec<Box<dyn Transformer>>,
    renderers: Vec<Box<dyn Renderer>>,
}

impl JournalBuilder {
    pub fn load(root: impl AsRef<Path>) -> Result<Self> {
        let config = Config::load(&root)?;

        Self::load_with_config(root, config)
    }

    pub fn load_with_config(root: impl AsRef<Path>, config: Config) -> Result<Self> {
        let source_path = root.as_ref().join(&config.journal.source);
        let table_of_contents = TableOfContents::load(&source_path)?;
        let builder = Self {
            root: root.as_ref().into(),
            config,
            table_of_contents,
            preprocessors: Vec::new(),
            transformers: Vec::new(),
            renderers: Vec::new(),
        };

        Ok(builder)
    }

    pub fn with_preprocessor(&mut self, preprocessor: impl Preprocessor + 'static) -> &mut Self {
        self.preprocessors.push(Box::new(preprocessor));

        self
    }

    pub fn with_transformer(&mut self, transformer: impl Transformer + 'static) -> &mut Self {
        self.transformers.push(Box::new(transformer));

        self
    }

    pub fn with_renderer(&mut self, renderer: impl Renderer + 'static) -> &mut Self {
        self.renderers.push(Box::new(renderer));

        self
    }

    pub fn build(self) -> Result<()> {
        let journal = self.load_journal()?;
        let journal = self.preprocess(journal)?;
        let journal = self.parse_items(journal)?;
        let journal = self.transform(journal)?;

        self.render(journal)
    }
}

impl JournalBuilder {
    fn load_journal(&self) -> Result<Journal> {
        let items = self.load_items(&self.table_of_contents.items)?;
        let journal = Journal {
            items,
            title: self.table_of_contents.title.clone(),
        };

        Ok(journal)
    }

    fn load_items(&self, toc_items: &[TOCItem]) -> Result<Vec<JournalItem>, anyhow::Error> {
        let source_path = self.root.join(&self.config.journal.source);
        let mut items = Vec::new();

        for item in toc_items {
            match item {
                TOCItem::Link(link) => {
                    let Some(ref location) = link.location else {
                        continue;
                    };

                    let entry = JournalEntry::load(link.name.clone(), &source_path, location)?;
                    items.push(JournalItem::Entry(entry));
                    let nested_items = self.load_items(&link.nested_items)?;
                    items.extend(nested_items);
                }
                TOCItem::SectionTitle(section) => {
                    let item = JournalItem::ChapterTitle(ChapterTitle {
                        title: section.title.clone(),
                    });

                    items.push(item)
                }
                TOCItem::Separator => items.push(JournalItem::Separator),
            }
        }

        Ok(items)
    }

    fn preprocess(&self, journal: Journal) -> Result<Journal> {
        let ctx = PreprocessorContext::new(self.root.clone(), self.config.clone());

        self.preprocessors
            .iter()
            .try_fold(journal, |journal, preprocessor| {
                preprocessor.run(&ctx, journal)
            })
    }

    fn parse_items(&self, journal: Journal) -> Result<Journal> {
        let items = journal
            .items
            .into_iter()
            .map(|item| {
                let JournalItem::Entry(entry) = item else { return Ok(item); };
                let entry = entry.parse()?;

                Ok(JournalItem::Entry(entry))
            })
            .collect::<Result<Vec<_>>>()?;

        let journal = Journal {
            title: journal.title,
            items,
        };

        Ok(journal)
    }

    fn transform(&self, journal: Journal) -> Result<Journal> {
        let ctx = TransformerContext::new(self.root.clone(), self.config.clone());

        self.transformers
            .iter()
            .try_fold(journal, |journal, preprocessor| {
                preprocessor.run(&ctx, journal)
            })
    }

    fn render(&self, journal: Journal) -> Result<()> {
        let ctx = RenderContext;

        // TODO: Parallelize renderers and let them all run to completion or error.
        for renderer in &self.renderers {
            renderer.render(&ctx, &journal)?;
        }

        Ok(())
    }
}
