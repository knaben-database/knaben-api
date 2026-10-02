use std::sync::Arc;
use actix_web::{get, web, HttpRequest, HttpResponse};
use chrono::{Duration, Local, Utc};
use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::{json, Value};
use sha1_smol::Sha1;
use crate::error_handling::{format_es_response, Error};
use crate::{incognitoer, TRANSPORT};
use crate::config::CONFIG;
use crate::redis_service::JsonCacherService;
use crate::utils::get_ip_address;
use crate::v2::query::{BrowseQuery, SearchQuery};

#[get("/search")]
pub async fn search(req: HttpRequest, query: web::Query<SearchQuery>, cache: web::Data<Arc<JsonCacherService>>) -> Result<HttpResponse, Error> {
    let user_agent = req.headers().get("User-Agent").and_then(|ua| ua.to_str().ok()).unwrap_or("-");
    let ip = get_ip_address(&req);
    let ip_str = ip.map(|ip|ip.to_string()).unwrap_or("None".to_string());

    log::debug!("[{} {}] {}", ip_str, user_agent, query.to_string());

    // Check cache and return early if hit
    let cache_key = format!("knaben-api:search:{}", Sha1::from(&query.to_string()).digest().to_string());
    if let Ok(Some(cached_result)) = cache.get(&cache_key).await {
        log::debug!("CACHE HIT: {ip_str} {user_agent}");
        return Ok(HttpResponse::Ok()
            .insert_header(("Cache-Control", "public, max-age=0, s-maxage=300, stale-while-revalidate=60"))
            .insert_header(("X-Origin-Cache", "HIT"))
            .json(cached_result));

    }

    if query.size > 300 {
        return Err(Error::new(
            400,
            "Size is too big, max is 300",
            "size_failed",
            Some("The size cannot be more than 300"),
        ));
    }

    let mut es_query = json!({});
    let mut must: Vec<Value> = vec!();
    let mut must_not: Vec<Value> = vec!();
    let mut filters: Vec<Value> = vec!();

    if let Some(order_by) = &query.order_by {
        let sort = incognitoer::format_sort(order_by).map_err(|e| Error::new(400, "Failed to order by", "sort_failed", Some(&e.to_string())))?;
        es_query["sort"][sort] = json!({
            "order": query.order_direction
        });
    }

    let field =
        incognitoer::format_field(&query.search_field).map_err(|e| Error::new(400, "Failed to format field", "format_field_failed", Some(&e.to_string())))?;

    if field == "Title" {
        must.push(json!({
            "simple_query_string": {
                "query": query.query,
                "minimum_should_match": "100%",
                "fields": [
                    "Title",
                    "Title.fullword"
                ],
                "default_operator": "AND",
                "analyzer": "my_search_analyzer"
            }
        }));
    } else if field == "Description" {
        must.push(json!({
            "simple_query_string": {
                "query": query.query,
                "minimum_should_match": "100%",
                "fields": [
                    "Description",
                ],
                "default_operator": "AND",
                "analyzer": "my_search_analyzer"
            }
        }));
    } else {
        must.push(json!({
            "match": {
                (field): {
                    "query": query.query,
                    "minimum_should_match": "100%"
                }
            }
        }));
    }


    if !query.categories.is_empty()  {
        filters.push(json!({
            "terms": {
                "Category": query.categories
            }
        }));
    }

    if !query.trackers.is_empty()  {
        filters.push(json!({
            "terms": {
                "TrackerId": query.trackers
            }
        }));
    }

    if !query.show_xxx {
        must_not.push(json!({
            "range": {
                "Category": {
                    "gte": 5000000,
                    "lt": 6000000
                }
            }
        }));
    }

    if !query.show_unsafe {
        filters.push(json!({
            "range": {
                "LastSeen": {
                    "gte": (Utc::now() - Duration::days(90)).timestamp_millis()
                }
            }
        }));
        filters.push(json!({
            "range": {
                "VirusDetection": {
                    "lt": 0.6
                }
            }
        }));
    }

    if !query.show_dead_torrents {
        filters.push(json!({ "range": { "Seeders": { "gt": 0 } } }));
        filters.push(json!({
            "bool": {
                "should": [
                    { "range": { "LiveSeeders": { "gt": 0 } } },
                    { "bool": { "must_not": { "exists": { "field": "LiveSeeders" } } } }
                ],
                "minimum_should_match": 1
            }
        }));
    }

    if let Some(seconds_since_last_seen) = query.seconds_since_last_seen {
        filters.push(json!({
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
                ],
                "minimum_should_match": 1
            }
        }));
    }

    // Join the must and must_not together
    if !must.is_empty() {
        es_query["query"]["bool"]["must"] = Value::Array(must);
    }
    if !must_not.is_empty() {
        es_query["query"]["bool"]["must_not"] = Value::Array(must_not);
    }
    if !filters.is_empty() {
        es_query["query"]["bool"]["filter"] = Value::Array(filters);
    }

    // Create an Elasticsearch client
    let client = Elasticsearch::new(TRANSPORT.to_owned());

    let response = client
        .search(SearchParts::Index(&[&CONFIG.index]))
        .body(es_query)
        .from(query.from as i64)
        .size(query.size as i64)
        .send()
        .await;

    let json_response = format_es_response(response).await?;

    let obscured = incognitoer::obscure(json_response).map_err(|e| Error::new(500, "Failed to filter result", "filter_failed", Some(&e.to_string())))?;

    // Cache response
    let _ = cache.set(&cache_key, &obscured, 30).await.map_err(|e| {
        log::warn!("FAILED TO CACHE RESPONSE: {e}");
    });

    log::debug!("NEW SEARCH: {ip_str} {user_agent}");

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=0, s-maxage=300, stale-while-revalidate=60"))
        .insert_header(("X-Origin-Cache", "MISS"))
        .json(obscured))
}

