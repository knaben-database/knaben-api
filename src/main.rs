use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse, HttpServer, web};
use elasticsearch::http::transport::Transport;
use lazy_static::lazy_static;
use std::sync::Arc;
use std::io::Write;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use env_logger::Builder;

mod error_handling;
mod incognitoer;
mod redis_service;
pub mod config;
mod rate_limiting;
mod utils;
mod v1;
mod v2;

use crate::error_handling::{Error, json_error_handler, query_error_handler};
use crate::redis_service::JsonCacherService;
use crate::config::CONFIG;

async fn create_redis_pool() -> Pool<RedisConnectionManager> {
    let manager = RedisConnectionManager::new(format!("redis://{}:{}/", CONFIG.redis_host, CONFIG.redis_port)).unwrap();
    Pool::builder().build(manager).await.unwrap()
}

lazy_static! {
    static ref TRANSPORT: Transport = {
        let transport = Transport::single_node(&format!("http://{}:{}", CONFIG.elasticsearch_host, CONFIG.elasticsearch_port));
        let _ = transport.as_ref().map_err(|e| {
            log::error!("Failed to connect to Elasticsearch: {}", e);
        });
        transport.unwrap()
    };
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut builder = Builder::from_env(env_logger::Env::new().default_filter_or("info"));

    if std::env::var("NO_LOG_DATETIME").map(|v| v == "1").unwrap_or(false) {
        // Custom format without timestamp
        builder
            .format(|buf, record| {
                writeln!(buf, "[{}] {}", record.level(), record.args())
            })
            .init();
    } else {
        builder
            .format(|buf, record| {
                writeln!(buf, "[{} {}] {}", buf.timestamp(), record.level(), record.args())
            })
            .init();
    }

    let redis_pool = create_redis_pool().await;
    let cache_service = Arc::new(JsonCacherService::new(redis_pool));

    HttpServer::new(move || {
        App::new()
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
            .app_data(web::QueryConfig::default().error_handler(query_error_handler))
            .app_data(web::Data::new(cache_service.clone()))
            .service(web::scope("/v1").configure(v1::configure))
            .service(web::scope("/v2").configure(v2::configure))
            .default_service(web::to(|| async {
                Err::<HttpResponse, _>(Error::new(404, "Invalid path", "invalid_path", None))
            }))
            .wrap(Logger::new("%{r}a \"%r\" %s %b \"%{User-Agent}i\" %T"))
    })
    .bind((CONFIG.addr, CONFIG.port))?
    .run()
    .await
}
