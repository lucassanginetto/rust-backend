use actix_web::web::Data;
use redis::{AsyncCommands, RedisResult, aio::ConnectionManager};
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Mutex;

pub const DEFAULT_EXPIRATION: u64 = 3600;

pub async fn get<T: DeserializeOwned>(
    key: &str,
    conn: &Data<Mutex<ConnectionManager>>,
) -> RedisResult<Option<T>> {
    let data: Option<String> = conn
        .lock()
        .expect("lock shouldn't be poisoned")
        .get(key)
        .await?;
    Ok(data.map(|json| {
        serde_json::from_str(&json).expect("json stored inside Redis should be valid object")
    }))
}

pub async fn set<T: Serialize>(
    key: &str,
    value: &T,
    ttl_seconds: u64,
    conn: &Data<Mutex<ConnectionManager>>,
) -> RedisResult<()> {
    let json = serde_json::to_string(value).unwrap();
    conn.lock()
        .expect("lock shouldn't be poisoned")
        .set_ex(key, json, ttl_seconds)
        .await
}

pub async fn del(key: &str, conn: &Data<Mutex<ConnectionManager>>) -> RedisResult<()> {
    conn.lock()
        .expect("lock shouldn't be poisoned")
        .del(key)
        .await
}
