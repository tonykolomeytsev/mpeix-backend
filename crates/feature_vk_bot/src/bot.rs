use std::{env, sync::Arc};

use anyhow::{anyhow, bail, ensure, Context};
use common_errors::errors::CommonError;
use domain_bot::{
    models::Reply, peer::repository::PlatformId, renderer::RenderTargetPlatform,
    usecases::GenerateReplyUseCase,
};
use domain_vk_bot::{
    usecases::ReplyToVkUseCase, ButtonActionType, Keyboard, KeyboardButton, KeyboardButtonAction,
    MessagePeerType, NewMessageObject, VkCallbackRequest, VkCallbackType,
};
use log::error;
use once_cell::sync::Lazy;

pub struct FeatureVkBot {
    pub(crate) config: Config,
    pub(crate) generate_reply_use_case: Arc<GenerateReplyUseCase>,
    pub(crate) reply_to_vk_use_case: Arc<ReplyToVkUseCase>,
}

pub(crate) struct Config {
    confirmation_code: String,
    secret: Option<String>,
    group_id: Option<i64>,
}

impl Default for Config {
    fn default() -> Self {
        let confirmation_code = env::var("VK_BOT_CONFIRMATION_CODE")
            .expect("Environment variable VK_BOT_CONFIRMATION_CODE not provided");
        let secret = env::var("VK_BOT_SECRET").ok();
        let group_id = env::var("VK_BOT_GROUP_ID")
            .ok()
            .and_then(|it| it.parse::<i64>().ok());

        Self {
            confirmation_code,
            secret,
            group_id,
        }
    }
}

macro_rules! button {
    ($label:expr, $color:expr $(,)?) => {
        KeyboardButton {
            action: KeyboardButtonAction {
                r#type: ButtonActionType::Text,
                label: $label.to_owned(),
                payload: Some("{}".to_owned()),
            },
            color: $color,
        }
    };
}

static KEYBOARD_INLINE_HELP: Lazy<Keyboard> = Lazy::new(|| Keyboard {
    buttons: vec![vec![button!("Помощь", Some("primary".to_owned()))]],
    inline: true,
    one_time: false,
});
static KEYBOARD_DEFAULT: Lazy<Keyboard> = Lazy::new(|| Keyboard {
    buttons: vec![
        vec![button!("Ближайшие пары", Some("primary".to_owned()))],
        vec![button!("Пары сегодня", None), button!("Пары завтра", None)],
        vec![button!("Помощь", None), button!("Сменить расписание", None)],
    ],
    inline: false,
    one_time: false,
});

impl FeatureVkBot {
    pub async fn reply(&self, callback: VkCallbackRequest) -> anyhow::Result<Option<String>> {
        ensure!(
            callback.secret == self.config.secret,
            CommonError::user("Request has invalid secret key")
        );
        if let Some(group_id) = self.config.group_id {
            ensure!(
                callback.group_id == group_id,
                CommonError::user(
                    "Field 'group_id' of the request does not match the one specified in the env"
                )
            )
        }

        match callback.r#type {
            VkCallbackType::Confirmation => Ok(Some(self.config.confirmation_code.to_owned())),
            VkCallbackType::MessageNew => {
                if let Some(NewMessageObject {
                    message,
                    client_info: _,
                }) = callback.object
                {
                    let reply = if let Some(text) = &message.text {
                        self.generate_reply_use_case
                            .generate_reply(PlatformId::Vk(message.peer_id), text)
                            .await
                            .unwrap_or_else(|e| {
                                error!("{e}");
                                Reply::InternalError
                            })
                    } else {
                        Reply::UnknownMessageType
                    };

                    let text =
                        domain_bot::renderer::render_message(&reply, RenderTargetPlatform::Vk);
                    let keyboard = self.render_keyboard(&reply, &message.peer_type());
                    self.reply_to_vk_use_case
                        .reply(&text, message.peer_id, keyboard)
                        .await
                        .with_context(|| "Error while sending reply to vk")?;

                    Ok(None)
                } else {
                    bail!(CommonError::internal(
                        "Callback with type 'message' has no field 'object'"
                    ))
                }
            }
            VkCallbackType::Unknown => {
                Err(anyhow!(CommonError::internal("Unsupported callback type")))
            }
        }
    }

    fn render_keyboard(&self, reply: &Reply, peer_type: &MessagePeerType) -> Option<Keyboard> {
        match (reply, peer_type) {
            (Reply::UnknownMessageType | Reply::UnknownCommand, _) => {
                Some(KEYBOARD_INLINE_HELP.to_owned())
            }
            (
                Reply::ScheduleSearchResults {
                    schedule_name: _,
                    results,
                    results_contains_person,
                },
                _,
            ) => Some(self.render_search_results_keyboard(results, *results_contains_person)),
            (_, MessagePeerType::GroupChat) => None,
            _ => Some(KEYBOARD_DEFAULT.to_owned()),
        }
    }

    fn render_search_results_keyboard(
        &self,
        results: &[String],
        results_contains_person: bool,
    ) -> Keyboard {
        if results_contains_person {
            return Keyboard {
                buttons: results.iter().map(|it| vec![button!(it, None)]).collect(),
                inline: true,
                one_time: false,
            };
        }

        let mut buttons: Vec<Vec<KeyboardButton>> = vec![];
        let mut iter = results.iter();
        let mut i = 0;

        while i < results.len() - 1 {
            if let (Some(btn1), Some(btn2)) = (iter.next(), iter.next()) {
                buttons.push(vec![button!(btn1, None), button!(btn2, None)]);
            }
            i += 2;
        }
        if let Some(btn) = iter.next() {
            buttons.push(vec![button!(btn, None)]);
        }
        Keyboard {
            buttons,
            inline: true,
            one_time: false,
        }
    }
}
