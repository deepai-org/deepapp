# DeepApp v2 Coverage Audit

Objective: implement DeepApp design v2 with a Docker-first workflow, TDD, and clean design.

## Concrete Deliverables

- Dockerized Rust compiler CLI named `deep`.
- Parser and AST inventory for v2 language declarations.
- Static analyzer for the core design rules that are currently enforceable from source text.
- Build artifact generator for deployment manifest, static report, runtime bundle description, and migration plan.
- MySQL-oriented SQL planning artifact for declared tables, indexes, indexed `where(...)` lookup/range query patterns, and primary-key `BETWEEN` scans.
- Ordered online migration planning metadata with DDL, safe additive column DDL, pre-deploy phase, lock-risk notes, resumability, checkpoint keys, and bail/resume policy.
- Executable runtime slice for compiled pages/endpoints, health checks, and compiler metadata routes.
- SSE/chunked-transfer runtime response path for endpoints declared with `response: stream`.
- Frontend asset compiler for page titles, component names, state declarations, and generated HTML.
- CDN-aware frontend metadata and browser bootstrap for cache policy, API base URL, and client-side page-data loading.
- In-memory runtime engine for declared data tables, indexed lookups, uniqueness, cache values, queues, and counters.
- Redis-backed runtime primitive layer for cache values with declared TTLs, FIFO/sorted-set queues, atomic counters, token-checked TTL locks, and topic publish when `REDIS_URL`/`--redis-url` is configured.
- Durable store snapshot support for rows, primary-key counters, cache, queues, and counters.
- Native handler slice that wires example endpoints into runtime cache, queue, counter, billing, and indexed store-query behavior.
- Endpoint policy runtime for `identity` checks and anonymous `rate_limit` enforcement.
- HTTP identity resolver for API-key row lookup, session-cookie cache lookup, and anonymous client fingerprint recording.
- Simple handler interpreter for endpoint `handle { { key: value } }` return objects, including string, bool, number, generated UUID placeholder, and path params.
- Model/provider catalog for `model_config` declarations with deterministic runtime resolution and provider fallback.
- Pricing catalog for `pricing` declarations with deterministic runtime usage charging.
- Task runner slice for cron, daemon, and worker declarations with deterministic runtime ticks.
- Task-level lock execution for cron, daemon, and worker ticks using parsed lock names and TTLs.
- Endpoint lock execution for routes that call functions containing `with lock ...`.
- Representative DeepApp v2 source file.
- Unit and CLI tests that encode compiler behavior before relying on implementation output.

## Evidence

- Docker workflow:
  - `Dockerfile` builds the release compiler image.
  - `docker-compose.yml` defines `compiler`, `runtime`, `redis`, `test`, and `fmt` services.
  - `Makefile` wraps Docker commands for build, test, check, build example, migration preview, and runtime start.
- TDD coverage:
  - `src/lib.rs` unit tests cover indexed query validation, unsafe `IN` query rejection, primary-key `BETWEEN` planning, secret/PII flow rejection, public page isolation, notification channel validation, cron contracts, deploy rules, construct inventory, and manifest routing.
  - Runtime tests cover `/ping`, page rendering, endpoint routing, metadata routes, 404s, and method rejection.
  - Frontend tests verify page declarations become generated HTML served by the runtime, with CDN API base URL, client-fetch data-loading metadata, browser page-data fetch bootstrap, and a `/__deep/page-data/{page}` JSON route.
  - Storage tests cover record creation/read, unique fields, indexed query enforcement, cache get/set, FIFO queues, counters, and lock acquire/release.
  - Redis tests cover real Redis cache get/set, sorted-set queue ordering, atomic counters, `SET NX EX` lock contention/release, and `PUBLISH` delivery to a Redis subscriber when `REDIS_URL` is configured.
  - Snapshot tests cover saving store state to JSON and restoring rows, cache entries, queue contents, and counters.
  - Handler tests cover research task creation/status, chat endpoint usage counters, and chat handler indexed store queries for sessions/messages.
  - Endpoint policy tests cover anonymous rejection for `logged_in | api_key`, logged-in/API-key acceptance, anonymous IP rate limits, and declared-window rollover.
  - HTTP identity tests cover API-key lookup through `UserApiKey`, session-cookie lookup through `user_session`, and anonymous `ClientInfo` fingerprint creation.
  - Handler interpreter tests cover parsing and evaluating simple endpoint return objects.
  - Model tests cover model catalog parsing, default model fallback, and provider failover.
  - Pricing tests cover catalog parsing, default model fallback, chat endpoint charge metadata, and usage cent counters.
  - Migration tests cover pre-deploy ordering, online/resumable metadata, checkpoint keys, generated DDL presence, online additive column DDL, and bail/resume policy metadata.
  - Task-runner tests cover cron/daemon ticks, worker job processing, task counters, task events, worker result queues, and skip behavior when a declared task lock is held.
  - Endpoint lock tests cover a route inheriting `billing[user.id]` from a called function and returning locked without mutating endpoint counters when the lock is held.
  - `tests/cli.rs` verifies the CLI writes all expected build artifacts.
