# Infrastructure Layer Rules

## Repositories

- Implement repository traits defined in `domain/ports/`.
- Repository structs hold a `Database` (or `PgPool`) and nothing else.
- One repository per aggregate root (e.g., `UserRepository`, `PostRepository`).
- Repository methods return domain types, never raw database rows.

## Queries

- All SQL goes through `sqlx::query_as!` or `sqlx::query!` for compile-time checking.
- Never write SQL in handlers or domain services.
- Use parameterized queries exclusively. No string interpolation in SQL.
- Run `cargo sqlx prepare --workspace` after adding or changing queries.

## Row Conversion

- Use `TryFrom<PgRow>` (not `From`) when converting database rows to domain types.
- Return `sqlx::Error` or a mapped domain error from `TryFrom`, never panic.
- Keep row structs private to the repository module.

## Migrations

- One migration file per schema change. Never edit existing migration files.
- Migration filenames are timestamped: `{timestamp}_{description}.sql`.
- Run `sqlx migrate add <description>` to create a new migration.
- Migrations run automatically at startup via `Database::migrate()`.

## Error Handling

- Map `sqlx::Error::RowNotFound` to the appropriate domain error (e.g., `UserError::NotFound`).
- Map unique constraint violations to domain errors (e.g., `UserError::EmailTaken`).
- Propagate unexpected errors with `anyhow::Context` for useful context messages.
- Never expose raw sqlx errors to the domain or API layers.
