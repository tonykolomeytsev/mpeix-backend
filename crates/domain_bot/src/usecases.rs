use std::{cmp::Ordering, collections::HashMap, sync::Arc};

use anyhow::anyhow;
use chrono::{Datelike, Local};
use common_errors::errors::CommonError;
use domain_schedule::usecases::{GetScheduleUseCase, SearchScheduleUseCase};
use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    models::{Peer, Reply, UserAction},
    peer::repository::{PeerRepository, PlatformId},
};

/// Create databases if needed and run migrations.
/// This use case must be started **STRICTLY** before the server starts.
pub struct InitDomainBotUseCase(pub(crate) Arc<PeerRepository>);

impl InitDomainBotUseCase {
    pub async fn init(&self) -> anyhow::Result<()> {
        self.0.init_peer_tables().await
    }
}

/// Determine [UserAction] from text sent by user
#[derive(Default)]
pub struct TextToActionUseCase;

lazy_static! {
    static ref MENTIONS_PATTERN: Regex = Regex::new(r#"(\[.*\],?)|(@\w+,?)"#).unwrap();
    static ref DAY_OF_WEEK_MAP: HashMap<i8, Vec<&'static str>> = HashMap::from([
        (1, vec!["пн", "понедельник", "mon", "monday"]),
        (2, vec!["вт", "вторник", "tue", "tuesday"]),
        (3, vec!["ср", "среда", "wed", "wednesday"]),
        (4, vec!["чт", "четверг", "thu", "thursday"]),
        (5, vec!["пт", "пятница", "fri", "friday"]),
        (6, vec!["сб", "суббота", "sat", "saturday"]),
    ]);
    static ref DAY_OF_WEEK_PATTERN: Regex = create_multipattern(
        r#"(пар[ыау]\s+)?((в|во)\s+)?"#,
        &DAY_OF_WEEK_MAP
            .values()
            .flatten()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        |a, b| format!("{a}{b}")
    );
    static ref REL_DAY_PTR_MAP: HashMap<i8, Vec<&'static str>> = HashMap::from([
        (
            3,
            vec![
                "Послепослезавтра",
                "Послепослезавтрашние",
                "Послепослезавтрашний"
            ]
        ),
        (2, vec!["Послезавтра", "Послезавтрашние", "Послезавтрашний"]),
        (1, vec!["Завтра", "Завтрашние", "Завтрашний", "/tomorrow"]),
        (0, vec!["Сегодня", "Сегодняшние", "Сегодняшний", "/today"]),
        (-1, vec!["Вчера", "Вчерашние", "Вчерашний", "/yesterday"]),
        (-2, vec!["Позавчера", "Позавчерашние", "Позавчерашний"]),
        (
            -3,
            vec!["Позапозавчера", "Позапозавчерашние", "Позапозавчерашний"]
        ),
    ]);
    static ref REL_DAY_PTR_PATTERN: Regex = create_multipattern(
        r#"(пар[ыау])?(день)?"#,
        &REL_DAY_PTR_MAP
            .values()
            .flatten()
            .map(|x| x.to_string())
            .collect::<Vec<String>>(),
        |a, b| format!(r#"(({a}\s+)?{b})|({b}(\s+{a})?)"#)
    );
}

impl TextToActionUseCase {
    pub fn text_to_action(&self, text: &str) -> anyhow::Result<UserAction> {
        let cleared_text = MENTIONS_PATTERN.replace_all(text, "").trim().to_lowercase();
        match cleared_text.as_str() {
            "старт" | "начать" | "start" | "/start" => Ok(UserAction::Start),
            "статус" | "ближайшие пары" | "ближайшие" | "status" | "/status" => {
                Ok(UserAction::UpcomingEvents)
            }
            "помощь" | "справка" | "помоги" | "help" | "/help" => {
                Ok(UserAction::Help)
            }
            "сменить" | "сменить группу" | "сменить расписание" | "change" | "/change" => {
                Ok(UserAction::ChangeScheduleIntent)
            }
            "эта неделя" | "/thisweek" => Ok(UserAction::WeekWithOffset(0)),
            "следующая неделя" | "/nextweek" => Ok(UserAction::WeekWithOffset(1)),
            cleared_text => {
                if DAY_OF_WEEK_PATTERN.is_match(cleared_text) {
                    let (requested_day_of_week, _) = DAY_OF_WEEK_MAP
                        .iter()
                        .find(|(_, v)| v.contains(&cleared_text))
                        .expect("Fatal error: text present in pattern but absent in map");
                    let requested_day_of_week = *requested_day_of_week as u32;
                    let current_day_of_week = Local::now().weekday().number_from_monday();
                    let day_offset = match current_day_of_week.cmp(&requested_day_of_week) {
                        Ordering::Equal => 0,
                        Ordering::Less => (requested_day_of_week - current_day_of_week) as i8,
                        Ordering::Greater => {
                            (requested_day_of_week + 7 - current_day_of_week) as i8
                        }
                    };
                    Ok(UserAction::DayWithOffset(day_offset))
                } else if REL_DAY_PTR_PATTERN.is_match(cleared_text) {
                    let (requested_day_offset, _) = REL_DAY_PTR_MAP
                        .iter()
                        .find(|(_, v)| v.contains(&cleared_text))
                        .expect("Fatal error: text present in pattern but absent in map");
                    Ok(UserAction::DayWithOffset(*requested_day_offset))
                } else {
                    Err(anyhow!(CommonError::user(format!(
                        "Unknown command: {text}"
                    ))))
                }
            }
        }
    }
}

fn create_multipattern<F: FnOnce(&str, &str) -> String>(
    prefix_pattern: &str,
    variants: &[String],
    merge: F,
) -> Regex {
    let names_pattern = variants.join("|");
    Regex::new(&merge(prefix_pattern, &names_pattern)).unwrap()
}

/// Generate response to user's message.
pub struct ReplyUseCase(
    pub(crate) Arc<TextToActionUseCase>,
    pub(crate) Arc<PeerRepository>,
    pub(crate) Arc<GetScheduleUseCase>,
    pub(crate) Arc<SearchScheduleUseCase>,
);

impl ReplyUseCase {
    pub async fn reply(&self, platform_id: PlatformId, text: &str) -> anyhow::Result<Reply> {
        let action = self.0.text_to_action(text)?;
        let peer = self.1.get_peer_by_platform_id(platform_id).await?;
        match action {
            UserAction::Start => {
                if peer.is_not_started() {
                    self.1
                        .save_peer(Peer {
                            selecting_schedule: true,
                            ..peer
                        })
                        .await?;
                    Ok(Reply::StartGreetings)
                } else {
                    Ok(Reply::AlreadyStarted {
                        schedule_name: peer.selected_schedule,
                    })
                }
            }
            UserAction::WeekWithOffset(offset) => {
                let schedule = self
                    .2
                    .get_schedule(
                        peer.selected_schedule,
                        peer.selected_schedule_type,
                        offset as i32,
                    )
                    .await?;
                Ok(Reply::Week(schedule))
            }
            _ => todo!(),
        }
    }
}
