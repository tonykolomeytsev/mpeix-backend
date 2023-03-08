use actix_web::{middleware, web::Data, App, HttpServer};
use anyhow::Context;
use common_actix::{define_app_error, get_address};
use di::create_app;
use domain_bot::usecases::InitDomainBotUseCase;
use feature_vk_bot::FeatureVkBot;

mod di;
mod routing;

pub struct AppVkBot {
    feature_vk_bot: FeatureVkBot,
    init_domain_bot_use_case: InitDomainBotUseCase,
}

define_app_error!(AppVkBotError);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    let app = Data::new(create_app());

    // we shall panic if init fails
    init_app_components(&app).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(app.clone())
            .service(routing::health)
            .service(routing::vk_callback_v1)
    })
    .bind(get_address())?
    .run()
    .await
}

async fn init_app_components(app: &AppVkBot) -> anyhow::Result<()> {
    app.init_domain_bot_use_case
        .init()
        .await
        .with_context(|| "domain_bot init error")
}
