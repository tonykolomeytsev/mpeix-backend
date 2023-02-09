# mpeix-backend/app_schedule_vk_bot

Microservice for VK Bot [**@mpeixbot**](https://vk.com/mpeixbot).

### Environment variables:
- App <sup>`app_schedule_vk_bot`</sup>:
  - `HOST` — app host name. In release mode default is `0.0.0.0`, in debug mode default is `127.0.0.1`.
  - `PORT` — app port. Default is `8080`.
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
- Inter Microservice Interaction <sup>`domain_bot`</sup>:
  - `APP_SCHEDULE_BASE_URL`<sup>**required**</sup> — Base url to reach `app_schedule` microservice.
