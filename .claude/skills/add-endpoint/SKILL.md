---
name: add-endpoint
description: Use when adding a new API endpoint or resource to this Rust workspace (axum + sqlx). Walks through domain, port, repository, service, DTOs, handler, route, OpenAPI registration, tests and the verification gate.
---

# Skill: Add Endpoint

Add a new API endpoint following the workspace's layered architecture.
Dependencies flow inward: `api -> domain`, `infra -> domain`. The `domain` crate has no axum or sqlx imports.

Use `crates/domain/src/users/` and `crates/api/src/handlers/users/` as the reference implementation.

## Steps

1. **Domain types** in `crates/domain/src/<feature>/` (create the folder and add `pub mod <feature>;` to `crates/domain/src/lib.rs` if the feature is new).
   - `model.rs`: entity, newtypes for validated values, constructor.
   - `errors.rs`: error enum for business violations (add variants if the feature exists).
   - `port.rs`: repository trait (`#[async_trait]`, `Send + Sync + 'static`) returning domain types and domain errors.
   - `service.rs`: only if there is business logic beyond plain CRUD.

2. **Database migration**, if the schema changes: follow the `add-migration` skill.

3. **Repository** in `crates/infra/src/repositories/<feature>_repository.rs` (register it in `repositories/mod.rs`).
   - Implement the port trait on a `Pg...Repository`.
   - Use `sqlx::query_as!` (compile-time checked), never string-built SQL.
   - Define a row struct and convert with `TryFrom<Row> for DomainEntity`.
   - Map database errors to domain errors.

4. **Wire into state** in `crates/api/src/state.rs`.
   - Construct the repository and service in `AppState::new`, and update `new_test` too.
   - Expose the service as an `Arc<...>` field (`AppState` derives `FromRef`).

5. **DTOs** in `crates/api/src/dtos/<feature>.rs` (register in `dtos/mod.rs`).
   - Request DTOs: `Deserialize` + `Validate` + `utoipa::ToSchema`.
   - Response DTOs: `Serialize` + `ToSchema`, exclude sensitive fields (password hash, internal data).
   - Implement `From<DomainEntity>` for the response DTO.
   - Reuse `dtos/common.rs` for pagination and error shapes.

6. **Handler** in `crates/api/src/handlers/<feature>/mod.rs` (register in `handlers/mod.rs`).
   - Extract with `ValidatedJson` (from `crates/api/src/extractors`) or standard axum extractors for path/query.
   - Parse raw fields into domain newtypes, call the service, return the response DTO.
   - Add `#[tracing::instrument(skip(state))]` (also skip anything sensitive).
   - Add `#[utoipa::path(...)]` with tag, responses and `security(("bearer_auth" = []))` if protected.
   - Keep it thin, under about 20 lines. No SQL or business logic.

7. **Route** in `crates/api/src/routes/mod.rs`.
   - Add the route to the matching group: `public_routes`, `auth_routes` (stricter rate limit, for credential endpoints) or `v1_routes` / `protected_routes` (behind `auth_middleware`).
   - Use the right method: POST create, PATCH partial update, PUT full replace, DELETE remove.
   - For protected handlers, take `AuthUser` as an argument when the handler needs the caller's identity.

8. **OpenAPI registration** in the `ApiDoc` struct in `routes/mod.rs`.
   - Add the handler to `paths(...)`, new DTOs to `components(schemas(...))`, and a new tag to `tags(...)` if needed.
   - Swagger is disabled in production, but the spec must stay accurate.

9. **Tests** in `crates/api/tests/`.
   - Happy path and the primary error case (validation failure, not found, unauthorized).
   - `tests/api/` holds HTTP-level tests, `tests/integration/` holds DB-backed tests with SQL fixtures. Reuse `common.rs` helpers.
   - Add unit tests in the domain crate if there is non-trivial business logic.

10. **Verify** with the full gate:
   ```
   cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && cargo sqlx prepare --workspace --check
   ```
