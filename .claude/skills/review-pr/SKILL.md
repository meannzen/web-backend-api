---
name: review-pr
description: Use when reviewing a pull request or branch diff in this Rust workspace. Checklist for architecture, correctness, database, security, testing and operational readiness.
---

# Skill: Review PR

Review a pull request for architecture, correctness, security, and operational readiness.
Get the diff with `git diff main...HEAD` (or `gh pr diff <n>`), then go through each section and report only real findings.

## Architecture

- [ ] Dependencies flow inward (`api -> domain`, `infra -> domain`). The `domain` crate has no axum, sqlx or other framework imports.
- [ ] Handlers are thin (extract, call service, respond). No SQL or business logic in handlers.
- [ ] New types are in the right crate: DTOs in `crates/api/src/dtos`, entities, ports and services in `crates/domain/src/<feature>`, rows and repositories in `crates/infra`.
- [ ] No database row types leaked into API responses.
- [ ] New services and repositories are wired in `AppState::new` and `new_test`.

## Correctness

- [ ] No `unwrap()` or `expect()` in request-handling code paths.
- [ ] Error types are appropriate: domain errors for business violations, `AppError` (`crates/api/src/error.rs`) for HTTP mapping.
- [ ] `?` propagation works through the full chain (infra error -> domain error -> `AppError`), and status codes are right (404, 409, 422 and so on).
- [ ] Newtypes are used for validated domain values (not raw `String`/`Uuid`).
- [ ] Partial updates use PATCH, not PUT.

## Database

- [ ] All queries use parameterized placeholders (no string interpolation).
- [ ] New queries use `query_as!` for compile-time checking.
- [ ] Row-to-domain conversions use `TryFrom`, not `From`.
- [ ] Migrations are backward-compatible or use expand/contract.
- [ ] `.sqlx/` metadata is updated if queries changed (`cargo sqlx prepare --workspace --check` passes).
- [ ] List endpoints use the cursor pagination helpers and have a matching index.

## Security

- [ ] Input validation at the API boundary (`ValidatedJson` with `Validate` on the DTO).
- [ ] Sensitive data excluded from response DTOs (password hash, internal IDs).
- [ ] Protected endpoints sit behind `auth_middleware` (in `protected_routes`) and use `AuthUser` where identity matters.
- [ ] Credential endpoints are in `auth_routes` (stricter rate limit).
- [ ] No secrets in logs (check `#[tracing::instrument]` skips and log fields).

## API contract

- [ ] New handlers have `#[utoipa::path]` and are listed in `ApiDoc` `paths(...)`.
- [ ] New DTOs derive `ToSchema` and are listed in `components(schemas(...))`.

## Testing

- [ ] Happy path test exists for new endpoints (`crates/api/tests`).
- [ ] Error path test exists for the primary failure case.
- [ ] Domain logic has unit tests if it contains non-trivial business rules.
- [ ] `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass.

## Operations

- [ ] New handlers have `#[tracing::instrument]` with appropriate skip/fields.
- [ ] No blocking calls on the async runtime (`spawn_blocking` for CPU work such as hashing).
- [ ] Outbound HTTP calls have timeouts configured.
- [ ] Background tasks handle shutdown via `CancellationToken`.
- [ ] New config values are added to `shared/src/config.rs` and `env.exmaple`.
