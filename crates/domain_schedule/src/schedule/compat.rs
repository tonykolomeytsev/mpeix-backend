use chrono::{DateTime, Local};
use common_in_memory_cache::Entry;
use domain_schedule_models::dto::v1::Schedule;
use serde::{Deserialize, Serialize};

/// Low-cost wrapper for Entry<Schedule> for safe reading from old cache entries,
/// generated by Kotlin backend.
#[derive(Debug, Deserialize)]
pub struct ReadingPersistentEntry {
    value: Schedule,
    #[serde(alias = "created", with = "datetime_serde")]
    created_at: Option<DateTime<Local>>,
    #[serde(alias = "accessed", with = "datetime_serde")]
    accessed_at: Option<DateTime<Local>>,
    #[serde(alias = "hitsNumber")]
    hits: u32,
}

/// Low-cost wrapper for Entry<Schedule>, just to write value to the persistent cache.
/// It lets us remove `serde` dependency from `common_in_memory_cache` crate.
/// Also it lets us get rid of backward compatibility logic from other crates.
#[derive(Debug, Serialize)]
pub struct WritingPersistentEntry<'a> {
    value: &'a Schedule,
    created_at: &'a DateTime<Local>,
    accessed_at: &'a DateTime<Local>,
    hits: u32,
}

impl From<ReadingPersistentEntry> for Entry<Schedule> {
    fn from(value: ReadingPersistentEntry) -> Self {
        Self {
            value: value.value,
            created_at: value.created_at.unwrap_or_else(Local::now),
            accessed_at: value.accessed_at.unwrap_or_else(Local::now),
            hits: value.hits,
        }
    }
}

#[inline]
pub fn writing(entry: &'_ Entry<Schedule>) -> WritingPersistentEntry<'_> {
    WritingPersistentEntry {
        value: &entry.value,
        created_at: &entry.created_at,
        accessed_at: &entry.accessed_at,
        hits: entry.hits,
    }
}

mod datetime_serde {
    use chrono::{DateTime, Local, NaiveDateTime, TimeZone};
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Local>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        if let Some(s) = s {
            // for backward compatibility with Kotlin generated cache entries,
            // remove '[Europe/Moscow]' at the end
            let s = s.chars().take_while(|it| it != &'[').collect::<String>();
            let naive = NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.f%z")
                .map_err(serde::de::Error::custom)?;

            // TODO: from_local_datetime always returns LocalResult::Single, but I need to get rid of unwrap()
            // for compatibility with future versions
            let local = Local.from_local_datetime(&naive).unwrap();
            return Ok(Some(local));
        }
        Ok(None)
    }
}