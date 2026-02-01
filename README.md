# Mango³ Trailers

Movies and TV Series recommendations.

### Build Requirements

- Rust 1.93.x

## Environment variables

| Name                     | Type    | Default                                              |
| ------------------------ | ------- | ---------------------------------------------------- |
| API_ADDRESS              | String  | 127.0.0.1:8000                                       |
| API_OLD_TOKENS           | String  |                                                      |
| API_TOKENS               | String  | trailers                                             |
| CACHE_REDIS_URL          | String  | redis://127.0.0.1:6379/1                             |
| CACHE_TTL                | Integer | 3600                                                 |
| DATABASE_MAX_CONNECTIONS | String  | postgres://mango3:mango3@127.0.0.1:5432/trailers_dev |
| MONITOR_REDIS_URL        | String  | redis://127.0.0.1:6379/0                             |
