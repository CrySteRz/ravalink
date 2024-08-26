use crate::state::initializer::StateClient;
use redis::AsyncCommands;
use redis::RedisResult;

pub struct State {
    state_client: StateClient,
}

impl State {
    pub fn new(state_client: StateClient) -> Self {
        State { state_client }
    }

    pub async fn get(&self, key: &str) -> RedisResult<Option<String>> {
        let mut conn = self.state_client.client.lock().await.get_multiplexed_async_connection().await?;
        conn.get(key).await
    }

    pub async fn set(&self, key: &str, value: &str) -> RedisResult<()> {
        let mut conn = self.state_client.client.lock().await.get_multiplexed_async_connection().await?;
        conn.set(key, value).await
    }

    pub async fn set_with_expiry(&self, key: &str, value: &str, seconds: u64) -> RedisResult<()> {
        let mut conn = self.state_client.client.lock().await.get_multiplexed_async_connection().await?;
        conn.set_ex(key, value, seconds).await
    }

    pub async fn delete(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.state_client.client.lock().await.get_multiplexed_async_connection().await?;
        conn.del(key).await
    }
}
