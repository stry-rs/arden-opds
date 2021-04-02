pub mod catalog;

use actix_web::{dev::HttpServiceFactory, web};

pub fn service() -> impl HttpServiceFactory + 'static {
    web::scope("/1.2").service(catalog::catalog)
}
