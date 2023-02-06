# mpeix-backend/app_schedule

Microservice responsible for working with the schedules via VK Bot [**@mpeixbot**](https://vk.com/mpeixbot).

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
  - `VK_BOT_CONFIRMATION_CODE`<sup>**required**</sup> — confirmation code provided by VK for group/community Callback API.
  - `VK_BOT_ACCESS_TOKEN`<sup>**required**</sup> — VK App access token.
  - `VK_BOT_SECRET` - Optional VK secret for Callback API.
  - `VK_BOT_GROUP_ID` - Allowed VK group/community id. If not specified, requests from any groups will be accepted by this service.
