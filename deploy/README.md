# Infrastructure

## Development

Use the root development environment file and start local infrastructure:

```bash
docker compose --env-file .env.dev -f docker-compose.dev.yml up -d
```

All Rust application services (`api-gateway`, `user-service`, `news-service`)
run locally with Cargo and connect to Dockerized NATS/PostgreSQL.

## Production-Oriented Baseline

Use the root production environment file and start infrastructure:

```bash
docker compose --env-file .env -f docker-compose.prod.yml up -d
```

This production file currently provisions shared infrastructure only.
Application containers should be added once `api-gateway`, `user-service`, and
`news-service` container builds are ready.
