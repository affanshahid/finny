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

use crate::process;
use crate::record::Money;
use crate::record::Record;
use crate::Subscription;

fn create_total_row(total: &Money, col_count: usize) -> Row {
    if col_count < 2 {
        panic!("table must have atleast 2 cols")
    }

    let empty_rows = col_count - 2;
    let mut row = Row::new();

    for _ in 0..empty_rows {
        row.add_cell(Cell::new(""));
    }
    row.add_cell(
        Cell::new("TOTAL")
            .add_attributes(vec![Attribute::Bold])
            .set_alignment(CellAlignment::Right),
    );

    row.add_cell(Cell::new(&total).fg(if total.amount().is_sign_positive() {
        Color::Green
    } else {
        Color::Red
    }));

    row
}

fn default_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

pub struct TransactionsTable<'a> {
    show_matchers: bool,
    records: &'a Vec<Record<'a>>,
}

impl<'a> TransactionsTable<'a> {
    pub fn new(records: &'a Vec<Record>, show_matchers: bool) -> TransactionsTable<'a> {
        TransactionsTable {
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
        row.add_cell(Cell::new(process::normalize_amount(&r.amount)).fg(
            if r.amount.is_positive() {
                Color::Green
            } else {
                Color::Red
            },
        ));

        row
    }
}

impl Display for TransactionsTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let header = if self.show_matchers {
            vec!["ID", "Time", "Pattern", "Reason", "Amount"]
        } else {
            vec!["ID", "Time", "Reason", "Amount"]
        };

        let col_count = header.len();

        let mut table = default_table();
        table
            .set_header(header)
            .add_rows(
                self.records
                    .iter()
                    .map(|r| self.record_to_row(r))
                    .collect::<Vec<_>>(),
            )
            .add_row(create_total_row(
                &process::calculate_total(&self.records.iter().map(|r| &r.amount).collect()),
                col_count,
            ));

        table.fmt(f)
    }
}

pub struct TotalsTable<'a> {
    records: &'a Vec<Record<'a>>,
}

impl<'a> TotalsTable<'a> {
    pub fn new(records: &'a Vec<Record<'a>>) -> TotalsTable<'a> {
        TotalsTable { records }
    }
}

impl Display for TotalsTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut totals: Vec<_> = process::group_totals(&self.records).into_iter().collect();
        totals.sort_by(|a, b| a.1.cmp(&b.1));

        let mut table = default_table();
        table
            .set_header(vec!["Source", "Total"])
            .add_rows(
                totals
                    .iter()
                    .map(|(k, v)| vec![Cell::new(k), Cell::new(v)])
                    .collect::<Vec<_>>(),
            )
            .add_row(create_total_row(
                &process::calculate_total(&totals.iter().map(|(_k, v)| v).collect()),
                2,
            ));

        table.fmt(f)
    }
}

pub struct SubscriptionsTable<'a> {
    records: &'a Vec<Record<'a>>,
}

impl<'a> SubscriptionsTable<'a> {
    pub fn new(records: &'a Vec<Record<'a>>) -> SubscriptionsTable<'a> {
        SubscriptionsTable { records }
    }
}

impl Display for SubscriptionsTable<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut subs: Vec<_> = process::get_subscriptions(&self.records)
            .into_iter()
            .map(|s| Subscription {
                amount: process::normalize_amount(&s.amount),
                ..s
            })
            .collect();
        subs.sort_by(|a, b| a.amount.cmp(&b.amount));

        let mut table = default_table();
        table
            .set_header(vec!["Source", "Day", "Amount"])
            .add_rows(
                subs.iter()
                    .map(|s| {
                        vec![
                            Cell::new(&s.source),
                            Cell::new(s.charge_date),
                            Cell::new(&s.amount),
                        ]
                    })
                    .collect::<Vec<_>>(),
            )
            .add_row(create_total_row(
                &process::calculate_total(&subs.iter().map(|s| &s.amount).collect()),
                3,
            ));

        table.fmt(f)
    }
}
