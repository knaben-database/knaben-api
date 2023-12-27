use actix_web::{get, web, App, HttpResponse, HttpServer};
use std::net::Ipv4Addr;
use crate::error_handling::{Error, json_error_handler};

mod error_handling;

const ADDR: Ipv4Addr = Ipv4Addr::new(127, 0, 0, 1);
const PORT: u16 = 8001;



#[get("/{version}")]
async fn api(path: web::Path<String>) -> Result<HttpResponse, Error> {
    let version = path.into_inner();
    if !version.eq_ignore_ascii_case("v1") {
        return Err(Error {
            msg: "Invalid version, must be v1".to_owned(),
            status: 400,
        });
    }
    Ok(HttpResponse::Ok().json("Hello!".to_string()))
}


#[tokio::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(api)
            .default_service(
                web::to(|| async {
                    Err::<HttpResponse, _>(Error {
                        msg: "Not found".to_owned(),
                        status: 404,
                    })
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