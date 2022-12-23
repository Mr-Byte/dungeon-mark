use std::{cell::RefCell, collections::HashMap, path::PathBuf, rc::Rc, str::FromStr};

use dungeon_mark::{
    build::{
        render::{RenderContext, Renderer},
        JournalBuilder,
    },
    config::Config,
    model::journal::{Journal, JournalEntry, JournalItem, Section, SectionLevel},
};
use serde::Deserialize;

fn test_dir() -> PathBuf {
    let mut current_dir = PathBuf::from_str(file!()).expect("unable to get path");
    current_dir.pop();

    current_dir
}

type JournalCell = Rc<RefCell<Option<Journal>>>;
type ConfigCell = Rc<RefCell<Option<Config>>>;

#[derive(Clone, Default)]
struct TestRenderer(JournalCell, ConfigCell);

impl TestRenderer {
    fn journal(&self) -> Journal {
        self.0.borrow_mut().take().expect("result was not set")
    }

    fn config(&self) -> Config {
        self.1.borrow_mut().take().expect("result was not set")
    }
}

impl Renderer for TestRenderer {
    fn name(&self) -> &str {
        "test_renderer"
    }

    fn render(&self, ctx: &RenderContext, journal: &Journal) -> anyhow::Result<()> {
        *self.0.borrow_mut() = Some(journal.clone());
        *self.1.borrow_mut() = Some(ctx.config.clone());

        Ok(())
    }
}

#[test]
fn it_loads_the_journal_as_expected() {
    let renderer = TestRenderer::default();
    let mut journal_builder = JournalBuilder::load(test_dir()).expect("failed to load journal");

    journal_builder.with_renderer(renderer.clone());
    journal_builder.build().expect("failed to build journal");

    let journal = renderer.journal();

    let expected = vec![JournalItem::Entry(JournalEntry {
        title: String::from("Entry 1"),
        body: None,
        sections: vec![Section {
            title: String::from("Test Entry"),
            level: SectionLevel::H1,
            body: String::from("This is a test entry!"),
            metadata: HashMap::new(),
            sections: Vec::new(),
        }],
        path: PathBuf::from_str("./entry_1.md").ok(),
    })];

    assert_eq!(expected, journal.items);
}

#[test]
fn it_loads_custom_configuration() {
    #[derive(Debug, Deserialize, PartialEq, Eq, Default)]
    #[serde(rename_all = "kebab-case")]
    struct TestData {
        test_item: String,
    }

    let renderer = TestRenderer::default();
    let mut journal_builder = JournalBuilder::load(test_dir()).expect("failed to load journal");

    journal_builder.with_renderer(renderer.clone());
    journal_builder.build().expect("failed to build journal");

    let expected = TestData {
        test_item: String::from("test"),
    };

    let config = renderer.config();
    let actual = config
        .get("test-section")
        .expect("should be deserializable");

    assert_eq!(expected, actual);
}
