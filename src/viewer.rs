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
use crate::process::NORMALIZED_CURRENCY;
use crate::record::Nature;
use crate::record::Record;

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
            Cell::new(format!(
                "{} {:.2}",
                NORMALIZED_CURRENCY,
                process::normalize_amount(r)
            ))
            .fg(match r.nature {
                Nature::Credit => Color::Green,
                Nature::Debit => Color::Red,
            }),
        ]
        .into()
    }

    fn compute_total_row(records: &Vec<Record>) -> Row {
        let total = records
            .iter()
            .map(|r| match r.nature {
                Nature::Credit => r.amount,
                Nature::Debit => -r.amount,
            })
            .reduce(|accum, current| accum + current)
            .unwrap();

        vec![
            Cell::new(""),
            Cell::new(""),
            Cell::new("TOTAL")
                .add_attributes(vec![Attribute::Bold, Attribute::SlowBlink])
                .set_alignment(CellAlignment::Right),
            Cell::new(format!("{} {:.2}", NORMALIZED_CURRENCY, total)).fg(if total >= 0.0 {
                Color::Green
            } else {
                Color::Red
            }),
        ]
        .into()
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