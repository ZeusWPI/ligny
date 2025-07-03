use std::{env, net::IpAddr, sync::OnceLock};

use dotenvy::dotenv;

static CONFIG: OnceLock<Config> = OnceLock::new();

pub struct Config {
    pub public: String,
    pub content: String,
    pub port: u16,
    pub address: IpAddr,
}

impl Config {
    pub fn initialize() {
        assert!(CONFIG.get().is_none());

        Config::get();
    }

    pub fn get() -> &'static Config {
        CONFIG.get_or_init(|| {
            dotenv().ok();
            Config {
                public: env::var("PUBLIC").unwrap_or("public".into()),
                content: env::var("CONTENT").unwrap_or("0_content".into()),
                port: env::var("PORT")
                    .map(|v| v.parse::<u16>().expect("PORT is invalid"))
                    .unwrap_or(8000),
                address: env::var("ADDRESS")
                    .unwrap_or(String::from("127.0.0.1"))
                    .parse()
                    .expect("ADDRESS is invalid"),
            }
        })
    }
}
