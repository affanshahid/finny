use crate::record::Record;

pub fn filter_out_sources(records: &Vec<Record>, sources: &Vec<String>) -> Vec<Record> {
    records
        .clone()
        .into_iter()
        .filter(|r| !sources.contains(&r.source))
        .collect()
}

pub fn filter_in_sources(records: &Vec<Record>, sources: &Vec<String>) -> Vec<Record> {
    records
        .clone()
        .into_iter()
        .filter(|r| sources.contains(&r.source))
        .collect()
}

pub fn fuzzy_filter_in_sources(records: &Vec<Record>, sources: &Vec<String>) -> Vec<Record> {
    records
        .clone()
        .into_iter()
        .filter(|r| {
            sources
                .iter()
                .any(|s| r.source.to_lowercase().contains(&s.to_lowercase()))
        })
        .collect()
}
