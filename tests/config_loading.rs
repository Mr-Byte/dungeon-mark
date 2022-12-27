use crate::common::TestRenderer;
use dungeon_mark::build::JournalBuilder;
use serde::Deserialize;

mod common;

#[test]
fn it_loads_custom_configuration() {
    #[derive(Debug, Deserialize, PartialEq, Eq, Default)]
    #[serde(rename_all = "kebab-case")]
    struct TestData {
        test_item: String,
    }

    let renderer = TestRenderer::default();
    let test_dir = common::test_dir();
    let mut journal_builder = JournalBuilder::load(test_dir).expect("failed to load journal");

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
