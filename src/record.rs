use std::str::FromStr;

use chrono::DateTime;
use chrono::Utc;
use rusty_money::iso::Currency;
use rusty_money::Money;
use serde::Deserialize;
use serde::Serialize;

use crate::config::CONFIG;
use crate::message::TextMessage;
use crate::parser::Matcher;
use crate::parser::RecordParser;

#[derive(Debug, strum_macros::Display, Clone, Serialize, Deserialize)]
pub enum Nature {
    #[allow(dead_code)]
    Credit,
    Debit,
}

impl FromStr for Nature {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Debit" => Ok(Nature::Debit),
            "Credit" => Ok(Nature::Credit),
            _ => Err(format!("expected 'Debit' or 'Credit', got: {}", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Record<'a> {
    pub matcher: &'a Matcher,
    pub message_id: u32,
    pub nature: Nature,
    pub account: String,
    pub amount: Money<'static, Currency>,
    pub source: String,
    pub time: DateTime<Utc>,
}

impl Record<'_> {
    pub fn parse_messages<'a>(messages: &'a Vec<TextMessage>) -> Vec<Record<'a>> {
        let parser = RecordParser::new(&CONFIG);

        messages.iter().filter_map(|m| parser.parse(m)).collect()
    }
}
