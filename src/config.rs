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

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    /// Configuration for the journal itself.
    pub journal: JournalConfig,

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
            journal: JournalConfig::default(),
            rest: Value::Table(Table::default()),
        }
    }
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let raw = Value::deserialize(deserializer)?;
        let Value::Table(mut table) = raw else {
            return Err(D::Error::custom("journal.toml must always be a toml table"));
        };

        let journal: JournalConfig = table
            .remove("journal")
            .map(|journal| journal.try_into().map_err(D::Error::custom))
            .transpose()?
            .unwrap_or_default();

        let config = Config {
            journal,
            rest: Value::Table(table),
        };

        Ok(config)
    }
}

impl FromStr for Config {
    type Err = Error;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        toml::from_str(source).with_context(|| "Attempted to parse invalid configuration file")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
pub struct JournalConfig {
    /// Optional title for the compendium.
    pub title: Option<String>,
    /// List of authors for the compendium.
    pub authors: Vec<String>,
    /// Optional description of the compendium.
    pub description: Option<String>,
    /// Relative path to the source location of the compendium.
    pub source: PathBuf,
}

impl Default for JournalConfig {
    fn default() -> Self {
        Self {
            title: None,
            authors: Vec::new(),
            description: None,
            source: PathBuf::from("./src"),
        }
    }
}
