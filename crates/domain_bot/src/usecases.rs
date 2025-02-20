use std::{cmp::Ordering, sync::Arc};

use anyhow::{anyhow, Context};
use chrono::{Datelike, Days, Local};
use common_errors::errors::CommonError;
use domain_schedule_models::{Classes, Day, ScheduleType};
use lazy_static::lazy_static;
use regex::Regex;

use crate::{
    models::{Peer, Reply, TimePrediction, UpcomingEventsPrediction, UserAction},
    peer::repository::{PeerRepository, PlatformId},
    schedule::repository::ScheduleRepository,
    search::repository::ScheduleSearchRepository,
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
pub struct TextToActionUseCase;

lazy_static! {
    static ref MENTIONS_PATTERN: Regex = Regex::new(r"(\[.*\],?)|(@\w+,?)").unwrap();
    static ref DAY_OF_WEEK_MAP: Vec<(i8, Vec<&'static str>)> = vec![
        (1, vec!["пн", "понедельник", "mon", "monday"]),
        (2, vec!["вт", "вторник", "tue", "tuesday"]),
        (3, vec!["ср", "среда", "wed", "wednesday"]),
        (4, vec!["чт", "четверг", "thu", "thursday"]),
        (5, vec!["пт", "пятница", "fri", "friday"]),
        (6, vec!["сб", "суббота", "sat", "saturday"]),
    ];
    static ref DAY_OF_WEEK_PATTERN: Regex = create_multipattern(
        r"(пар[ыау]\s+)?((в|во)\s+)?",
        &DAY_OF_WEEK_MAP
            .iter()
            .flat_map(|(_, v)| v)
            .map(|x| x.to_lowercase())
            .collect::<Vec<String>>(),
        |a, b| format!("^({a}{b})$")
    );
    static ref REL_DAY_PTR_MAP: Vec<(i8, Vec<&'static str>)> = vec![
        (2, vec!["послезавтра", "послезавтрашние", "послезавтрашний"]),
        (-2, vec!["позавчера", "позавчерашние", "позавчерашний"]),
        (0, vec!["сегодня", "сегодняшние", "сегодняшний", "/today"]),
        (-1, vec!["вчера", "вчерашние", "вчерашний", "/yesterday"]),
        (1, vec!["завтра", "завтрашние", "завтрашний", "/tomorrow"]),
    ];
    static ref REL_DAY_PTR_PATTERN: Regex = create_multipattern(
        r#"(пар[ыау])?(день)?"#,
        &REL_DAY_PTR_MAP
            .iter()
            .flat_map(|(_, v)| v)
            .map(|x| x.to_lowercase())
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
            "неделя" | "эта неделя" | "/thisweek" => {
                Ok(UserAction::WeekWithOffset(0))
            }
            "следующая неделя" | "/nextweek" => Ok(UserAction::WeekWithOffset(1)),
            "прошлая неделя" | "/prevweek" => Ok(UserAction::WeekWithOffset(-1)),
            cleared_text => {
                if DAY_OF_WEEK_PATTERN.is_match(cleared_text) {
                    let (requested_day_of_week, _) = DAY_OF_WEEK_MAP
                        .iter()
                        .find(|(_, v)| v.iter().any(|it| cleared_text.contains(it)))
                        .ok_or_else(|| {
                            CommonError::internal(
                                "Error: text present in pattern but absent in map (day of week)",
                            )
                        })?;
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
                        .find(|(_, v)| v.iter().any(|it| cleared_text.contains(it)))
                        .ok_or_else(|| {
                            CommonError::internal(
                                "Error: text present in pattern but absent in map (rel day ptr)",
                            )
                        })?;
                    Ok(UserAction::DayWithOffset(*requested_day_offset))
                } else {
                    Ok(UserAction::Unknown(cleared_text.to_owned()))
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
///
/// The main logic for generating responses to user messages is described here.
/// During the preparation of responses, asynchronous requests to the `app_schedule`
/// microservice can be made. All logic related to caching is implemented on the
/// side of the `app_schedule` microservice.
pub struct GenerateReplyUseCase(
    pub(crate) Arc<TextToActionUseCase>,
    pub(crate) Arc<PeerRepository>,
    pub(crate) Arc<ScheduleRepository>,
    pub(crate) Arc<ScheduleSearchRepository>,
    pub(crate) Arc<GetUpcomingEventsUseCase>,
);

impl GenerateReplyUseCase {
    /// Generate [Reply] model from user request for further text reply rendering.
    pub async fn generate_reply(
        &self,
        platform_id: PlatformId,
        text: &str,
    ) -> anyhow::Result<Reply> {
        let action = self.0.text_to_action(text)?;
        let peer = self.1.get_peer_by_platform_id(platform_id).await?;
        // handle initial state
        if peer.selected_schedule.is_empty() && !matches!(&action, UserAction::Unknown(_)) {
            return if peer.selecting_schedule {
                Ok(Reply::ReadyToChangeSchedule)
            } else {
                self.handle_start(peer).await
            };
        }
        match action {
            UserAction::Start => self.handle_start(peer).await,
            UserAction::WeekWithOffset(offset) => self.handle_week_with_offset(peer, offset).await,
            UserAction::DayWithOffset(offset) => self.handle_day_with_offset(peer, offset).await,
            UserAction::Unknown(q) => {
                if peer.selecting_schedule || peer.selected_schedule.is_empty() {
                    self.handle_schedule_search(peer, &q).await
                } else {
                    Ok(Reply::UnknownCommand)
                }
            }
            UserAction::ChangeScheduleIntent => {
                self.1
                    .save_peer(Peer {
                        selecting_schedule: true,
                        ..peer
                    })
                    .await?;
                Ok(Reply::ReadyToChangeSchedule)
            }
            UserAction::Help => Ok(Reply::ShowHelp),
            UserAction::UpcomingEvents => self.4.handle_upcoming_events(peer).await,
        }
    }

    /// Process `/start` command.
    /// This command can usually be sent by new bot users.
    async fn handle_start(&self, peer: Peer) -> anyhow::Result<Reply> {
        if peer.selected_schedule.is_empty() {
            self.1
                .save_peer(Peer {
                    selecting_schedule: true,
                    ..peer
                })
                .await?;
            Ok(Reply::StartGreetings)
        } else {
            let schedule_name = peer.selected_schedule.to_owned();
            self.reset_schedule_selection_if_needed(peer).await?;
            Ok(Reply::AlreadyStarted { schedule_name })
        }
    }

    /// Process `/thisweek` and `/nextweek` commands
    /// with `offset` equals 0 and 1 respectively.
    async fn handle_week_with_offset(&self, peer: Peer, offset: i8) -> anyhow::Result<Reply> {
        let schedule = self
            .2
            .get_schedule(
                &peer.selected_schedule,
                &peer.selected_schedule_type,
                offset,
            )
            .await?;
        self.reset_schedule_selection_if_needed(peer).await?;
        Ok(Reply::Week {
            week_offset: offset,
            week: schedule
                .weeks
                .first()
                .ok_or_else(|| anyhow!(CommonError::internal("Schedule does not have week")))?
                .clone(),
            schedule_type: schedule.r#type,
        })
    }

    /// Process `/today`, `/tomorrow` and other commands about specific day schedules.
    async fn handle_day_with_offset(&self, peer: Peer, offset: i8) -> anyhow::Result<Reply> {
        let current_date = Local::now().date_naive();
        let selected_date = match offset.cmp(&0) {
            Ordering::Equal => Some(current_date),
            Ordering::Greater => current_date.checked_add_days(Days::new(offset as u64)),
            Ordering::Less => current_date.checked_sub_days(Days::new(-offset as u64)),
        }
        .ok_or_else(|| anyhow!(CommonError::user("Invalid day offset")))?;
        let week_offset =
            selected_date.iso_week().week() as i8 - current_date.iso_week().week() as i8;
        let schedule = self
            .2
            .get_schedule(
                &peer.selected_schedule,
                &peer.selected_schedule_type,
                week_offset,
            )
            .await?;
        let day = schedule
            .weeks
            .iter()
            .flat_map(|week| &week.days)
            .find(|day| day.date == selected_date).cloned()
            // mock day without classes
            .unwrap_or_else(|| Day {
                day_of_week: selected_date.weekday().number_from_monday() as u8,
                date: selected_date,
                classes: Vec::with_capacity(0),
            });
        self.reset_schedule_selection_if_needed(peer).await?;
        Ok(Reply::Day {
            day_offset: offset,
            day,
            schedule_type: schedule.r#type,
        })
    }

    /// Process uncnown commands which may be a schedule change request commands.
    ///
    /// We suggest search results if it is not possible to switch to the specified schedule.
    async fn handle_schedule_search(&self, peer: Peer, q: &str) -> anyhow::Result<Reply> {
        let search_results = self
            .3
            .search_schedule(q, None)
            .await
            .with_context(|| "Error while processing schedule change")?;
        if let Some(candidate) = search_results.iter().find(|it| it.name.to_lowercase() == q) {
            self.1
                .save_peer(Peer {
                    selected_schedule: candidate.name.to_owned(),
                    selected_schedule_type: candidate.r#type.to_owned(),
                    selecting_schedule: false,
                    ..peer
                })
                .await?;
            Ok(Reply::ScheduleChangedSuccessfully(
                candidate.name.to_owned(),
            ))
        } else if !search_results.is_empty() {
            let mut results = search_results;
            let max_idx = results.len();
            results.sort_by(|a, b| {
                let idx_a = a.name.to_lowercase().find(q).or(Some(max_idx));
                let idx_b = b.name.to_lowercase().find(q).or(Some(max_idx));
                idx_a.cmp(&idx_b)
            });
            let results_contains_person = results
                .iter()
                .any(|it| matches!(it.r#type, ScheduleType::Person));

            Ok(Reply::ScheduleSearchResults {
                schedule_name: q.to_owned(),
                results_contains_person,
                results: if results_contains_person {
                    results.into_iter().take(3).map(|it| it.name).collect()
                } else {
                    results.into_iter().take(6).map(|it| it.name).collect()
                },
            })
        } else {
            Ok(Reply::CannotFindSchedule(q.to_owned()))
        }
    }

    async fn reset_schedule_selection_if_needed(&self, peer: Peer) -> anyhow::Result<()> {
        if peer.selecting_schedule {
            self.1
                .save_peer(Peer {
                    selecting_schedule: false,
                    ..peer
                })
                .await?;
        }
        Ok(())
    }
}

/// Use case which generates a response similar to the mpeix dashboard page content.
///
/// In simple words, shows upcoming events, if any.
/// Shows how much time is left until the next pair,
/// shows which pair has already started and is running.
pub struct GetUpcomingEventsUseCase(pub(crate) Arc<ScheduleRepository>);

impl GetUpcomingEventsUseCase {
    pub async fn handle_upcoming_events(&self, peer: Peer) -> anyhow::Result<Reply> {
        // load all days for current and next week
        let mut days: Vec<Day> = Vec::with_capacity(14);
        self.0
            .get_schedule(&peer.selected_schedule, &peer.selected_schedule_type, 0)
            .await?
            .weeks
            .iter_mut()
            .for_each(|week| days.append(&mut week.days));
        self.0
            .get_schedule(&peer.selected_schedule, &peer.selected_schedule_type, 1)
            .await?
            .weeks
            .iter_mut()
            .for_each(|week| days.append(&mut week.days));
        // remove all past days, (and also current day if it has only past classes)
        let local_datetime = Local::now();
        let current_date = local_datetime.date_naive();
        let current_time = local_datetime.time();
        days.retain(|day| {
            if day.date == current_date {
                // keep current day only if it has classes right now or in the future
                day.classes.iter().any(|cls| cls.time.end > current_time)
            } else {
                // keep all future days
                day.date > current_date
            }
        });
        // early return if there are no actual days
        use UpcomingEventsPrediction::*;
        if days.is_empty() {
            return Ok(Reply::UpcomingEvents {
                prediction: NoClassesNextWeek,
                schedule_type: peer.selected_schedule_type,
            });
        }
        // check first near day for classes
        let actual_day = days.first().expect("Can't be empty due to early return");
        let actual_day_is_current_day = actual_day.date == current_date;

        if actual_day_is_current_day {
            // today we can have classes in progress or only future classes
            if let Some(started_classes) = actual_day
                .classes
                .iter()
                .find(|cls| cls.time.start < current_time && cls.time.end > current_time)
            {
                // we have classes in progress
                let rest_of_future_classes = actual_day
                    .classes
                    .iter()
                    .filter(|cls| cls.time.start > current_time)
                    .cloned()
                    .collect::<Vec<Classes>>();
                Ok(Reply::UpcomingEvents {
                    prediction: ClassesTodayStarted {
                        in_progress: Box::new(started_classes.clone()),
                        future_classes: if rest_of_future_classes.is_empty() {
                            None
                        } else {
                            Some(rest_of_future_classes)
                        },
                    },
                    schedule_type: peer.selected_schedule_type,
                })
            } else {
                // we do not have classes in progress, only future classes
                let future_classes = actual_day
                    .classes
                    .iter()
                    .filter(|cls| cls.time.start > current_time)
                    .cloned()
                    .collect::<Vec<Classes>>();
                let time_prediction = TimePrediction::WithinOneDay(
                    future_classes
                        .first()
                        .expect("Cannot be empty, because actual_day has classes anyway")
                        .time
                        .start
                        .signed_duration_since(current_time),
                );
                Ok(Reply::UpcomingEvents {
                    prediction: ClassesTodayNotStarted {
                        time_prediction,
                        future_classes,
                    },
                    schedule_type: peer.selected_schedule_type,
                })
            }
        } else {
            // in the future days we can have only classes in future
            let first_classes_start_time = actual_day
                .classes
                .first()
                .expect("Cannot be empty, because actual_day has classes anyway")
                .time
                .start;
            let time_prediction = TimePrediction::WithinAWeek {
                date: actual_day.date,
                duration: actual_day
                    .date
                    .and_time(first_classes_start_time)
                    .signed_duration_since(local_datetime.naive_local()),
            };
            Ok(Reply::UpcomingEvents {
                prediction: ClassesInNDays {
                    time_prediction,
                    future_classes: actual_day.classes.to_vec(),
                },
                schedule_type: peer.selected_schedule_type,
            })
        }
    }
}

#[cfg(test)]
mod t2a_tests {
    use crate::models::UserAction;

    use super::TextToActionUseCase;

    macro_rules! test_t2a {
        ($name:tt, $exp:expr, $inputs:expr) => {
            #[test]
            fn $name() {
                let use_case = TextToActionUseCase;
                for text in $inputs {
                    assert_eq!(use_case.text_to_action(text).unwrap(), $exp);
                }
            }
        };
    }

    test_t2a!(
        action_start,
        UserAction::Start,
        ["старт", "начать", "start", "/start"]
    );

    test_t2a!(
        action_status,
        UserAction::UpcomingEvents,
        ["статус", "ближайшие пары", "ближайшие", "status", "/status"]
    );

    test_t2a!(
        action_help,
        UserAction::Help,
        ["помощь", "справка", "помоги", "help", "/help"]
    );

    test_t2a!(
        action_change,
        UserAction::ChangeScheduleIntent,
        [
            "сменить",
            "сменить группу",
            "сменить расписание",
            "change",
            "/change",
        ]
    );

    test_t2a!(
        action_week_offset_0,
        UserAction::WeekWithOffset(0),
        ["неделя", "эта неделя", "/thisweek"]
    );

    test_t2a!(
        action_week_offset_1,
        UserAction::WeekWithOffset(1),
        ["следующая неделя", "/nextweek"]
    );

    test_t2a!(
        action_week_offset_m1,
        UserAction::WeekWithOffset(-1),
        ["прошлая неделя", "/prevweek"]
    );

    test_t2a!(
        action_today,
        UserAction::DayWithOffset(0),
        [
            "сегодня",
            "сегодняшние",
            "сегодняшний",
            "/today",
            "пары сегодня",
            "сегодняшние пары"
        ]
    );

    test_t2a!(
        action_yesterday,
        UserAction::DayWithOffset(-1),
        [
            "вчера",
            "вчерашние",
            "вчерашний",
            "/yesterday",
            "пары вчера",
            "вчерашние пары"
        ]
    );

    test_t2a!(
        action_tomorrow,
        UserAction::DayWithOffset(1),
        [
            "завтра",
            "завтрашние",
            "завтрашний",
            "/tomorrow",
            "пары завтра",
            "завтрашние пары"
        ]
    );

    test_t2a!(
        action_day_after_tomorrow,
        UserAction::DayWithOffset(2),
        ["послезавтра", "послезавтрашние", "послезавтрашний"]
    );

    test_t2a!(
        action_day_before_yesterday,
        UserAction::DayWithOffset(-2),
        ["позавчера", "позавчерашние", "позавчерашний"]
    );

    #[test]
    fn action_day_of_week() {
        let use_case = TextToActionUseCase;
        for text in [
            "понедельник",
            "вторник",
            "среда",
            "четверг",
            "пятница",
            "суббота",
            "пн",
            "вт",
            "ср",
            "чт",
            "пт",
            "сб",
        ] {
            let result = use_case.text_to_action(text).unwrap();
            assert!(matches!(result, UserAction::DayWithOffset(_)));
        }
    }
}
