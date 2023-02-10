use std::env;

use actix_web::{middleware, web::Data, App, HttpServer};
use anyhow::Context;
use common_actix::{define_app_error, get_address};
use di::create_app;
use domain_bot::usecases::InitDomainBotUseCase;
use feature_telegram_bot::FeatureTelegramBot;

mod di;
mod routing;

pub struct AppTelegramBot {
    feature_telegram_bot: FeatureTelegramBot,
    init_domain_bot_use_case: InitDomainBotUseCase,
}

define_app_error!(AppTelegramBotError);

#[actix_web::main]

async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "1");
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
            .service(routing::telegram_webhook_v1)
    })
    .bind(get_address())?
    .run()
    .await
}

async fn init_app_components(app: &AppTelegramBot) -> anyhow::Result<()> {
    app.init_domain_bot_use_case
        .init()
        .await
        .with_context(|| "domain_bot init error")?;
    app.feature_telegram_bot
        .set_webhook()
        .await
        .with_context(|| "Set webhook error")
}
