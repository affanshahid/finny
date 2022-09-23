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

pub struct Viewer<'a> {
    show_matchers: bool,
    records: Vec<Record<'a>>,
}

impl Viewer<'_> {
    pub fn new(records: Vec<Record>, show_matchers: bool) -> Viewer {
        Viewer {
            records,
            show_matchers,
        }
    }

    fn record_to_row(&self, r: &Record) -> Row {
        let mut row = Row::new();
        row.add_cell(Cell::new(r.message_id));
        row.add_cell(Cell::new(r.time.format("%a, %d/%m/%y %I:%M %p")));
        if self.show_matchers {
            row.add_cell(Cell::new(&r.matcher.id));
        }
        row.add_cell(Cell::new(&r.source));
        row.add_cell(Cell::new(Self::normalized_amount(&r)).fg(match r.nature {
            Nature::Credit => Color::Green,
            Nature::Debit => Color::Red,
        }));

        row
    }

    fn compute_total_row(&self) -> Row {
        let total = self
            .records
            .iter()
            .map(|r| Self::normalized_amount(r).amount().clone())
            .reduce(|accum, current| accum + current)
            .map(|total| Money::from_decimal(total, NORMALIZED_CURRENCY))
            .unwrap();

        let mut row = vec![
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
        ];

        if self.show_matchers {
            row.insert(0, Cell::new(""));
        }
        row.into()
    }

    fn normalized_amount(r: &Record) -> Money<'static, Currency> {
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
            result =
                Money::from_decimal(result.amount() * Decimal::NEGATIVE_ONE, result.currency());
        }

        result
    }
}

impl Display for Viewer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        let header = if self.show_matchers {
            vec!["ID", "Time", "Pattern", "Reason", "Amount"]
        } else {
            vec!["ID", "Time", "Reason", "Amount"]
        };

        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(header)
            .add_rows(
                self.records
                    .iter()
                    .map(|r| self.record_to_row(r))
                    .collect::<Vec<_>>(),
            )
            .add_row(self.compute_total_row());

        table.fmt(f)
    }
}
