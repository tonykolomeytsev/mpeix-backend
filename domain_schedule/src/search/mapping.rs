use anyhow::bail;
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::{ScheduleSearchResult, ScheduleType};
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
    dbg!(&mpei_results);
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

/// Because we cannot implement trait `actix_web::FromRequest` for `ScheduleType`.
/// They belongs to different crates and no one belongs this crate.
/// I do not want to add `actix-web` dependency to `domain_schedule_models` crate.
fn parse_schedule_type(r#type: &str) -> anyhow::Result<ScheduleType> {
    match r#type {
        "group" => Ok(ScheduleType::Group),
        "person" => Ok(ScheduleType::Person),
        "room" => Ok(ScheduleType::Room),
        _ => bail!(CommonError::internal(format!(
            "Unsupported schedule type: {type}"
        ))),
    }
}
