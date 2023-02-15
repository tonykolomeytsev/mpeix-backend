use chrono::{Datelike, Weekday};
use domain_schedule_models::dto::v1::{Classes, Day, ScheduleType, Week};

use crate::models::{Reply, TimePrediction, UpcomingEventsPrediction};
use std::fmt::Write;

pub enum RenderTargetPlatform {
    Vk,
    Telegram,
}

/// Turn the [Reply] response model into the text of the message, for further sending to social networks.
pub fn render_message(reply: &Reply, platform: RenderTargetPlatform) -> String {
    match reply {
        Reply::StartGreetings => include_str!("../res/msg_cannot_find_schedule.txt").to_owned(),
        Reply::AlreadyStarted { schedule_name: _ } => {
            include_str!("../res/msg_already_started.txt").to_owned()
        }
        Reply::Week {
            week_offset,
            week,
            schedule_type,
        } => {
            let mut buf = String::with_capacity(4096);
            render_week(*week_offset, week, schedule_type, &mut buf);
            buf
        }
        Reply::Day {
            day_offset,
            day,
            schedule_type,
        } => {
            let mut buf = String::with_capacity(2048);
            render_day(*day_offset, day, schedule_type, &mut buf, false);
            buf
        }
        Reply::UpcomingEvents {
            prediction,
            schedule_type,
        } => {
            let mut buf = String::with_capacity(2048);
            render_upcoming_events(prediction, schedule_type, &mut buf);
            buf
        }
        Reply::ScheduleChangedSuccessfully(schedule_name) => format!(
            include_str!("../res/msg_schedule_changed_successfully.txt"),
            schedule_name = &schedule_name
        ),
        Reply::ScheduleSearchResults {
            schedule_name,
            results: _,
            results_contains_person: _,
        } => format!(
            include_str!("../res/msg_schedule_search_results.txt"),
            schedule_name = &schedule_name
        ),
        Reply::CannotFindSchedule(q) => {
            format!(
                include_str!("../res/msg_cannot_find_schedule.txt"),
                schedule_name = q
            )
        }
        Reply::ReadyToChangeSchedule => {
            include_str!("../res/msg_ready_to_change_schedule.txt").to_owned()
        }
        Reply::ShowHelp => match platform {
            RenderTargetPlatform::Telegram => {
                include_str!("../res/msg_show_help_telegram.txt").to_owned()
            }
            RenderTargetPlatform::Vk => include_str!("../res/msg_show_help_vk.txt").to_owned(),
        },
        Reply::UnknownCommand => match platform {
            RenderTargetPlatform::Telegram => {
                include_str!("../res/msg_unknown_command_telegram.txt").to_owned()
            }
            RenderTargetPlatform::Vk => {
                include_str!("../res/msg_unknown_command_vk.txt").to_owned()
            }
        },
        Reply::UnknownMessageType => match platform {
            RenderTargetPlatform::Telegram => {
                include_str!("../res/msg_unknown_message_type_telegram.txt").to_owned()
            }
            RenderTargetPlatform::Vk => {
                include_str!("../res/msg_unknown_message_type_vk.txt").to_owned()
            }
        },
        Reply::InternalError => match platform {
            RenderTargetPlatform::Telegram => {
                include_str!("../res/msg_internal_error_telegram.txt").to_owned()
            }
            RenderTargetPlatform::Vk => include_str!("../res/msg_internal_error_vk.txt").to_owned(),
        },
    }
}

fn render_upcoming_events(
    prediction: &UpcomingEventsPrediction,
    schedule_type: &ScheduleType,
    buf: &mut String,
) {
    use UpcomingEventsPrediction::*;
    match prediction {
        NoClassesNextWeek => buf.push_str("В ближайшие несколько дней нет пар"),
        ClassesTodayNotStarted {
            time_prediction,
            future_classes,
        } => {
            render_time_prediction(time_prediction, buf);
            for (i, cls) in future_classes.iter().enumerate() {
                if i > 0 {
                    buf.push_str("\n\n");
                }
                render_classes(cls, schedule_type, buf);
            }
        }
        ClassesTodayStarted {
            in_progress,
            future_classes,
        } => {
            buf.push_str("Пара уже началась:\n\n");
            render_classes(in_progress, schedule_type, buf);
            if let Some(classes) = future_classes {
                buf.push_str("\n\nДалее:\n\n");
                for (i, cls) in classes.iter().enumerate() {
                    if i > 0 {
                        buf.push_str("\n\n");
                    }
                    render_classes(cls, schedule_type, buf);
                }
            }
        }
        ClassesInNDays {
            time_prediction,
            future_classes,
        } => {
            render_time_prediction(time_prediction, buf);
            for (i, cls) in future_classes.iter().enumerate() {
                if i > 0 {
                    buf.push_str("\n\n");
                }
                render_classes(cls, schedule_type, buf);
            }
        }
    }
}

fn render_time_prediction(time_prediction: &TimePrediction, buf: &mut String) {
    match time_prediction {
        TimePrediction::WithinOneDay(duration) => {
            buf.push_str("Ближайшая пара начнется через ");
            render_duration(duration, buf)
        }
        TimePrediction::WithinAWeek { date, duration } => {
            if duration.num_hours() < 24 {
                buf.push_str("Ближайшая пара начнется через ");
                render_duration(duration, buf)
            } else {
                buf.push_str("Ближайшие пары ");
                buf.push_str(render_day_of_week_gen(date.weekday()));
                buf.push_str(", ");
                buf.push_str(&date.day().to_string());
                buf.push(' ');
                buf.push_str(render_month(date.month()));
            }
        }
    }
    buf.push_str(":\n\n");
}

