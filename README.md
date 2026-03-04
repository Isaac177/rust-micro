# rust-micro

A microservices architecture built in Rust. Services communicate exclusively over NATS (request-reply). The API Gateway is the single entry point for all HTTP clients — no service is exposed directly.

## Architecture

```
Client
  │
  ▼
api-gateway  (HTTP :8080)
  │
  │  NATS request-reply
  ├──────────────────────▶  user-service
  │                               │
  │                               │  NATS request-reply
  └──────────────────────▶  news-service ──────────────▶  user-service
```

### Key design decisions

- **Gateway is the only HTTP server.** All other services are internal and speak NATS only.
- **Service-to-service communication is direct over NATS.** When news-service needs author data it publishes to `users.get` itself — it does not go back through the gateway.
- **Gateway mints JWTs.** Auth is a transport concern. user-service owns credentials and password hashing (argon2), gateway owns token issuance (access + refresh, stateless HS256).
- **`NatsResponse<T>`** is a typed envelope used for operations that can fail with domain errors (register, authenticate, get). List operations return their response type directly.
- **No service discovery.** Services know each other only by NATS subject strings defined in the shared `contracts` crate.

## Workspace layout

```
rust-micro/
├── crates/
│   ├── contracts/   # Shared NATS subjects, request/response types (serde)
│   └── common/      # Shared utilities (unused for now)
└── services/
    ├── api-gateway/  # HTTP server, JWT auth, NATS client, route handlers
    ├── user-service/ # User registration, authentication, password hashing
    └── news-service/ # Articles, enriched with author data from user-service
```

## Services

### api-gateway

- **HTTP framework:** Axum 0.8
- **Middleware:** CORS, request ID injection, `require_auth` (JWT Bearer validation)
- **Public routes:** `POST /api/v1/auth/register`, `POST /api/v1/auth/login`, `POST /api/v1/auth/refresh`, `GET /health`, `GET /ready`
- **Protected routes** (require `Authorization: Bearer <access_token>`): `GET /api/v1/users`, `GET /api/v1/news/articles`, `GET /api/v1/news/articles/{id}`
- **JWT:** HS256, access token (15 min), refresh token (7 days). Claims include `sub`, `email`, `display_name`, `token_type`.
- **Readiness check** verifies live NATS and Redis connections.

### user-service

- **Database:** PostgreSQL (via sqlx, migrations run on startup)
- **Password hashing:** argon2 (Argon2id default params)
- **NATS subjects:**
  | Subject | Description |
  |---|---|
  | `users.list` | Paginated user list |
  | `users.register` | Create user, returns `NatsResponse` (error: `email_taken`) |
  | `users.authenticate` | Verify credentials, returns `NatsResponse` (error: `invalid_credentials`, `account_disabled`) |
  | `users.get` | Fetch single user by ID, returns `NatsResponse` (error: `not_found`) |

### news-service

- **Database:** PostgreSQL (via sqlx, migrations run on startup)
- **NATS subjects:**
  | Subject | Description |
  |---|---|
  | `news.articles.list` | Paginated published articles |
  | `news.articles.get` | Single article enriched with author details (calls `users.get` internally) |

## Internal file structure per service

Each service follows the same three-layer pattern:

```
src/<domain>/
  mod.rs        # module declarations
  nats.rs       # subscribe, dispatch, reply — no business logic
  handlers.rs   # business logic — deserializes payload, calls repository, returns serialized bytes
  repository.rs # SQL queries only
```

`nats.rs` contains only `serve()` (subscription loop + match dispatch) and `reply()` (publish helper). Handlers own deserialization, logic, and serialization.

## Shared contracts crate

All NATS subjects and message types live in `crates/contracts`. Both gateway and services import from here. Adding a new inter-service call means:

1. Add subject + Request/Response structs to `contracts`
2. Add handler in the producer service
3. Add one line to the NATS dispatch match
4. Call from the consumer

## Infrastructure

| Service | Port |
|---|---|
| api-gateway (HTTP) | 8080 |
| NATS | 4222 |
| Redis | 6379 |
| user-service DB (Postgres) | 6433 |
| news-service DB (Postgres) | 6434 |

## Getting started

```bash
# start infrastructure
docker compose up -d

# run migrations
export DATABASE_URL=postgres://user_service:user_service_dev@127.0.0.1:6433/user_service
sqlx migrate run --source services/user-service/migrations

# run services (in separate terminals)
cargo run -p user-service
cargo run -p news-service
cargo run -p api-gateway
```

## Auth flow

```bash
# register — returns access + refresh tokens
curl -X POST localhost:8080/api/v1/auth/register \
  -H 'Content-Type: application/json' \
  -d '{"email":"user@example.com","password":"password123","display_name":"Alice"}'

# login
curl -X POST localhost:8080/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"user@example.com","password":"password123"}'

# refresh
curl -X POST localhost:8080/api/v1/auth/refresh \
  -H 'Content-Type: application/json' \
  -d '{"refresh_token":"<token>"}'

# authenticated request
curl localhost:8080/api/v1/users \
  -H 'Authorization: Bearer <access_token>'
```
