# mpeix-backend/app_schedule_telegram_bot

Microservice responsible for working with the schedules via Telegram Bot [**@mpeixbot**](https://mpeixbot.t.me).

### Environment variables:
- App <sup>`app_schedule`</sup>:
  - `HOST` — app host name. In release mode default is `0.0.0.0`, in debug mode default is `127.0.0.1`.
  - `PORT` — app port. Default is `8080`.
  - `RUST_LOG` logging verbosity. Default is `info`. See all available values in [env_logger documentation](https://docs.rs/env_logger/latest/env_logger/).
- Database <sup>`common_database`</sup>:
  - `POSTGRES_PASSWORD`<sup>**required**</sup> — password for PostgreSQL database.
  - `POSTGRES_USER` - postgres user. Default is `postgres`.
  - `POSTGRES_DB` - database name. Default is the same as user name.
  - `POSTGRES_HOST` - database hostname. Default is `postgres`.
  - `POSTGRES_PORT` - database port. Default is `5432`.
- VK Schedule Bot <sup>`feature_vk_bot`</sup>:
  - `TELEGRAM_BOT_ACCESS_TOKEN`<sup>**required**</sup> — Telegram Bot access token.
  - `TELEGRAM_BOT_SECRET`<sup>**required**</sup> - Telegram secret part of endpoint for Webhook API.
