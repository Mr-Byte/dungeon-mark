# dungeon-mark

CLI to produce Foundry VTT compendiums and HTML websites from a single source

# Internals

A DungeonMark journal goes through several steps:

- Load
- Preprocess
- Parse
- Transform
- Render

The preprocess, transform, and render steps can be configured by adding
additional sub-steps to the configuration of a journal.
