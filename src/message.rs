use std::error;
use std::fmt::Display;

use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use rusqlite::params;
use rusqlite::Connection;

const NSECS_SINCE_2001: i64 = 978307200000000000;
const QUERY: &str = "
select 
	m.ROWID as id, 
	m.text as text, 
	m.date as century_epoch 
from handle h 
join message m 
	on h.ROWID = m.handle_id  
where 
	h.id in {IDS} and
	m.date between ? and ?
order by
    m.date;
";

pub struct TextMessage {
    pub id: u32,
    pub text: String,
    pub time: DateTime<Utc>,
}

#[derive(Debug)]
pub enum Error {
    HomeDirNotFound,
    SqliteError(rusqlite::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::HomeDirNotFound => write!(f, "unable to get home dir"),
            Error::SqliteError(err) => write!(f, "{}", err),
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(error: rusqlite::Error) -> Error {
        Error::SqliteError(error)
    }
}

impl TextMessage {
    pub fn fetch(
        source: &Vec<&str>,
        start: &DateTime<Utc>,
        end: &DateTime<Utc>,
    ) -> Result<Vec<TextMessage>, Error> {
        let in_param = String::from("(")
            + &source
                .iter()
                .map(|s| format!("\"{}\"", s).to_string())
                .collect::<Vec<String>>()
                .join(",")
            + ")";

        let mut home = match home::home_dir() {
            Some(path) => path,
            None => return Err(Error::HomeDirNotFound),
        };
        home.push("Library/Messages/chat.db");

        let conn = Connection::open(home)?;
        let mut stmt = conn.prepare(&QUERY.replace("{IDS}", &in_param))?;

        let msgs = stmt.query_map(
            params![
                start.timestamp_nanos() - NSECS_SINCE_2001,
                (end.timestamp_nanos() - NSECS_SINCE_2001),
            ],
            |row| {
                Ok(TextMessage {
                    id: row.get(0)?,
                    text: row.get(1)?,
                    time: TextMessage::parse_time_from_century_epoch(row.get(2)?),
                })
            },
        )?;

        Ok(msgs.collect::<Result<Vec<_>, _>>()?)
    }

    fn parse_time_from_century_epoch(century_epoch: i64) -> DateTime<Utc> {
        let epoch = century_epoch + NSECS_SINCE_2001;
        Utc.timestamp(epoch / 1_000_000_000, (epoch % 1_000_000_000) as u32)
    }
}
