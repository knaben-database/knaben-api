use crate::error_handling::{json_error_handler, Error};
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use chrono::{Duration, Utc};
use elasticsearch::http::response::Response;
use elasticsearch::{Elasticsearch, SearchParts};
use lazy_static::lazy_static;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;

mod error_handling;
mod incognitoer;

const ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 8001;
const INDEX: &str = "torrent_cache2";
const KNABEN_DOMAIN: &str = "knaben.eu";
const PROXY_DOMAIN: &str = "knaben.xyz";

lazy_static! {
    static ref SORT_DIRECTIONS: HashSet<&'static str> = {
        let mut sort_options: HashSet<&str> = HashSet::new();
        sort_options.insert("desc");
        sort_options.insert("asc");
        return sort_options;
    };
}

lazy_static! {
    static ref SEARCH_OPTIONS: HashSet<&'static str> = {
        let mut search_options: HashSet<&str> = HashSet::new();
        search_options.insert("score");
        return search_options;
    };
}

lazy_static! {
    static ref LIVE_MAPPING: HashMap<&'static str, &'static str> = {
        let mut live_mapping: HashMap<&str, &str> = HashMap::new();
        live_mapping.insert("Seeders", "LiveSeeders");
        live_mapping.insert("Peers", "LivePeers");
        live_mapping.insert("Grabs", "LiveGrabs");
        return live_mapping;
    };
}

#[derive(Deserialize)]
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
}

#[post("/{version}")]
async fn api_post(path: web::Path<String>, body: web::Json<QueryBody>) -> Result<HttpResponse, Error> {
    let version = path.into_inner();
    if !version.eq_ignore_ascii_case("v1") {
        return Err(Error::new(
            400,
            "Invalid version, must be v1",
            "no_path_found",
            Some("Path must be /v1, that is the only path that is supported right now"),
        ));
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

        let field = match body.search_field.as_ref() {
            Some(f) => Some(
                incognitoer::format_field(f).map_err(|e| Error::new(400, "Failed to format field", "format_field_failed", Some(&e.to_string())))?,
            ),
            None => None,
        };

        if is_percentage {
            let field = field.unwrap_or("Title".to_string());
            let match_query = json!({
                "match": {
                    field: {
                        "query": q,
                        "minimum_should_match": search_type
                    }
                }
            });
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
                    ))
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
    let client = Elasticsearch::default();
    let mut response: Value = client
        .search(SearchParts::Index(&[INDEX]))
        .body(&query)
        .from(from as i64)
        .size(size as i64)
        .send()
        .await
        .map_err(|e| Error::new(500, "Failed to parse response body", "parse_failed", Some(&e.to_string())))
        .and_then(|r| Ok(serde_response(r)))?
        .await?;

    println!("basic query: {}", query);

    let incognitoed_result = if let Some(second_sort) = &query
        .get("sort")
        .and_then(|serde_obj| serde_obj.as_object())
        .and_then(|obj| obj.keys().next())
        .and_then(|sn| LIVE_MAPPING.get(sn.as_str()))
    {
        // Elasticsearch has no way of intersecting sorts, so we have to do 2 requests here
        let sort_direction = body.order_direction.clone().unwrap_or_else(|| "desc".to_string()).to_lowercase();
        query["sort"] = json!({
            second_sort.to_string() : {
                "order": sort_direction
            }
        });
        let response_live: Value = client
            .search(SearchParts::Index(&[INDEX]))
            .body(&query)
            .from(from as i64)
            .size(size as i64)
            .send()
            .await
            .map_err(|e| Error::new(500, "Failed to parse response body", "parse_failed", Some(&e.to_string())))
            .and_then(|r| Ok(serde_response(r)))?
            .await?;
        println!("live query: {}", query);

        let response = intersect_results(response, response_live, size, sort_direction)
            .await
            .map_err(|e| Error::new(500, "Failed to intersect results", "parse_failed", Some(&e.to_string())))?;
        incognitoer::obscure(response).map_err(|e| Error::new(500, "Failed to filter result", "filter_failed", Some(&e.to_string())))?
    } else {
        incognitoer::obscure(response).map_err(|e| Error::new(500, "Failed to filter result", "filter_failed", Some(&e.to_string())))?
    };

    Ok(HttpResponse::Ok().json(incognitoed_result))
}

