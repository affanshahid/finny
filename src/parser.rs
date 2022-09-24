use chrono::DateTime;
use chrono::Local;
use chrono::ParseError;
use chrono::TimeZone;
use chrono::Utc;
use regex::Captures;
use regex::Regex;
use rust_decimal::Decimal;
use rusty_money::iso;
use rusty_money::MoneyError;
use serde::Deserialize;
use std::error;
use std::fmt::Display;
use std::num::ParseIntError;

use crate::message::TextMessage;
use crate::record::Money;
use crate::record::Record;
use crate::wrapper::Currency;

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

#[derive(Deserialize, Debug)]
pub enum Nature {
    #[allow(dead_code)]
    Credit,
    Debit,
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
#[serde(tag = "type", content = "config")]
pub enum Value<T: Clone, R: ValueParser<T>> {
    Fixed(T),
    FromMatch { group: String, parser: R },
}

impl<T: Clone, R: ValueParser<T>> Value<T, R> {
    fn extract(&self, captures: &Captures) -> Result<T, Error> {
        match self {
            Value::Fixed(value) => Ok(value.clone()),
            Value::FromMatch { group, parser } => Ok(parser.parse(&captures[&group as &str])?),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ValuesConfig {
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
    pub nature: Nature,
    pub values: ValuesConfig,
}

pub struct RecordParser<'a> {
    matchers: &'a Vec<Matcher>,
}

impl<'a> RecordParser<'a> {
    pub fn new(matchers: &'a Vec<Matcher>) -> RecordParser<'a> {
        RecordParser {
            matchers: &matchers,
        }
    }

    pub fn parse(&self, msg: &TextMessage) -> Option<Record<'a>> {
        let matcher = self
            .matchers
            .iter()
            .find(|m| m.pattern.is_match(&msg.text))?;

        let captures = matcher
            .pattern
            .captures(&msg.text)
            .expect("expected all captures to match");

        match RecordParser::parse_record(&matcher, &captures, msg) {
            Ok(record) => Some(record),
            Err(err) => {
                println!(
                    "error while parsing record. message: {}, matcher-id: {}, pattern: {}, : {}",
                    msg.text, matcher.id, matcher.pattern, err
                );
                None
            }
        }
    }

    fn parse_record(
        matcher: &'a Matcher,
        captures: &Captures,
        msg: &TextMessage,
    ) -> Result<Record<'a>, Error> {
        let values = &matcher.values;
        Ok(Record {
            message_id: msg.id,
            account: values.account.extract(captures)?,
            amount: RecordParser::canonical_amount(
                &Money::from_str(
                    &values.amount.extract(captures)?,
                    values.currency.extract(captures)?.0,
                )?,
                &matcher.nature,
            ),
            source: values.source.extract(captures)?,
            time: values.time.extract(captures)?,
            matcher,
        })
    }

    fn canonical_amount(money: &Money, nature: &Nature) -> Money {
        Money::from_decimal(
            money.amount()
                * match nature {
                    Nature::Credit => Decimal::ONE,
                    Nature::Debit => Decimal::NEGATIVE_ONE,
                },
            money.currency(),
        )
    }
}
