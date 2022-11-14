use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread, time,
};

use frankenstein::{AllowedUpdate, GetUpdatesParams, TelegramApi, Update};

use super::telapi::Telapi;
use crate::{
    bot::{fsm, telapi, telorker::Telorker},
    config::Config,
    db::{survey_table, user_table},
    user_data::UserData,
};
use log::*;

pub struct Teladler {
    api: Arc<Telapi>,
    update_params: GetUpdatesParams,
    user_data: Arc<Mutex<UserData>>,
    buffer: VecDeque<Update>,
}

impl Teladler {
    pub fn new() -> Teladler {
        let api = Arc::new(telapi::api().clone());
        let user_data = Arc::new(Mutex::new(UserData::new()));
        let buffer = VecDeque::new();
        let update_params = GetUpdatesParams::builder()
            .allowed_updates(vec![
                AllowedUpdate::Message,
                AllowedUpdate::ChannelPost,
                AllowedUpdate::CallbackQuery,
            ])
            .build();

        Teladler {
            api,
            update_params,
            user_data,
            buffer,
        }
    }

    fn start_timer_thread(&self) {
        let api = self.api.clone();
        let user_data = self.user_data.clone();
        thread::spawn(move || loop {
            let chat_id_collection = user_data.lock().unwrap().find_expired();
            if  !chat_id_collection.is_empty() {
                Self::initiate_survey(api.clone(), user_data.clone(), chat_id_collection);
            }
            let chat_id_collection = user_data.lock().unwrap().find_with_issues();
            if  !chat_id_collection.is_empty() {
                Self::initiate_survey(api.clone(), user_data.clone(), chat_id_collection);
            }
            thread::sleep(time::Duration::from_secs(60));
        });
    }

    fn initiate_survey(
        api: Arc<Telapi>,
        user_data: Arc<Mutex<UserData>>,
        chat_id_collection: Vec<i64>,
    ) {
        for chat_id in chat_id_collection {
            if chat_id == 0 {
                continue;
            }
            let chat = frankenstein::Chat::builder()
                .id(chat_id)
                .type_field(frankenstein::ChatType::Private)
                .build();
            let msg = frankenstein::Message::builder()
                .message_id(-1)
                .date(0)
                .chat(chat)
                .text(format!(
                    "/{}",
                    fsm::Event::Survey.to_string().to_lowercase()
                ))
                .build();
            let update = Update {
                update_id: 0,
                content: frankenstein::UpdateContent::Message(msg),
            };
            Telorker::new(api.clone(), update, user_data.clone()).run();
        }
    }

    pub fn exec(&mut self) -> ! {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(Config::telegram_pool_thread_number() as usize)
            .build()
            .unwrap();
        let _ = user_table::User::new().create_table();
        let _ = survey_table::SurveyEntry::new().create_table();

        info!("Starting the Guardian bot");
        let interval = time::Duration::from_millis(100);

        self.start_timer_thread();
        loop {
            while let Some(update) = self.fetch() {
                let api = self.api.clone();
                let user_data = self.user_data.clone();
                thread_pool.spawn(move || {
                    Telorker::new(api, update, user_data).run();
                });
            }
            thread::sleep(interval);
        }
    }

    fn fetch(&mut self) -> Option<Update> {
        if let Some(update) = self.buffer.pop_front() {
            return Some(update);
        }

        match self.api.get_updates(&self.update_params) {
            Ok(updates) => {
                updates
                    .result
                    .into_iter()
                    .for_each(|u| self.buffer.push_back(u));

                if let Some(last_update) = self.buffer.back() {
                    self.update_params.offset = Some((last_update.update_id + 1).into());
                }

                self.buffer.pop_front()
            }

            Err(err) => {
                error!("Failed to fetch updates {:?}", err);
                None
            }
        }
    }
}
