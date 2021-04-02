use actix_web::{get, HttpRequest, Responder};

#[get("/catalog")]
pub async fn catalog(_req: HttpRequest) -> impl Responder {
    tracing::info!("Hello from the catalog");
    "Hello World!"
}
