use chrono::DateTime;
use chrono::Utc;
use clap::Parser;
use finny::config::Config;
use finny::message::TextMessage;
use finny::process::filter_out_sources;
use finny::record::Record;
use finny::tables::SubscriptionsTable;
use finny::tables::TotalsTable;
use finny::tables::TransactionsTable;
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
        default_value_t=chronoutil::shift_months(Utc::now(), -3),
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

    /// Path to the matchers config
    #[clap(short, long, value_parser, default_value = "./config.yml")]
    config: String,

    #[clap(subcommand)]
    subcommand: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Shows a table of transactions
    Transactions {
        /// Show pattern id when displaying transactions
        #[clap(short = 'p', long, value_parser, action)]
        show_matcher: bool,
    },

    /// Shows aggregated totals for each source
    Totals,

    /// Shows detected subscriptions from your data
    Subscriptions,
}

fn main() {
    let args = Args::parse();
    let msgs = TextMessage::fetch(
        &args.contacts.iter().map(|s| &s[..]).collect(),
        &args.start,
        &args.end,
    )
    .unwrap();

    let config = Config::new(&args.config).expect("Error parsing configuration");
    let mut records = Record::parse_messages(&config.matchers, &msgs);
    records = filter_out_sources(&records, &args.exclude_sources);

    match args.subcommand {
        Command::Transactions { show_matcher } => {
            let v = TransactionsTable::new(&records, show_matcher);
            println!("{}", v);
        }
        Command::Totals => {
            let v = TotalsTable::new(&records);
            println!("{}", v);
        }
        Command::Subscriptions => {
            let v = SubscriptionsTable::new(&records);
            println!("{}", v);
        }
    }
}
