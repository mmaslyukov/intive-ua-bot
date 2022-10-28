use crate::config::Config;
use isahc::config::RedirectPolicy;
use isahc::prelude::*;
use isahc::HttpClient;
use once_cell::sync::OnceCell;

pub fn client() -> &'static HttpClient {
    static CLIENT: OnceCell<HttpClient> = OnceCell::new();
    CLIENT.get_or_init(|| {
        HttpClient::builder()
            .redirect_policy(RedirectPolicy::Limit(10))
            .timeout(Config::request_timeout_in_seconds().duration())
            .build()
            .unwrap()
    })
}
