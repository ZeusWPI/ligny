use std::{env, net::IpAddr, path::PathBuf, sync::OnceLock};

use dotenvy::dotenv;

static CONFIG: OnceLock<Config> = OnceLock::new();

pub struct Config {
    pub public: String,
    pub content: PathBuf,
    pub static_dir: PathBuf,
    pub index_name: String,
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
                content: PathBuf::from(env::var("CONTENT").unwrap_or("0_content".into())),
                static_dir: PathBuf::from(env::var("STATIC").unwrap_or("static".into())),
                index_name: env::var("INDEX").unwrap_or("index.json".into()),
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
