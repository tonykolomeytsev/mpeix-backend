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
    pub message_id: i32,
    pub from: Option<User>,
    pub chat: Chat,
    pub text: Option<String>,
}

/// https://core.telegram.org/bots/api/#callbackquery
#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub id: String,
    pub message: Message,
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
#[derive(Debug, Serialize)]
pub struct InlineKeyboardMarkup {
    pub inline_keyboard: Vec<Vec<InlineKeyboardButton>>,
}

/// https://core.telegram.org/bots/api/#inlinekeyboardbutton
#[derive(Debug, Serialize)]
pub struct InlineKeyboardButton {
    pub text: String,
}

/// https://core.telegram.org/bots/api/#replykeyboardmarkup
#[derive(Debug, Serialize)]
pub struct ReplyKeyboardMarkup {
    pub keyboard: Vec<Vec<KeyboardButton>>,
    pub one_time_keyboard: bool,
    /// Placeholder length should be from 1 to 64 chars
    pub input_field_placeholder: Option<String>,
}

/// https://core.telegram.org/bots/api/#keyboardbutton
#[derive(Debug, Serialize)]
pub struct KeyboardButton {
    pub text: String,
}

/// https://core.telegram.org/bots/api/#replykeyboardremove
pub struct ReplyKeyboardRemove {
    pub remove_keyboard: bool,
}
