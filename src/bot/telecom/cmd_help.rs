use frankenstein::Message;
use once_cell::sync::OnceCell;
use std::{fmt, sync::Arc, io::Error};

use crate::bot::telapi::Telapi;

use super::{BotCmd, Command, Telecom};

static HELP: OnceCell<String> = OnceCell::new(); //format!("{}", Telecom::cmd_to_string(&BotCmd::Help)).as_str();

pub struct Help {
    api: Arc<Telapi>,
    msg: Message,
}
impl Help {
    pub fn make_command(api: Arc<Telapi>, msg: Message) -> Box<dyn Command> {
        Box::new(Help { api, msg })
    }
}

impl Command for Help {
    // fn message(&self) -> &Message {
    //     &self.msg
    // }
    // fn api(&self) -> Arc<Telapi> {
    //     self.api.clone()
    // }
    fn execute(&self) -> Result<(), Error> {
        let text = HELP.get_or_init(|| {
            format!(
                "{} - Prints this help\n\n",
                Telecom::cmd_to_string(&BotCmd::Help)
            ) + format!(
                "{} - Start Guardian bot\n\n",
                Telecom::cmd_to_string(&BotCmd::Start)
            )
            .as_str()
                + format!("DUMMY\n\n").as_str()
        });
        return Ok(())
    }

    // fn response(&self) -> String {
    //     "".to_string()
    // }
}
