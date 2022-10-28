pub mod cmd_help;
pub mod cmd_test;

use frankenstein::Message;
use log::info;
use once_cell::sync::OnceCell;
use rayon::iter::Once;
use std::{
    collections::{hash_map, HashMap},
    fmt,
    sync::{Arc, Mutex}, io::Error,
};
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use self::cmd_help::Help;

use super::telapi::{Telapi};

#[derive(Debug, EnumIter, PartialEq, Hash)]
pub enum BotCmd {
    Help,
    Start,
}

impl fmt::Display for BotCmd {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
        // or, alternatively:
        // write!(f, "{:?}", self)
    }
}

pub trait Command {
    // fn response(&self) -> String;
    // fn message(&self) -> &Message;
    // fn api(&self) -> Arc<Telapi>;
    fn execute(&self) -> Result<(), Error>;

    fn reply(&self, text: String) {
        // let msg = self.message();
        // if let Err(error) =
        //     self.api()
        //         .reply_with_text_message(msg.chat.id, text, Some(msg.message_id))
        // {
        //     log::error!("Failed to reply to update {:?} {:?}", error, msg);
        // }
    }

    // fn _execute(&self, message: &Message) {
    //     info!(
    //         "{:?} wrote: {}",
    //         message.chat.id,
    //         message.text.as_ref().unwrap()
    //     );

    //     let text = self.response();
    //     // self.reply_to_message(message, text)
    // }

    // fn reply_to_message(&self, message: &Message, text: String) {
    //     if let Err(error) =
    //         self.api()
    //             .reply_with_text_message(message.chat.id, text, Some(message.message_id))
    //     {
    //         error!("Failed to reply to update {:?} {:?}", error, message);
    //     }
    // }
}

type TelHash = HashMap<String, fn(Arc<Telapi>, Message) -> Box<dyn Command>>;
static TELCOM: OnceCell<Telecom> = OnceCell::new();
pub struct Telecom {
    cmd_table: TelHash,
}

impl Telecom {
    pub fn telcom() -> &'static Self {
        TELCOM.get_or_init(|| Telecom::new())
    }

    fn new() -> Self {
        let mut cmd_table: TelHash = HashMap::new();

        cmd_table.insert(Self::cmd_to_string(&BotCmd::Help), Help::make_command);

        Telecom { cmd_table }
    }

    pub fn make_cmd_handler(&self, msg: Message, api: Arc<Telapi>) -> Option<Box<dyn Command>> {
        if let Some(cmd_name) = &msg.text {
            if let Some(ch) = self.cmd_table.get(cmd_name) {
                return Some(ch(api, msg));
            }
        }
        None
    }

    pub fn cmd_to_string(cmd: &BotCmd) -> String {
        format!("/{}", cmd.to_string()).to_lowercase()
    }

    pub fn cmd_from_string(name: &str) -> Option<BotCmd> {
        for cmd in BotCmd::iter() {
            if name == Self::cmd_to_string(&cmd) {
                return Some(cmd.into());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Telecom;

    #[test]
    fn test_cmd_to_string() {
        assert_eq!(super::Telecom::cmd_to_string(&super::BotCmd::Help), "/help")
    }

    #[test]
    fn test_cmd_from_string() {
        assert_eq!(
            super::Telecom::cmd_from_string("/help").unwrap(),
            super::BotCmd::Help
        );
    }
}