fn get_es_hits(es_value: Value) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let obj = es_value.as_object().ok_or("Root is not an object")?;
    let hits = obj.get("hits").ok_or("No 'hits' field in root")?;
    let hits_obj = hits.as_object().ok_or("'hits' is not an object")?;
    let hits_arr = hits_obj.get("hits").ok_or("'hits' field in 'hits' object is missing")?;
    let arr = hits_arr.as_array().ok_or("'hits' is not an array")?;

    Ok(arr.to_owned())
}

async fn intersect_results(
    base_result: Value,
    live_result: Value,
    max_results: u16,
    order_direction: String,
) -> Result<Value, Box<dyn std::error::Error>> {
    let mut intersected_value = base_result.clone();
    let mut base_arr = get_es_hits(base_result)?;
    let mut live_arr = get_es_hits(live_result)?;

    // Since we're popping and appending, it needs to be reversed before doing the heavy lifting
    base_arr.reverse();
    live_arr.reverse();

    let mut ids: HashSet<&str> = HashSet::new();
    let mut intersected_hits: Vec<Value> = vec![];
    for _ in 0..max_results {
        // Find the array with the highest/lowest value and pop it
        let live_value = live_arr
            .first()
            .and_then(|v| v.as_object())
            .and_then(|hit| hit.get("sort"))
            .and_then(|s_value| s_value.as_array())
            .and_then(|s_arr| s_arr.first())
            .and_then(|n| n.as_i64());

        let base_value = base_arr
            .first()
            .and_then(|v| v.as_object())
            .and_then(|hit| hit.get("sort"))
            .and_then(|s_value| s_value.as_array())
            .and_then(|s_arr| s_arr.first())
            .and_then(|n| n.as_i64());

        if live_value.is_none() && base_value.is_none() {
            break;
        }

        let next_live_entry = {
            loop {
                let entry = live_arr.pop();
                if entry.is_none() {
                    return None;
                }
                let entry = entry.unwrap();
                if entry.as_str().is_none() {
                    return None;
                }
                let id = entry.as_str().unwrap();
                if !ids.contains(id) {
                    return Some(entry);
                }
            }
        };

        if let Some(entry) = live_arr.pop() {
            ids.insert(entry["_id"]);
            intersected_hits.push(entry);
        }
        if let Some(entry) = base_arr.pop() {
            ids.insert(entry["_id"]);
            intersected_hits.push(entry);
        }
    }
    intersected_value["hits"]["hits"] = Value::Array(intersected_hits);
    Ok(intersected_value)
}

#[get("/{version}")]
async fn api(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let version = path.into_inner();
    if !version.eq_ignore_ascii_case("v1") {
        return Err(Error::new(
            400,
            "Invalid version, must be v1",
            "no_path_found",
            Some("Path must be /v1, that is the only path that is supported right now"),
        ));
    }

    // Create an Elasticsearch client
    let client = Elasticsearch::default();

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
    let response = client
        .search(SearchParts::Index(&[INDEX]))
        .body(query)
        .send()
        .await
        .map_err(|e| Error::new(500, "Failed to parse response body", "parse_failed", Some(&e.to_string())))?;

    let formatted_result = serde_response(response).await?;
    let incognitoed_result =
        incognitoer::obscure(formatted_result).map_err(|e| Error::new(500, "Failed to filter result", "filter_failed", Some(&e.to_string())))?;
    Ok(HttpResponse::Ok().json(incognitoed_result))
}

async fn serde_response(r: Response) -> Result<Value, Error> {
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

    return Ok(value);
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .service(api)
            .service(api_post)
            .default_service(web::to(|| async {
                Err::<HttpResponse, _>(Error::new(404, "Invalid path", "invalid_path", None))
            }))
            .app_data(web::JsonConfig::default().error_handler(json_error_handler))
            .wrap(Logger::new("%{r}a \"%r\" %s %b \"%{User-Agent}i\" %T"))
    })
    .bind((ADDR, PORT))?
    .run()
    .await
}
