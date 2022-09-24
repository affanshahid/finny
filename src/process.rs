use std::borrow::Borrow;
use std::collections::HashMap;
use std::vec;

use chrono::Datelike;
use lazy_static::lazy_static;
use rust_decimal_macros::dec;
use rusty_money::iso;
use rusty_money::iso::Currency;
use rusty_money::Exchange;
use rusty_money::ExchangeRate;

use crate::record::Money;
use crate::record::Record;

pub fn filter_out_sources<'a>(records: &Vec<Record<'a>>, sources: &Vec<String>) -> Vec<Record<'a>> {
    records
        .clone()
        .into_iter()
        .filter(|r| !sources.contains(&r.source))
        .collect()
}

pub fn filter_in_sources<'a>(records: &Vec<Record<'a>>, sources: &Vec<String>) -> Vec<Record<'a>> {
    records
        .clone()
        .into_iter()
        .filter(|r| sources.contains(&r.source))
        .collect()
}

pub fn fuzzy_filter_in_sources<'a>(
    records: &Vec<Record<'a>>,
    sources: &Vec<String>,
) -> Vec<Record<'a>> {
    records
        .clone()
        .into_iter()
        .filter(|r| {
            sources
                .iter()
                .any(|s| r.source.to_lowercase().contains(&s.to_lowercase()))
        })
        .collect()
}

pub const NORMALIZED_CURRENCY: &Currency = iso::PKR;

lazy_static! {
    //FIXME: remove when https://github.com/varunsrin/rusty_money/pull/75 is merged
    static ref RATES: Vec<ExchangeRate<'static, Currency>> =
        vec![
            ExchangeRate::new(iso::USD, iso::PKR, dec!(237)).unwrap(),
            ExchangeRate::new(iso::SGD, iso::PKR, dec!(158)).unwrap()
        ];
    static ref EXCHANGE: Exchange<'static, Currency> = {
        let mut exchange: Exchange<'static, Currency> = Exchange::new();
        for rate in RATES.iter() {
            exchange.set_rate(&rate);
        }
        exchange
    };
}

pub fn calculate_total(moneys: &Vec<impl Borrow<Money>>) -> Money {
    moneys
        .iter()
        .map(|r| normalize_amount(r.borrow()).amount().clone())
        .reduce(|accum, current| accum + current)
        .map(|total| Money::from_decimal(total, NORMALIZED_CURRENCY))
        .unwrap()
}

pub fn normalize_amount(amount: &Money) -> Money {
    //FIXME: remove clone when https://github.com/varunsrin/rusty_money/pull/76 is merged
    let mut result = amount.clone();

    if amount.currency() != NORMALIZED_CURRENCY {
        let rate = EXCHANGE
            .get_rate(amount.currency(), NORMALIZED_CURRENCY)
            .expect(&format!(
                "currency rate not configured: {}",
                amount.currency().iso_alpha_code
            ));

        result = rate.convert(result).unwrap();
        let _f = 1;
    }

    result
}

pub fn group<'a>(records: &Vec<Record<'a>>) -> HashMap<String, Vec<Record<'a>>> {
    let mut map = HashMap::new();

    for record in records {
        match map.get_mut(&record.source) {
            None => {
                map.insert(record.source.clone(), vec![record.clone()]);
            }
            Some(list) => list.push(record.clone()),
        };
    }

    map
}

pub fn group_totals(records: &Vec<Record>) -> HashMap<String, Money> {
    group(&records)
        .into_iter()
        .map(|(k, v)| (k, calculate_total(&v.iter().map(|r| &r.amount).collect())))
        .collect()
}

pub struct Subscription {
    pub source: String,
    pub amount: Money,
    pub charge_date: u32,
}

pub fn get_subscriptions(records: &Vec<Record>) -> Vec<Subscription> {
    let groups = group(records);

    // filter out groups with only a single charge
    let groups = groups.iter().filter(|(_k, v)| v.len() > 1);

    // filter out groups with records that aren't spaced exactly one month apart
    let groups = groups.filter(|(_k, v)| {
        for i in 1..v.len() {
            let cur_date = v[i].time.date();
            let prev_date = v[i - 1].time.date();

            if cur_date == prev_date || cur_date.day() != prev_date.day() {
                return false;
            }
        }

        true
    });

    // filter out groups with records that don't have the same amount
    let groups = groups.filter(|(_k, v)| {
        for i in 1..v.len() {
            if v[i].amount != v[i - 1].amount {
                return false;
            }
        }

        true
    });

    groups
        .map(|(k, v)| Subscription {
            source: k.clone(),
            amount: v[0].amount.clone(),
            charge_date: v[0].time.date().day(),
        })
        .collect()
}
