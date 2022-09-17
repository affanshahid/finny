use std::fmt::Display;
use std::vec;

use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::modifiers::UTF8_SOLID_INNER_BORDERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::Attribute;
use comfy_table::Cell;
use comfy_table::CellAlignment;
use comfy_table::Color;
use comfy_table::ContentArrangement;
use comfy_table::Row;
use comfy_table::Table;
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

pub const NORMALIZED_CURRENCY: &Currency = iso::PKR;

lazy_static! {
    //FIXME: remove when https://github.com/varunsrin/rusty_money/pull/75 is merged
    static ref RATES: Vec<ExchangeRate<'static, Currency>> =
        vec![ExchangeRate::new(iso::USD, iso::PKR, dec!(237)).unwrap(),];
    static ref EXCHANGE: Exchange<'static, Currency> = {
        let mut exchange: Exchange<'static, Currency> = Exchange::new();
        for rate in RATES.iter() {
            exchange.set_rate(&rate);
        }
        exchange
    };
}

pub struct Viewer {
    records: Vec<Record>,
}

impl Viewer {
    pub fn new(records: Vec<Record>) -> Viewer {
        Viewer { records: records }
    }

    fn record_to_row(r: &Record) -> Row {
        vec![
            Cell::new(r.message_id),
            Cell::new(r.time.format("%a, %d/%m/%y %I:%M %p")),
            Cell::new(&r.source),
            Cell::new(Self::normalized_amount(&r)).fg(match r.nature {
                Nature::Credit => Color::Green,
                Nature::Debit => Color::Red,
            }),
        ]
        .into()
    }

    fn compute_total_row(records: &Vec<Record>) -> Row {
        let total = records
            .iter()
            .map(|r| Self::normalized_amount(r).amount().clone())
            .reduce(|accum, current| accum + current)
            .map(|total| Money::from_decimal(total, NORMALIZED_CURRENCY))
            .unwrap();

        vec![
            Cell::new(""),
            Cell::new(""),
            Cell::new("TOTAL")
                .add_attributes(vec![Attribute::Bold])
                .set_alignment(CellAlignment::Right),
            Cell::new(&total).fg(if total.amount().is_sign_positive() {
                Color::Green
            } else {
                Color::Red
            }),
        ]
        .into()
    }

    fn normalized_amount(r: &Record) -> Money<'static, Currency> {
        //FIXME: remove clone when https://github.com/varunsrin/rusty_money/pull/76 is merged
        let mut result = r.amount.clone();

        if r.amount.currency() != NORMALIZED_CURRENCY {
            let rate = EXCHANGE
                .get_rate(r.amount.currency(), NORMALIZED_CURRENCY)
                .unwrap();

            result = rate.convert(result).unwrap()
        }

        if let Nature::Debit = r.nature {
            result =
                Money::from_decimal(result.amount() * Decimal::NEGATIVE_ONE, result.currency());
        }

        result
    }
}

impl Display for Viewer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();

        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["ID", "Time", "Reason", "Amount"])
            .add_rows(
                self.records
                    .iter()
                    .map(Self::record_to_row)
                    .collect::<Vec<_>>(),
            )
            .add_row(Self::compute_total_row(&self.records));

        table.fmt(f)
    }
}
