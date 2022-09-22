use std::error;
use std::fmt::Display;
use std::num::ParseIntError;
use std::str::FromStr;

use crate::wrapper::Currency;
use chrono::DateTime;
use chrono::Local;
use chrono::ParseError;
use chrono::TimeZone;
use chrono::Utc;
use regex::Captures;
use regex::Regex;
use rusty_money::iso;
use rusty_money::Money;
use rusty_money::MoneyError;
use serde::Deserialize;

use crate::config::Config;
use crate::message::TextMessage;
use crate::record::Nature;
use crate::record::Record;

#[derive(Debug)]
pub struct Error(String);

impl error::Error for Error {}

impl From<MoneyError> for Error {
    fn from(error: MoneyError) -> Self {
        Error(format!(
            "error parsing monetary value: {}",
            &error.to_string()
        ))
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Self {
        Error(format!("error parsing int: {}", &error.to_string()))
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error(format!("error parsing time: {}", &error.to_string()))
    }
}

impl From<<Nature as FromStr>::Err> for Error {
    fn from(error: <Nature as FromStr>::Err) -> Self {
        Error(format!("error parsing Nature: {}", &error.to_string()))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait ValueParser<T: Clone> {
    fn parse(&self, v: &str) -> Result<T, Error>;
}

#[derive(Debug, Deserialize)]
pub struct StringParser;

impl ValueParser<String> for StringParser {
    fn parse(&self, val: &str) -> Result<String, Error> {
        Ok(val.trim().to_string())
    }
}

#[derive(Debug, Deserialize)]
pub struct CurrencyParser;

impl ValueParser<Currency> for CurrencyParser {
    fn parse(&self, val: &str) -> Result<Currency, Error> {
        iso::find(val)
            .map(|c| Currency(c))
            .ok_or(Error("currency not recognized".to_string()))
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum DateTimeParser {
    FormattedDateTime(String),
    FormattedDateTimeWithAppend { format: String, suffix: String },
}

impl ValueParser<DateTime<Utc>> for DateTimeParser {
    fn parse(&self, val: &str) -> Result<DateTime<Utc>, Error> {
        match self {
            DateTimeParser::FormattedDateTime(format) => {
                Ok(Local.datetime_from_str(val, &format)?.with_timezone(&Utc))
            }
            DateTimeParser::FormattedDateTimeWithAppend { format, suffix } => Ok(Local
                .datetime_from_str(&format!("{}{}", val, suffix), &format)?
                .with_timezone(&Utc)),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NatureParser;

impl ValueParser<Nature> for NatureParser {
    fn parse(&self, val: &str) -> Result<Nature, Error> {
        Ok(val.parse()?)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "config")]
pub enum Value<T: Clone, R: ValueParser<T>> {
    Fixed(T),
    FromMatch { group: String, parser: R },
}

#[derive(Debug, Deserialize)]
pub struct ValuesConfig {
    pub nature: Value<Nature, NatureParser>,
    pub account: Value<String, StringParser>,
    pub amount: Value<String, StringParser>,
    pub currency: Value<Currency, CurrencyParser>,
    pub source: Value<String, StringParser>,
    pub time: Value<DateTime<Utc>, DateTimeParser>,
}

#[derive(Debug, Deserialize)]
pub struct Matcher {
    pub id: String,
    #[serde(with = "serde_regex")]
    pub pattern: Regex,
    pub values: ValuesConfig,
}

pub struct RecordParser<'a> {
    matchers: &'a Vec<Matcher>,
}

impl<'a> RecordParser<'a> {
    pub fn new(config: &'a Config) -> RecordParser<'a> {
        RecordParser {
            matchers: &config.matchers,
        }
    }

    pub fn parse(&self, msg: &TextMessage) -> Option<Record> {
        let matcher = self
            .matchers
            .iter()
            .find(|m| m.pattern.is_match(&msg.text))?;

        let captures = matcher
            .pattern
            .captures(&msg.text)
            .expect("expected all captures to match");

        match RecordParser::parse_record(&matcher.values, &captures, msg) {
            Ok(record) => Some(record),
            Err(err) => {
                println!("error while parsing record: {}", err);
                None
            }
        }
    }

    fn parse_record(
        values: &ValuesConfig,
        captures: &Captures,
        msg: &TextMessage,
    ) -> Result<Record, Error> {
        Ok(Record {
            message_id: msg.id,
            nature: RecordParser::extract(&values.nature, captures)?,
            account: RecordParser::extract(&values.account, captures)?,
            amount: Money::from_str(
                &RecordParser::extract(&values.amount, captures)?,
                RecordParser::extract(&values.currency, captures)?.0,
            )?,
            source: RecordParser::extract(&values.source, captures)?,
            time: RecordParser::extract(&values.time, captures)?,
        })
    }

    fn extract<T: Clone, R: ValueParser<T>>(
        value: &Value<T, R>,
        captures: &Captures,
    ) -> Result<T, Error> {
        match value {
            Value::Fixed(value) => Ok(value.clone()),
            Value::FromMatch { group, parser } => Ok(parser.parse(&captures[&group as &str])?),
        }
    }
}
