use serde_derive::{Deserialize, Serialize};
use dotenvy::dotenv;
use std::env;
use crate::utils::constants::{DEFAULT_JOB_EXPIRATION_TIME_SECONDS, DEFAULT_BOT_IDLE_TIME_SECONDS};

#[derive(Deserialize, Clone, Serialize)]
pub struct ServerConfig {
    pub discord_bot_id: u64,
    pub discord_bot_token: String,
    pub worker_id: Option<String>,
    pub job_expiration_time_seconds: u64,
    pub bot_idle_time_seconds: u64,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct KafkaConfig {
    pub kafka_uri: String,
    pub kafka_topic: String,
    pub kafka_use_ssl: Option<bool>,
    pub kafka_use_sasl: Option<bool>,
    pub kafka_username: Option<String>,
    pub kafka_password: Option<String>,
    pub kafka_ssl_cert: Option<String>,
    pub kafka_ssl_key: Option<String>,
    pub kafka_ssl_ca: Option<String>,
}

#[derive(Deserialize, Clone, Serialize)]
pub struct Config {
    pub config: ServerConfig,
    pub kafka: KafkaConfig,
}

pub fn init_config() -> Config {
    dotenv().ok();

    Config {
        config: ServerConfig {
            discord_bot_id: env::var("DISCORD_BOT_ID")
                .expect("DISCORD_BOT_ID must be set")
                .parse()
                .expect("DISCORD_BOT_ID must be a valid u64"),
            discord_bot_token: env::var("DISCORD_BOT_TOKEN")
                .expect("DISCORD_BOT_TOKEN must be set"),
            worker_id: env::var("WORKER_ID").ok().or_else(|| Some(nanoid::nanoid!())),
            job_expiration_time_seconds: DEFAULT_JOB_EXPIRATION_TIME_SECONDS,
            bot_idle_time_seconds: DEFAULT_BOT_IDLE_TIME_SECONDS,
        },
        kafka: KafkaConfig {
            kafka_uri: env::var("KAFKA_URI").expect("KAFKA_URI must be set"),
            kafka_topic: env::var("KAFKA_TOPIC").expect("KAFKA_TOPIC must be set"),
            kafka_use_ssl: env::var("KAFKA_USE_SSL").ok().map(|v| v == "true"),
            kafka_use_sasl: env::var("KAFKA_USE_SASL").ok().map(|v| v == "true"),
            kafka_username: env::var("KAFKA_USERNAME").ok(),
            kafka_password: env::var("KAFKA_PASSWORD").ok(),
            kafka_ssl_cert: env::var("KAFKA_SSL_CERT").ok(),
            kafka_ssl_key: env::var("KAFKA_SSL_KEY").ok(),
            kafka_ssl_ca: env::var("KAFKA_SSL_CA").ok(),
        },
    }
}