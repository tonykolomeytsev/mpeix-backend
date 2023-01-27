# mpeix-backend/app_schedule

Microservice responsible for working with the schedules of groups, teachers and rooms.

### Environment variables:
- App address:
  - `HOST` - app host name. 
      In release mode default is `0.0.0.0`, in debug mode default is `127.0.0.1`.
  - `PORT` - app working port. Default is `8080`.
- Schedule cache:
  - `SCHEDULE_CACHE_CAPACITY` - in-memory LRU cache capacity. Default is `500` items.
  - `SCHEDULE_CACHE_MAX_HITS` - cache expiration policy by hits. By default `10` hits.
  - `SCHEDULE_CACHE_LIFETIME_HOURS` - cache expiration policy by creation date. Default is `6` hours.
  - `SCHEDULE_CACHE_DIR` - dir to store schedule cache files. Default is `./cache`.
- Schedule Id cache:
  - `SCHEDULE_ID_CACHE_CAPACITY` - in-memory LRU cache capacity. Default is `3000` items.
  - `SCHEDULE_ID_CACHE_MAX_HITS` - cache expiration policy by hits. By default `20` hits.
  - `SCHEDULE_ID_CACHE_LIFETIME_HOURS` - cache expiration policy by creation date. Default is `12` hours.
- Logging:
  - `RUST_LOG` logging verbosity. Default is `info`.
