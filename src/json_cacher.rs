use bb8::{Pool, RunError};
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};

pub struct JsonCacherService {
    pool: Pool<RedisConnectionManager>,
}

impl JsonCacherService {
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        Self { pool }
    }

    pub async fn get(&self, key: &String) -> Result<Option<String>, RunError<RedisError>> {
        let mut conn = self.pool.get().await?;
        Ok(conn.get(key).await?)
    }

    pub async fn set(
        &self,
        key: &str,
        value: &String,
        ttl_seconds: u64,
    ) -> Result<(), RunError<RedisError>> {
        let mut conn = self.pool.get().await?;
        conn.set_ex::<_, _, ()>(key, value, ttl_seconds).await?;
        Ok(())
    }

    pub async fn get_or_set<F, Fut>(
        &self,
        key: &str,
        ttl_seconds: u64,
        loader: F,
    ) -> Result<String, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: futures::Future<Output = Result<String, Box<dyn std::error::Error>>>,
    {
        if let Some(cached) = self.get(&key.to_string()).await? {
            return Ok(cached);
        }

        let value = loader().await?;
        self.set(key, &value, ttl_seconds).await?;
        Ok(value)
    }
}