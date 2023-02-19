use chrono::NaiveDate;
use domain_schedule_models::{Classes, Day, ScheduleType, Week};

/// Representation of database row from table 'peer'
pub struct Peer {
    pub id: i64,
    pub selected_schedule: String,
    pub selected_schedule_type: ScheduleType,
    pub selecting_schedule: bool,
}

/// Input actions for the bot
#[derive(Debug, PartialEq, Eq)]
pub enum UserAction {
    /// User just started communicating with the bot and sent the "Start" command
    Start,
    /// User requested the entire schedule for a certain week
    WeekWithOffset(i8),
    /// User requested the schedule for a certain day
    DayWithOffset(i8),
    /// User requested a schedule change
    ChangeScheduleIntent,
    /// User requested an upcoming events (like as mpeix dashboard page)
    UpcomingEvents,
    /// User requested help
    Help,
    /// Maybe user types new chedule to change... who knows?
    Unknown(String),
}

/// Rendered reply to answer
pub enum Reply {
    StartGreetings,
    AlreadyStarted {
        schedule_name: String,
    },
    Week {
        week_offset: i8,
        week: Week,
        schedule_type: ScheduleType,
    },
    Day {
        day_offset: i8,
        day: Day,
        schedule_type: ScheduleType,
    },
    UpcomingEvents {
        prediction: UpcomingEventsPrediction,
        schedule_type: ScheduleType,
    },
    ScheduleChangedSuccessfully(String),
    ScheduleSearchResults {
        schedule_name: String,
        results: Vec<String>,
        results_contains_person: bool,
    },
    CannotFindSchedule(String),
    ReadyToChangeSchedule,
    ShowHelp,
    UnknownCommand,
    /// Type for non-text messages
    UnknownMessageType,
    /// Type for default error message
    InternalError,
}

pub enum UpcomingEventsPrediction {
    NoClassesNextWeek,
    ClassesTodayNotStarted {
        time_prediction: TimePrediction,
        future_classes: Vec<Classes>,
    },
    ClassesTodayStarted {
        in_progress: Box<Classes>,
        future_classes: Option<Vec<Classes>>,
    },
    ClassesInNDays {
        time_prediction: TimePrediction,
        future_classes: Vec<Classes>,
    },
}

pub enum TimePrediction {
    WithinOneDay(chrono::Duration),
    WithinAWeek {
        date: NaiveDate,
        duration: chrono::Duration,
    },
}
