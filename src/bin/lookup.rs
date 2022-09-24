use std::collections::HashMap;

use chrono::DateTime;
use chrono::Utc;
use clap::Parser;
use finny::config::Config;
use finny::message::TextMessage;
use finny::record::Record;

/// Inspect messages
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

    /// Sources to filter in
    #[clap(long, value_parser)]
    sources: Option<Vec<String>>,

    /// Sources to filter in fuzzily
    #[clap(long, value_parser)]
    sources_fuzzy: Option<Vec<String>>,

    /// Path to the matchers config
    #[clap(short, long, value_parser, default_value = "./config.yml")]
    config: String,
}

fn main() {
    let args = Args::parse();
    let msgs = TextMessage::fetch(
        &args.contacts.iter().map(|s| &s[..]).collect(),
        &args.start,
        &args.end,
    )
    .unwrap();

    let mut msg_id_map = HashMap::new();

    for msg in &msgs {
        msg_id_map.insert(msg.id, msg);
    }

    let config = Config::new(&args.config).expect("Error parsing configuration");
    let mut records = Record::parse_messages(&config.matchers, &msgs);

    if let Some(sources) = args.sources {
        records = finny::filter_in_sources(&records, &sources);
    }

    if let Some(sources) = args.sources_fuzzy {
        records = finny::fuzzy_filter_in_sources(&records, &sources)
    }

    dbg!(records
        .iter()
        .map(|r| *msg_id_map.get(&r.message_id).unwrap())
        .collect::<Vec<&TextMessage>>());
}
