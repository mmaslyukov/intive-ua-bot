use core::fmt;
use std::str::FromStr;

pub mod fsm;
pub mod telandler;
pub mod telapi;
pub mod telecom;
pub mod telorker;
pub mod utils;

#[derive(Debug)]
pub enum Error {
    Default,
    Verbose(String),
}

impl Error {
    pub fn make_verbose(text: &str) -> Self {
        Self::Verbose(String::from_str(text).unwrap())
    }
    pub fn wrap<T>(self) -> Result<T, Error> {
        Err(self)
    }
    pub fn msg(&self) -> Option<String> {
        match self {
            Error::Verbose(text) => Some(text.clone()),
            _ => None,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl From<Result<telecom::ReplyEnum, Error>> for Error {
    fn from(result: Result<telecom::ReplyEnum, Error>) -> Self {
        match result {
            Ok(_) => panic!("Shall not be the case"),
            Err(err) => err,
        }
    }
}
