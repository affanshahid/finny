use crate::record::Record;

pub fn filter_out_sources<'a>(records: &Vec<Record<'a>>, sources: &Vec<String>) -> Vec<Record<'a>> {
    records
        .clone()
        .into_iter()
        .filter(|r| !sources.contains(&r.source))
        .collect()
}

pub fn filter_in_sources<'a>(records: &Vec<Record<'a>>, sources: &Vec<String>) -> Vec<Record<'a>> {
    records
        .clone()
        .into_iter()
        .filter(|r| sources.contains(&r.source))
        .collect()
}

pub fn fuzzy_filter_in_sources<'a>(
    records: &Vec<Record<'a>>,
    sources: &Vec<String>,
) -> Vec<Record<'a>> {
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
