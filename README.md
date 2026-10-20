``web backenda api``

## Local development

```sh
cp env.exmaple .env                 # then add DATABASE_URL (used by sqlx macros / tests), e.g.
                                    # DATABASE_URL=postgres://web-app:web-app@localhost:5432/web-app
docker compose up -d                # start Postgres (waits on healthcheck with --wait)
cargo run -p api                    # applies migrations on startup (see note below on first build)
cargo test --workspace              # integration tests use sqlx::test against the same server
docker compose down -v              # stop and delete the data volume
```

Note: `APPLICATION__ENVIRONMENT` defaults to `production` when unset (no Swagger UI, strict CORS);
`env.exmaple` sets it to `development` for local use.

sqlx's compile-time query checks run against the live database, so on a fresh DB the first build fails
with `relation "users" does not exist`. Either build offline using the checked-in `.sqlx` data
(`SQLX_OFFLINE=true cargo build`), or apply the migrations first (`cargo install sqlx-cli`, then
`sqlx migrate run`). After changing a query, refresh the offline data with `cargo sqlx prepare --workspace`.
