# DeepApp

DeepApp is a prototype compiler and runtime for the DeepApp v2 language idea: one source file describes data, HTTP endpoints, pages, background work, model configuration, pricing, and deployment metadata.

This repository currently contains a Rust implementation of a compiler frontend, static analyzer, artifact generator, and small local HTTP runtime. It is useful for experimenting with the language shape and validating compiler/runtime behavior, but it is not a production DeepApp implementation yet.

## What Works

- Parse a representative DeepApp v2 application.
- Run static checks for indexed queries, private page access, secret/PII flows, notification channels, cron/daemon contracts, deploy rules, endpoint identity, and rate limits.
- Generate JSON artifacts for routes, services, storage, migrations, frontend assets, background tasks, models, and pricing.
- Run a local HTTP runtime for compiled pages and endpoints.
- Serve runtime metadata through `/__deep/*` routes.
- Exercise in-memory storage, cache, queues, counters, snapshots, model fallback, pricing, rate limiting, and deterministic task ticks.

See [docs/coverage-audit.md](docs/coverage-audit.md) for the detailed implementation audit and [docs/user-tutorial.md](docs/user-tutorial.md) for a user walkthrough.

## Quick Start

The intended workflow is Docker-first.

```sh
docker compose build compiler
docker compose run --rm compiler check examples/chat.deep
docker compose run --rm compiler build examples/chat.deep --out build
docker compose up runtime
```

Then test the running app:

```sh
curl -fsS http://127.0.0.1:8080/ping
curl -fsS http://127.0.0.1:8080/
curl -fsS http://127.0.0.1:8080/__deep/manifest
```

Stop it with:

```sh
docker compose down
```

## CLI

The Docker `compiler` service runs the `deep` CLI.

```sh
docker compose run --rm compiler check examples/chat.deep
docker compose run --rm compiler build examples/chat.deep --out build
docker compose run --rm compiler migrate examples/chat.deep --preview
docker compose run --rm compiler ast examples/chat.deep
docker compose run --rm compiler run examples/chat.deep --addr 0.0.0.0:8080
```

The runtime can persist and restore an in-memory snapshot:

```sh
docker compose run --rm compiler run examples/chat.deep \
  --addr 0.0.0.0:8080 \
  --snapshot build/runtime-snapshot.json
```

## Make Targets

```sh
make docker-build
make docker-test
make docker-check
make docker-build-example
make docker-migrate-preview
make docker-run
```

## Generated Artifacts

Running `deep build` writes these files into `build/`:

- `manifest.json`: services, routes, endpoint identity/rate-limit policy, CDN, health check, rollback.
- `static-report.json`: language unit counts, invariants, checked rules.
- `runtime-bundle.json`: runtime capability summary.
- `migrations.json`: storage reconciliation plan.
- `storage-catalog.json`: data schemas, fields, indexes, uniqueness, privacy/security flags.
- `frontend-assets.json`: generated page assets.
- `task-catalog.json`: cron, daemon, and worker metadata.
- `model-catalog.json`: model/provider metadata.
- `pricing-catalog.json`: category/model prices in cents.

## Runtime Routes

With `docker compose up runtime`, the example app exposes:

```text
GET  /ping
GET  /
POST /hacking_is_a_serious_crime
POST /start_deep_research
GET  /check_research_status/{task_id}
```

Metadata and debug routes:

```text
GET  /__deep/manifest
GET  /__deep/static-report
GET  /__deep/tasks
POST /__deep/task-tick
GET  /__deep/snapshot
GET  /__deep/models
GET  /__deep/resolve-model/{model}
GET  /__deep/pricing
GET  /__deep/price/{category}/{model}
```

Some endpoint behavior is intentionally deterministic for the prototype. For example:

```sh
curl -fsS -X POST http://127.0.0.1:8080/hacking_is_a_serious_crime
curl -fsS -H 'Api-Key: dev-key' -X POST http://127.0.0.1:8080/start_deep_research
curl -fsS http://127.0.0.1:8080/check_research_status/research-1
```

## Development

Run tests:

```sh
docker compose run --rm test
```

Run the formatter check:

```sh
docker compose run --rm fmt
```

Local Rust commands also work when the matching toolchain is installed:

```sh
cargo test --locked
cargo run -- build examples/chat.deep --out build
```

## Project Layout

```text
src/lib.rs              compiler, analyzer, runtime, storage, tests
src/main.rs             CLI entrypoint
examples/chat.deep      representative DeepApp v2 app
tests/cli.rs            CLI integration test
docs/user-tutorial.md   user-facing walkthrough
docs/coverage-audit.md  implementation coverage and known gaps
build/                  generated example artifacts
```

## Current Limits

This is a working compiler/runtime prototype, not a complete production implementation of DeepApp v2.

Known gaps include:

- Arbitrary handler bodies are not fully interpreted.
- Authentication is deterministic header classification, not real user/session lookup.
- Rate limiting uses in-memory wall-clock buckets, not a distributed limiter.
- Pricing is catalog lookup plus counters, not a billing ledger or payment integration.
- Storage is in-memory with snapshot support, not an always-on durable database service.
- Frontend output is generated HTML metadata, not a reactive browser runtime.
- Cron, daemon, and worker behavior runs through deterministic ticks, not real schedulers.
- External providers such as OpenAI, Stripe, AWS, and S3 are represented as metadata, not real calls.

The goal of the repo is to keep tightening those gaps behind tests while preserving a Docker-first workflow.
