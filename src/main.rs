use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse, HttpServer, get, post, web};
use chrono::{Duration, Local, Utc};
use elasticsearch::http::response::Response;
use elasticsearch::http::transport::Transport;
use elasticsearch::{Elasticsearch, SearchParts};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use sha1_smol::Sha1;

mod error_handling;
mod incognitoer;
mod json_cacher;
pub mod config;

use crate::error_handling::{Error, json_error_handler};
use crate::json_cacher::JsonCacherService;
use crate::config::CONFIG;

async fn create_redis_pool() -> Pool<RedisConnectionManager> {
    let manager = RedisConnectionManager::new(format!("redis://{}:{}/", CONFIG.redis_host, CONFIG.redis_port)).unwrap();
    Pool::builder().build(manager).await.unwrap()
}

lazy_static! {
    static ref TRANSPORT: Transport = {
        let transport = Transport::single_node(&format!("http://{}:{}", CONFIG.elasticsearch_host, CONFIG.elasticsearch_port));
        let _ = transport.as_ref().map_err(|e| {
            eprintln!("{}", e);
        });
        transport.unwrap()
    };
}

lazy_static! {
    static ref SORT_DIRECTIONS: HashSet<&'static str> = {
        let mut sort_options: HashSet<&str> = HashSet::new();
        sort_options.insert("desc");
        sort_options.insert("asc");
        sort_options
    };
}

lazy_static! {
    static ref SEARCH_OPTIONS: HashSet<&'static str> = {
        let mut search_options: HashSet<&str> = HashSet::new();
        search_options.insert("score");
        search_options
    };
}

#[derive(Deserialize, Serialize, Hash)]
struct QueryBody {
    search_type: Option<String>,
    query: Option<String>,
    search_field: Option<String>,
    order_by: Option<String>,
    order_direction: Option<String>,
    categories: Option<Vec<u64>>,
    from: Option<u16>,
    size: Option<u16>,
    hide_xxx: Option<bool>,
    hide_unsafe: Option<bool>,
    seconds_since_last_seen: Option<i64>,
}
impl fmt::Display for QueryBody {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}


