use actix_web::{get, web, App, HttpResponse, HttpServer};
use elasticsearch::{Elasticsearch, SearchParts};
use serde_json::json;
use std::net::Ipv4Addr;
use crate::error_handling::{Error, json_error_handler};

mod error_handling;
mod incognitoer;

const ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 8001;



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
        .search(SearchParts::Index(&["torrent_cache2"]))
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