- Language design coverage:
  - `examples/chat.deep` includes module, types, identity, model config, notification channels, user/API-key/client data, queue, cache, cached function, counter, topic, pricing, storage, functions, worker, endpoints, page, cron, daemon, services, CDN, deploy rules, and invariants.
  - `build/static-report.json` records counts for parsed language units.
  - `build/manifest.json` records services, routes, route kind/method, endpoint identity/rate-limit policies, CDN, health check, and rollback policy.
  - `build/manifest.json` records endpoint lock names and lock TTLs inherited from direct endpoint locks or called locked functions.
  - `build/migrations.json` records ordered pre-deploy migration steps with DDL, online/resumable flags, lock-risk notes, safe additive column DDL, checkpoint keys, and bail/resume policy.
  - `build/sql-plan.json` records MySQL table DDL, index DDL, indexed lookup plans, indexed range plans, and primary-key `BETWEEN` plans derived from `data` declarations and handler query expressions.
  - `build/frontend-assets.json` records page assets generated from `view` blocks, including cache policy, API base URL, and client-fetch data-loading bootstrap metadata.
  - `build/storage-catalog.json` records data schemas, fields, indexes, and privacy/security flags used by the runtime store.
  - `build/task-catalog.json` records cron, daemon, and worker execution metadata, including parsed lock names and lock TTLs.
  - `build/model-catalog.json` records model configs, providers, reasoning effort, and cost metadata.
  - `build/pricing-catalog.json` records category/model prices in cents.
  - `build/redis-catalog.json` records Redis primitive backends, TTLs, and sorted queue order fields derived from `queue`, `cache`, `cached fn`, `counter`, and `topic` declarations.
- Runtime coverage:
  - `deep run examples/chat.deep --addr 0.0.0.0:8080` serves compiled routes.
  - Docker smoke test verified `GET /ping`, `GET /__deep/static-report`, `GET /`, and `POST /hacking_is_a_serious_crime`.
  - Docker smoke test verified `POST /start_deep_research` followed by `GET /check_research_status/research-1`.
  - Docker smoke test verified `GET /__deep/tasks` and `POST /__deep/task-tick`.
  - Docker smoke test verified `GET /__deep/snapshot` after runtime mutations.
  - Docker smoke test verified `POST /__deep/publish/{topic}` through Redis pub/sub.
  - Docker smoke test verified interpreted fallback response for uncached `GET /check_research_status/{task_id}`.
  - Docker smoke test verified `GET /__deep/models`, `GET /__deep/resolve-model/{model}`, and chat endpoint model/provider metadata.
  - Docker smoke test verified `GET /__deep/pricing`, `GET /__deep/price/{category}/{model}`, and chat endpoint charge metadata.
  - Docker smoke test verified anonymous rejection and API-key acceptance for `/start_deep_research`, plus anonymous chat rate-limit rejection.
  - Docker smoke test verified `POST /hacking_is_a_serious_crime` uses `text/event-stream` and `Transfer-Encoding: chunked`.
  - Runtime can opt into Redis primitives with `deep run --redis-url redis://...`; Docker Compose wires the `runtime` service to the `redis` service through `REDIS_URL`.
- Verified commands:
  - `cargo test --locked`
  - `docker compose run --rm test`
  - `docker compose run --rm -e REDIS_URL=redis://redis:6379 test`
  - `docker compose run --rm fmt`
  - `docker compose build compiler`
  - `docker compose build compiler runtime`
  - `docker compose run --rm compiler check examples/chat.deep`
  - `docker compose run --rm compiler build examples/chat.deep --out build`
  - `docker compose run --rm compiler migrate examples/chat.deep --preview`
  - `docker compose up -d runtime` plus HTTP smoke checks, followed by `docker compose down`
  - `docker compose up -d runtime` with Redis-backed endpoint smoke checks, direct Redis key inspection, sorted-set type check, and TTL checks, followed by `docker compose down`
  - `docker compose up -d runtime` with a pre-held Redis task lock and `/__deep/task-tick` smoke check, followed by `docker compose down`
  - `docker compose up -d runtime` with a pre-held Redis endpoint billing lock and locked chat endpoint smoke check, followed by `docker compose down`

## Current Limits

This is a working compiler and runtime prototype, not a complete production implementation of every DeepApp v2 runtime promise. It does not yet interpret arbitrary DeepApp handler bodies beyond simple return objects plus native example handlers, emit machine code, provide an always-on durable storage service, render full reactive browser behavior from `view` blocks, execute real scheduled background loops, or call real providers such as Stripe/OpenAI/AWS. SQL and migration output are MySQL-oriented planning artifacts, not a live database backend, live migration runner, or complete optimizer. Streaming routes use SSE/chunked transfer, but their chunks are deterministic prototype payloads rather than provider-backed token streams. Frontend assets carry CDN/cache/API metadata and bootstrap credentialed page-data fetches outside cached HTML, but there is not yet a full reactive browser data-loading runtime. HTTP identity resolves API keys, session cookies, and anonymous client fingerprints through prototype data/cache lookups, but it is not a real Django/Allauth/Google Auth integration or durable user/session service. Redis can back cache values with declared TTLs, FIFO/sorted-set queues, counters, TTL locks, and topic publish. Native example handlers execute cache, queue, counter, billing, and indexed store-query operations, but arbitrary handler execution remains incomplete. Task ticks acquire parsed cron/daemon/worker locks before running, and endpoint routes can inherit locks from called functions. Rate limiting uses the configured counter backend, but without stale-bucket cleanup or a production policy engine. Pricing is represented as deterministic catalog lookup and usage counters, not a real billing ledger or payment integration. Those remain future implementation work beyond the compiler, HTTP runtime, SQL/migration planning artifacts, frontend asset, native example handlers, endpoint policy slice, simple handler interpreter, model/provider catalog, pricing slice, task-runner tick, in-memory storage/cache/queue, Redis primitive backend, and snapshot slice represented here.
