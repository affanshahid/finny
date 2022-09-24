use chrono::DateTime;
use chrono::Utc;
use rusty_money::iso::Currency;

use crate::message::TextMessage;
use crate::parser::Matcher;
use crate::parser::RecordParser;

pub type Money = rusty_money::Money<'static, Currency>;

#[derive(Debug, Clone)]
pub struct Record<'a> {
    pub matcher: &'a Matcher,
    pub message_id: u32,
    pub account: String,
    pub amount: Money,
    pub source: String,
    pub time: DateTime<Utc>,
}

impl Record<'_> {
    pub fn parse_messages<'a>(
        matchers: &'a Vec<Matcher>,
        messages: &'a Vec<TextMessage>,
    ) -> Vec<Record<'a>> {
        let parser = RecordParser::new(matchers);

        messages.iter().filter_map(|m| parser.parse(m)).collect()
    }
}
