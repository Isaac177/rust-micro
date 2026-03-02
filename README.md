I’m building a professional Rust microservices backend manually to learn senior-level Rust backend engineering. Continue from the current state and keep the same teaching style and constraints.

Project goals
- Rust microservices architecture
- Services:
    - api-gateway
    - user-service
    - news-service
- Communication:
    - api-gateway is the only public HTTP entrypoint
    - internal service-to-service communication uses NATS
    - api-gateway transforms HTTP requests into NATS request/reply messages
    - services also publish/consume domain events through NATS
- Data ownership:
    - user-service owns user data
    - news-service owns news/article data
- Cross-service data strategy:
    - prefer async event-driven replication/projections for frequently needed data
    - avoid unnecessary synchronous runtime coupling between services
- Databases:
    - PostgreSQL per service
    - migrations required
    - seed strategy required
- Infra:
    - Docker for infra only in dev
    - Rust services run locally in dev
    - production can containerize all app services later
- Infra components currently intended:
    - NATS
    - Redis
    - PostgreSQL for user-service
    - PostgreSQL for news-service

Engineering principles
- start simple, structure for growth
- domain-oriented modules
- clear separation of transport, application, and persistence
- fail fast on startup misconfiguration
- readiness should verify real dependencies
- explicit types and DTOs
- maintainable code over clever code
- production-minded organization
- strong error handling
- observability/logging
- health/readiness checks
- config via environment
- scalable folder structure

Framework/ecosystem
- Rust
- Axum
- Tokio
- SQLx
- PostgreSQL
- NATS Rust client
- Redis client for gateway rate limiting

How I want you to work with me
- First inspect what I already have before proposing changes
- Do not modify files unless I explicitly ask, except Cargo.toml updates are allowed when I ask to proceed to the next technical step
- Tell me the next step only
- Generate the next file or code snippet for me to type manually so I learn the syntax
- After each step, explain the concepts behind what we wrote
- Do not explain lines mechanically
- Explain the Rust/software concepts first, then connect them to the code
- Use and reference these docs when explaining concepts:
    - https://doc.rust-lang.org/book/
    - https://doc.rust-lang.org/rust-by-example/
    - https://google.github.io/comprehensive-rust/
- When needed, also use official docs for Axum, SQLx, Tokio, and NATS
- Teach me like I’m trying to become a strong Rust backend engineer
- Keep explanations practical and tied to what we are building

Current repo shape
- Cargo workspace
- services:
    - services/api-gateway
    - services/user-service
    - services/news-service
- shared crates:
    - crates/common
    - crates/contracts
- infra files at repo root:
    - docker-compose.dev.yml
    - docker-compose.prod.yml
- env files:
    - root `.env.dev` for dev infra
    - root `.env` for prod infra
    - service-specific env files for api-gateway/user-service/news-service

Current infra status
- infra is already live
- Docker Compose files are at repo root, not under deploy
- dev compose is infra only:
    - NATS
    - Redis
    - user Postgres
    - news Postgres
- app services are not containerized in dev
- Redis has been added for future rate limiting
- local Postgres host ports were changed to avoid conflicts:
    - user-db -> 6433
    - news-db -> 6434

Important architectural decision already made
- No API gateway debates: api-gateway is required
- api-gateway is the only public HTTP service
- user-service and news-service are internal and should primarily use NATS
- NATS is used for request/reply and pub/sub events
- For frequent cross-service reads, prefer event-driven projection/replication over live lookups

API gateway professional feature plan
We explicitly want the api-gateway to be production-minded and include these concerns over time:
1. config
2. tracing/logger
3. error handling
4. request ID middleware
5. CORS
6. auth skeleton
7. authorization mapping / permissions
8. Redis-backed rate limiting
9. metrics
10. health and readiness
    Also important later:
- security headers
- request size limits
- timeout policy
- proxy/IP trust rules
- normalized error responses
- resilience rules
- structured logs
- service communication controls
- graceful shutdown

Current api-gateway code status
We have started building api-gateway first.

Current files are flat in `services/api-gateway/src/`:
- `main.rs`
- `config.rs`
- `telemetry.rs`
- `error.rs`

Current implemented pieces
1. Config loading exists
- typed `GatewayConfig`
- `AppEnv` enum
- env-based loading
- fail-fast required var loading
- local `.env.dev` loading

2. Tracing/logger exists
- tracing initialized from `LOG_LEVEL`

3. Error type exists
- `AppError` enum
- maps to HTTP status codes
- maps to stable error codes
- implements `IntoResponse`

Important note
- I flattened single-file modules intentionally
- if a module is only one file, keep it as `config.rs`, `telemetry.rs`, `error.rs`
- do not force a folder + `mod.rs` unless there is a real need

What should be implemented next
The next step is:
- make `api-gateway` a real running Axum HTTP service
- start the server
- expose `GET /health`
- keep the process alive
