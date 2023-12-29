use actix_web::{get, web, App, HttpResponse, HttpServer, post};
use elasticsearch::{Elasticsearch, SearchParts};
use chrono::{Utc, Duration};
use serde_json::json;
use std::net::Ipv4Addr;
use serde::Deserialize;
use crate::error_handling::{Error, json_error_handler};

mod error_handling;
mod incognitoer;

const ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 8001;
const INDEX: &str = "torrent_cache2";
const KNABEN_DOMAIN: &str = "knaben.eu";
const PROXY_DOMAIN: &str = "knaben.xyz";


#[derive(Deserialize)]
struct QueryBody {
    query: Option<String>,
    order_by: Option<String>, // DONE
    order_direction: Option<String>, // DONE
    categories: Option<Vec<i32>>, // DONE
    from: Option<u16>, // DONE
    size: Option<u16>, // DONE
    hide_xxx: Option<bool>, // DONE
    hide_unsafe: Option<bool>, // DONE
}


#[post("/{version}")]
async fn api_post(path: web::Path<String>, body: web::Json<QueryBody>) -> Result<HttpResponse, Error> {
    let version = path.into_inner();
    if !version.eq_ignore_ascii_case("v1") {
        return Err(Error::new(400, "Invalid version, must be v1", "no_path_found", Some("Path must be /v1, that is the only path that is supported right now")));
    }

    let from = body.from.unwrap_or(0);
    let size = body.size.unwrap_or(150);

    let mut query = json!({
        "query": {}
    });

    if let Some(order_by) = &body.order_by {
        let sort = incognitoer::format_sort(order_by.clone()).map_err(|e| Error::new(400, "Failed to order by", "sort_failed", Some(&e.to_string())))?;
        let sort_direction = body.order_direction.clone().unwrap_or_else(|| "desc".to_string());
        if sort_direction.to_lowercase() != "desc" && sort_direction.to_lowercase() != "asc" {
            return Err(Error::new(400, "Order direction not correct", "sort_failed", Some("Order direction needs to be either desc or asc or not defined")));
        }
        query["sort"][sort] = json!({
            "order": sort_direction
        });
    }

    if let Some(categories) = &body.categories {
        query["query"]["bool"]["filter"] = json!({
            "terms": {
                "Category": categories
            }
        });
    }

    if body.hide_xxx.unwrap_or(false) {
        query["query"]["bool"]["must_not"] = json!({
            "range": {
                "Category": {
                    "gte": 5000000,
                    "lt": 6000000
                }
            }
        });
    }

    if body.hide_unsafe.unwrap_or(true) {
        query["query"]["bool"]["must"] = json!({
            "range": {
                "LastSeen": {
                    "gte": (Utc::now() - Duration::days(90)).timestamp_millis()
                }
            }
        });
        query["query"]["bool"]["must_not"] = json!({
            "range": {
                "VirusDetection": {
                    "gt": 0.6
                }
            }
        });
    }

    // Create an Elasticsearch client
    let client = Elasticsearch::default();


    let response = client
        .search(SearchParts::Index(&[INDEX]))
        .body(query)
        .from(from as i64)
        .size(size as i64)
        .send().await;

    let r = response.map_err(|e| Error::new(500, "Failed to execute search", "query_failed", Some(&e.to_string())))?;
    let r = r.error_for_status_code().map_err(|e| Error::new(500, "Failed to execute search", "query_failed", Some(&e.to_string())))?;
    let value = r.json::<serde_json::Value>().await.map_err(|e| Error::new(500, "Failed to parse response body", "parse_failed", Some(&e.to_string())))?;

    let filtered_value = incognitoer::obscure(value).map_err(|e| Error::new(500, "Failed to filter result", "filter_failed", Some(&e.to_string())))?;

    Ok(HttpResponse::Ok().json(filtered_value))
}

#[get("/{version}")]
async fn api(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let version = path.into_inner();
    if !version.eq_ignore_ascii_case("v1") {
        return Err(Error::new(400, "Invalid version, must be v1", "no_path_found", Some("Path must be /v1, that is the only path that is supported right now")));
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
        .await;

    let r = response.map_err(|e| Error::new(500, "Failed to execute search", "query_failed", Some(&e.to_string())))?;
    let r = r.error_for_status_code().map_err(|e| Error::new(500, "Failed to execute search", "query_failed", Some(&e.to_string())))?;
    let value = r.json::<serde_json::Value>().await.map_err(|e| Error::new(500, "Failed to parse response body", "parse_failed", Some(&e.to_string())))?;

    let filtered_value = incognitoer::obscure(value).map_err(|e| Error::new(500, "Failed to filter result", "filter_failed", Some(&e.to_string())))?;

    Ok(HttpResponse::Ok().json(filtered_value))
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(api)
            .service(api_post)
            .default_service(
                web::to(|| async {
                    Err::<HttpResponse, _>(Error::new(404, "Invalid path", "invalid_path", None))
                }),
            )
            .app_data(
                web::JsonConfig::default()
                    .error_handler(json_error_handler),
            )
    })
        .bind((ADDR, PORT))?
        .run()
        .await
}