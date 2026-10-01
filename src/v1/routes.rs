use actix_web::{HttpResponse, get, post, web, HttpRequest};
use chrono::{Duration, Local, Utc};
use elasticsearch::{Elasticsearch, SearchParts};
use lazy_static::lazy_static;
use serde_json::json;
use std::collections::HashSet;
use std::sync::Arc;
use rand::Rng;
use sha1_smol::Sha1;
use tokio::time::sleep;

use crate::error_handling::{format_es_response, Error};
use crate::redis_service::JsonCacherService;
use crate::config::CONFIG;
use crate::{incognitoer, rate_limiting, TRANSPORT};
use crate::rate_limiting::BlockMethod;
use crate::utils::get_ip_address;
use crate::v1::query::QueryBody;

const MATCH_ALL_CACHE_KEY: &str = "knaben-api:match_all";

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

#[get("")]
pub async fn api(req: HttpRequest, cache: web::Data<Arc<JsonCacherService>>) -> Result<HttpResponse, Error> {
    let user_agent = req.headers().get("User-Agent").and_then(|ua| ua.to_str().ok()).unwrap_or("-");
    let ip = get_ip_address(&req);
    let ip_str = ip.map(|ip|ip.to_string()).unwrap_or("None".to_string());

    if let Ok(Some(cached_result)) = cache.get(&MATCH_ALL_CACHE_KEY.to_string()).await {
        log::debug!("CACHE HIT: {ip_str} {user_agent}");
        return Ok(HttpResponse::Ok()
            .insert_header(("Cache-Control", "public, max-age=0, s-maxage=300, stale-while-revalidate=60"))
            .json(cached_result));
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
    let json_response = format_es_response(response).await?;
    let obscured = incognitoer::obscure(json_response).map_err(|e| Error::new(500, "Failed to filter result", "filter_failed", Some(&e.to_string())))?;
    // Cache response
    let _ = cache.set(MATCH_ALL_CACHE_KEY, &obscured, 30).await.map_err(|e| {
        log::warn!("FAILED TO CACHE RESPONSE: {e}");
    });

    log::debug!("NEW SEARCH: {ip_str} {user_agent}");

    Ok(HttpResponse::Ok().insert_header(("Cache-Control", "public, max-age=0, s-maxage=300, stale-while-revalidate=60")).json(obscured))
}

#[post("")]
pub async fn api_post(req: HttpRequest, body: web::Json<QueryBody>, cache: web::Data<Arc<JsonCacherService>>) -> Result<HttpResponse, Error> {
    let user_agent = req.headers().get("User-Agent").and_then(|ua| ua.to_str().ok()).unwrap_or("-");
    let ip = get_ip_address(&req);
    let ip_str = ip.map(|ip|ip.to_string()).unwrap_or("None".to_string());

    log::debug!("[{} {}] {}", ip_str, user_agent, body.to_string());

    // Checks against hardcoded blocking rules
    match rate_limiting::is_blocked(&req) {
        // Try to hide that we're sending them a bullshit response
        BlockMethod::Troll => {
            log::info!("BLOCKED TROLL: {ip_str} {user_agent}");
            let sleep_ms = rand::rng().random_range(100..=21000);
            sleep(tokio::time::Duration::from_millis(sleep_ms)).await;

            // Try to get a random cached response
            if let Ok(Some((_, cached_result))) = cache.random_entry("knaben-api:*").await {
                return Ok(HttpResponse::Ok().json(cached_result));
            }

            // Check if we have the catch-all cached response
            if let Ok(Some(cached_result)) = cache.get(&MATCH_ALL_CACHE_KEY.to_string()).await {
                return Ok(HttpResponse::Ok().json(cached_result));
            }

            // Otherwise generate a fresh troll response
            return rate_limiting::troll_response();
        }
        _ => {}
    }

    // Check cache and return early if hit
    let cache_key = format!("knaben-api:{}", Sha1::from(&body.to_string()).digest().to_string());
    if let Ok(Some(cached_result)) = cache.get(&cache_key).await {
        log::debug!("CACHE HIT: {ip_str} {user_agent}");
        return Ok(HttpResponse::Ok()
            .insert_header(("Cache-Control", "public, max-age=0, s-maxage=300, stale-while-revalidate=60"))
            .json(cached_result));
    }

    // Dynamic rate limiting
    // if rate_limiting::is_rate_limited(&req).await {
    //     return Err(Error::new(429, "Too many concurrent connections", "rate_limited", None));
    // }

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
            incognitoer::format_sort(order_by).map_err(|e| Error::new(400, "Failed to order by", "sort_failed", Some(&e.to_string())))?;
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

    let json_response = format_es_response(response).await?;

    let obscured = incognitoer::obscure(json_response).map_err(|e| Error::new(500, "Failed to filter result", "filter_failed", Some(&e.to_string())))?;

    // Cache response
    let _ = cache.set(&cache_key, &obscured, 30).await.map_err(|e| {
        log::warn!("FAILED TO CACHE RESPONSE: {e}");
    });

    log::debug!("NEW SEARCH: {ip_str} {user_agent}");

    Ok(HttpResponse::Ok().json(obscured))
}