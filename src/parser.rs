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

#[derive(Debug, Deserialize, Clone)]
pub struct StringParser;

impl StringParser {
    fn parse(&self, val: &str) -> Result<String, Error> {
        Ok(val.trim().to_string())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CurrencyParser;

impl CurrencyParser {
    fn parse(&self, val: &str) -> Result<Currency, Error> {
        iso::find(val)
            .map(|c| Currency(c))
            .ok_or(Error("currency not recognized".to_string()))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DateTimeParser(pub String);

impl DateTimeParser {
    fn parse(&self, val: &str) -> Result<DateTime<Utc>, Error> {
        Ok(Local.datetime_from_str(val, &self.0)?.with_timezone(&Utc))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DateTimeWithAppendParser {
    pub format: String,
    pub suffix: String,
}

impl DateTimeWithAppendParser {
    fn parse(&self, val: &str) -> Result<DateTime<Utc>, Error> {
        Ok(Local
            .datetime_from_str(&format!("{}{}", val, self.suffix), &self.format)?
            .with_timezone(&Utc))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct NatureParser;

impl NatureParser {
    fn parse(&self, val: &str) -> Result<Nature, Error> {
        Ok(val.parse()?)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "config")]
pub enum Parser {
    String(StringParser),
    Currency(CurrencyParser),
    DateTime(DateTimeParser),
    DateTimeWithAppend(DateTimeWithAppendParser),
    Nature(NatureParser),
}

impl Parser {
    fn parse_string(&self, val: &str) -> Result<String, Error> {
        match self {
            Parser::String(parser) => parser.parse(val),
            _ => panic!("given parser does not parse expected data"),
        }
    }

    fn parse_currency(&self, val: &str) -> Result<Currency, Error> {
        match self {
            Parser::Currency(parser) => parser.parse(val),
            _ => panic!("given parser does not parse expected data"),
        }
    }

    fn parse_datetime(&self, val: &str) -> Result<DateTime<Utc>, Error> {
        match self {
            Parser::DateTime(parser) => parser.parse(val),
            Parser::DateTimeWithAppend(parser) => parser.parse(val),
            _ => panic!("given parser does not parse expected data"),
        }
    }

    fn parse_nature(&self, val: &str) -> Result<Nature, Error> {
        match self {
            Parser::Nature(parser) => parser.parse(val),
            _ => panic!("given parser does not parse expected data"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "config")]
pub enum Value<T> {
    Fixed(T),
    FromMatch { group: String, parser: Parser },
}

#[derive(Debug, Deserialize, Clone)]
pub struct ValuesConfig {
    pub nature: Value<Nature>,
    pub account: Value<String>,
    pub amount: Value<String>,
    pub currency: Value<Currency>,
    pub source: Value<String>,
    pub time: Value<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Clone)]
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

    pub fn parse_record(
        values: &ValuesConfig,
        captures: &Captures,
        msg: &TextMessage,
    ) -> Result<Record, Error> {
        Ok(Record {
            message_id: msg.id,
            nature: RecordParser::extract_nature(&values.nature, captures)?,
            account: RecordParser::extract_string(&values.account, captures)?,
            amount: Money::from_str(
                &RecordParser::extract_string(&values.amount, captures)?,
                RecordParser::extract_currency(&values.currency, captures)?.0,
            )?,
            source: RecordParser::extract_string(&values.source, captures)?,
            time: RecordParser::extract_datetime(&values.time, captures)?,
        })
    }

    fn extract_string(value: &Value<String>, captures: &Captures) -> Result<String, Error> {
        match value {
            Value::Fixed(value) => Ok(value.clone()),
            Value::FromMatch { group, parser } => {
                Ok(parser.parse_string(&captures[&group as &str])?)
            }
        }
    }

    fn extract_datetime(
        value: &Value<DateTime<Utc>>,
        captures: &Captures,
    ) -> Result<DateTime<Utc>, Error> {
        match value {
            Value::Fixed(value) => Ok(value.clone()),
            Value::FromMatch { group, parser } => {
                Ok(parser.parse_datetime(&captures[&group as &str])?)
            }
        }
    }

    fn extract_currency(value: &Value<Currency>, captures: &Captures) -> Result<Currency, Error> {
        match value {
            Value::Fixed(value) => Ok(value.clone()),
            Value::FromMatch { group, parser } => {
                Ok(parser.parse_currency(&captures[&group as &str])?)
            }
        }
    }

    fn extract_nature(value: &Value<Nature>, captures: &Captures) -> Result<Nature, Error> {
        match value {
            Value::Fixed(value) => Ok(value.clone()),
            Value::FromMatch { group, parser } => {
                Ok(parser.parse_nature(&captures[&group as &str])?)
            }
        }
    }
}
