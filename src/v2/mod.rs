mod routes;
mod query;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    (cfg)
        .service(routes::search)
        .service(routes::browse);
}