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
use crate::record::Nature;
use crate::record::Record;

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
        row.add_cell(Cell::new(process::normalize_amount(&r)).fg(match r.nature {
            Nature::Credit => Color::Green,
            Nature::Debit => Color::Red,
        }));

        row
    }

    fn create_total_row(&self) -> Row {
        let total = process::calculate_total(&self.records);

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
            .add_row(self.create_total_row());

        table.fmt(f)?;
        write!(f, "\n")?;

        let mut table = Table::new();
        let mut totals: Vec<_> = process::group_totals(&self.records).into_iter().collect();
        totals.sort_by(|a, b| a.1.amount().cmp(b.1.amount()));

        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .apply_modifier(UTF8_SOLID_INNER_BORDERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(vec!["Source", "Total"])
            .add_rows(
                totals
                    .iter()
                    .map(|(k, v)| vec![Cell::new(k), Cell::new(v)])
                    .collect::<Vec<_>>(),
            );

        table.fmt(f)
    }
}
