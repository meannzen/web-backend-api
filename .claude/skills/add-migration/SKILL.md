---
name: add-migration
description: Use when changing the Postgres schema (new table, column, index, constraint) with sqlx migrations in this repo. Covers conventions, backward compatibility, applying, and updating the .sqlx offline metadata.
---

# Skill: Add Migration

Add a database migration safely. Migrations live in `migrations/` at the workspace root.

## Steps

1. **Create the migration file.**
   ```
   cargo sqlx migrate add <description>
   ```
   This creates a timestamped `.sql` file in `migrations/`. See `migrations/*_users.sql` for the existing style.

2. **Write the SQL.** Follow these conventions:
   - `UUID PRIMARY KEY DEFAULT gen_random_uuid()` for IDs.
   - `TIMESTAMPTZ NOT NULL DEFAULT NOW()` for timestamps.
   - Add indexes on columns used in WHERE/JOIN/ORDER BY (cursor pagination needs an index matching the sort keys).
   - Add a trigger calling `set_updated_at()` (defined in the users migration) if the table has `updated_at`.
   - Consider case-insensitive uniqueness (citext or ICU collation) for user-facing text fields like email.

3. **Check backward compatibility.** Before applying:
   - Will the currently deployed application still work against the new schema?
   - If adding a NOT NULL column, does it have a DEFAULT?
   - If renaming or dropping a column, is the old application still deployed?
   - For destructive changes, use expand/contract: add the new structure, migrate data, and remove the old structure in a later migration.

4. **Apply and prepare.** Needs `DATABASE_URL` (see `env.exmaple`, and `docker-compose.yml` for local Postgres).
   ```
   cargo sqlx migrate run
   cargo sqlx prepare --workspace
   ```

5. **Update queries.** Update any `query_as!` calls in `crates/infra/src/repositories/` that reference changed columns or tables, and the row structs and `TryFrom` conversions with them.

6. **Verify.**
   ```
   cargo sqlx prepare --workspace --check
   cargo test --workspace
   ```

7. **Commit** the migration file together with the updated `.sqlx/` directory.
