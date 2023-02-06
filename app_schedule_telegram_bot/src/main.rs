use common_actix::define_app_error;
use feature_telegram_bot::FeatureTelegramBot;

mod routing;

pub struct AppTelegramBot {
    feature_telegram_bot: FeatureTelegramBot,
}

define_app_error!(AppTelegramBotError);

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    todo!()
}
