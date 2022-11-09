use crate::config::Config;
use crate::http_client;
use frankenstein::ErrorResponse;
use frankenstein::InlineKeyboardButton;
use frankenstein::InlineKeyboardMarkup;
use frankenstein::KeyboardButton;
use frankenstein::ParseMode;
use frankenstein::ReplyKeyboardMarkup;
use frankenstein::ReplyMarkup;
use frankenstein::SendMessageParams;
use frankenstein::TelegramApi;
use isahc::prelude::*;
use isahc::Request;
use log::*;
use once_cell::sync::OnceCell;
use std::path::PathBuf;

use super::telecom::ReplyInline;

static API: OnceCell<Telapi> = OnceCell::new();

pub type Telerow = Vec<KeyboardButton>;
pub type Teleboard = Vec<Telerow>;

pub type TelerowInline = Vec<InlineKeyboardButton>;
pub type TeleboardInline = Vec<TelerowInline>;

#[derive(Clone, Debug)]
pub struct Telapi {
    pub api_url: String,
}

#[derive(Debug)]
pub enum Error {
    HttpError(HttpError),
    ApiError(ErrorResponse),
}

#[derive(Eq, PartialEq, Debug)]
pub struct HttpError {
    pub code: u16,
    pub message: String,
}

impl Default for Telapi {
    fn default() -> Self {
        Self::new()
    }
}

impl Telapi {
    pub fn new() -> Self {
        let token = Config::telegram_bot_token();
        let base_url = Config::telegram_base_url();
        let api_url = format!("{}{}", base_url, token);
        Self { api_url }
    }

    pub fn send_text_message(&self, chat_id: i64, message: String) -> Result<(), Error> {
        self.reply_with_text_message(chat_id, message, None)
    }

    pub fn reply_with_keyboard(&self, chat_id: i64, keyboard: Teleboard) -> Result<(), Error> {
        // let keyboard_markup = InlineKeyboardMarkup::builder().inline_keyboard(keyboard).build();
        let keyboard_markup = ReplyKeyboardMarkup::builder()
            .resize_keyboard(true)
            .keyboard(keyboard)
            .build();

        let send_message_params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text("hello!")
            // .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard_markup))
            .reply_markup(ReplyMarkup::ReplyKeyboardMarkup(keyboard_markup))
            .build();

        match self.send_message(&send_message_params) {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(
                    "Failed to send message {:?}: {:?}",
                    err, send_message_params
                );
                Err(err)
            }
        }
    }

    pub fn reply_with_keyboard_inline(
        &self,
        chat_id: i64,
        reply: ReplyInline,
    ) -> Result<(), Error> {
        let keyboard_markup = InlineKeyboardMarkup::builder()
            .inline_keyboard(reply.keyboard)
            .build();

        let send_message_params = SendMessageParams::builder()
            .chat_id(chat_id)
            .text(reply.text)
            .reply_markup(ReplyMarkup::InlineKeyboardMarkup(keyboard_markup))
            .build();

        match self.send_message(&send_message_params) {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(
                    "Failed to send message {:?}: {:?}",
                    err, send_message_params
                );
                Err(err)
            }
        }
    }

    pub fn reply_with_text_message(
        &self,
        chat_id: i64,
        message: String,
        message_id: Option<i32>,
    ) -> Result<(), Error> {
        let send_message_params = match message_id {
            None => SendMessageParams::builder()
                .chat_id(chat_id)
                .text(message)
                .parse_mode(ParseMode::Html)
                .build(),

            Some(message_id_value) => SendMessageParams::builder()
                .chat_id(chat_id)
                .text(message)
                .parse_mode(ParseMode::Html)
                .reply_to_message_id(message_id_value)
                .build(),
        };

        match self.send_message(&send_message_params) {
            Ok(_) => Ok(()),
            Err(err) => {
                error!(
                    "Failed to send message {:?}: {:?}",
                    err, send_message_params
                );
                Err(err)
            }
        }
    }
}

impl TelegramApi for Telapi {
    type Error = Error;

    fn request<T1: serde::ser::Serialize, T2: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: Option<T1>,
    ) -> Result<T2, Error> {
        let url = format!("{}/{}", self.api_url, method);

        let request_builder = Request::post(url).header("Content-Type", "application/json");

        let mut response = match params {
            None => {
                let request = request_builder.body(())?;
                http_client::client().send(request)?
            }
            Some(data) => {
                let json = serde_json::to_string(&data).unwrap();
                let request = request_builder.body(json)?;
                http_client::client().send(request)?
            }
        };

        let mut bytes = Vec::new();
        response.copy_to(&mut bytes)?;

        let parsed_result: Result<T2, serde_json::Error> = serde_json::from_slice(&bytes);

        match parsed_result {
            Ok(result) => Ok(result),
            Err(_) => {
                let parsed_error: Result<ErrorResponse, serde_json::Error> =
                    serde_json::from_slice(&bytes);

                match parsed_error {
                    Ok(result) => Err(Error::ApiError(result)),
                    Err(error) => {
                        let message = format!("{:?} {:?}", error, std::str::from_utf8(&bytes));

                        let error = HttpError { code: 500, message };

                        Err(Error::HttpError(error))
                    }
                }
            }
        }
    }

    // isahc doesn't support multipart uploads
    // https://github.com/sagebind/isahc/issues/14
    // but it's fine because this bot doesn't need this feature
    fn request_with_form_data<T1: serde::ser::Serialize, T2: serde::de::DeserializeOwned>(
        &self,
        _method: &str,
        _params: T1,
        _files: Vec<(&str, PathBuf)>,
    ) -> Result<T2, Error> {
        let error = HttpError {
            code: 500,
            message: "isahc doesn't support form data requests".to_string(),
        };

        Err(Error::HttpError(error))
    }
}

pub fn api() -> &'static Telapi {
    API.get_or_init(|| Telapi::new())
}

impl From<isahc::http::Error> for Error {
    fn from(error: isahc::http::Error) -> Self {
        let message = format!("{:?}", error);

        let error = HttpError { code: 500, message };

        Error::HttpError(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        let message = format!("{:?}", error);

        let error = HttpError { code: 500, message };

        Error::HttpError(error)
    }
}

impl From<isahc::Error> for Error {
    fn from(error: isahc::Error) -> Self {
        let message = format!("{:?}", error);

        let error = HttpError { code: 500, message };

        Error::HttpError(error)
    }
}
