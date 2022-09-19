use std::error;
use std::fmt::Display;
use std::num::ParseIntError;

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

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait ParseableValue {}

trait ValueParser<T: ParseableValue> {
    fn parse(&self, val: &str) -> Result<T, Error>;
}

#[derive(Debug, Deserialize, Clone)]
pub struct StringParser;

impl ParseableValue for String {}

impl ValueParser<String> for StringParser {
    fn parse(&self, val: &str) -> Result<String, Error> {
        Ok(val.to_string())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct CurrencyParser;

impl ParseableValue for Currency {}

impl ValueParser<Currency> for CurrencyParser {
    fn parse(&self, val: &str) -> Result<Currency, Error> {
        iso::find(val)
            .map(|c| Currency(c))
            .ok_or(Error("currency not recognized".to_string()))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct DateTimeParser(pub String);

impl ParseableValue for DateTime<Utc> {}

impl ValueParser<DateTime<Utc>> for DateTimeParser {
    fn parse(&self, val: &str) -> Result<DateTime<Utc>, Error> {
        Ok(Local.datetime_from_str(val, &self.0)?.with_timezone(&Utc))
    }
}

impl ParseableValue for Nature {}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "config")]
pub enum Parser {
    String(StringParser),
    Currency(CurrencyParser),
    DateTime(DateTimeParser),
}

impl ValueParser<String> for Parser {
    fn parse(&self, val: &str) -> Result<String, Error> {
        match self {
            Parser::String(parser) => parser.parse(val),
            _ => panic!("given parser does not parse expected data"),
        }
    }
}

impl ValueParser<Currency> for Parser {
    fn parse(&self, val: &str) -> Result<Currency, Error> {
        match self {
            Parser::Currency(parser) => parser.parse(val),
            _ => panic!("given parser does not parse expected data"),
        }
    }
}

impl ValueParser<DateTime<Utc>> for Parser {
    fn parse(&self, val: &str) -> Result<DateTime<Utc>, Error> {
        match self {
            Parser::DateTime(parser) => parser.parse(val),
            _ => panic!("given parser does not parse expected data"),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", content = "config")]
pub enum Value<T: ParseableValue> {
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
            nature: RecordParser::extract_value_nature(&values.nature, captures)?,
            account: RecordParser::extract_value_string(&values.account, captures)?,
            amount: Money::from_str(
                &RecordParser::extract_value_string(&values.amount, captures)?,
                RecordParser::extract_value_currency(&values.currency, captures)?.0,
            )?,
            source: RecordParser::extract_value_string(&values.source, captures)?,
            time: RecordParser::extract_value_datetime(&values.time, captures)?,
        })
    }

    fn extract_value_string(value: &Value<String>, captures: &Captures) -> Result<String, Error> {
        match value {
            Value::Fixed(value) => Ok(value.clone()),
            Value::FromMatch { group, parser } => Ok(parser.parse(&captures[&group as &str])?),
        }
    }

    fn extract_value_datetime(
        value: &Value<DateTime<Utc>>,
        captures: &Captures,
    ) -> Result<DateTime<Utc>, Error> {
        match value {
            Value::Fixed(value) => Ok(value.clone()),
            Value::FromMatch { group, parser } => Ok(parser.parse(&captures[&group as &str])?),
        }
    }

    fn extract_value_currency(
        value: &Value<Currency>,
        captures: &Captures,
    ) -> Result<Currency, Error> {
        match value {
            Value::Fixed(value) => Ok(value.clone()),
            Value::FromMatch { group, parser } => Ok(parser.parse(&captures[&group as &str])?),
        }
    }

    fn extract_value_nature(value: &Value<Nature>, _captures: &Captures) -> Result<Nature, Error> {
        match value {
            Value::Fixed(value) => Ok(value.clone()),
            Value::FromMatch {
                group: _,
                parser: _,
            } => todo!(),
        }
    }
}
