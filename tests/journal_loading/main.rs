use std::{
    env,
    path::{Path, PathBuf},
    str::FromStr,
};

use dungeon_mark::journal::DMJournal;

#[test]
fn it_loads_the_journal_as_expected() {
    let mut current_dir = PathBuf::from_str(file!()).expect("unable to get path");
    current_dir.pop();

    let journal = DMJournal::load(current_dir).expect("failed to load");

    assert!(journal.journal.items.len() >= 1);
}
