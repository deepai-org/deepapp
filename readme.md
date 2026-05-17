To build a compiler for **DeepApp v2**, you have to recognize that it isn't just a programming language; it is a **tierless (or isomorphic) language combined with an Infrastructure-as-Code (IaC) engine**. 

Here is how you would practically build it, what you would base it on, and how it is categorized.

### 1. What kind of language is it?
*   **Tierless / Isomorphic:** Like Ur/Web, Darklang, or Winglang. The programmer writes one unified AST, and the compiler partitions it into client, server, and database code.
*   **Effect-Tracked:** The type system doesn't just track data types (`int`, `string`); it tracks *data provenance and security* (e.g., `@pii`, `@secret`, `@no_index`). This requires **taint analysis** and **Information Flow Control (IFC)** at compile time.
*   **Declarative Infrastructure:** It contains embedded configuration (cron, CDN, GPU workers) that dictates how the compiled binary is deployed.

### 2. What would you base it on?
Practically, you would not write a machine-code generator (like LLVM) from scratch. You would write a **transpiler/compiler frontend** that emits code for existing, highly optimized runtimes. 

*   **The Compiler Itself:** **Rust**. It has the best ecosystem for building fast, reliable compilers (using libraries like `rowan` for syntax trees, `salsa` for incremental compilation, and `miette` for beautiful error messages).
*   **The Server Target:** **Go (Golang)** or **Rust**. Go is highly recommended here because its goroutines map perfectly to DeepApp's `spawn`, `parallel`, and `channel` constructs. Its standard library handles HTTP and concurrency effortlessly.
*   **The Frontend Target:** **SolidJS + TypeScript**. DeepApp’s reactive UI (`state`, `view`, `on_load`) maps perfectly to SolidJS’s fine-grained reactivity and signals. The compiler would emit optimized JS.
*   **The Database Engine:** **PostgreSQL + Redis**. Even though the user doesn't see them, the compiler would generate Postgres schemas (for `data`) and Redis commands (for `cache`, `queue`, `lock`). Alternatively, for a truly single-binary approach, embed **SQLite (libsql)** and **BadgerDB** (for key-value/queues) directly into the Go/Rust server binary.
*   **The Infrastructure Target:** **Pulumi** or **Terraform (JSON)**. The compiler outputs a deployment plan that a cloud provider can execute.

### 3. How would you write the compiler? (The Pipeline)

**Phase 1: Parsing & AST Generation**
You would write a recursive descent parser (or use a parser combinator like `chumsky` in Rust) to turn the DeepApp syntax into an Abstract Syntax Tree (AST). 

**Phase 2: Semantic Analysis (The "Secret Sauce")**
This is where DeepApp actually works. You run multiple passes over the AST:
*   **Type Checking:** Standard type inference.
*   **Taint Analysis:** Ensure `@secret` never flows into `log` or `notify` nodes. Ensure `@pii` is masked.
*   **Database Query Validation:** Walk the AST for DB queries (`User |> where(...)`). Check the referenced fields against the `data` definitions. If a field lacks an `index` or has `@no_index`, throw a compiler error.
*   **Tier Splitting (Graph Partitioning):** Analyze the `page` and `component` blocks. If a function accesses the DB or secrets, tag it `SERVER`. If it accesses DOM/UI state, tag it `CLIENT`. If it's a pure function, tag it `SHARED`.

**Phase 3: Lowering & Code Generation**
You split the enriched AST into three distinct compilation targets:
1.  **Backend (Go/Rust):** 
    *   Generate HTTP handlers for `endpoint` blocks.
    *   Generate hidden REST/RPC endpoints for the frontend to call server-side functions.
    *   Convert `data` blocks into SQL `CREATE TABLE` statements (migrations) and generate an internal ORM.
2.  **Frontend (JS/Wasm):**
    *   Compile `view` blocks into DOM creation code (e.g., SolidJS JSX).
    *   Replace server-tagged function calls with `fetch()` calls to the generated backend RPC endpoints.
3.  **Infrastructure (JSON/YAML):**
    *   Extract `cron`, `cdn`, `worker`, and `services` blocks.
    *   Generate a `manifest.json` or Dockerfiles/Terraform code to orchestrate the deployment.

### Summary
To build this, you are essentially building a **smart router for code**. You write a Rust program that reads DeepApp syntax, enforces strict rules about how data moves, and then translates that single file into a Go backend, a SolidJS frontend, a Postgres schema, and a Terraform deployment script.


