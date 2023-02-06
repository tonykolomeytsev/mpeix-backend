use common_actix::define_app_error;
use feature_vk_bot::FeatureVkBot;

mod routing;

pub struct AppVkBot {
    feature_vk_bot: FeatureVkBot,
}

define_app_error!(AppVkBotError);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    todo!()
}
