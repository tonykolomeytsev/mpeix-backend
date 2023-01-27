# mpeix-backend::app_schedule

Microservice responsible for working with the schedules of groups, teachers and rooms.

### Environment variables:
- `HOST` - app host name. 
    In release mode default is `0.0.0.0`, in debug mode default is `127.0.0.1`.
- `PORT` - app working port. Default is `8080`.
- `CACHE_CAPACITY` - in-memory LRU cache capacity. Default is `500` items.
- `CACHE_MAX_HITS` - cache expiration policy by hits. By default `10` hits.
- `CACHE_LIFETIME_HOURS` - cache expiration policy by creation date. Default is `6` hours.
- `CACHE_DIR` - dir to store schedule cache files. Default is `./cache`.
- `RUST_LOG` logging verbosity. Default is `info`.
