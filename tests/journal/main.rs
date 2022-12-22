// use std::{collections::HashMap, path::PathBuf, str::FromStr};

// use dungeon_mark::journal::{JournalEntry, JournalItem, Section, SectionLevel};
// use serde::Deserialize;

// fn test_dir() -> PathBuf {
//     let mut current_dir = PathBuf::from_str(file!()).expect("unable to get path");
//     current_dir.pop();

//     current_dir
// }

// #[test]
// fn it_loads_the_journal_as_expected() {
//     let journal = DMJournal::load(test_dir()).expect("failed to load");
//     let expected = vec![JournalItem::Entry(JournalEntry {
//         title: String::from("Entry 1"),
//         body: None,
//         sections: vec![Section {
//             title: String::from("Test Entry"),
//             level: SectionLevel::H1,
//             body: String::from("This is a test entry!"),
//             metadata: HashMap::new(),
//             sections: Vec::new(),
//         }],
//         path: PathBuf::from_str("./entry_1.md").ok(),
//     })];

//     assert_eq!(expected, journal.journal.items);
// }

// #[test]
// fn it_loads_custom_configuration() {
//     #[derive(Debug, Deserialize, PartialEq, Eq, Default)]
//     #[serde(rename_all = "kebab-case")]
//     struct TestData {
//         test_item: String,
//     }

//     let journal = DMJournal::load(test_dir()).expect("failed to load");
//     let expected = TestData {
//         test_item: String::from("test"),
//     };

//     let actual = journal
//         .config
//         .get("test-section")
//         .expect("should be deserializable");

//     assert_eq!(expected, actual);
// }
