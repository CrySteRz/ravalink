use redis::Client;
use std::sync::Arc;
use tokio::sync::Mutex;
use redis::RedisResult;

pub struct StateClient {
    pub client: Arc<Mutex<Client>>,
}

impl StateClient {
    pub fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        Ok(StateClient {
            client: Arc::new(Mutex::new(client)),
        })
    }
}