#[get("/browse")]
pub async fn browse(req: HttpRequest, query: web::Query<BrowseQuery>, cache: web::Data<Arc<JsonCacherService>>) -> Result<HttpResponse, Error> {
    let user_agent = req.headers().get("User-Agent").and_then(|ua| ua.to_str().ok()).unwrap_or("-");
    let ip = get_ip_address(&req);
    let ip_str = ip.map(|ip|ip.to_string()).unwrap_or("None".to_string());

    log::debug!("[{} {}] {}", ip_str, user_agent, query.to_string());

    // Check cache and return early if hit
    let cache_key = format!("knaben-api:browse:{}", Sha1::from(&query.to_string()).digest().to_string());
    if let Ok(Some(cached_result)) = cache.get(&cache_key).await {
        log::debug!("CACHE HIT: {ip_str} {user_agent}");
        return Ok(HttpResponse::Ok()
            .insert_header(("Cache-Control", "public, max-age=0, s-maxage=300, stale-while-revalidate=60"))
            .insert_header(("X-Origin-Cache", "HIT"))
            .json(cached_result));

    }

    if query.size > 300 {
        return Err(Error::new(
            400,
            "Size is too big, max is 300",
            "size_failed",
            Some("The size cannot be more than 300"),
        ));
    }

    let mut es_query = json!({});
    let mut must: Vec<Value> = vec!();
    let mut must_not: Vec<Value> = vec!();
    let mut filters: Vec<Value> = vec!();

    let sort = incognitoer::format_sort(&query.order_by).map_err(|e| Error::new(400, "Failed to order by", "sort_failed", Some(&e.to_string())))?;
    es_query["sort"][sort] = json!({
        "order": query.order_direction
    });

    if !query.categories.is_empty()  {
        filters.push(json!({
            "terms": {
                "Category": query.categories
            }
        }));
    }

    if !query.trackers.is_empty()  {
        filters.push(json!({
            "terms": {
                "TrackerId": query.trackers
            }
        }));
    }

    if !query.show_xxx {
        must_not.push(json!({
            "range": {
                "Category": {
                    "gte": 5000000,
                    "lt": 6000000
                }
            }
        }));
    }

    if !query.show_unsafe {
        filters.push(json!({
            "range": {
                "LastSeen": {
                    "gte": (Utc::now() - Duration::days(90)).timestamp_millis()
                }
            }
        }));
        filters.push(json!({
            "range": {
                "VirusDetection": {
                    "lt": 0.6
                }
            }
        }));
    }

    if !query.show_dead_torrents {
        filters.push(json!({ "range": { "Seeders": { "gt": 0 } } }));
        filters.push(json!({
            "bool": {
                "should": [
                    { "range": { "LiveSeeders": { "gt": 0 } } },
                    { "bool": { "must_not": { "exists": { "field": "LiveSeeders" } } } }
                ],
                "minimum_should_match": 1
            }
        }));
    }

    if let Some(seconds_since_last_seen) = query.seconds_since_last_seen {
        filters.push(json!({
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
                ],
                "minimum_should_match": 1
            }
        }));
    }

    // Join the must and must_not together
    if must.is_empty() && must_not.is_empty() && filters.is_empty() {
        es_query["query"]["match_all"] = json!({});
    }
    if !must.is_empty() {
        es_query["query"]["bool"]["must"] = Value::Array(must);
    }
    if !must_not.is_empty() {
        es_query["query"]["bool"]["must_not"] = Value::Array(must_not);
    }
    if !filters.is_empty() {
        es_query["query"]["bool"]["filter"] = Value::Array(filters);
    }

    // Create an Elasticsearch client
    let client = Elasticsearch::new(TRANSPORT.to_owned());

    let response = client
        .search(SearchParts::Index(&[&CONFIG.index]))
        .body(&es_query)
        .from(query.from as i64)
        .size(query.size as i64)
        .send()
        .await;

    let json_response = format_es_response(response).await?;

    let obscured = incognitoer::obscure(json_response).map_err(|e| Error::new(500, "Failed to filter result", "filter_failed", Some(&e.to_string())))?;

    // Cache response
    let _ = cache.set(&cache_key, &obscured, 30).await.map_err(|e| {
        log::warn!("FAILED TO CACHE RESPONSE: {e}");
    });

    log::debug!("NEW SEARCH: {ip_str} {user_agent}");

    Ok(HttpResponse::Ok()
        .insert_header(("Cache-Control", "public, max-age=0, s-maxage=300, stale-while-revalidate=60"))
        .insert_header(("X-Origin-Cache", "MISS"))
        .json(obscured))
}