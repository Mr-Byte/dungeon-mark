use std::{collections::HashMap, path::PathBuf, str::FromStr};

use dungeon_mark::journal::{DMJournal, JournalEntry, JournalItem, Section, SectionLevel};

#[test]
fn it_loads_the_journal_as_expected() {
    let mut current_dir = PathBuf::from_str(file!()).expect("unable to get path");
    current_dir.pop();

    let journal = DMJournal::load(current_dir).expect("failed to load");
    let expected = vec![JournalItem::Entry(JournalEntry {
        name: String::from("Entry 1"),
        body: None,
        sections: vec![Section {
            title: String::from("Test Entry"),
            level: SectionLevel::H1,
            body: String::from("This is a test entry!"),
            metadata: HashMap::new(),
            sections: Vec::new(),
        }],
        entry_path: PathBuf::from_str("./entry_1.md").ok(),
    })];

    assert_eq!(expected, journal.journal.items);
}
