use serde::{Deserialize, Serialize};

/// https://core.telegram.org/bots/api/#update
#[derive(Debug, Deserialize)]
pub struct Update {
    pub update_id: i32,
    pub message: Option<Message>,
    pub callback_query: Option<CallbackQuery>,
}

/// https://core.telegram.org/bots/api/#message
#[derive(Debug, Deserialize)]
pub struct Message {
    pub message_id: i64,
    pub from: Option<User>,
    pub chat: Chat,
    pub text: Option<String>,
}

/// https://core.telegram.org/bots/api/#callbackquery
#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    pub message: Option<Message>,
    pub data: Option<String>,
}

/// https://core.telegram.org/bots/api/#user
#[derive(Debug, Deserialize)]
pub struct User {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
}

/// https://core.telegram.org/bots/api/#chat
#[derive(Debug, Deserialize)]
pub struct Chat {
    pub id: i64,
    #[serde(default)]
    pub r#type: ChatType,
    pub title: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatType {
    Private,
    Group,
    #[serde(alias = "supergroup")]
    SuperGroup,
    Channel,
    Unknown,
}

impl Default for ChatType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// https://core.telegram.org/bots/api/#inlinekeyboardmarkup
#[derive(Debug, Serialize, Clone)]
pub struct InlineKeyboardMarkup {
    pub inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

/// https://core.telegram.org/bots/api/#inlinekeyboardbutton
#[derive(Debug, Serialize, Clone)]
pub struct InlineKeyboardButton {
    pub text: String,
    pub callback_data: String,
}

/// https://core.telegram.org/bots/api/#replykeyboardmarkup
#[derive(Debug, Serialize, Clone)]
pub struct ReplyKeyboardMarkup {
    pub keyboard: Vec<Vec<KeyboardButton>>,
    pub one_time_keyboard: bool,
}

/// https://core.telegram.org/bots/api/#keyboardbutton
#[derive(Debug, Serialize, Clone)]
pub struct KeyboardButton {
    pub text: String,
}

/// https://core.telegram.org/bots/api/#replykeyboardremove
#[derive(Debug, Serialize, Clone)]
pub struct ReplyKeyboardRemove {
    pub remove_keyboard: bool,
}

#[derive(Debug, Serialize, Clone)]
pub enum CommonKeyboardMarkup {
    Inline(InlineKeyboardMarkup),
    Reply(ReplyKeyboardMarkup),
    Remove(ReplyKeyboardRemove),
}
