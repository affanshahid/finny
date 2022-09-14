use chrono::DateTime;
use chrono::Utc;
use clap::Parser;
use message::TextMessage;
use record::Record;

mod message;
mod parse;
mod record;

/// Calculate your expenses from messages sent by your bank
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about=None)]
struct Args {
    /// Space separated list of contact numbers
    #[clap(
        short,
        long,
        value_parser,
        default_values_t=vec!["8012".to_string(),"9355".to_string()]
    )]
    contacts: Vec<String>,

    /// Start date and time between which to perform analysis
    #[clap(
        short,
        long,
        value_parser=str::parse::<DateTime<Utc>>,
        default_value_t=chronoutil::shift_months(Utc::now(), -1),
    )]
    start: DateTime<Utc>,

    /// End date and time between which to perform analysis
    #[clap(
        short,
        long,
        value_parser=str::parse::<DateTime<Utc>>,
        default_value_t=Utc::now(),
    )]
    end: DateTime<Utc>,
}

fn main() {
    let args = Args::parse();
    let msgs = TextMessage::fetch(
        &args.contacts.iter().map(|s| &s[..]).collect(),
        &args.start,
        &args.end,
    )
    .unwrap();

    let records = Record::parse_messages(&msgs);
    println!("{:#?}", records)
}