fn render_week(_: i8, week: &Week, schedule_type: &ScheduleType, buf: &mut String) {
    if let n @ 0..=17 = week.week_of_semester {
        write!(buf, "Расписание на {n} учебную неделю\n\n").unwrap();
    } else {
        buf.push_str("Расписание на неделю\n\n")
    }

    if week.days.is_empty() {
        buf.push_str("Нет пар 🤷");
        return;
    }

    for (i, day) in week.days.iter().enumerate() {
        if i > 0 {
            buf.push_str("\n\n");
        }
        render_day(0, day, schedule_type, buf, true);
    }
}

fn render_day(
    day_offset: i8,
    day: &Day,
    schedule_type: &ScheduleType,
    buf: &mut String,
    inside_week: bool,
) {
    if !inside_week {
        buf.push_str("Расписание ");
    }

    if day_offset == 0 && !inside_week {
        buf.push_str("сегодня\n\n")
    } else {
        if inside_week {
            buf.push_str("📅 ");
            buf.push_str(render_day_of_week(day.date.weekday()));
        } else {
            buf.push_str(render_day_of_week_gen(day.date.weekday()));
        }
        buf.push_str(", ");
        buf.push_str(&day.date.day().to_string());
        buf.push(' ');
        buf.push_str(render_month(day.date.month()));
        buf.push_str("\n\n");
    };

    if !day.classes.is_empty() {
        for (i, cls) in day.classes.iter().enumerate() {
            if i > 0 {
                buf.push_str("\n\n");
            }
            render_classes(cls, schedule_type, buf);
        }
    } else {
        buf.push_str("Нет пар 🤷")
    };
}

fn render_classes(cls: &Classes, schedule_type: &ScheduleType, buf: &mut String) {
    buf.push_str(render_emoji_number(cls.number));
    buf.push(' ');
    buf.push_str(&cls.name);
    if !cls.raw_type.is_empty() {
        buf.push_str(" (");
        buf.push_str(&cls.raw_type);
        buf.push_str(")\n");
    }
    match (schedule_type, cls.groups.is_empty(), cls.person.is_empty()) {
        (ScheduleType::Person, false, _) => {
            buf.push_str("🎓 ");
            buf.push_str(&cls.groups);
            buf.push('\n');
        }
        (_, _, false) => {
            buf.push_str("👨‍🏫 ");
            buf.push_str(&cls.person);
            buf.push('\n');
        }
        _ => (),
    };
    if !cls.place.is_empty() {
        buf.push_str("🚪 ");
        buf.push_str(&cls.place);
        buf.push('\n');
    }
    buf.push_str("🕖 С ");
    buf.push_str(&cls.time.start.format("%H:%M").to_string());
    buf.push_str(" до ");
    buf.push_str(&cls.time.end.format("%H:%M").to_string());
}

#[inline]
fn render_emoji_number<'a>(num: i8) -> &'a str {
    match num {
        1 => "1️⃣",
        2 => "2️⃣",
        3 => "3️⃣",
        4 => "4️⃣",
        5 => "5️⃣",
        6 => "6️⃣",
        7 => "7️⃣",
        8 => "8️⃣",
        9 => "9️⃣",
        _ => "🟢",
    }
}

#[inline]
fn render_day_of_week<'a>(weekday: Weekday) -> &'a str {
    match weekday.number_from_monday() {
        1 => "понедельник",
        2 => "вторник",
        3 => "среда",
        4 => "четверг",
        5 => "пятница",
        6 => "суббота",
        7 => "воскресенье",
        _ => unreachable!(),
    }
}

#[inline]
fn render_day_of_week_gen<'a>(weekday: Weekday) -> &'a str {
    match weekday.number_from_monday() {
        1 => "в понедельник",
        2 => "во вторник",
        3 => "в среду",
        4 => "в четверг",
        5 => "в пятницу",
        6 => "в субботу",
        7 => "в воскресенье",
        _ => unreachable!(),
    }
}

#[inline]
fn render_month<'a>(month: u32) -> &'a str {
    match month {
        1 => "января",
        2 => "февраля",
        3 => "марта",
        4 => "апреля",
        5 => "мая",
        6 => "июня",
        7 => "июля",
        8 => "августа",
        9 => "сентября",
        10 => "октября",
        11 => "ноября",
        12 => "декабря",
        _ => "",
    }
}

fn render_duration(duration: &chrono::Duration, buf: &mut String) {
    let h = duration.num_hours();
    let m = duration.num_minutes() % 60;
    match (h, m) {
        (h, m) if h > 0 && m > 0 => {
            render_hours(h as i8, buf);
            buf.push(' ');
            render_minutes(m as i8, buf);
        }
        (h, 0) if h > 0 => render_hours(h as i8, buf),
        (0, m) if m > 0 => render_minutes(m as i8, buf),
        _ => (),
    }
}

fn render_minutes(m: i8, buf: &mut String) {
    if let m @ 11..=19 = m {
        write!(buf, "{m} минут").unwrap();
        return;
    }
    match m % 10 {
        1 => write!(buf, "{m} минуту"),
        2..=4 => write!(buf, "{m} минуты"),
        _ => write!(buf, "{m} минут"),
    }
    .unwrap()
}

fn render_hours(h: i8, buf: &mut String) {
    if let h @ 11..=19 = h {
        write!(buf, "{h} часов").unwrap();
        return;
    }
    match h % 10 {
        1 => write!(buf, "{h} час"),
        2..=4 => write!(buf, "{h} часа"),
        _ => write!(buf, "{h} часов"),
    }
    .unwrap()
}
