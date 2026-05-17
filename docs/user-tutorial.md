# DeepApp User Tutorial

This tutorial shows how to use the current DeepApp prototype as an app author. It focuses on the CLI, the example app, generated artifacts, and the local runtime.

DeepApp is still a prototype. It can parse and check a representative v2 app, generate build artifacts, and run a small HTTP runtime for compiled routes. Some v2 ideas are represented as deterministic prototype behavior rather than production integrations.

## Prerequisites

Use Docker for the normal workflow:

```sh
docker compose build compiler
```

You can also run the Rust binary directly with `cargo`, but Docker is the intended path for this repo.

## The Example App

Start with [examples/chat.deep](../examples/chat.deep). It demonstrates the main DeepApp concepts:

- `data` declarations for built-in storage schemas.
- `endpoint` declarations for HTTP routes.
- `page` declarations for frontend assets.
- `queue`, `cache`, `cached fn`, `counter`, and `topic` declarations.
- `model_config` and `pricing` declarations.
- `worker`, `cron`, `daemon`, `services`, `cdn`, and `deploy_rules` blocks.
- Compile-time safety checks for indexed queries, private data, secrets, notification channels, and operational contracts.

## Check A DeepApp File

Run the compiler checks:

```sh
docker compose run --rm compiler check examples/chat.deep
```

Expected result:

```text
DeepApp check passed: examples/chat.deep
```

If the compiler finds a problem, it prints a DeepApp error code. For example, querying a `@no_index` field can produce a `DB021` error, and notifying an undeclared channel can produce `NOTIFY002`.

## Build Artifacts

Generate artifacts into `build/`:

```sh
docker compose run --rm compiler build examples/chat.deep --out build
```

The build writes JSON files that describe the app:

- `manifest.json`: routes, services, CDN settings, endpoint identity policy, rate limits, health check, and rollback policy.
- `static-report.json`: language-unit counts, invariants, and checked compiler rules.
- `runtime-bundle.json`: what runtime capabilities the app needs.
- `migrations.json`: ordered pre-deploy migration plan with online/resumable metadata, DDL, lock-risk notes, and checkpoint keys.
- `sql-plan.json`: MySQL-oriented table DDL, index DDL, and indexed query plans.
- `storage-catalog.json`: data schemas, indexes, uniqueness, and privacy/security flags.
- `frontend-assets.json`: generated page assets with cache policy, API base URL, and client-fetch data-loading metadata.
- `task-catalog.json`: cron, daemon, and worker metadata.
- `model-catalog.json`: model provider metadata.
- `pricing-catalog.json`: category/model prices in cents.
- `redis-catalog.json`: Redis primitive backends, TTLs, and sorted queue order fields.

Inspect one artifact:

```sh
sed -n '1,120p' build/manifest.json
```

## Preview Migrations

Preview storage changes without applying anything:

```sh
docker compose run --rm compiler migrate examples/chat.deep --preview
```

In this prototype, non-preview migration application is intentionally gated.

## Run The App

Start the runtime:

```sh
docker compose up runtime
```

In another terminal, test the health check:

```sh
curl -fsS http://127.0.0.1:8080/ping
```

Open the compiled page:

```sh
curl -fsS http://127.0.0.1:8080/
```

Stop the runtime with `Ctrl-C`, or from another shell:

```sh
docker compose down
```

## Runtime Metadata Routes

The runtime exposes compiler metadata routes:

```sh
curl -fsS http://127.0.0.1:8080/__deep/manifest
curl -fsS http://127.0.0.1:8080/__deep/static-report
curl -fsS http://127.0.0.1:8080/__deep/tasks
curl -fsS http://127.0.0.1:8080/__deep/models
curl -fsS http://127.0.0.1:8080/__deep/pricing
curl -fsS http://127.0.0.1:8080/__deep/snapshot
```

## Redis-Backed Primitives

By default, the prototype uses in-memory maps for cache, queue, counter, and lock primitives. To run those primitives against Redis:

```sh
docker compose up -d redis
docker compose run --rm compiler run examples/chat.deep \
  --addr 0.0.0.0:8080 \
  --redis-url redis://redis:6379
```

With Redis configured, cache values are stored as Redis strings with declared TTLs, queues use FIFO Redis lists or sorted sets based on `sorted_by`, counters use atomic increments, and locks use token-checked `SET NX EX` acquisition.

Resolve a model through the runtime catalog:

```sh
curl -fsS http://127.0.0.1:8080/__deep/resolve-model/gpt-5
```

Look up pricing:

```sh
curl -fsS http://127.0.0.1:8080/__deep/price/chat/gpt-5
```

## Call Endpoints

Call the public chat endpoint:

```sh
curl -i -N -X POST http://127.0.0.1:8080/hacking_is_a_serious_crime
```

The prototype response is served as `text/event-stream` with HTTP chunked transfer. Its `data:` payload includes route metadata, selected model/provider metadata, and usage charge information.

Call the research endpoint without credentials:

```sh
curl -sS -o /tmp/deepapp-response.json -w '%{http_code}\n' \
  -X POST http://127.0.0.1:8080/start_deep_research
```

Expected status:

```text
401
```

Call it with an API key header:

```sh
curl -fsS -H 'Api-Key: dev-key' \
  -X POST http://127.0.0.1:8080/start_deep_research
```

Then check the generated task status:

```sh
curl -fsS http://127.0.0.1:8080/check_research_status/research-1
```

## Run Background Work Once

DeepApp task declarations compile into a task catalog. The prototype runtime can execute one deterministic task tick:

```sh
curl -fsS -X POST http://127.0.0.1:8080/__deep/task-tick
```

This exercises cron, daemon, and worker metadata without starting real long-running schedulers.

## Common Editing Loop

1. Edit `examples/chat.deep`, or create another `.deep` file.
2. Run:

   ```sh
   docker compose run --rm compiler check examples/chat.deep
   ```

3. Build artifacts:

   ```sh
   docker compose run --rm compiler build examples/chat.deep --out build
   ```

4. Run tests when changing compiler/runtime behavior:

   ```sh
   docker compose run --rm test
   docker compose run --rm fmt
   ```

5. Start the runtime:

   ```sh
   docker compose up runtime
   ```

## What To Expect From The Prototype

Implemented prototype behavior includes parsing, static checks, artifact generation, generated page HTML, endpoint routing, metadata routes, in-memory storage, Redis-backed cache/queues/counters/locks when configured, declared Redis TTL and sorted-queue metadata, snapshots, endpoint identity policy, rate limits, model resolution, pricing, and deterministic task ticks.

Not yet production-grade: arbitrary handler execution, real user/session authentication, pub/sub execution, full `with lock ...` handler interpretation, distributed rate-limit cleanup/policy, real billing, provider calls, reactive browser behavior, real scheduled background loops, and always-on durable storage services.
