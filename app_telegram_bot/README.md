# mpeix-backend/app_schedule_telegram_bot

Microservice for Telegram Bot [**@MpeixBot**](https://mpeixbot.t.me).

### Environment variables:
- App <sup>`app_schedule_telegram_bot`</sup>:
  - `HOST` — app host name. In release mode default is `0.0.0.0`, in debug mode default is `127.0.0.1`.
  - `PORT` — app port. Default is `8080`.
- Database <sup>`common_database`</sup>:
  - `POSTGRES_PASSWORD`<sup>**required**</sup> — password for PostgreSQL database.
  - `POSTGRES_USER` - postgres user. Default is `postgres`.
  - `POSTGRES_DB` - database name. Default is the same as user name.
  - `POSTGRES_HOST` - database hostname. Default is `postgres`.
  - `POSTGRES_PORT` - database port. Default is `5432`.
- VK Schedule Bot <sup>`feature_vk_bot`</sup>:
  - `TELEGRAM_BOT_ACCESS_TOKEN`<sup>**required**</sup> — Telegram Bot access token.
  - `TELEGRAM_BOT_SECRET`<sup>**required**</sup> — Telegram secret part of endpoint for Webhook API.
  - `TELEGRAM_BOT_WEBHOOK_URL`<sup>**required**</sup> — This microservice endpoint url for Webhook API.
- Inter Microservice Interaction <sup>`domain_bot`</sup>:
  - `APP_SCHEDULE_BASE_URL`<sup>**required**</sup> — Base url to reach `app_schedule` microservice.
