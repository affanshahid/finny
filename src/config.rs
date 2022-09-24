use std::error;
use std::fs;
use std::io;

use serde::Deserialize;
use strum_macros::Display;
use yaml2json_rs::Style;
use yaml2json_rs::Yaml2Json;

use crate::parser::Matcher;

#[derive(Debug, Display)]
pub enum Error {
    FileReadFailure(io::Error),
    YamlConversionFailure(yaml2json_rs::Yaml2JsonError),
    DeError(serde_json::error::Error),
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::FileReadFailure(error)
    }
}

impl From<yaml2json_rs::Yaml2JsonError> for Error {
    fn from(error: yaml2json_rs::Yaml2JsonError) -> Self {
        Error::YamlConversionFailure(error)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(error: serde_json::error::Error) -> Self {
        Error::DeError(error)
    }
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub matchers: Vec<Matcher>,
}

impl Config {
    pub fn new(path: &str) -> Result<Config, Error> {
        let cfg_str = fs::read_to_string(path)?;
        let converter = Yaml2Json::new(Style::PRETTY);
        let cfg_str = converter.document_to_string(&cfg_str)?;
        Ok(serde_json::from_str(&cfg_str)?)
    }
}
