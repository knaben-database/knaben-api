use bb8::{Pool, RunError};
use bb8_redis::RedisConnectionManager;
use redis::{AsyncCommands, RedisError};
use serde_json::Value;
use anyhow;
use rand::Rng;

pub struct JsonCacherService {
    pool: Pool<RedisConnectionManager>,
}

impl JsonCacherService {
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        Self { pool }
    }

    pub async fn get(&self, key: &String) -> anyhow::Result<Option<Value>> {
        let mut conn = self.pool.get().await?;
        let content: Option<String> = conn.get(key).await?;
        match content {
            Some(content) => {
                Ok(Some(serde_json::from_str(&content)?))
            },
            None => Ok(None)
        }
    }

    pub async fn set(
        &self,
        key: &str,
        value: &Value,
        ttl_seconds: u64,
    ) -> Result<(), RunError<RedisError>> {
        let mut conn = self.pool.get().await?;
        conn.set_ex::<_, _, ()>(key, value.to_string(), ttl_seconds).await?;
        Ok(())
    }

    pub async fn get_or_set<F, Fut>(
        &self,
        key: &str,
        ttl_seconds: u64,
        loader: F,
    ) -> Result<Value, Box<dyn std::error::Error>>
    where
        F: FnOnce() -> Fut,
        Fut: futures::Future<Output = Result<Value, Box<dyn std::error::Error>>>,
    {
        if let Some(cached) = self.get(&key.to_string()).await? {
            return Ok(cached);
        }

        let value = loader().await?;
        self.set(key, &value, ttl_seconds).await?;
        Ok(value)
    }

    /// Get a random key matching a pattern (e.g., "website-api:*")
    pub async fn random_key(&self, pattern: &str) -> anyhow::Result<Option<String>> {
        let mut conn = self.pool.get().await?;
        let mut cursor = 0;

        // First, collect some matching keys
        let mut matching_keys = Vec::new();

        loop {
            let (next_cursor, keys): (i64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(100)  // Check 100 keys at a time
                .query_async(&mut *conn)
                .await?;

            matching_keys.extend(keys);
            cursor = next_cursor;

            // Stop after finding some matches or scanning everything
            if !matching_keys.is_empty() || cursor == 0 {
                break;
            }
        }

        // Pick a random key from what we found
        if matching_keys.is_empty() {
            Ok(None)
        } else {
            let idx = rand::rng().random_range(0..matching_keys.len());
            Ok(Some(matching_keys[idx].clone()))
        }
    }

    /// Get a random entry (key + value) matching a pattern
    pub async fn random_entry(&self, pattern: &str) -> anyhow::Result<Option<(String, Value)>> {
        // First get a random key
        if let Some(key) = self.random_key(pattern).await? {
            // Then get the value for that key
            if let Some(value) = self.get(&key).await? {
                return Ok(Some((key, value)));
            }
        }

        Ok(None)
    }

    /// Get a random value matching a pattern (just the value, not the key)
    pub async fn random_value(&self, pattern: &str) -> anyhow::Result<Option<Value>> {
        if let Some((_, value)) = self.random_entry(pattern).await? {
            return Ok(Some(value));
        }

        Ok(None)
    }

    /// Get all keys matching a pattern (use with caution on large datasets)
    pub async fn keys(&self, pattern: &str) -> anyhow::Result<Vec<String>> {
        let mut conn = self.pool.get().await?;
        let mut cursor = 0;
        let mut all_keys = Vec::new();

        loop {
            let (next_cursor, keys): (i64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(1000)
                .query_async(&mut *conn)
                .await?;

            all_keys.extend(keys);
            cursor = next_cursor;

            if cursor == 0 {
                break;
            }
        }

        Ok(all_keys)
    }
}