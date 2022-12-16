use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    str::FromStr,
};
use toml::{value::Table, Value};

use crate::error::{Error, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Configuration for the compendium itself.
    pub compendium: CompendiumConfig,

    /// Any remaining configuration for renderers and preprocessors.
    rest: Value,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Config> {
        let mut buffer = String::new();
        File::open(path)
            .with_context(|| "Failed to open config file")?
            .read_to_string(&mut buffer)
            .with_context(|| "Failed to read config file")?;

        Config::from_str(&buffer)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            compendium: CompendiumConfig::default(),
            rest: Value::Table(Table::default()),
        }
    }
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        toml::from_str(source).with_context(|| "Attempted to parse invalid configuration file")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct CompendiumConfig {
    /// Optional title for the compendium.
    pub title: Option<String>,
    /// List of authors for the compendium.
    pub authors: Vec<String>,
    /// Optional description of the compendium.
    pub description: Option<String>,
    /// Relative path to the source location of the compendium.
    pub source: PathBuf,
}

impl Default for CompendiumConfig {
    fn default() -> Self {
        Self {
            title: None,
            authors: Vec::new(),
            description: None,
            source: PathBuf::from("compendium"),
        }
    }
}
