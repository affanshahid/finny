use std::error;
use std::fmt::Display;
use std::num::ParseIntError;

use chrono::Local;
use chrono::NaiveDate;
use chrono::ParseError;
use chrono::TimeZone;
use chrono::Utc;
use lazy_static::lazy_static;
use regex::Captures;
use regex::Regex;
use rusty_money::iso;
use rusty_money::Money;
use rusty_money::MoneyError;

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

pub struct Matcher {
    pub id: &'static str,
    pub pattern: Regex,
    pub factory: Box<dyn Fn(Captures, &TextMessage) -> Result<Record, Error> + Sync>,
}

lazy_static! {
    pub static ref MATCHERS: Vec<Matcher> = vec![
        Matcher {
            id: "habib-metro-cash-withdraw",
            pattern: Regex::new(
                r"HABIBMETRO - Alert: (?P<datetime>[0-9-]+ [0-9:]+) (?P<account>.+) Amt (?P<amount>.+) Debited /Switch Withdrawal (?P<location>.+), Avl. Bal .+ CR"
            ).unwrap(),
            factory: Box::new(|captures, msg| {
                Ok(Record {
                    message_id: msg.id,
                    nature: Nature::Debit,
                    account: captures["account"].to_string(),
                    amount: Money::from_str(&captures["amount"], iso::PKR)?,
                    source: captures["location"].trim().to_string(),
                    time: Local.datetime_from_str(&captures["datetime"], "%d-%m-%y %H:%M")?.with_timezone(&Utc),
                })
            }),
        },
        Matcher {
            id: "js-credit-card-used",
            pattern: Regex::new(r"Dear .+, your JS Bank credit card ending with (?P<card>.+) has been used for (?P<currency>[A-Z]+) (?P<amount>.+) at (?P<location>.+) on (?P<datetime>.+ at \d\d:\d\d:\d\d)\.").unwrap(),
            factory: Box::new(|captures, msg| {
                Ok(Record {
                    message_id: msg.id,
                    nature: Nature::Debit,
                    account: captures["card"].to_string(),
                    amount:  Money::from_str(
                        &captures["amount"],
                        iso::find(&captures["currency"]).ok_or(Error(
                            "currency not recognized: ".to_string() + &captures["currency"],
                        ))?
                    )?,
                    source: captures["location"].to_string(),
                    time: Local.datetime_from_str(&captures["datetime"], "%d/%m/%y at %H:%M:%S")?.with_timezone(&Utc)
                })
            }),
        },
        Matcher {
            id: "js-credit-card-online-used",
            pattern: Regex::new(r"Dear .+, your JS Bank credit card ending with (?P<card>.+) has been used for (?P<currency>[A-Z]+) (?P<amount>.+) at (?P<location>.+) on (?P<datetime>.+) at (?P<hour>\d+)\.").unwrap(),
            factory: Box::new(|captures, msg| {
                Ok(Record {
                    message_id: msg.id,
                    nature: Nature::Debit,
                    account: captures["card"].to_string(),
                    amount:  Money::from_str(
                        &captures["amount"],
                        iso::find(&captures["currency"]).ok_or(Error(
                            "currency not recognized: ".to_string() + &captures["currency"],
                        ))?
                    )?,
                    source: captures["location"].to_string(),
                    time:  Local
                        .from_local_datetime(
                            &NaiveDate::parse_from_str(&captures["datetime"], "%d/%m/%y")?.and_hms(
                                captures["hour"].parse()?,
                                0,
                                0,
                            ),
                        )
                        .single()
                        .ok_or(Error("unable to covert to DateTime".to_string()))?
                        .with_timezone(&Utc)
                })
            }),
        },
        Matcher{
            id: "js-cash-withdrawl",
            pattern: Regex::new(r"Acct. (?P<account>.+) debited by (?P<currency>.+) (?P<amount>.+) due to (?P<reason>.+) at (?P<datetime>.+ hrs on .+).Current Balance: .+").unwrap(),
            factory: Box::new(|captures, msg| {
                Ok(Record {
                    message_id: msg.id,
                    nature: Nature::Debit,
                    account: captures["account"].to_string(),
                    amount:  Money::from_str(
                        &captures["amount"],
                        iso::find(&captures["currency"]).ok_or(Error(
                            "currency not recognized: ".to_string() + &captures["currency"],
                        ))?
                    )?,
                    source: captures["reason"].to_string(),
                    time: Local.datetime_from_str(&captures["datetime"], "%H:%M hrs on %d-%m-%Y")?.with_timezone(&Utc)
                })
            })
        }
    ];
}
