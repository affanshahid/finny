use chrono::TimeZone;
use chrono::Utc;
use message::TextMessage;
use record::Record;

mod message;
mod parse;
mod record;

fn main() {
    let msgs = TextMessage::fetch(
        vec!["8012", "9355"],
        Utc.ymd(2022, 1, 1).and_hms(0, 0, 0),
        Utc.ymd(2022, 9, 15).and_hms(0, 0, 0),
    )
    .unwrap();

    let records = Record::parse_messages(&msgs);
    println!("{:#?}", records)
}
