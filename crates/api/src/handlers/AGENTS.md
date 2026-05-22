# HTTP Layer Rules

## Handlers

- Handlers should be concise (5-20 lines): extract input, invoke service logic, generate response.
- Convert raw input to domain types within the handler, such as `UserName::parse(&payload.name)`.
- Return `AppResult<T>` (a `Result<T, AppError>` type alias).
- Apply `StatusCode::CREATED` for POST requests, `StatusCode::NO_CONTENT` for DELETE requests.
- Use `#[debug_handler]` during development for improved compiler error messages.

## Extractors

- Prefer `ValidatedJson<T>` over plain `Json<T>` when request bodies require validation.
- Use `ValidatedQuery<T>` for query parameters needing validation.
- Use `AuthUser` extractor for routes requiring authentication.
- Use `MaybeAuthUser` when authentication is optional (to differentiate missing headers from invalid ones).
- Body-consuming extractors (`Json`, `ValidatedJson`) should appear last in handler parameters.

## Middleware

- Apply `route_layer()` for authentication concerns (applies only to matched routes).
- Use `.layer()` for application-wide concerns like tracing, compression, and timeouts.
- Middleware order is significant: request ID -> TraceLayer -> request ID propagation -> timeout -> body limit -> CORS.

## DTOs

- Request DTOs should derive `Deserialize` and `Validate`.
- Response DTOs should derive `Serialize` (and `ToSchema` if using utoipa).
- Response DTOs must exclude sensitive data such as password_hash and internal identifiers.
- Implement `From<DomainEntity>` to transform domain types into response DTOs.

## Routes

- Use PATCH (rather than PUT) for partial updates with optional fields.
- Organize routes hierarchically: `/api/v1/users`, `/api/v1/posts`.
- Place health check endpoints outside the versioned API: `/health`, `/health/ready`.
