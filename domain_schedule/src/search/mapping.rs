use domain_schedule_models::dto::v1::ScheduleSearchResult;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{dto::mpei::MpeiSearchResult, parse_schedule_type};

lazy_static! {
    static ref SPACES_PATTERN: Regex = Regex::new(r"\s{2,}").unwrap();
}

pub(crate) fn map_search_models(
    mpei_results: Vec<MpeiSearchResult>,
) -> anyhow::Result<Vec<ScheduleSearchResult>> {
    let mut output = Vec::with_capacity(mpei_results.len());
    for res in mpei_results {
        output.push(ScheduleSearchResult {
            name: SPACES_PATTERN.replace_all(&res.label, " ").to_string(),
            description: res.description.trim().to_owned(),
            id: res.id.to_string(),
            r#type: parse_schedule_type(&res.r#type)?,
        })
    }
    Ok(output)
}