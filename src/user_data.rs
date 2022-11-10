use chrono::Utc;
use typed_builder::TypedBuilder as Builder;

use std::collections::HashMap;

use crate::{
    bot::{
        fsm::{self, Data, State},
        telecom::{ReplyEnum, UserInput},
        Error,
    },
    db::report,
};

// static USER_DATA: OnceCell<Box<Arc<UserData>>> = OnceCell::new();

#[derive(Builder)]
pub struct UserData {
    user_data_table: HashMap<i64, fsm::State>,
}

impl UserData {
    pub fn new() -> Self {
        let hash_map: HashMap<i64, fsm::State> = HashMap::new();
        Self::builder().user_data_table(hash_map).build().init()
    }

    pub fn init(mut self) -> Self {
        if let Ok(report) = report::Report::new().select_all() {
            for r in report {
                self.user_data_table
                    .insert(r.chat_id.value, Data::from(r).wrap(fsm::State::Idle));
            }
        }
        self
    }

    pub fn collect_chat_ids(&self) -> Vec<i64> {
        self.user_data_table.iter().map(|i| *i.0).collect()
    }

    pub fn find_expired(&self) -> Vec<i64> {
        let mut chat_id_collection = Vec::new();
        for u in &self.user_data_table {
            if Utc::now().signed_duration_since(u.1.data().utc) > chrono::Duration::hours(24) {
                chat_id_collection.push(u.1.data().chat_id);
            }
        }
        chat_id_collection
    }

    pub fn handle_incoming_v2(&mut self, user_input: &UserInput) -> Result<ReplyEnum, Error> {
        let mut state = self
            .user_data_table
            .entry(user_input.chat_id)
            .or_insert_with_key(|key| {
                Data::builder()
                    .chat_id(key.to_owned())
                    .build()
                    .wrap(State::New)
            })
            .to_owned();

        state = state.consume_as_str(&user_input.text).transit();
        let reply = state.reply();
        self.user_data_table
            .insert(user_input.chat_id, state.clone());
        reply.ok_or(Error::make_verbose(&format!(
            "Skipped input: {}",
            user_input.text
        )))
    }
}
