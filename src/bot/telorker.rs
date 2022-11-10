use std::sync::{Arc, Mutex};

use frankenstein::{Update, UpdateContent};

use crate::user_data::UserData;

use super::{
    telapi::Telapi,
    telecom::{self, Telecom},
};

pub struct Telorker {
    api: Arc<Telapi>,
    update: Update,
    user_data: Arc<Mutex<UserData>>,
}

impl Telorker {
    pub fn new(api: Arc<Telapi>, update: Update, user_data: Arc<Mutex<UserData>>) -> Telorker {
        Telorker {
            api: api,
            update,
            user_data,
        }
    }

    pub fn run(&self) {
        let update = match &self.update.content {
            UpdateContent::Message(message) => telecom::Update::Message(message.clone()),
            UpdateContent::ChannelPost(channel_post) => {
                telecom::Update::Message(channel_post.clone())
            }
            UpdateContent::CallbackQuery(callback_query) => {
                telecom::Update::CallbackQuery(callback_query.clone())
            }
            _ => return,
        };

        Telecom::handle(self.api.clone(), update, self.user_data.clone());
    }
}