#[post("/{version}")]
async fn api_post(path: web::Path<String>, body: web::Json<QueryBody>, cache: web::Data<Arc<JsonCacherService>>) -> Result<HttpResponse, Error> {
    let version = path.into_inner();
    if !version.eq_ignore_ascii_case("v1") {
        return Err(Error::new(
            400,
            "Invalid version, must be v1",
            "no_path_found",
            Some("Path must be /v1, that is the only path that is supported right now"),
        ));
    }
    
    // Check cache and return early if hit
    let cache_key = format!("knaben-api:{}", Sha1::from(&body.to_string()).digest().to_string());
    if let Ok(Some(cached_result)) = cache.get(&cache_key).await {
        return Ok(HttpResponse::Ok().json(cached_result));
    }

    let from = body.from.unwrap_or(0);
    let mut size = body.size.unwrap_or(150);
    if size > 300 {
        // No we don't want a single person to query 12 MB
        size = 300;
    }

    let mut query = json!({});
    let mut must = json!([]);
    let mut must_not = json!([]);

    if let Some(order_by) = &body.order_by {
        let sort =
            incognitoer::format_sort(order_by.clone()).map_err(|e| Error::new(400, "Failed to order by", "sort_failed", Some(&e.to_string())))?;
        let sort_direction = body.order_direction.clone().unwrap_or_else(|| "desc".to_string()).to_lowercase();
        if !SORT_DIRECTIONS.contains(sort_direction.as_str()) {
            return Err(Error::new(
                400,
                "Order direction not correct",
                "sort_failed",
                Some("Order direction needs to be either desc or asc or not defined"),
            ));
        }
        query["sort"][sort] = json!({
            "order": sort_direction
        });
    }

    if let Some(q) = &body.query {
        let search_type = body.search_type.clone().unwrap_or("100%".to_string()).to_lowercase();
        let is_percentage = match search_type.strip_suffix('%') {
            Some(v) => v.parse::<u8>().is_ok(),
            None => false,
        };

        let field = match &body.search_field {
            Some(f) => Some(
                incognitoer::format_field(f).map_err(|e| Error::new(400, "Failed to format field", "format_field_failed", Some(&e.to_string())))?,
            ),
            None => None,
        };

        if is_percentage {
            let field = field.unwrap_or("Title".to_string());
            let match_query = if field == "Title" {
                json!({
                    "simple_query_string": {
                      "query": q,
                      "minimum_should_match": search_type,
                      "fields": [
                        "Title",
                        "Title.fullword"
                      ],
                      "default_operator": "AND",
                      "analyzer": "my_search_analyzer"
                    }
                })
            } else {
                json!({
                    "match": {
                        field: {
                            "query": q,
                            "minimum_should_match": search_type
                        }
                    }
                })
            };
            must.as_array_mut()
                .ok_or(Error::new(
                    500,
                    "Failed to build query",
                    "query_building_exception",
                    Some("Failed to mutate must minimum_should_match as array"),
                ))?
                .push(match_query);
        } else if SEARCH_OPTIONS.contains(search_type.as_str()) {
            match search_type.as_str() {
                "score" => {
                    let query_string = match field {
                        Some(f) => json!({
                            "query_string": {
                                "query": q,
                                "default_field": f
                            }
                        }),
                        None => json!({
                            "query_string": {
                                "query": q
                            }
                        }),
                    };

                    must.as_array_mut()
                        .ok_or(Error::new(
                            500,
                            "Failed to build query",
                            "query_building_exception",
                            Some("Failed to mutate must query_string as array"),
                        ))?
                        .push(query_string);
                },
                _ => {
                    return Err(Error::new(
                        400,
                        "Invalid search option",
                        "invalid_search_option",
                        Some("The option provided needs to be \"score\" or \"100%\"."),
                    ));
                },
            }
        } else {
            return Err(Error::new(
                400,
                "Invalid search option",
                "invalid_search_option",
                Some("The option provided is not valid."),
            ));
        }
    }

    if let Some(categories) = &body.categories {
        query["query"]["bool"]["filter"] = json!({
            "terms": {
                "Category": categories
            }
        });
    }

    if body.hide_xxx.unwrap_or(false) {
        let range_query = json!({
            "range": {
                "Category": {
                    "gte": 5000000,
                    "lt": 6000000
                }
            }
        });
        must_not
            .as_array_mut()
            .ok_or(Error::new(
                500,
                "Failed to build query",
                "query_building_exception",
                Some("Failed to mutate must_not range as array"),
            ))?
            .push(range_query);
    }

    if body.hide_unsafe.unwrap_or(true) {
        let range_query = json!({
            "range": {
                "LastSeen": {
                    "gte": (Utc::now() - Duration::days(90)).timestamp_millis()
                }
            }
        });

        must.as_array_mut()
            .ok_or(Error::new(
                500,
                "Failed to build query",
                "query_building_exception",
                Some("Failed to mutate must range Last seen as array"),
            ))?
            .push(range_query);
        let range_query = json!({
            "range": {
                "VirusDetection": {
                    "gt": 0.6
                }
            }
        });
        must_not
            .as_array_mut()
            .ok_or(Error::new(
                500,
                "Failed to build query",
                "query_building_exception",
                Some("Failed to mutate must_not range virus detection as array"),
            ))?
            .push(range_query);
    }

    if let Some(seconds_since_last_seen) = body.seconds_since_last_seen {
        let range_query = json!({
            "bool": {
                "should": [
                    {
                        "range": {
                            "LastSeen": {
                                "gte": (Utc::now() - Duration::seconds(seconds_since_last_seen)).with_timezone(&Local).to_rfc3339().split(".").collect::<Vec<&str>>()[0]
                            }
                        }
                    },
                    {
                        "range": {
                            "CheckedMagnet": {
                                "gte": (Utc::now() - Duration::seconds(seconds_since_last_seen)).with_timezone(&Local).to_rfc3339().split(".").collect::<Vec<&str>>()[0]
                            }
                        }
                    }
                ]
            }
        });
        must.as_array_mut()
            .ok_or(Error::new(
                500,
                "Failed to build query",
                "query_building_exception",
                Some("Failed to mutate must range seconds_since_last_seen as array"),
            ))?
            .push(range_query);
    }

    // Join the must and must_not together
    if !must
        .as_array()
        .ok_or(Error::new(
            500,
            "Failed to build query",
            "query_building_exception",
            Some("Failed to mutate must to array when finishing of joining to main query"),
        ))?
        .is_empty()
    {
        query["query"]["bool"]["must"] = must;
    }

    if !must_not
        .as_array()
        .ok_or(Error::new(
            500,
            "Failed to build query",
            "query_building_exception",
            Some("Failed to mutate must_not to array when finishing of joining to main query"),
        ))?
        .is_empty()
    {
        query["query"]["bool"]["must_not"] = must_not;
    }

    // Create an Elasticsearch client
    let client = Elasticsearch::new(TRANSPORT.to_owned());

    let response = client
        .search(SearchParts::Index(&[&CONFIG.index]))
        .body(query)
        .from(from as i64)
        .size(size as i64)
        .send()
        .await;

    let json_response = format_response(response).await?;

    let obscured = incognitoer::obscure(json_response).map_err(|e| Error::new(500, "Failed to filter result", "filter_failed", Some(&e.to_string())))?;

    // Cache response
    let _ = cache.set(&cache_key, &obscured, 30).await.map_err(|e| {
        eprintln!("FAILED TO CACHE RESPONSE: {e}");
    });
    
    Ok(HttpResponse::Ok().json(obscured))
}

