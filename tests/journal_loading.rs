use crate::common::TestRenderer;
use dungeon_mark::{
    build::JournalBuilder,
    model::journal::{JournalEntry, JournalItem, Section, SectionLevel},
};
use std::{collections::HashMap, env, path::PathBuf, str::FromStr};

mod common;

fn test_dir() -> PathBuf {
    env::current_dir()
        .expect("Unable to get working directory")
        .join("data")
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
        level: 1,
    })];

    assert_eq!(expected, journal.items);
}
