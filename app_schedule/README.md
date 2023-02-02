# mpeix-backend/app_schedule

Microservice responsible for working with the schedules of groups, teachers and rooms.

### Environment variables:
- App <sup>`app_schedule`</sup>:
  - `HOST` — app host name. In release mode default is `0.0.0.0`, in debug mode default is `127.0.0.1`.
  - `PORT` — app port. Default is `8080`.
- Database <sup>`common_database`</sup>:
  - `POSTGRES_PASSWORD`<sup>**required**</sup> — password for PostgreSQL database.
  - `POSTGRES_USER` - postgres user. Default is `postgres`.
  - `POSTGRES_DB` - database name. Default is the same as user name.
  - `POSTGRES_HOST` - database hostname. Default is `postgres`.
  - `POSTGRES_PORT` - database port. Default is `5432`.
- Schedule cache <sup>`domain_schedule_cache`</sup>:
  - `SCHEDULE_CACHE_CAPACITY` — in-memory LRU cache capacity. Default is `500` items.
  - `SCHEDULE_CACHE_MAX_HITS` — cache expiration policy by hits. Default is `10` hits.
  - `SCHEDULE_CACHE_LIFETIME_HOURS` — cache expiration policy by creation date. Default is `6` hours.
  - `SCHEDULE_CACHE_DIR` — dir to store schedule cache files. Default is `./cache`.
- Schedule Id cache <sup>`domain_schedule_cache`</sup>:
  - `SCHEDULE_ID_CACHE_CAPACITY` — in-memory LRU cache capacity. Default is `3000` items.
  - `SCHEDULE_ID_CACHE_MAX_HITS` — cache expiration policy by hits. Default is `20` hits.
  - `SCHEDULE_ID_CACHE_LIFETIME_HOURS` — cache expiration policy by creation date. Default is `12` hours.
- Schedule Search cache <sup>`domain_schedule_cache`</sup>:
  - `SCHEDULE_SEARCH_CACHE_CAPACITY` — in-memory LRU cache capacity. Default is `3000` items.
  - `SCHEDULE_SEARCH_CACHE_LIFETIME_MINUTES` — cache expiration policy by creation date. Default is `5` minutes.
- Schedule shift rules:
  - `SCHEDULE_SHIFT_CONFIG_PATH` — path to config with "schedule shift rules". 
    By default, the built-in default config will be used, which can be found here: [default_schedule_shift.toml](https://github.com/tonykolomeytsev/mpeix-backend/blob/master/domain_schedule_shift/res/default_schedule_shift.toml)
- Logging <sup>`app_schedule`</sup>:
  - `RUST_LOG` logging verbosity. Default is `info`. See all available values in [env_logger documentation](https://docs.rs/env_logger/latest/env_logger/).
