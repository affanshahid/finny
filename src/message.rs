use std::error;
use std::rc::Rc;

use chrono::DateTime;
use chrono::TimeZone;
use chrono::Utc;
use rusqlite::params;
use rusqlite::types::Value;
use rusqlite::Connection;
use strum_macros::Display;

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
	h.id in rarray(?) and
	m.date between ? and ?
order by
    m.date;
";

#[derive(Debug)]
pub struct TextMessage {
    pub id: u32,
    pub text: String,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Display)]
pub enum Error {
    HomeDirNotFound,
    SqliteError(rusqlite::Error),
}

impl error::Error for Error {}

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
        let mut home = match home::home_dir() {
            Some(path) => path,
            None => return Err(Error::HomeDirNotFound),
        };
        home.push("Library/Messages/chat.db");

        let conn = Connection::open(home)?;
        rusqlite::vtab::array::load_module(&conn)?;

        let mut stmt = conn.prepare(&QUERY)?;
        let ids: Rc<Vec<_>> = Rc::new(
            source
                .iter()
                .map(ToString::to_string)
                .map(Value::from)
                .collect(),
        );

        let msgs = stmt.query_map(
            params![
                ids,
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
