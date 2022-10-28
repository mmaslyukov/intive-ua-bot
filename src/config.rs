use std::fmt::{Debug, Display};
use std::time::Duration;
use std::{env, str::FromStr};

pub struct Config;
pub struct Timeout {
    sec: u64,
}
impl Timeout {
    pub fn new(sec: u64) -> Timeout {
        Timeout { sec }
    }
    pub fn seconds(&self) -> u64 {
        self.sec
    }
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.sec)
    }
}

impl Config {
    pub fn telegram_base_url() -> String {
        Self::read_var_with_default("TELEGRAM_BASE_URL", "https://api.telegram.org/bot")
    }

    pub fn telegram_bot_token() -> String {
        Self::read_var("TELEGRAM_BOT_TOKEN")
    }

    pub fn telegram_pool_thread_number() -> u64 {
        Self::read_var_with_default("TELEGRAM_POOL_THREAD_NUMBER", 3)
    }

    pub fn request_timeout_in_seconds() -> Timeout {
        Timeout::new(Self::read_var_with_default("REQUEST_TIMEOUT", 5))
    }

    pub fn owner_telegram_id() -> Option<i64> {
        panic!("Not implemented");
    }

    fn read_var_with_default<T, V>(name: &str, default_value: V) -> T
    where
        T: FromStr + Debug,
        V: Display,
    {
        let value = env::var(name).unwrap_or_else(|_| default_value.to_string());

        value
            .parse()
            .unwrap_or_else(|_| panic!("{} can not be parsed", name))
    }

    fn read_var<T>(name: &str) -> T
    where
        T: FromStr + Debug,
    {
        let value = env::var(name).unwrap_or_else(|_| panic!("{} must be set", name));

        value
            .parse()
            .unwrap_or_else(|_| panic!("{} can not be parsed", name))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_read_var_with_default() {
        assert_eq!(
            super::Config::read_var_with_default::<i32, _>("DUMMY", 2),
            2
        );
        assert_eq!(
            super::Config::read_var_with_default::<String, _>("DUMMY", "Hi"),
            "Hi"
        );
    }
}
