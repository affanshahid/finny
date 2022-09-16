use chrono::DateTime;
use chrono::Utc;
use rusty_money::iso::Currency;
use tabled::Tabled;

use crate::message::TextMessage;
use crate::parse::MATCHERS;

#[derive(Debug, strum_macros::Display, Clone)]
pub enum Nature {
    #[allow(dead_code)]
    Credit,
    Debit,
}

#[derive(Debug, Clone, Tabled)]
pub struct Record {
    pub message_id: u32,
    pub nature: Nature,
    pub account: String,
    pub currency: &'static Currency,
    pub amount: f64,
    pub source: String,
    pub time: DateTime<Utc>,
}

impl Record {
    pub fn parse_messages(messages: &Vec<TextMessage>) -> Vec<Record> {
        messages
            .iter()
            .filter_map(|msg| {
                Some((
                    msg,
                    MATCHERS.iter().find(|m| m.pattern.is_match(&msg.text))?,
                ))
            })
            .filter_map(|(msg, matcher)| {
                match (matcher.factory)(matcher.pattern.captures(&msg.text).unwrap(), msg) {
                    Ok(r) => Some(r),
                    Err(err) => {
                        println!("unable to parse: {}: error: {}", msg.text, err);
                        None
                    }
                }
            })
            .collect()
    }
}
