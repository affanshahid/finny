use chrono::DateTime;
use chrono::Utc;
use clap::Parser;
use finny::message::TextMessage;
use finny::process::filter_out_sources;
use finny::record::Record;
use finny::viewer::Viewer;
use lazy_static::lazy_static;

lazy_static! {
    static ref DEFAULT_EXCLUDE_SOURCES: Vec<String> =
        vec!["JS Credit Card Bill Pay From IB".to_string()];
}

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
        default_value_t=chronoutil::shift_months(Utc::now(), -2),
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

    /// Sources to filter out
    #[clap(long, value_parser, default_values_t=DEFAULT_EXCLUDE_SOURCES.iter())]
    exclude_sources: Vec<String>,

    /// Show pattern id when displaying transactions
    #[clap(short = 'p', long, value_parser, default_value_t = false)]
    show_matcher: bool,
}

fn main() {
    let args = Args::parse();
    let msgs = TextMessage::fetch(
        &args.contacts.iter().map(|s| &s[..]).collect(),
        &args.start,
        &args.end,
    )
    .unwrap();

    let mut records = Record::parse_messages(&msgs);
    records = filter_out_sources(&records, &args.exclude_sources);

    let v = Viewer::new(records, args.show_matcher);
    println!("{}", v);
}
