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

DeepApp should absorb APIs and runtime semantics for open-source infrastructure where that makes the language simpler and safer, such as Redis for cache, queues, counters, pub/sub, and locks. Proprietary services should remain provider integrations or adapters. Their operational details should not become DeepApp language/runtime internals.

## Next Steps

### Must-haves

These are blocking for any serious production path.

1. Real storage backend

   The biggest gap. DeepAI has MySQL tables with hundreds of millions of rows where query patterns matter enormously: PK-range scans, no joins on huge tables, and `BETWEEN` over `IN`. In-memory storage with snapshots is not a path to production. DeepApp needs to emit real SQL against a real database, and the query planner needs to respect indexes declared in `data` blocks. The `@no_index` annotation on `ChatSession.created_at` is a good signal, but it needs to actually drive query generation.

2. Real migrations on live data

   `deep migrate --preview` is useful, but DeepAI runs migrations on tables with 300M+ rows where `ALTER TABLE` can lock the table for minutes. Production migration support needs:

   - Migration ordering guarantees, including `migrate before deploy` from `deploy_rules`.
   - Safe column-add semantics that avoid table rewrites.
   - The ability to bail out and resume.

3. Handler bodies that actually execute

   The `handle` blocks are the application. Until `resolve_model(...).chat(request.messages)` actually calls OpenAI and `ChatMessage |> where(_.session == session)` actually queries the database, the language is still mostly a spec rather than a runtime.

4. Real authentication and identity resolution

   DeepAI's identity model is messy: `owner_id` vs `client_info_id`, anonymous-to-logged-in conversion, Django Allauth, and Google Auth. The `identity RequestIdentity` block captures the shape, but deterministic header classification is not enough. DeepApp needs session cookies, API key lookup against a real user table, and anonymous-to-owner dedup logic.

5. Redis as a real service

   DeepAI uses Redis for distributed locks, caching, sorted-set queues, and pub/sub. The `cache`, `queue`, `counter`, and `lock` primitives in `.deep` map well to these concepts, but they need a real Redis backend rather than in-memory maps.

### High-priority

These would block most useful application slices.

6. Streaming responses

   The chat endpoint declares `response: stream`. This needs real SSE or chunked transfer encoding, not a JSON blob.

7. CDN-aware page serving

   The `cdn` block and `cache private` page annotation are pointed in the right direction, but the critical constraint is stricter: never render user data in cached pages; load it client-side via the correct app/API base URL. That needs to be enforced in generated frontend behavior, not only declared.

8. Worker/GPU integration

   The `worker stable_diffusion` block is expressive, but the boundary matters. DeepApp should absorb the open-source pieces it can own, such as Docker image contracts and Redis-backed queues. Proprietary services such as Vast and Salad should remain provider adapters. DeepApp should call those adapters through typed interfaces rather than absorbing their private APIs or operational details into the core language/runtime.

9. Distributed locks that work under concurrency

   `with lock billing[user.id]` needs Redis `SETNX` with TTL, not in-process mutexes.
