# Trailers (aka Filmstrip)

Movies and TV Series recommendations.

### Build Requirements

- Rust 1.93.x
- PostgreSQL 18.x

## Run Requirements

- PostgreSQL 18.x
- Redis 8.x
- Yt-dlp 2026.x

## Environment variables

| Name                     | Type    | Default                                              | Packages        |
| ------------------------ | ------- | ---------------------------------------------------- | --------------- |
| API_ADDRESS              | String  | 127.0.0.1:8005                                       | api             |
| API_CLIENT_IP_SOURCE     | String  | ConnectInfo                                          | api             |
| API_SERVE_STORAGE        | Boolean | true                                                 | api             |
| DATABASE_MAX_CONNECTIONS | Number  | 5                                                    | api,monitor     |
| DATABASE_URL             | String  | postgres://mango3:mango3@127.0.0.1:5432/trailers_dev | api,monitor     |
| MONITOR_REDIS_URL        | String  | redis://127.0.0.1:6379/1                             | api,cli,monitor |
| STORAGE_PATH             | String  | ./storage/                                           | api,monitor     |
| STORAGE_URL              | String  | http://127.0.0.1:8005/storage/                       | api             |
| TMDB_API_KEY             | String  |                                                      | monitor         |
| YT_DLP_PROXY             | String  |                                                      | monitor         |

Other environment variables: https://github.com/mangocubed/toolbox#environment-variables
