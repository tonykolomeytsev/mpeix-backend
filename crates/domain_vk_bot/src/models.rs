use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct VkCallbackRequest {
    #[serde(default)]
    pub r#type: VkCallbackType,
    pub group_id: i32,
    pub event_id: i32,
    pub secret: Option<String>,
    pub object: Option<NewMessageObject>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VkCallbackType {
    Confirmation,
    NewMessage,
    Unknown,
}

impl Default for VkCallbackType {
    fn default() -> Self {
        VkCallbackType::Unknown
    }
}

#[derive(Debug, Deserialize)]
pub struct NewMessageObject {
    pub message: Message,
    pub client_info: ClientInfo,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    pub id: i32,
    pub date: u64,
    pub peer_id: i32,
    pub chat_id: i32,
    pub from_id: i32,
    pub text: Option<String>,
    pub payload: Option<String>,
}

#[derive(Debug, Clone)]
pub enum MessagePeerType {
    GroupChat,
    Community,
    User,
}

impl Message {
    pub fn peer_type(&self) -> MessagePeerType {
        if self.peer_id > 2000000000 {
            MessagePeerType::GroupChat
        } else if self.peer_id < 0 {
            MessagePeerType::Community
        } else {
            MessagePeerType::User
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ClientInfo {
    #[serde(default)]
    pub button_actions: Vec<ButtonActionType>,
    pub keyboard: bool,
    pub inline_keyboard: bool,
    pub carousel: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ButtonActionType {
    Text,
    #[serde(alias = "vkpay")]
    VkPay,
    OpenApp,
    Location,
    OpenLink,
    OpenPhoto,
    Callback,
    IntentSubscribe,
    IntentUnsubscribe,
    Unknown,
}

impl Default for ButtonActionType {
    fn default() -> Self {
        ButtonActionType::Unknown
    }
}
