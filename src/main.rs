mod opds;
mod services;

mod logger;

use {
    crate::logger::TracingLogger,
    actix_web::{App, HttpServer},
    tracing_log::LogTracer,
    tracing_subscriber::{fmt::Layer, layer::SubscriberExt as _, Registry},
};

#[actix_web::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    LogTracer::init()?;

    tracing::subscriber::set_global_default(Registry::default().with(Layer::default()))?;

    HttpServer::new(|| {
        App::new()
            .service(services::admin())
            .service(services::v1_2())
            .wrap(TracingLogger)
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await?;

    Ok(())
}
