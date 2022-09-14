use std::error;
use std::fmt::Display;
use std::num::ParseFloatError;
use std::str::FromStr;

use chrono::Local;
use chrono::ParseError;
use chrono::TimeZone;
use chrono::Utc;
use lazy_static::lazy_static;
use regex::Captures;
use regex::Regex;

use crate::message::TextMessage;
use crate::record::Nature;
use crate::record::Record;

#[derive(Debug)]
pub struct Error(String);

impl error::Error for Error {}

impl From<ParseFloatError> for Error {
    fn from(error: <f64 as FromStr>::Err) -> Self {
        Error(format!(
            "error parsing monetary value: {}",
            &error.to_string()
        ))
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

pub struct Matcher {
    pub pattern: Regex,
    pub factory: Box<dyn Fn(Captures, &TextMessage) -> Result<Record, Error> + Sync>,
}

lazy_static! {
    pub static ref MATCHERS: Vec<Matcher> = vec![
        Matcher {
            pattern: Regex::new(
                r"HABIBMETRO - Alert: (?P<datetime>[0-9-]+ [0-9:]+) (?P<account>.+) Amt (?P<amount>.+) Debited /Switch Withdrawal (?P<location>.+), Avl. Bal .+ CR"
            ).unwrap(),
            factory: Box::new(|captures, msg| {
                Ok(Record {
                    message_id: msg.id,
                    nature: Nature::Debit,
                    account: captures["account"].to_string(),
                    currency: String::from("PKR"),
                    amount: parse_monetary_value(&captures["amount"])?,
                    source: captures["location"].trim().to_string(),
                    time: Local.datetime_from_str(&captures["datetime"], "%d-%m-%y %H:%M")?.with_timezone(&Utc),
                })
            }),
        },
        Matcher {
            pattern: Regex::new(r"Dear .+, your JS Bank credit card ending with (?P<card>.+) has been used for (?P<currency>[A-Z]+) (?P<amount>.+) at (?P<location>.+) on (?P<datetime>.+ at .+)\.").unwrap(),
            factory: Box::new(|captures, msg| {
                Ok(Record {
                    message_id: msg.id,
                    nature: Nature::Debit,
                    account: captures["card"].to_string(),
                    currency: captures["currency"].to_string(),
                    amount:  parse_monetary_value(&captures["amount"])?,
                    source: captures["location"].to_string(),
                    time: Local.datetime_from_str(&captures["datetime"], "%d/%m/%y at %H:%M:%S")?.with_timezone(&Utc)
                })
            }),
        },
        Matcher{
            pattern: Regex::new(r"Acct. (?P<account>.+) debited by (?P<currency>.+) (?P<amount>.+) due to (?P<reason>.+) at (?P<datetime>.+ hrs on .+).Current Balance: .+").unwrap(),
            factory: Box::new(|captures, msg| {
                Ok(Record {
                    message_id: msg.id,
                    nature: Nature::Debit,
                    account: captures["account"].to_string(),
                    currency: captures["currency"].to_string(),
                    amount: parse_monetary_value(&captures["amount"])?,
                    source: captures["reason"].to_string(),
                    time: Local.datetime_from_str(&captures["datetime"], "%H:%M hrs on %d-%m-%Y")?.with_timezone(&Utc)
                })
            })
        }
    ];
}

fn parse_monetary_value(s: &str) -> Result<f64, ParseFloatError> {
    s.replace(",", "").parse()
}
