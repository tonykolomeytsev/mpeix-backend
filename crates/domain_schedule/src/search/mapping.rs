use common_errors::errors::CommonError;
use domain_schedule_models::{ScheduleSearchResult, ScheduleType};
use lazy_static::lazy_static;
use regex::Regex;

use crate::dto::mpei::MpeiSearchResult;

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
            r#type: res
                .r#type
                .parse::<ScheduleType>()
                .map_err(CommonError::internal)?,
        })
    }
    Ok(output)
}
