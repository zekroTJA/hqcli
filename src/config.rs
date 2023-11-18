use std::{ops::Deref, path::Path};

use anyhow::Result;
use figment::{
    providers::{Format, Json, Toml, Yaml},
    Figment,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub endpoint: String,
    pub session: Session,
    pub defaults: Option<Defaults>,
}

#[derive(Deserialize)]
pub struct Session {
    pub key: String,
    pub value: String,
}

#[derive(Deserialize)]
pub struct Defaults {
    #[serde(rename = "startTime")]
    pub start_time: Option<String>,
    pub pause: Option<String>,
}

impl Config {
    pub fn init() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("could not resolve current users home directory"))?;

        Ok(Figment::new()
            .merge(Toml::file(home_dir.join(".config").join("hqcli.toml")))
            .merge(Toml::file("hqcli.toml"))
            .merge(Yaml::file(home_dir.join(".config").join("hqcli.yaml")))
            .merge(Yaml::file("hqcli.yaml"))
            .merge(Json::file(home_dir.join(".config").join("hqcli.json")))
            .merge(Json::file("hqcli.json"))
            .extract()?)
    }

    pub fn from_file<T: AsRef<Path>>(path: T) -> Result<Self> {
        let ext = path.as_ref().extension().unwrap_or_default();
        let mut figment = Figment::new();

        figment = match ext.to_string_lossy().deref() {
            "yml" | "yaml" => figment.merge(Yaml::file(path)),
            "toml" => figment.merge(Toml::file(path)),
            "json" => figment.merge(Json::file(path)),
            _ => anyhow::bail!("invalid config file type"),
        };

        Ok(figment.extract()?)
    }
}
