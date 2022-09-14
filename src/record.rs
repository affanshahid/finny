use chrono::DateTime;
use chrono::Utc;

use crate::message::TextMessage;
use crate::parse::MATCHERS;

#[derive(Debug, strum_macros::Display)]
pub enum Nature {
    #[allow(dead_code)]
    Credit,
    Debit,
}

#[derive(Debug)]
pub struct Record {
    pub message_id: u32,
    pub nature: Nature,
    pub account: String,
    pub currency: String,
    pub amount: f64,
    pub source: String,
    pub time: DateTime<Utc>,
}

impl Record {
    pub fn parse_messages(messages: &Vec<TextMessage>) -> Vec<Record> {
        messages
            .iter()
            .filter_map(
                |msg| match MATCHERS.iter().find(|m| m.pattern.is_match(&msg.text)) {
                    Some(matcher) => {
                        match (matcher.factory)(matcher.pattern.captures(&msg.text).unwrap(), msg) {
                            Ok(record) => Some(record),
                            Err(err) => {
                                println!("unable to parse: {}", err);
                                None
                            }
                        }
                    }
                    None => None,
                },
            )
            .collect()
    }
}
