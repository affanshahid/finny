use std::collections::HashMap;

use lazy_static::lazy_static;
use rusty_money::iso;
use rusty_money::iso::Currency;

use crate::record::Record;

pub const NORMALIZED_CURRENCY: &Currency = iso::PKR;
lazy_static! {
    static ref EXCHANGE_RATES: HashMap<&'static str, f64> = {
        let mut m = HashMap::new();
        m.insert("USDPKR", 237.0);
        m
    };
}

pub fn filter_sources(records: &Vec<Record>, sources: &Vec<String>) -> Vec<Record> {
    records
        .clone()
        .into_iter()
        .filter(|r| !sources.contains(&r.source))
        .collect()
}

pub fn normalize_amount(record: &Record) -> f64 {
    let mut amount = record.amount;

    if record.currency != NORMALIZED_CURRENCY {
        amount = convert_currency(amount, record.currency, NORMALIZED_CURRENCY)
    }

    if amount >= 0.0 {
        amount
    } else {
        -amount
    }
}

fn convert_currency(amount: f64, from: &Currency, to: &Currency) -> f64 {
    amount * EXCHANGE_RATES[&format!("{}{}", from.iso_alpha_code, to.iso_alpha_code) as &str]
}