DeepApp v2 — The Whole Stack Is the Language

  ---
  Philosophy Shift

  v1 was a DSL that generated Python/TS/SQL. v2 is the language. You write DeepApp, and DeepApp is what runs. There is no
  "target language" underneath — the compiler emits machine code, bytecode, or whatever the runtime needs, but the
  programmer never sees or thinks in anything else.

  The key decisions:

  ┌───────────────────────────┬───────────────────────────────┬───────────────────┐
  │         Category          │ Absorbed into language/stdlib │ Remains a library │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ HTTP server               │ yes                           │ —                 │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ Database (relational)     │ yes (built-in storage engine) │ —                 │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ Redis-like caching/queues │ yes (built-in)                │ —                 │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ Image processing          │ yes                           │ —                 │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ PDF generation            │ yes                           │ —                 │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ HTML/CSS rendering        │ yes                           │ —                 │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ Reactive frontend         │ yes                           │ —                 │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ JSON/XML/SSE              │ yes                           │ —                 │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ Crypto/hashing            │ yes                           │ —                 │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ Email sending             │ yes (SMTP is stdlib)          │ —                 │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ Stripe                    │ —                             │ library           │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ OpenAI/Novita/Exa         │ —                             │ library           │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ AWS (S3, ECS, CloudWatch) │ —                             │ library           │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ RevenueCat                │ —                             │ library           │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ Sentry                    │ —                             │ library           │
  ├───────────────────────────┼───────────────────────────────┼───────────────────┤
  │ Google BigQuery           │ —                             │ library           │
  └───────────────────────────┴───────────────────────────────┴───────────────────┘

  ---
  Design Principles

  1. One language for everything. Server logic, database schema, queries, frontend UI, background jobs, deployment — all
  the same syntax, same types, same toolchain.
  2. The database is part of the program. You don't "connect" to a database. Your data declarations are the database.
  Queries are expressions, not strings.
  3. The frontend is part of the program. No separate TypeScript files. UI components are written in the same language,
  with the compiler deciding what runs server-side and what ships to the browser.
  4. No escape hatches. If something can't be expressed, the language is incomplete and must be extended. This forces the
  language to actually be good enough.
  5. Third-party services are libraries with typed contracts. They can fail, have latency, cost money. The type system
  knows this.

  ---
  1. Computation Model

  Values and Types

  -- Scalars
  x = 42
  name = "hello"
  price = $9.99
  active = true
  id = uuid()
  when = now()
  how_long = 30s

  -- Compounds
  colors = ["red", "green", "blue"]
  config = { timeout: 30s, retries: 3 }
  result = ok(42) | err("failed")

  -- The type system
  type Name = string max 190
  type Email = string @pii format email
  type Money = decimal places 2 currency USD
  type Duration = int unit seconds
  type Url = string format url
  type Json = any serializable

  Functions

  fn fibonacci(n: int) -> int {
    if n <= 1 { n }
    else { fibonacci(n - 1) + fibonacci(n - 2) }
  }

  fn greet(name: Name) -> string {
    "Hello, {name}!"
  }

  Pattern Matching

  fn describe(status: JobStatus) -> string {
    match status {
      pending        => "Waiting..."
      running(pct)   => "Running: {pct}%"
      completed(url) => "Done: {url}"
      failed(err)    => "Failed: {err}"
    }
  }

  Iteration

  for user in users where user.active {
    send_reminder(user)
  }

  items = users
    |> filter(_.active)
    |> map(_.email)
    |> take(100)

  Error Handling

  No exceptions. Results are values.

  fn divide(a: int, b: int) -> int or DivisionError {
    if b == 0 { fail DivisionError("division by zero") }
    a / b
  }

  -- Caller must handle
  result = divide(10, 0)
  match result {
    ok(n)  => print(n)
    err(e) => print(e.message)
  }

  -- Or propagate with ?
  fn compute(x: int) -> int or DivisionError {
    y = divide(x, 2)?
    y + 1
  }

  ---
  2. Data — The Database Is the Program

  There is no SQL. There is no ORM. Data declarations define both the schema and the storage engine.

  Declaring Data

  data User {
    id: pk
    email: Email unique
    username: string max 30 unique
    password_hash: string @secret
    created_at: datetime default now()
    locked: bool default false

    index email
    index username
  }

  data ChatSession {
    id: pk
    owner: User?
    client_info: ClientInfo?
    title: string max 500 default ""
    created_at: datetime default now() @no_index

    index owner
    index client_info
  }

  data ChatMessage {
    id: pk
    session: ChatSession
    role: "user" | "assistant"
    content: string
    attachments: [Attachment]
    created_at: datetime default now()

    index session
  }

  @no_index is not just documentation — it affects what queries the compiler allows.

  Queries Are Expressions

  -- Simple lookup
  user = User[id: 42]
  user = User[email: "foo@bar.com"]

  -- Filtered collections
  active_pros = User
    |> where(_.subscription.active)
    |> where(_.locked == false)

  -- Aggregation
  session_counts = ChatMessage
    |> where(_.session.id > max_pk(ChatSession) - 50000)
    |> group_by(_.session)
    |> count()

  -- Joins are explicit and warned
  -- The compiler knows table sizes from runtime statistics
  combined = ChatSession
    |> join(ChatMessage on _.id == ChatMessage.session.id)
    -- COMPILER WARNING: large-table join. Consider separate queries.

  Query Safety

  The compiler tracks which fields are indexed:

  -- OK: uses primary key range
  recent = ChatSession |> where(_.id > max_pk(ChatSession) - 50000)

  -- OK: uses indexed field
  by_owner = ChatSession |> where(_.owner == some_user)

  -- ERROR: created_at is @no_index
  bad = ChatSession |> where(_.created_at > now() - 1d)
  -- Compile error: DB021: ChatSession.created_at is not indexed.
  -- Use a primary-key range or add an index.

  Mutations

  -- Create
  new_user = User.create(
    email: "foo@bar.com",
    username: "foo",
    password_hash: hash("secret")
  )

  -- Update
  user.email = "new@bar.com"
  user.save()

  -- Bulk update
  User |> where(_.locked) |> where(_.created_at < now() - 365d) |> delete()

  -- Transactions
  transaction {
    wallet.balance -= cost
    InvoiceLineItem.create(user: user, amount: cost)
  }

  Migrations

  Schema changes are detected automatically:

  deep migrate --preview

  + ChatSession.pinned: bool default false
  ~ User.email: max 190 -> max 254
  - ChatLog.legacy_field

  Safe: yes (all additive or nullable)

  deep migrate --apply

  Dangerous changes require explicit approval in the source:

  data ChatSession {
    -- ...
    @migration(drop_column, approved: "2026-05-16 kevin")
    -- removed: legacy_field
  }

  ---
  3. Frontend — Same Language, Browser Target

  No TypeScript. No HTML templates. The UI is written in DeepApp and compiled to run in the browser.

  Pages

  page pricing at /pricing cache public {
    -- This entire block compiles to browser code
    -- The compiler knows it's public-cached: no user state allowed here

    view {
      h1 "Pricing"

      plans = [
        { name: "Free", price: $0, features: ["5 images/day", "Basic chat"] },
        { name: "Pro", price: $9.99, features: ["Unlimited", "All models", "Priority"] },
      ]

      row {
        for plan in plans {
          card {
            h2 plan.name
            price_display plan.price per "month"
            list plan.features
            button "Get Started" action navigate("/signup?plan={plan.name}")
          }
        }
      }

      -- User-specific content loads after page render
      on_load {
        status = fetch /api/me/pricing-status
        if status.is_pro {
          show badge("Current Plan") on plans[1]
        }
      }
    }
  }

  Interactive Components

  component ChatInput {
    state message: string = ""
    state sending: bool = false

    view {
      textarea bind message
        placeholder "Type a message..."
        on_submit { send() }

      button "Send"
        disabled (message.empty or sending)
        action send()
    }

    fn send() {
      sending = true
      response = stream POST /hacking_is_a_serious_crime
        body { messages: [{ role: "user", content: message }] }

      message = ""
      sending = false
      emit message_sent(response)
    }
  }

  Compile Targets

  The compiler decides what runs where:

  page dashboard at /dashboard cache private {
    -- SERVER: authentication check, initial data load
    user = require_login()
    initial_data = load_dashboard(user)

    -- CLIENT: reactive UI
    view {
      sidebar { ... }
      main_content {
        chat_panel(initial_data.recent_sessions)
      }
    }
  }

  The compiler splits this into:
  - Server: renders initial HTML with data
  - Client: hydrates interactive components

  No manual "this is server code" / "this is client code" boundary — the compiler infers it from what the code does (DB
  access = server, DOM manipulation = client, pure computation = either).

  Styling

  No CSS files. Styling is part of the component:

  component PricingCard {
    style {
      padding: 24
      border_radius: 12
      shadow: medium
      background: white

      on hover {
        shadow: large
        transform: translate_y(-2)
      }

      at mobile {
        padding: 16
      }
    }

    view {
      h2 plan.name style { font_size: 24, weight: bold }
      -- ...
    }
  }

  ---
  4. HTTP — Built Into the Language

  No framework. The language is the web server.

  Endpoints

  endpoint POST /hacking_is_a_serious_crime {
    identity: any
    rate_limit: 30/min by ip when anonymous

    request {
      messages: [{ role: string, content: string }]
      model: string default "gpt-4.1"
      session_uuid: uuid?
    }

    response: stream chunk {
      content: string
      done: bool
      thinking_time: duration?
    }

    handle {
      check_sensitivity(request.messages)
      model = resolve_model(request.model)

      stream for chunk in model.chat(request.messages) {
        yield { content: chunk.text, done: chunk.final }
      }

      finally {
        record_usage(identity, model)
      }
    }
  }

  Request/Response

  The language handles serialization, validation, content negotiation:

  endpoint GET /api/user/{user_id: int} {
    identity: api_key | logged_in

    response {
      id: int
      email: Email
      username: string
      subscription: { tier: string, active: bool }
    }

    handle {
      user = User[id: user_id] or fail NotFound
      { id: user.id, email: user.email, username: user.username,
        subscription: user.subscription_summary() }
    }
  }

  Streaming

  SSE is a native construct:

  stream for chunk in provider.chat(messages) {
    yield { content: chunk.delta, done: chunk.finished }
  }

  WebSockets are native:

  websocket /ws/voice {
    on connect(ctx) {
      ctx.user = authenticate(ctx.headers)
    }

    on message(ctx, audio_bytes: bytes) {
      text = speech_to_text(audio_bytes)
      response = chat(text)
      audio = text_to_speech(response)
      ctx.send(audio)
    }
  }

  ---
  5. Concurrency

  No threads, no async/await ceremony. The runtime handles it.

  -- Parallel execution
  (nsfw_result, oai_result) = parallel {
    detect_nsfw_regex(messages)
    detect_nsfw_oai(messages)
  } timeout 4s

  -- Background work
  spawn {
    send_welcome_email(user)
  }

  -- Channels
  channel jobs: ImageJob capacity 1000

  -- Producer
  jobs.send(new_job)

  -- Consumer
  for job in jobs.receive() {
    process(job)
  }

  The compiler decides the concurrency model (green threads, OS threads, event loop) based on the workload shape.

  ---
  6. Storage Engine — Redis Absorbed

  There is no separate Redis. The language has built-in constructs for everything Redis does:

  Caching

  cached fn expensive_query(user: User) -> DashboardData ttl 5m {
    -- computed once, cached automatically
    -- cache key derived from function name + arguments
    -- invalidated on ttl or manual invalidation
  }

  -- Manual cache
  cache user_session[session_id: uuid] ttl 1h {
    type: SessionData
  }

  user_session[my_id] = session_data
  data = user_session[my_id]  -- or none if expired

  Queues

  queue chat_tasks: ChatTask sorted_by created_at {
    capacity: 10000
    ttl: 1h
  }

  -- Push
  chat_tasks.push(task)

  -- Pop (blocks until available or timeout)
  task = chat_tasks.pop() timeout 5s

  -- Length
  depth = chat_tasks.length

  Locks

  lock billing[user_id: int] timeout 30s {
    -- guaranteed exclusive
    -- auto-released on exit or crash
    -- distributed across all instances
  }

  -- Usage
  with lock billing[user.id] {
    charge(user, amount)
  }

  Counters and Stats

  counter api_calls[ip: string, model: string] window 60s

  -- Increment
  api_calls[request.ip, model.name] += 1

  -- Read
  recent = api_calls[request.ip, model.name]
  if recent > 30 { fail RateLimited(retry_after: 60s) }

  Pub/Sub

  topic model_updates: ModelEvent

  -- Publish
  model_updates.publish(ModelEvent.scaled_up("sdxl", 5))

  -- Subscribe
  on model_updates receive event {
    log "Model {event.model} {event.action}"
  }

  ---
  7. AI Model Integration

  AI providers are libraries, but the patterns of using them are stdlib.

  Chat Models

  import openai
  import novita

  model_config chat_models {
    "gpt-5" {
      providers: [openai.chat, novita.chat]
      thinking: hidden
      reasoning_effort: medium
      max_tokens: 16000
      cost: 3 credits
    }

    "deepseek-reasoner" {
      providers: [novita.chat]
      thinking: think_tags
      strip_thinking: true
      cost: 2 credits
    }

    "gpt-4.1-nano" {
      providers: [openai.chat]
      thinking: none
      cost: 1 credit
      temperature: 0.7
    }
  }

  fn resolve_model(name: string) -> ChatModel {
    chat_models[name] or chat_models["gpt-4.1-nano"]
  }

  Provider Failover

  -- The language handles failover natively
  -- providers list = priority order, automatic failover on error
  response = model.chat(messages)
  -- tries openai first, falls back to novita if openai fails
  -- retries with exponential backoff per provider config

  Streaming with Thinking

  stream for chunk in model.chat(messages) {
    match chunk {
      thinking(text) => yield { thinking: text, content: "" }
      content(text)  => yield { thinking: "", content: text }
      done           => break
    }
  }

  ---
  8. Image/Video/Audio Processing — Absorbed

  No PIL, no wand, no ffmpeg bindings. Media processing is stdlib.

  -- Load and transform
  img = image.load(uploaded_bytes)
  thumb = img |> resize(width: 256) |> crop_center(256, 256)
  safe = img |> strip_exif() |> convert(format: webp, quality: 85)

  -- Analyze
  is_nsfw = img |> classify_nsfw()
  dimensions = img.size  -- { width: int, height: int }

  -- Generate from AI (provider is a library)
  import replicate
  generated = replicate.generate_image(prompt: "a sunset", model: "flux")

  -- Video
  video = video.load(url)
  gif = video |> clip(0s, 5s) |> to_gif(fps: 10, width: 320)
  thumbnail = video |> frame_at(2s) |> resize(width: 640)

  -- Audio
  audio = audio.load(uploaded_bytes)
  transcript = speech_to_text(audio)  -- stdlib wraps whisper or similar

  ---
  9. Identity and Auth — Built In

  identity RequestIdentity =
      api_key(key: UserApiKey, user: User)
    | logged_in(user: User, session: Session)
    | anonymous(client: ClientInfo)

  -- Authentication is a language construct
  fn authenticate(request: Request) -> RequestIdentity {
    if header "Api-Key" exists {
      key = UserApiKey[key: header("Api-Key")] or fail Unauthorized
      api_key(key: key, user: key.user)
    }
    else if request.session.user exists {
      logged_in(user: request.session.user, session: request.session)
    }
    else {
      client = get_or_create_client_info(request)
      anonymous(client: client)
    }
  }

  -- The identity dedup problem is a stdlib function
  fn resolve_owner(identity: RequestIdentity) -> User? {
    match identity {
      api_key(_, user)    => user
      logged_in(user, _)  => user
      anonymous(client)   => {
        -- if this client_info ever appeared with a logged-in user, return that user
        Session |> where(_.client_info == client and _.user != none)
                |> first()
                |> map(_.user)
      }
    }
  }

  -- Login/signup
  fn signup(email: Email, password: string) -> User or SignupError {
    if User[email: normalize_email(email)] exists {
      fail SignupError.email_taken
    }

    user = User.create(
      email: email,
      username: generate_username(),
      password_hash: hash(password)
    )

    send_verification_email(user)
    user
  }

  ---
  10. Billing — Language-Level Money

  -- Money is a built-in type with safety rules
  -- You cannot add dollars to credits, or forget currency

  data Wallet {
    user: User unique
    balance: Money default $0.00
  }

  data Subscription {
    user: User
    tier: free | pro_web | pro_mobile
    active: bool default true
    grandfathered: bool default false
    stripe_id: string?

    monthly_price: Money = match {
      tier == free         => $0.00
      grandfathered        => $4.99
      _                    => $9.99
    }

    can_charge: bool = tier == pro_web and user.has_payment_method

    unique (user) where active  -- only one active subscription per user
  }

  -- Pricing table
  pricing {
    chat {
      default:              $0.01
      "gpt-5":             $0.03
      "deepseek-reasoner": $0.02
    }

    image_generation {
      default:               $0.05
      "stable-diffusion-xl": $0.04
      "flux-proxy":          $0.08
    }
  }

  -- The charge function
  fn charge_for_usage(user: User, category: string, model: string) {
    cost = pricing[category][model]

    with lock billing[user.id] {
      wallet = Wallet[user: user]
      wallet.balance -= cost

      if wallet.balance < $0.00 {
        if user.subscription.auto_top_up_enabled {
          top_up(user)
        } else {
          user.locked = true
          user.save()
          notify errors "Locked {user.email}: insufficient balance"
        }
      }
    }
  }

  -- Auto top-up
  fn top_up(user: User) {
    import stripe

    amount = calculate_top_up_amount(user)

    stripe.charge(
      customer: user.profile.stripe_customer_id,
      amount: amount,
      idempotency: "topup:{user.id}:{current_cycle_id(user)}"
    )

    wallet = Wallet[user: user]
    wallet.balance += amount
    wallet.save()

    notify notifications "Auto top-up {amount} for {user.email}"
  }

  ---
  11. Notifications — Stdlib

  -- Channels are declared once
  notification_channels {
    errors:        slack when production, log when dev
    user_activity: slack when production, log when dev
    notifications: slack when production, log when dev
  }

  -- Usage is just a function call
  notify errors "Billing failed: {error}"
  notify user_activity "Locked {user.email}"
  notify notifications "New signup: {user.email}"

  -- The compiler verifies channel names exist
  notify billing_alerts "..."
  -- ERROR: NOTIFY002: unknown channel 'billing_alerts'

  ---
  12. Email — Stdlib

  fn send_verification_email(user: User) {
    token = user.profile.email_verification_token
    link = "https://deepai.org/verify_email?key={token}"

    email to user.email
      from "noreply@deepai.org"
      subject "DeepAI Email Verification"
      body text {
        Hello,

        Please verify your email address by clicking this link: {link}

        Thanks for using DeepAI!
        - DeepAI Team
      }
  }

  ---
  13. Cron and Daemons

  cron daily_billing schedule "0 3 * * *" resources { cpu: 4096, mem: 8192 } {
    healthcheck: "https://hc-ping.com/{secret HEALTHCHECK_BILLING}"

    with lock daily_billing timeout 30m {
      today = now().date

      for user in User |> where(_.subscription.active)
                       |> where(_.needs_invoice(today)) {
        cycle = current_billing_cycle(user)
        generate_invoice(user, cycle)
      }
    }

    on error(e) {
      notify errors "daily_billing failed: {e}"
    }
  }

  daemon chat_task_checker schedule "*/5 * * * *" runtime 280s {
    healthcheck: "https://hc-ping.com/{secret HEALTHCHECK_TASKS}"

    with lock chat_task_checker {
      loop {
        task = chat_tasks.pop() timeout 5s

        match task {
          some(t) => process_task(t)
          none    => sleep 5s
        }
      }
    }
  }

  The compiler enforces:
  - daemon runtime < schedule interval
  - lock is present
  - healthcheck is present
  - on error handler exists

  ---
  14. Services and Deployment

  -- Service topology
  services {
    web {
      endpoints: [pages, static_assets]
      cache_fronted: true
      instances: 2..10
    }

    chat {
      endpoints: [/hacking_is_a_serious_crime, /check_chat_task_status, /check_sensitivity]
      instances: 2..8
    }

    image {
      endpoints: [/api/*, /job-view-file/*]
      instances: 2..6
    }

    chat_history {
      endpoints: [/save_chat_session, /get_chat_session, /search_chat_history, ...]
      instances: 1..4
    }
  }

  -- The compiler derives which services are affected by a code change
  -- No manual "which services to deploy" reasoning needed

  Deploy Rules (Compiler-Enforced)

  deploy_rules {
    -- Migrations run before code that uses new schema
    migration_before_deploy: true

    -- CDN refresh required when templates/static change
    cdn_refresh_on: [pages, static_assets]

    -- Health check must pass before traffic shifts
    health_check: GET /ping expect 200

    -- Rollback on failure
    rollback: automatic
  }

  deep deploy

  The compiler analyzes the diff, determines affected services, orders migrations, and produces a deploy plan:

  Diff: apps/base/billing.deep (modified)

  Affected:
    services: web, chat, image, chat_history (shared billing code)
    crons: daily_billing, lock_check_daemon

  Plan:
    1. Run migration 0046 (adds Wallet.auto_top_up_cooldown)
    2. Deploy all services
    3. Restart affected crons
    4. No CDN refresh needed (no frontend changes)

  Proceed? [y/n]

  ---
  15. Privacy and Security — Type-Level

  -- PII is tracked through the type system
  data User {
    email: string @pii
    password_hash: string @secret
    ip_address: string @pii
  }

  -- Secrets cannot appear in logs or notifications
  notify errors "Key: {api_key.key}"
  -- ERROR: SEC002: @secret field cannot be interpolated

  -- PII requires explicit handling
  log "User logged in: {user.email}"
  -- ERROR: LOG004: @pii field in log. Use redact(user.email) or hash(user.email)

  -- Correct:
  log "User logged in: {hash(user.email)}"

  -- Data deletion (GDPR)
  fn delete_user_data(user: User) {
    -- The compiler knows all @pii fields across all data types
    -- and can generate a complete deletion plan
    purge_pii(user)
  }

  ---
  16. Testing

  Since there's no test suite currently (manual testing + healthchecks), the language provides contracts and invariants
  rather than traditional unit tests:

  -- Invariants are always-true statements the compiler verifies at boundaries
  invariant "wallet never negative without lock"
    for_all charge_for_usage calls:
      requires lock billing[user.id] held

  invariant "public pages never access user"
    for_all pages where cache == public:
      never reads User, Session, or Wallet

  invariant "cron jobs have healthchecks"
    for_all cron declarations:
      has healthcheck field

  -- Contracts on functions
  fn generate_invoice(user: User, cycle: BillingCycle) -> Invoice
    requires user.subscription.active
    requires cycle.end <= now()
    ensures result.total >= $0.00
    ensures result.line_items.length > 0 or result.is_zero_usage_statement
  {
    ...
  }

  -- Property-based exploration (replaces manual testing)
  explore "billing never double-charges" {
    given user = random User with subscription.active
    given cycle = current_billing_cycle(user)

    when generate_invoice(user, cycle) called twice {
      assert second_call returns already_invoiced
    }
  }

  ---
  17. File Storage — Stdlib

  -- File storage is a built-in concept
  storage images {
    backend: s3  -- implementation detail, could be local in dev
    bucket: "deepai-images"
    public_url: "https://images.deepai.org"
  }

  -- Upload
  path = images.store(image_bytes, name: "{uuid()}.webp")
  url = images.url(path)  -- "https://images.deepai.org/abc123.webp"

  -- Download
  data = images.load(path)

  ---
  18. GPU Workers

  worker stable_diffusion {
    image: "sdxl-worker:latest"
    providers: [vast, salad]

    queue: sorted_by submitted_at capacity 5000

    scale {
      metric: queue.length / 10
      min: 1
      max: 20
      cooldown: 2m
    }

    input {
      prompt: string
      width: int default 1024
      height: int default 1024
      steps: int default 30
    }

    output {
      image_url: Url
      seed: int
    }

    process(job) {
      -- This code runs on the GPU worker
      result = run_inference(job.prompt, job.width, job.height, job.steps)
      path = images.store(result.bytes, name: "{job.id}.webp")
      { image_url: images.url(path), seed: result.seed }
    }
  }

  -- Submit from an endpoint
  endpoint POST /api/text2img {
    identity: api_key | logged_in

    handle {
      user = require_owner(identity)

      charge_for_usage(user, "image_generation", "sdxl")

      job = stable_diffusion.submit(
        prompt: request.prompt,
        width: request.width,
        height: request.height
      )

      -- Return immediately with job ID, or wait
      match request.wait {
        true  => job.await(timeout: 120s)
        false => { job_id: job.id, status_url: "/job-status/{job.id}" }
      }
    }
  }

  ---
  19. CDN — Built Into the Hosting Model

  cdn {
    domain: "deepai.org"
    origin: services.web

    rules {
      -- All pages cached by default
      cache_all_pages: true

      -- These paths bypass cache
      bypass: ["/dashboard*", "/api/*"]

      -- Only GET/HEAD through CDN
      methods: [GET, HEAD]

      -- POST/PUT/DELETE must go to api domain
      api_domain: "api.deepai.org"
    }
  }

  -- The compiler enforces this everywhere:
  -- Any endpoint accepting POST is automatically routed to api_domain
  -- Any page accessing user state is automatically cache: private
  -- Any fetch() in frontend code auto-prefixes the correct domain

  The app_base_url pattern disappears. The compiler handles it:

  -- In a page component:
  on_load {
    status = fetch GET /api/me/status
    -- Compiler emits: fetch("https://api.deepai.org/api/me/status")
    -- in production, or fetch("/api/me/status") in dev
  }

  ---
  20. Complete Example: Chat Feature

  One file that defines the full chat feature — data, API, frontend, background tasks:

  module chat

  -- Data
  data ChatStyle {
    id: pk
    name: string max 100
    system_prompt: string
    image_url: Url?
    is_public: bool default true
    creator: User?
    popularity: int default 0

    index popularity
    index creator
  }

  data ChatSession {
    id: pk
    uuid: uuid unique default uuid()
    owner: User?
    client_info: ClientInfo?
    style: ChatStyle?
    title: string max 500 default ""
    created_at: datetime default now() @no_index

    index owner
    index client_info
    index uuid
  }

  data ChatMessage {
    id: pk
    session: ChatSession
    role: "user" | "assistant"
    content: string
    thinking: string?
    model: string
    created_at: datetime default now()

    index session
  }

  -- Background task for deep research / extended thinking
  queue research_tasks: ResearchTask sorted_by created_at {
    ttl: 1h
  }

  type ResearchTask {
    session_uuid: uuid
    messages: [{ role: string, content: string }]
    model: string
  }

  type ResearchStatus =
      pending
    | thinking(elapsed: duration)
    | streaming(content: string)
    | done(content: string)
    | failed(error: string)

  cached fn research_status[task_id: uuid] -> ResearchStatus ttl 1h

  -- Chat endpoint
  endpoint POST /hacking_is_a_serious_crime {
    identity: any
    rate_limit: 30/min by ip when anonymous

    request {
      messages: [{ role: string, content: string }]
      model: string default "gpt-4.1"
      style_id: int?
      session_uuid: uuid?
    }

    response: stream {
      content: string
      thinking: string?
      done: bool
    }

    handle {
      import openai
      import novita

      -- Sensitivity check (parallel)
      (regex_safe, oai_safe) = parallel {
        check_nsfw_regex(request.messages)
        check_nsfw_oai(request.messages)
      } timeout 4s

      if not regex_safe or not oai_safe {
        fail ContentViolation("Message flagged as inappropriate")
      }

      -- Resolve model
      model = resolve_model(request.model)

      -- Get or create session
      session = if request.session_uuid {
        ChatSession[uuid: request.session_uuid] or ChatSession.create(
          uuid: request.session_uuid,
          owner: resolve_owner(identity),
          client_info: get_client_info(request),
          style: request.style_id and ChatStyle[id: request.style_id]
        )
      }

      -- Stream response
      full_response = ""

      stream for chunk in model.chat(request.messages) {
        full_response += chunk.content
        yield { content: chunk.content, thinking: chunk.thinking, done: false }
      }

      yield { content: "", thinking: none, done: true }

      -- Save to history
      spawn {
        ChatMessage.create(
          session: session,
          role: "user",
          content: request.messages.last.content,
          model: request.model
        )
        ChatMessage.create(
          session: session,
          role: "assistant",
          content: full_response,
          model: request.model
        )
      }

      -- Record usage
      owner = resolve_owner(identity)
      if owner {
        charge_for_usage(owner, "chat", request.model)
      }
    }
  }

  -- Deep research endpoint
  endpoint POST /start_deep_research {
    identity: logged_in | api_key

    request {
      messages: [{ role: string, content: string }]
      session_uuid: uuid
    }

    handle {
      task_id = uuid()
      research_status[task_id] = pending

      research_tasks.push(ResearchTask {
        session_uuid: request.session_uuid,
        messages: request.messages,
        model: "deepseek-reasoner"
      })

      { task_id: task_id }
    }
  }

  endpoint GET /check_research_status/{task_id: uuid} {
    identity: any

    handle {
      research_status[task_id] or fail NotFound
    }
  }

  -- Frontend
  page chat at / {
    cache: public

    view {
      component ChatPage {
        state sessions: [ChatSession] = []
        state current_session: uuid? = none
        state messages: [ChatMessage] = []
        state input: string = ""
        state streaming: bool = false
        state model: string = "gpt-4.1"

        on_load {
          sessions = fetch GET /get_my_chat_sessions
        }

        view {
          row {
            sidebar {
              for session in sessions {
                session_item(session)
                  selected (session.uuid == current_session)
                  action { load_session(session.uuid) }
              }
            }

            main {
              message_list {
                for msg in messages {
                  chat_bubble(msg)
                }
              }

              ChatInput(
                value: input,
                model: model,
                on_send: send_message
              )
            }
          }
        }

        fn send_message() {
          streaming = true
          messages += [{ role: "user", content: input }]
          input = ""

          assistant_msg = { role: "assistant", content: "" }
          messages += [assistant_msg]

          stream from POST /hacking_is_a_serious_crime
            body { messages: messages, model: model, session_uuid: current_session }
            on chunk(c) {
              assistant_msg.content += c.content
            }
            on done {
              streaming = false
            }
        }

        fn load_session(uuid: uuid) {
          current_session = uuid
          messages = fetch GET /get_chat_session body { session_uuid: uuid }
        }
      }
    }
  }

  ---
  21. What the Compiler Does

  Given the above, deep build produces:

  1. A single binary (or runtime image) containing:
    - HTTP server (handles all endpoints)
    - Database engine (manages all data types, migrations, queries)
    - Cache/queue engine (manages all ephemeral state, locks, counters)
    - Background task runner (cron, daemons)
    - Frontend assets (HTML, JS, CSS — compiled from the page/component declarations)
    - Worker binaries (for GPU tasks)
  2. A deploy manifest describing:
    - Which services exist
    - How they route
    - How they scale
    - CDN rules
    - Health checks
  3. Static analysis reports:
    - All known footguns checked
    - All invariants verified
    - All @pii/@secret flows traced
    - All billing paths locked
    - All large-table queries validated

  ---
  22. What You Need to Know

  To build this app, you need to know:
  - DeepApp syntax
  - Your domain (billing rules, chat models, safety policies)
  - Your library APIs (Stripe, OpenAI, AWS)

  You don't need to know:
  - Python
  - TypeScript
  - SQL
  - HTML/CSS (well, you need to understand layout concepts)
  - Redis commands
  - Django conventions
  - Docker
  - nginx/CDN configuration
  - cron syntax

  ---
  23. The Core Bet, Revised

  v1's bet: "encode operational knowledge as compile errors."

  v2's bet: "the entire app is one coherent program, and the language is smart enough to turn it into distributed
  infrastructure."

  You write what the product does. The compiler figures out:
  - What runs on the server vs browser
  - What needs a database vs cache vs ephemeral memory
  - What needs a lock
  - What needs a background worker
  - What order to deploy
  - What to invalidate when something changes

  The motto:

  Write the product. The language handles the infrastructure.


