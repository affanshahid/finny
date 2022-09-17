use crate::record::Record;

pub fn filter_sources(records: &Vec<Record>, sources: &Vec<String>) -> Vec<Record> {
    records
        .clone()
        .into_iter()
        .filter(|r| !sources.contains(&r.source))
        .collect()
}
