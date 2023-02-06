mod di;
mod routing;

use actix_web::{middleware, web::Data, App, HttpServer};
use anyhow::Context;
use common_actix::{define_app_error, get_address};
use di::AppComponent;
use domain_schedule::usecases::InitDomainScheduleUseCase;
use feature_schedule::v1::FeatureSchedule;

pub struct AppSchedule {
    feature_schedule: FeatureSchedule,
    init_domain_schedule_use_case: InitDomainScheduleUseCase,
}

define_app_error!(AppScheduleError);

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
            .service(routing::health)
            .service(routing::get_id_v1)
            .service(routing::get_schedule_v1)
            .service(routing::search_schedule_v1)
    })
    .bind(get_address())?
    .run()
    .await
}

async fn init_app_components(app: &AppSchedule) -> anyhow::Result<()> {
    app.init_domain_schedule_use_case
        .init()
        .await
        .with_context(|| "domain_schedule init error")
}
