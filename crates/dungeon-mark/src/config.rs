use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    str::FromStr,
};
use toml::value::Table;

use crate::error::{Error, Result};

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// Configuration for the journal itself.
    pub journal: JournalConfig,

    /// Configuration for the build process.
    pub build: BuildConfig,

    #[serde(flatten)]
    rest: Table,
}

impl Config {
    /// Load the config file from the specified path.
    pub fn load(path: impl AsRef<Path>) -> Result<Config> {
        let path = path.as_ref().join("journal.toml");
        let config: Self = fs::read_to_string(&path)
            .with_context(|| format!("Failed to open journal.toml: {}", path.display()))?
            .parse()
            .with_context(|| "Failed to deserialize journal.toml")?;

        Ok(config)
    }

    /// Attempt to retrieve the specified key and deserialize it to the target type.
    /// The target type must implement `Default` which will be returned in the event
    /// that the specified key could not be found.
    pub fn get<'de, D>(&self, key: &str) -> Result<D>
    where
        D: Deserialize<'de> + Default,
    {
        let Some(item) = self.rest.get(key).cloned() else {
            return Ok(Default::default());
        };

        let item = item.try_into()?;

        Ok(item)
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

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
pub struct BuildConfig {
    pub renderers: Vec<RendererConfig>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, rename_all = "kebab-case")]
pub struct RendererConfig {
    pub name: String,
    /// Optional command, if this is not set the name will be used as a fallback for the command to run.
    pub command: Option<String>,
}
