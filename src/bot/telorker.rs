use std::sync::Arc;

use frankenstein::{Update, UpdateContent};

use crate::config::Config;

use super::{telapi::Telapi, telecom::Telecom};

pub struct Telorker {
    api: Arc<Telapi>,
    update: Update,
}

impl Telorker {
    pub fn new(api: Arc<Telapi>, update: Update) -> Telorker {
        Telorker { api, update }
    }

    pub fn run(&self) {
        let message = match &self.update.content {
            UpdateContent::Message(message) => message,
            UpdateContent::ChannelPost(channel_post) => channel_post,
            _ => return,
        };

        if let Some(owner_id) = Config::owner_telegram_id() {
            if message.from.is_none() {
                return;
            }

            if message.from.as_ref().unwrap().id as i64 != owner_id {
                return;
            }
        }
        if let Some(h) = Telecom::telcom().make_cmd_handler(message.clone(), self.api.clone()) {
            h.execute();
        }
    }
}
