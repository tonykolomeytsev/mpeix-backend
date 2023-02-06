use domain_schedule_models::dto::v1::{Day, Schedule, ScheduleType};

/// Representation of database row from table 'peer'
pub struct Peer {
    pub id: i64,
    pub selected_schedule: String,
    pub selected_schedule_type: ScheduleType,
    pub selecting_schedule: bool,
}

impl Peer {
    pub fn is_not_started(&self) -> bool {
        self.selected_schedule.is_empty() && !self.selecting_schedule
    }
}

/// Input actions for the bot
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
    AlreadyStarted { schedule_name: String },
    Week(Schedule),
    Day(Option<Day>),
    ScheduleChangedSuccessfully(String),
    ScheduleSearchResults(Vec<String>),
    CannotFindSchedule,
    ReadyToChangeSchedule,
    ShowHelp,
}
