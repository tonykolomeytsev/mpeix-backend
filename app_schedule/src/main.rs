mod di;
mod errors;
mod routing;

use actix_web::{middleware, web::Data, App, HttpServer};
use anyhow::Context;
use di::AppComponent;
use domain_bot::usecases::InitDomainBotUseCase;
use domain_schedule::usecases::InitDomainScheduleUseCase;
use feature_schedule::v1::FeatureSchedule;
use feature_telegram_bot::FeatureTelegramBot;
use feature_vk_bot::FeatureVkBot;
use log::info;
use routing::*;

pub struct AppSchedule {
    feature_schedule: FeatureSchedule,
    feature_telegram_bot: FeatureTelegramBot,
    feature_vk_bot: FeatureVkBot,
    init_domain_schedule_use_case: InitDomainScheduleUseCase,
    init_domain_bot_use_case: InitDomainBotUseCase,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", envmnt::get_or("RUST_LOG", "info"));
    env_logger::init();
    let app = Data::new(AppComponent::create_app());

    // we shall panic if init fails
    init_app_components(&app).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .app_data(app.clone())
            .service(are_you_alive)
            .service(get_id_v1)
            .service(get_schedule_v1)
            .service(search_schedule_v1)
    })
    .bind(get_addr())?
    .run()
    .await
}

async fn init_app_components(app: &AppSchedule) -> anyhow::Result<()> {
    app.init_domain_schedule_use_case
        .init()
        .await
        .with_context(|| "domain_schedule init error")?;
    app.init_domain_bot_use_case
        .init()
        .await
        .with_context(|| "domain_bot init error")?;
    Ok(())
}

fn get_addr() -> (String, u16) {
    let host = envmnt::get_or(
        "HOST",
        if cfg!(debug_assertions) {
            "127.0.0.1"
        } else {
            "0.0.0.0"
        },
    );
    let port = envmnt::get_u16("PORT", 8080);
    info!("Starting server on {}:{}", host, port);
    (host, port)
}
