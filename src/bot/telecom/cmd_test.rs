
use std::{sync::Arc, io::Error};

use frankenstein::Message;
use crate::bot::telapi::{Telapi};

use super::{Command, Telecom};


pub trait Exec {
    fn exec(self) -> Result<Box<dyn Reply>, Error>;

}
pub trait Msg {
    fn message(self, msg: Message) -> Box<dyn Exec>;
}

pub trait Api {
    fn api(self, api: Arc<Telapi>) -> Box<dyn Msg>;
}

pub trait Reply {
    fn reply(self) -> Result<(), Error>;
}




pub struct Dummy<T:Command> {
    obj: T,
}

impl<T:Command> Dummy<T> {
    fn execute(&self) {
        self.obj.execute();
    }
}




pub struct Test1 {
    api: Option<Arc<Telapi>>,
    msg: Option<Message>,
    reply: Option<String>
}
impl Command for Test1 {
    fn execute(&self) -> Result<(), Error> {
        Ok(())
    }
}

impl Test1 {
    pub fn start() -> Box<dyn Api> {
        Box::new(Test1{api: None, msg: None, reply:None})
    }
}


impl Api for Test1 {
    fn api(mut self, api: Arc<Telapi>) -> Box<dyn Msg> {
        self.api = Some(api);
        Box::new(self)
    }
}

impl Msg for Test1 {
    fn message(mut self, msg: Message) -> Box<dyn Exec> {
        self.msg = Some(msg);
        Box::new(self)
    }
}

impl Exec for Test1 {
    fn exec(mut self) -> Result<Box<dyn Reply>, Error> {
        self.reply = Some("Hi".to_string());
        Ok(Box::new(self))
    }
}

impl Reply for Test1 {
    fn reply(mut self) -> Result<(), Error> {
        Ok(())
    }
}

// pub struct Test2;
// impl Command for Test2 {
//     fn execute(&self) -> Result<(), Error> {
//         Ok(())
//     }
// }

// pub struct Handler;
// impl Handler {
//     pub fn make(name: &str) -> Dummy<Command> {

//     }
// }

struct Material<T: Language> {
    sentense: String,
    word: String,
    language: T,
}

impl<T: Language> Material<T> {
    fn say(&self) {
        self.language.translate(&self.sentense);
        self.language.translate(&self.word);
    }
}

struct English;

trait Language {
    fn translate(&self, original: &str) -> String;
}

impl Language for English {
    fn translate(&self, original: &str) -> String {
        original.into()
    }
}




#[cfg(test)]
mod tests {
    use super::Telecom;

    #[test]
    fn test_test() {
        super::Test1::start().api(api).message(msg).exec().unwrap().reply();
    }

}