#[get("/{version}")]
async fn api(path: web::Path<String>, cache: web::Data<Arc<JsonCacherService>>) -> Result<HttpResponse, Error> {
    let version = path.into_inner();
    if !version.eq_ignore_ascii_case("v1") {
        return Err(Error::new(
            400,
            "Invalid version, must be v1",
            "no_path_found",
            Some("Path must be /v1, that is the only path that is supported right now"),
        ));
    }

    let cache_key = "knaben-api:match_all";
    if let Ok(Some(cached_result)) = cache.get(&cache_key.to_string()).await {
        return Ok(HttpResponse::Ok().json(cached_result));
    }

    // Create an Elasticsearch client
    let client = Elasticsearch::new(TRANSPORT.to_owned());

    // Define the search query
    let query = json!({
        "query": {
            "match_all": {}
        },
        "sort": {
            "PublishDate":{
                "order": "desc"
            }
        },
        "size": 150
    });

    // Perform the search query
    let response = client.search(SearchParts::Index(&[&CONFIG.index])).body(query).send().await;
    let json_response = format_response(response).await?;
    let obscured = incognitoer::obscure(json_response).map_err(|e| Error::new(500, "Failed to filter result", "filter_failed", Some(&e.to_string())))?;
    // Cache response
    let _ = cache.set(cache_key, &obscured, 30).await.map_err(|e| {
        eprintln!("FAILED TO CACHE RESPONSE: {e}");
    });

    Ok(HttpResponse::Ok().json(obscured))
}

async fn format_response(r: Result<Response, elasticsearch::Error>) -> Result<Value, Error> {
    let r = r.map_err(|e| Error::new(500, "Failed to execute search", "query_failed", Some(&e.to_string())))?;
    let status_code = r.status_code();
    let value = r
        .json::<Value>()
        .await
        .map_err(|e| Error::new(500, "Failed to parse response body", "parse_failed", Some(&e.to_string())))?;

    if !status_code.is_success() {
        if let Some(error) = value.get("error") {
            let error_type = error.get("type").and_then(Value::as_str).unwrap_or("Unknown error type");
            let reason = error.get("reason").and_then(Value::as_str);
            let detailed_reason = error
                .get("root_cause")
                .and_then(Value::as_array)
                .and_then(|arr| arr.get(0))
                .and_then(|cause| cause.get("reason"))
                .and_then(Value::as_str);
            let detaileder_reason = error
                .get("failed_shards")
                .and_then(Value::as_array)
                .and_then(|arr| arr.get(0))
                .and_then(|cause| cause.get("reason"))
                .and_then(|cause| cause.get("caused_by"))
                .and_then(|cause| cause.get("reason"))
                .and_then(Value::as_str);
            let error_message = format!(
                "Failed to execute search: {}",
                detailed_reason.unwrap_or(reason.unwrap_or("Unknown reason"))
            );
            return Err(Error::new(
                500,
                &error_message,
                error_type,
                detaileder_reason.or(detailed_reason).or(Some("Unknown reason")),
            ));
        }
        return Err(Error::new(500, "Failed to execute search", "query_failed", Some("Unknown error")));
    }

    Ok(value)
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let redis_pool = create_redis_pool().await;
    let cache_service = Arc::new(JsonCacherService::new(redis_pool));

    HttpServer::new(move || {
        App::new()
            .service(api)
            .service(api_post)
            .default_service(web::to(|| async {
                Err::<HttpResponse, _>(Error::new(404, "Invalid path", "invalid_path", None))
            }))
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
            .app_data(web::Data::new(cache_service.clone()))
            .wrap(Logger::new("%{r}a \"%r\" %s %b \"%{User-Agent}i\" %T"))
    })
    .bind((CONFIG.addr, CONFIG.port))?
    .run()
    .await
}
