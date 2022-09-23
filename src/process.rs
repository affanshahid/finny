use lazy_static::lazy_static;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use rusty_money::iso;
use rusty_money::iso::Currency;
use rusty_money::Exchange;
use rusty_money::ExchangeRate;
use rusty_money::Money;

use crate::record::Nature;
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

pub fn calculate_total(records: &Vec<Record>) -> Money<'static, Currency> {
    records
        .iter()
        .map(|r| normalize_amount(r).amount().clone())
        .reduce(|accum, current| accum + current)
        .map(|total| Money::from_decimal(total, NORMALIZED_CURRENCY))
        .unwrap()
}

pub fn normalize_amount(r: &Record) -> Money<'static, Currency> {
    //FIXME: remove clone when https://github.com/varunsrin/rusty_money/pull/76 is merged
    let mut result = r.amount.clone();

    if r.amount.currency() != NORMALIZED_CURRENCY {
        let rate = EXCHANGE
            .get_rate(r.amount.currency(), NORMALIZED_CURRENCY)
            .expect(&format!(
                "currency rate not configured: {}",
                r.amount.currency().iso_alpha_code
            ));

        result = rate.convert(result).unwrap();
        let _f = 1;
    }

    if let Nature::Debit = r.nature {
        result = Money::from_decimal(result.amount() * Decimal::NEGATIVE_ONE, result.currency());
    }

    result
}
