use frankenstein::{CallbackQuery, Message};
use std::str::FromStr;
use std::sync::Mutex;
use std::{fmt, sync::Arc};

use crate::bot::telapi::Telapi;
use crate::user_data::UserData;
use typed_builder::TypedBuilder as Builder;

use super::telapi::{Teleboard, TeleboardInline};

pub enum Update {
    Message(Message),
    CallbackQuery(CallbackQuery),
    None,
}

#[derive(Clone, Debug)]
pub enum ReplyEnum {
    Text(String),
    KeyboardMenu(ReplyMenu),
    KeyboardInline(ReplyInline),
    None,
}

#[derive(Clone, Debug)]
pub struct ReplyInline {
    pub text: String,
    pub keyboard: TeleboardInline,
}
impl ReplyInline {
    pub fn new(text: &str, keyboard: TeleboardInline) -> Self {
        Self {
            text: text.to_string(),
            keyboard,
        }
    }
}
#[derive(Clone, Debug)]
pub struct ReplyMenu {
    pub text: String,
    pub keyboard: Teleboard,
}
impl ReplyMenu {
    pub fn new(text: &str, keyboard: Teleboard) -> Self {
        Self {
            text: text.to_string(),
            keyboard,
        }
    }
}

#[derive(Builder, Debug, Default)]
pub struct User {
    pub first_name: String,
    #[builder(setter(into, strip_option), default)]
    pub last_name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub username: Option<String>,
}
#[derive(Builder, Debug)]
pub struct UserInput {
    pub chat_id: i64,
    pub text: String,
    pub date: u64,
    // pub user: User,
}

impl fmt::Display for UserInput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl UserInput {
    pub fn from_msg(msg: &Message) -> Self {
        Self::builder()
            .chat_id(msg.chat.id)
            .text(msg.text.clone().unwrap())
            .date(msg.date)
            .build()
    }
    pub fn from_query(query: &CallbackQuery) -> Self {
        Self::builder()
            .chat_id(query.message.clone().unwrap().chat.id)
            .text(query.data.clone().unwrap())
            .date(query.message.clone().unwrap().date)
            // .user(User::default())
            .build()
    }
}

pub struct Telecom {}

impl Telecom {
    fn handle_user_input(api: Arc<Telapi>, user_data: Arc<Mutex<UserData>>, user_input: UserInput) {
        log::debug!("Got input: {}", user_input);
        let result = user_data.lock().unwrap().handle_incoming_v2(&user_input);
        if result.is_ok() {
            Self::reply(api, &user_input, result.unwrap())
        } else {
            log::error!(
                "Error: `{}",
                result
                    .err()
                    .unwrap()
                    .msg()
                    .unwrap_or_else(|| String::from_str("unknonw").unwrap())
            );
        }
    }

    pub fn handle(api: Arc<Telapi>, update: Update, user_data: Arc<Mutex<UserData>>) {
        match update {
            Update::Message(msg) => {
                Self::handle_user_input(api, user_data, UserInput::from_msg(&msg))
            }
            Update::CallbackQuery(query) => {
                Self::handle_user_input(api, user_data, UserInput::from_query(&query))
            }
            _ => {}
        }
    }
    pub fn reply(api: Arc<Telapi>, user_input: &UserInput, reply: ReplyEnum) {
        match reply {
            ReplyEnum::Text(txt) => {
                api.reply_with_text_message(user_input.chat_id, txt, None)
                    .map_err(|e| log::error!("error: {:?}", e))
                    .unwrap();
            }
            ReplyEnum::KeyboardMenu(menu) => {
                api.reply_with_keyboard(user_input.chat_id, menu)
                    .map_err(|e| log::error!("error: {:?}", e))
                    .unwrap();
            }
            ReplyEnum::KeyboardInline(kbrd) => {
                api.reply_with_keyboard_inline(user_input.chat_id, kbrd)
                    .map_err(|e| log::error!("error: {:?}", e))
                    .unwrap();
            }
            ReplyEnum::None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::{BotQuery, Command, Telecom};

    // #[test]
    // pub fn test_bot_query_enum() {
    //     assert_eq!(
    //         BotQuery::Register,
    //         BotQuery::from_string_begins("register_new_user").unwrap()
    //     );
    //     assert_eq!(None, BotQuery::from_string_begins("/register_new_user"));
    // }
}
