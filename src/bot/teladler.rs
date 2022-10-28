use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use frankenstein::{AllowedUpdate, GetUpdatesParams, TelegramApi, Update};

use crate::{
    bot::{telapi, telorker::Telorker},
    config::Config,
};

use super::telapi::Telapi;

pub struct Teladler {
    api: Arc<Telapi>,
    // api : Arc<Mutex<Telapi>>,
    update_params: GetUpdatesParams,

    buffer: VecDeque<Update>,
}

impl Teladler {
    pub fn new() -> Teladler {
        let api = Arc::new(telapi::api().clone());
        // let api = Arc::new(Mutex::new(telegram_client_api::api().clone()));
        let buffer = VecDeque::new();
        let update_params = GetUpdatesParams::builder()
            .allowed_updates(vec![AllowedUpdate::Message, AllowedUpdate::ChannelPost])
            .build();

        Teladler {
            api,
            update_params,
            buffer,
        }
    }

    pub fn exec(&mut self) -> ! {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(Config::telegram_pool_thread_number() as usize)
            .build()
            .unwrap();

        log::info!("Starting the Guardian bot");
        let interval = Duration::from_secs(1);

        loop {
            while let Some(update) = self.fetch() {
                let api = self.api.clone();
                thread_pool.spawn(move || {
                    Telorker::new(api, update).run();
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
                log::error!("Failed to fetch updates {:?}", err);
                None
            }
        }
    }
}
