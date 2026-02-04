# Mango³ Trailers

Movies and TV Series recommendations.

### Build Requirements

- Rust 1.93.x
- PostgreSQL 18.x

## Run Requirements

- PostgreSQL 18.x
- Redis 8.x
- Yt-dlp 2026.x

## Environment variables

| Name                         | Type    | Default                                              | Packages        |
| ---------------------------- | ------- | ---------------------------------------------------- | --------------- |
| API_ADDRESS                  | String  | 127.0.0.1:8000                                       | api             |
| API_SERVE_STORAGE            | Boolean | true                                                 | api             |
| API_TOKENS                   | String  | trailers                                             | api             |
| CACHE_REDIS_URL              | String  | redis://127.0.0.1:6379/1                             | api             |
| CACHE_TTL                    | Number  | 3600                                                 | api             |
| DATABASE_MAX_CONNECTIONS     | Number  | 5                                                    | api,monitor     |
| DATABASE_URL                 | String  | postgres://mango3:mango3@127.0.0.1:5432/trailers_dev | api,monitor     |
| IP_GEO_API_KEY               | String  |                                                      | monitor         |
| MAILER_ENABLE                | Boolean | false                                                | monitor         |
| MAILER_SENDER_ADDRESS        | String  | Mango³ dev <no-reply@localhost>                      | monitor         |
| MAILER_SMTP_ADDRESS          | String  | localhost                                            | monitor         |
| MAILER_SMTP_PASSWORD         | String  |                                                      | monitor         |
| MAILER_SMTP_SECURITY         | String  | none                                                 | monitor         |
| MAILER_SMTP_USERNAME         | String  |                                                      | monitor         |
| MAILER_SUPPORT_EMAIL_ADDRESS | String  | support@localhost                                    | monitor         |
| MONITOR_REDIS_URL            | String  | redis://127.0.0.1:6379/0                             | api,cli,monitor |
| STORAGE_PATH                 | String  | ./storage/                                           | api,monitor     |
| STORAGE_URL                  | String  | http://127.0.0.1:8000/storage/                       | api             |
| USERS_SESSION_TOKEN_LENGTH   | Number  | 64                                                   | api             |
| TMDB_API_KEY                 | String  |                                                      | monitor         |
| YT_DLP_PROXY                 | String  |                                                      | monitor         |
