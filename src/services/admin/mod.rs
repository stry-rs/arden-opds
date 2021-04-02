use actix_web::{dev::HttpServiceFactory, web};

pub fn service() -> impl HttpServiceFactory + 'static {
    web::scope("/admin")
}
