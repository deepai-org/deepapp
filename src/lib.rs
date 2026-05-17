use redis::Commands;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Program {
    pub module: Option<String>,
    pub data: BTreeMap<String, DataDecl>,
    pub pages: Vec<PageDecl>,
    pub endpoints: Vec<EndpointDecl>,
    pub crons: Vec<CronDecl>,
    pub daemons: Vec<DaemonDecl>,
    pub services: Vec<ServiceDecl>,
    pub cdn: Option<CdnDecl>,
    pub notification_channels: BTreeSet<String>,
    pub invariants: Vec<String>,
    pub declarations: Vec<LanguageDecl>,
    pub model_catalog: Vec<ModelSpec>,
    pub pricing_catalog: Vec<PricingRule>,
    pub deploy_rules: Option<DeployRules>,
    pub sources: Vec<SourceFinding>,
    pub redis_primitives: Vec<RedisPrimitiveSpec>,
    pub functions: Vec<FunctionDecl>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DataDecl {
    pub name: String,
    pub fields: BTreeMap<String, FieldDecl>,
    pub indexes: BTreeSet<String>,
    pub unique_constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldDecl {
    pub name: String,
    pub raw_type: String,
    pub pii: bool,
    pub secret: bool,
    pub no_index: bool,
    pub unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageDecl {
    pub name: String,
    pub path: String,
    pub cache: CacheMode,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CacheMode {
    Public,
    Private,
    Unspecified,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EndpointDecl {
    pub method: String,
    pub path: String,
    pub identity: Option<String>,
    pub rate_limit: Option<RateLimitDecl>,
    pub response_stream: bool,
    pub lock_name: Option<String>,
    pub lock_ttl_seconds: Option<u64>,
    pub handler_response: Option<HandlerResponse>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RateLimitDecl {
    pub limit: u32,
    pub window: String,
    pub key: String,
    pub condition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandlerResponse {
    pub fields: BTreeMap<String, HandlerValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HandlerValue {
    String(String),
    Number(i64),
    Bool(bool),
    Param(String),
    GeneratedUuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CronDecl {
    pub name: String,
    pub schedule: Option<String>,
    pub has_healthcheck: bool,
    pub has_lock: bool,
    pub lock_name: Option<String>,
    pub lock_ttl_seconds: Option<u64>,
    pub has_error_handler: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonDecl {
    pub name: String,
    pub schedule: Option<String>,
    pub runtime: Option<String>,
    pub has_healthcheck: bool,
    pub has_lock: bool,
    pub lock_name: Option<String>,
    pub lock_ttl_seconds: Option<u64>,
    pub has_error_handler: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServiceDecl {
    pub name: String,
    pub endpoints: Vec<String>,
    pub min_instances: Option<u32>,
    pub max_instances: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CdnDecl {
    pub domain: Option<String>,
    pub api_domain: Option<String>,
    pub bypass: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageDecl {
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeployRules {
    pub migration_before_deploy: bool,
    pub cdn_refresh_on: Vec<String>,
    pub health_check: Option<String>,
    pub rollback: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceFinding {
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FunctionDecl {
    pub name: String,
    pub lock_name: Option<String>,
    pub lock_ttl_seconds: Option<u64>,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BuildArtifacts {
    pub manifest: DeployManifest,
    pub report: StaticReport,
    pub runtime: RuntimeBundle,
    pub migrations: Vec<MigrationStep>,
    pub sql_plan: SqlPlan,
    pub storage_catalog: Vec<DataDecl>,
    pub frontend_assets: Vec<FrontendAsset>,
    pub task_catalog: Vec<TaskSpec>,
    pub model_catalog: Vec<ModelSpec>,
    pub pricing_catalog: Vec<PricingRule>,
    pub redis_catalog: Vec<RedisPrimitiveSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SqlPlan {
    pub dialect: String,
    pub tables: Vec<SqlTablePlan>,
    pub queries: Vec<SqlQueryPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SqlTablePlan {
    pub table: String,
    pub create_table: String,
    pub indexes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SqlQueryPlan {
    pub source: String,
    pub table: String,
    pub field: String,
    pub index: String,
    pub strategy: String,
    pub sql: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelSpec {
    pub config: String,
    pub name: String,
    pub providers: Vec<String>,
    pub reasoning_effort: Option<String>,
    pub cost: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelResolution {
    pub requested: String,
    pub selected_model: String,
    pub selected_provider: Option<String>,
    pub fallback_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PricingRule {
    pub category: String,
    pub model: String,
    pub cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RedisPrimitiveSpec {
    pub kind: String,
    pub name: String,
    pub ttl_seconds: Option<u64>,
    pub queue_order: Option<String>,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChargeResult {
    pub category: String,
    pub model: String,
    pub cents: i64,
    pub usage_count: i64,
    pub total_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeployManifest {
    pub module: Option<String>,
    pub services: Vec<ServicePlan>,
    pub routes: Vec<RoutePlan>,
    pub cdn: Option<CdnDecl>,
    pub health_check: String,
    pub rollback: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServicePlan {
    pub name: String,
    pub endpoints: Vec<String>,
    pub min_instances: u32,
    pub max_instances: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoutePlan {
    pub path: String,
    pub target: String,
    pub cache: CacheMode,
    pub method: Option<String>,
    pub kind: RouteKind,
    pub identity: Option<String>,
    pub rate_limit: Option<RateLimitDecl>,
    pub response_stream: bool,
    pub lock_name: Option<String>,
    pub lock_ttl_seconds: Option<u64>,
    pub handler_response: Option<HandlerResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RouteKind {
    Page,
    Endpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StaticReport {
    pub data_types: usize,
    pub endpoints: usize,
    pub pages: usize,
    pub crons: usize,
    pub daemons: usize,
    pub language_units: BTreeMap<String, usize>,
    pub invariants: Vec<String>,
    pub checked_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeBundle {
    pub contains_http_server: bool,
    pub contains_storage_engine: bool,
    pub contains_cache_queue_engine: bool,
    pub contains_task_runner: bool,
    pub contains_frontend_assets: bool,
    pub contains_worker_support: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationStep {
    pub table: String,
    pub action: String,
    pub order: u32,
    pub phase: String,
    pub online: bool,
    pub resumable: bool,
    pub lock_risk: String,
    pub ddl: Vec<String>,
    pub safe_column_add: bool,
    pub additive_ddl: Vec<String>,
    pub checkpoint_key: String,
    pub resume_policy: String,
    pub bail_out: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrontendAsset {
    pub page: String,
    pub path: String,
    pub cache: CacheMode,
    pub api_base_url: Option<String>,
    pub data_loading: String,
    pub title: String,
    pub components: Vec<String>,
    pub states: Vec<String>,
    pub html: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskSpec {
    pub name: String,
    pub kind: TaskKind,
    pub schedule: Option<String>,
    pub runtime: Option<String>,
    pub healthcheck: bool,
    pub lock: bool,
    pub lock_name: Option<String>,
    pub lock_ttl_seconds: Option<u64>,
    pub on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskKind {
    Cron,
    Daemon,
    Worker,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskRun {
    pub name: String,
    pub kind: TaskKind,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeResponse {
    pub status: u16,
    pub content_type: String,
    pub body: String,
    pub chunked: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeIdentity {
    Anonymous,
    LoggedIn,
    ApiKey,
}

#[derive(Debug, Clone)]
pub struct RuntimeApp {
    artifacts: BuildArtifacts,
    store: MemoryStore,
}

pub type JsonRecord = BTreeMap<String, serde_json::Value>;

#[derive(Debug, Clone)]
pub struct MemoryStore {
    inner: Arc<Mutex<StoreInner>>,
    redis: Option<RedisPrimitiveStore>,
    primitives: Vec<RedisPrimitiveSpec>,
}

#[derive(Debug, Clone)]
struct RedisPrimitiveStore {
    client: redis::Client,
    namespace: String,
}

#[derive(Debug, Clone)]
struct StoreInner {
    schemas: BTreeMap<String, DataDecl>,
    rows: BTreeMap<String, BTreeMap<u64, JsonRecord>>,
    next_pk: BTreeMap<String, u64>,
    cache: BTreeMap<String, serde_json::Value>,
    queues: BTreeMap<String, VecDeque<serde_json::Value>>,
    counters: BTreeMap<String, i64>,
    locks: BTreeMap<String, LockEntry>,
    topics: BTreeMap<String, Vec<serde_json::Value>>,
}

#[derive(Debug, Clone)]
struct LockEntry {
    token: String,
    expires_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreSnapshot {
    pub rows: BTreeMap<String, BTreeMap<u64, JsonRecord>>,
    pub next_pk: BTreeMap<String, u64>,
    pub cache: BTreeMap<String, serde_json::Value>,
    pub queues: BTreeMap<String, Vec<serde_json::Value>>,
    pub counters: BTreeMap<String, i64>,
    pub topics: BTreeMap<String, Vec<serde_json::Value>>,
}

impl RedisPrimitiveStore {
    fn new(redis_url: &str, namespace: &str) -> anyhow::Result<Self> {
        Ok(Self {
            client: redis::Client::open(redis_url)?,
            namespace: namespace.into(),
        })
    }

    fn connection(&self) -> redis::RedisResult<redis::Connection> {
        self.client.get_connection()
    }

    fn key(&self, kind: &str, name: &str) -> String {
        format!("deepapp:{}:{}:{}", self.namespace, kind, name)
    }

    fn ping(&self) -> anyhow::Result<()> {
        let mut connection = self.connection()?;
        redis::cmd("PING").query::<String>(&mut connection)?;
        Ok(())
    }

    fn cache_set(
        &self,
        key: &str,
        value: &serde_json::Value,
        ttl_seconds: Option<u64>,
    ) -> anyhow::Result<()> {
        let mut connection = self.connection()?;
        let encoded = serde_json::to_string(value)?;
        let redis_key = self.key("cache", key);
        if let Some(ttl) = ttl_seconds {
            connection.set_ex::<_, _, ()>(redis_key, encoded, ttl)?;
        } else {
            connection.set::<_, _, ()>(redis_key, encoded)?;
        }
        Ok(())
    }

    fn cache_get(&self, key: &str) -> anyhow::Result<Option<serde_json::Value>> {
        let mut connection = self.connection()?;
        let encoded: Option<String> = connection.get(self.key("cache", key))?;
        encoded
            .map(|value| serde_json::from_str(&value).map_err(Into::into))
            .transpose()
    }

    fn queue_push(
        &self,
        queue: &str,
        value: &serde_json::Value,
        spec: Option<&RedisPrimitiveSpec>,
    ) -> anyhow::Result<()> {
        let mut connection = self.connection()?;
        let encoded = serde_json::to_string(value)?;
        let redis_key = self.key("queue", queue);
        if spec.and_then(|spec| spec.queue_order.as_ref()).is_some() {
            let score = redis_queue_score(value, spec.and_then(|spec| spec.queue_order.as_deref()));
            redis::cmd("ZADD")
                .arg(&redis_key)
                .arg(score)
                .arg(encoded)
                .query::<usize>(&mut connection)?;
        } else {
            connection.rpush::<_, _, usize>(&redis_key, encoded)?;
        }
        if let Some(ttl) = spec.and_then(|spec| spec.ttl_seconds) {
            connection.expire::<_, ()>(&redis_key, ttl as i64)?;
        }
        Ok(())
    }

    fn queue_pop(
        &self,
        queue: &str,
        spec: Option<&RedisPrimitiveSpec>,
    ) -> anyhow::Result<Option<serde_json::Value>> {
        let mut connection = self.connection()?;
        let encoded: Option<String> = if spec.and_then(|spec| spec.queue_order.as_ref()).is_some() {
            let values: Vec<(String, f64)> = redis::cmd("ZPOPMIN")
                .arg(self.key("queue", queue))
                .arg(1)
                .query(&mut connection)?;
            values.into_iter().next().map(|(value, _)| value)
        } else {
            connection.lpop(self.key("queue", queue), None)?
        };
        encoded
            .map(|value| serde_json::from_str(&value).map_err(Into::into))
            .transpose()
    }

    fn counter_add(&self, key: &str, amount: i64) -> anyhow::Result<i64> {
        let mut connection = self.connection()?;
        Ok(connection.incr(self.key("counter", key), amount)?)
    }

    fn counter_get(&self, key: &str) -> anyhow::Result<i64> {
        let mut connection = self.connection()?;
        let value: Option<i64> = connection.get(self.key("counter", key))?;
        Ok(value.unwrap_or(0))
    }

    fn lock_acquire(&self, key: &str, token: &str, ttl_seconds: u64) -> anyhow::Result<bool> {
        let mut connection = self.connection()?;
        let reply: Option<String> = redis::cmd("SET")
            .arg(self.key("lock", key))
            .arg(token)
            .arg("NX")
            .arg("EX")
            .arg(ttl_seconds)
            .query(&mut connection)?;
        Ok(reply.as_deref() == Some("OK"))
    }

    fn lock_release(&self, key: &str, token: &str) -> anyhow::Result<bool> {
        let mut connection = self.connection()?;
        let deleted: i32 = redis::Script::new(
            "if redis.call('GET', KEYS[1]) == ARGV[1] then return redis.call('DEL', KEYS[1]) else return 0 end",
        )
        .key(self.key("lock", key))
        .arg(token)
        .invoke(&mut connection)?;
        Ok(deleted == 1)
    }

    fn topic_publish(&self, topic: &str, value: &serde_json::Value) -> anyhow::Result<i64> {
        let mut connection = self.connection()?;
        let encoded = serde_json::to_string(value)?;
        Ok(connection.publish(self.key("topic", topic), encoded)?)
    }
}

fn redis_queue_score(value: &serde_json::Value, order_field: Option<&str>) -> f64 {
    order_field
        .and_then(|field| value.get(field))
        .and_then(|value| {
            value.as_f64().or_else(|| {
                value
                    .as_str()
                    .and_then(|text| text.parse::<f64>().ok())
                    .or_else(|| Some(current_epoch_seconds() as f64))
            })
        })
        .unwrap_or_else(|| current_epoch_seconds() as f64)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum StoreError {
    #[error("STORE001: unknown data type '{data}'")]
    UnknownData { data: String },
    #[error("STORE002: unknown field '{data}.{field}'")]
    UnknownField { data: String, field: String },
    #[error("STORE003: {data}.{field} is not indexed")]
    UnindexedField { data: String, field: String },
    #[error("STORE004: duplicate unique value for {data}.{field}")]
    DuplicateUnique { data: String, field: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DeepError {
    #[error("PARSE001: could not parse declaration near line {line}: {text}")]
    Parse { line: usize, text: String },
    #[error("DB021: {data}.{field} is not indexed. Use a primary-key range or add an index.")]
    QueryOnUnindexedField { data: String, field: String },
    #[error("DB022: {data}.{field} uses IN. Use BETWEEN or indexed range scans instead.")]
    QueryUsesInList { data: String, field: String },
    #[error("SEC002: @secret field cannot be interpolated into log or notify: {data}.{field}")]
    SecretFlow { data: String, field: String },
    #[error("LOG004: @pii field in log. Use redact() or hash(): {data}.{field}")]
    PiiLog { data: String, field: String },
    #[error("PAGE001: public page '{page}' reads private data type '{data}'")]
    PublicPageReadsPrivateData { page: String, data: String },
    #[error("NOTIFY002: unknown channel '{channel}'")]
    UnknownNotificationChannel { channel: String },
    #[error("CRON001: cron '{name}' must declare healthcheck, lock, and on error handler")]
    InvalidCron { name: String },
    #[error(
        "DAEMON001: daemon '{name}' must declare healthcheck, lock, on error handler, schedule, and runtime"
    )]
    InvalidDaemon { name: String },
    #[error(
        "DEPLOY001: deploy_rules must set migration_before_deploy, health_check, and automatic rollback"
    )]
    InvalidDeployRules,
}

pub fn compile_source(source: &str) -> Result<BuildArtifacts, Vec<DeepError>> {
    let program = parse(source).map_err(|e| vec![e])?;
    analyze(&program)?;
    Ok(generate(&program))
}

pub fn compile_file(path: &Path) -> anyhow::Result<BuildArtifacts> {
    let source = fs::read_to_string(path)?;
    compile_source(&source).map_err(|errors| anyhow::anyhow!(format_errors(&errors)))
}

pub fn runtime_from_file(path: &Path) -> anyhow::Result<RuntimeApp> {
    Ok(RuntimeApp::new(compile_file(path)?))
}

pub fn runtime_from_file_with_redis(path: &Path, redis_url: &str) -> anyhow::Result<RuntimeApp> {
    RuntimeApp::with_redis(compile_file(path)?, redis_url)
}

pub fn format_errors(errors: &[DeepError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn parse(source: &str) -> Result<Program, DeepError> {
    let mut program = Program {
        module: None,
        data: BTreeMap::new(),
        pages: Vec::new(),
        endpoints: Vec::new(),
        crons: Vec::new(),
        daemons: Vec::new(),
        services: Vec::new(),
        cdn: None,
        notification_channels: BTreeSet::new(),
        invariants: Vec::new(),
        declarations: Vec::new(),
        model_catalog: Vec::new(),
        pricing_catalog: Vec::new(),
        deploy_rules: None,
        sources: Vec::new(),
        redis_primitives: Vec::new(),
        functions: Vec::new(),
    };

    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let line = strip_comment(lines[i]).trim().to_string();
        if line.is_empty() {
            i += 1;
            continue;
        }
        if let Some(name) = line.strip_prefix("module ") {
            program.module = Some(name.trim().to_string());
            i += 1;
        } else if line.starts_with("data ") {
            let (header, body, next) = capture_block(&lines, i)?;
            let decl = parse_data(&header, &body, i + 1)?;
            program.sources.push(SourceFinding {
                kind: "data".into(),
                name: decl.name.clone(),
            });
            program.data.insert(decl.name.clone(), decl);
            i = next;
        } else if line.starts_with("page ") {
            let (header, body, next) = capture_block(&lines, i)?;
            let decl = parse_page(&header, &body, i + 1)?;
            program.sources.push(SourceFinding {
                kind: "page".into(),
                name: decl.name.clone(),
            });
            program.pages.push(decl);
            i = next;
        } else if line.starts_with("endpoint ") {
            let (header, body, next) = capture_block(&lines, i)?;
            let decl = parse_endpoint(&header, &body, i + 1)?;
            program.sources.push(SourceFinding {
                kind: "endpoint".into(),
                name: format!("{} {}", decl.method, decl.path),
            });
            program.endpoints.push(decl);
            i = next;
        } else if line.starts_with("cron ") {
            let (header, body, next) = capture_block(&lines, i)?;
            let decl = parse_cron(&header, &body);
            program.crons.push(decl);
            i = next;
        } else if line.starts_with("daemon ") {
            let (header, body, next) = capture_block(&lines, i)?;
            let decl = parse_daemon(&header, &body);
            program.daemons.push(decl);
            i = next;
        } else if line.starts_with("services ") {
            let (_, body, next) = capture_block(&lines, i)?;
            program.services = parse_services(&body);
            i = next;
        } else if line.starts_with("cdn ") {
            let (_, body, next) = capture_block(&lines, i)?;
            program.cdn = Some(parse_cdn(&body));
            i = next;
        } else if line.starts_with("model_config ") {
            let (header, body, next) = capture_block(&lines, i)?;
            let name = header
                .split_whitespace()
                .nth(1)
                .unwrap_or("models")
                .to_string();
            program.sources.push(SourceFinding {
                kind: "model_config".into(),
                name: name.clone(),
            });
            program.declarations.push(LanguageDecl {
                kind: "model_config".into(),
                name: name.clone(),
            });
            program
                .model_catalog
                .extend(parse_model_config(&name, &body));
            i = next;
        } else if line.starts_with("pricing ") {
            let (_, body, next) = capture_block(&lines, i)?;
            program.sources.push(SourceFinding {
                kind: "pricing".into(),
                name: "pricing".into(),
            });
            program.declarations.push(LanguageDecl {
                kind: "pricing".into(),
                name: "pricing".into(),
            });
            program.pricing_catalog = parse_pricing(&body);
            i = next;
        } else if line.starts_with("deploy_rules ") {
            let (_, body, next) = capture_block(&lines, i)?;
            program.deploy_rules = Some(parse_deploy_rules(&body));
            i = next;
        } else if line.starts_with("notification_channels ") {
            let (_, body, next) = capture_block(&lines, i)?;
            program.notification_channels = parse_notification_channels(&body);
            i = next;
        } else if line.starts_with("queue ") {
            let (header, body, next) = if line.contains('{') {
                capture_block(&lines, i)?
            } else {
                (line.clone(), String::new(), i + 1)
            };
            let decl = parse_redis_primitive("queue", &header, &body);
            program.sources.push(SourceFinding {
                kind: "queue".into(),
                name: decl.name.clone(),
            });
            program.declarations.push(LanguageDecl {
                kind: "queue".into(),
                name: decl.name.clone(),
            });
            program.redis_primitives.push(decl);
            i = next;
        } else if line.starts_with("cache ") {
            let (header, body, next) = if line.contains('{') {
                capture_block(&lines, i)?
            } else {
                (line.clone(), String::new(), i + 1)
            };
            let decl = parse_redis_primitive("cache", &header, &body);
            program.sources.push(SourceFinding {
                kind: "cache".into(),
                name: decl.name.clone(),
            });
            program.declarations.push(LanguageDecl {
                kind: "cache".into(),
                name: decl.name.clone(),
            });
            program.redis_primitives.push(decl);
            i = next;
        } else if line.starts_with("cached fn ") {
            let decl = parse_redis_primitive("cached_fn", &line, "");
            program.sources.push(SourceFinding {
                kind: "cached_fn".into(),
                name: decl.name.clone(),
            });
            program.declarations.push(LanguageDecl {
                kind: "cached_fn".into(),
                name: decl.name.clone(),
            });
            program.redis_primitives.push(decl);
            i += 1;
        } else if line.starts_with("topic ") {
            let decl = parse_redis_primitive("topic", &line, "");
            program.sources.push(SourceFinding {
                kind: "topic".into(),
                name: decl.name.clone(),
            });
            program.declarations.push(LanguageDecl {
                kind: "topic".into(),
                name: decl.name.clone(),
            });
            program.redis_primitives.push(decl);
            i += 1;
        } else if line.starts_with("counter ") {
            let decl = parse_redis_primitive("counter", &line, "");
            program.sources.push(SourceFinding {
                kind: "counter".into(),
                name: decl.name.clone(),
            });
            program.declarations.push(LanguageDecl {
                kind: "counter".into(),
                name: decl.name.clone(),
            });
            program.redis_primitives.push(decl);
            i += 1;
        } else if line.starts_with("fn ") {
            let (header, body, next) = capture_block(&lines, i)?;
            let decl = parse_function(&header, &body);
            program.sources.push(SourceFinding {
                kind: "function".into(),
                name: decl.name.clone(),
            });
            program.declarations.push(LanguageDecl {
                kind: "function".into(),
                name: decl.name.clone(),
            });
            program.functions.push(decl);
            i = next;
        } else if line.starts_with("invariant ") {
            program.invariants.push(line);
            i += 1;
        } else if let Some(decl) = parse_language_decl(&line) {
            program.sources.push(SourceFinding {
                kind: decl.kind.clone(),
                name: decl.name.clone(),
            });
            program.declarations.push(decl);
            i += 1;
        } else {
            i += 1;
        }
    }

    Ok(program)
}

fn strip_comment(line: &str) -> &str {
    line.split_once("--").map_or(line, |(left, _)| left)
}

fn capture_block(lines: &[&str], start: usize) -> Result<(String, String, usize), DeepError> {
    let mut header = String::new();
    let mut body = String::new();
    let mut depth = 0i32;
    let mut seen_open = false;

    for (offset, raw) in lines[start..].iter().enumerate() {
        let line_no = start + offset + 1;
        let line = strip_comment(raw);
        if offset == 0 {
            header = line.trim().to_string();
        } else {
            body.push_str(line);
            body.push('\n');
        }
        for ch in line.chars() {
            match ch {
                '{' => {
                    depth += 1;
                    seen_open = true;
                }
                '}' => depth -= 1,
                _ => {}
            }
        }
        if seen_open && depth == 0 {
            return Ok((header, body, start + offset + 1));
        }
        if depth < 0 {
            return Err(DeepError::Parse {
                line: line_no,
                text: raw.to_string(),
            });
        }
    }

    Err(DeepError::Parse {
        line: start + 1,
        text: lines[start].to_string(),
    })
}

fn parse_data(header: &str, body: &str, line: usize) -> Result<DataDecl, DeepError> {
    let re = Regex::new(r"^data\s+([A-Za-z][A-Za-z0-9_]*)").unwrap();
    let name = re
        .captures(header)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| DeepError::Parse {
            line,
            text: header.to_string(),
        })?;

    let mut fields = BTreeMap::new();
    let mut indexes = BTreeSet::from(["id".to_string()]);
    let mut unique_constraints = Vec::new();

    for raw in body.lines() {
        let line = raw.trim();
        if line.is_empty() || line == "}" {
            continue;
        }
        if let Some(indexed) = line.strip_prefix("index ") {
            indexes.insert(indexed.trim().to_string());
            continue;
        }
        if line.starts_with("unique ") {
            unique_constraints.push(line.to_string());
            continue;
        }
        if let Some((field, rest)) = line.split_once(':') {
            let field = field.trim();
            let rest = rest.trim();
            let decl = FieldDecl {
                name: field.to_string(),
                raw_type: rest
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .trim_end_matches(',')
                    .to_string(),
                pii: rest.contains("@pii"),
                secret: rest.contains("@secret"),
                no_index: rest.contains("@no_index"),
                unique: rest.contains(" unique"),
            };
            if decl.unique || decl.raw_type == "pk" {
                indexes.insert(field.to_string());
            }
            fields.insert(field.to_string(), decl);
        }
    }

    Ok(DataDecl {
        name,
        fields,
        indexes,
        unique_constraints,
    })
}

fn parse_page(header: &str, body: &str, line: usize) -> Result<PageDecl, DeepError> {
    let re = Regex::new(r"^page\s+([A-Za-z][A-Za-z0-9_]*)\s+at\s+(\S+)(.*)$").unwrap();
    let caps = re.captures(header).ok_or_else(|| DeepError::Parse {
        line,
        text: header.to_string(),
    })?;
    let trailing = caps.get(3).map_or("", |m| m.as_str());
    let cache = if trailing.contains("cache public") || body.contains("cache: public") {
        CacheMode::Public
    } else if trailing.contains("cache private") || body.contains("cache: private") {
        CacheMode::Private
    } else {
        CacheMode::Unspecified
    };
    Ok(PageDecl {
        name: caps[1].to_string(),
        path: caps[2].to_string(),
        cache,
        body: body.to_string(),
    })
}

fn parse_endpoint(header: &str, body: &str, line: usize) -> Result<EndpointDecl, DeepError> {
    let re = Regex::new(r"^endpoint\s+([A-Z]+)\s+(\S+)").unwrap();
    let caps = re.captures(header).ok_or_else(|| DeepError::Parse {
        line,
        text: header.to_string(),
    })?;
    let identity = body
        .lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("identity:").map(|v| v.trim().to_string()));
    Ok(EndpointDecl {
        method: caps[1].to_string(),
        path: caps[2].to_string(),
        identity,
        rate_limit: parse_rate_limit(body),
        response_stream: body.contains("response: stream") || body.contains("response: stream "),
        lock_name: parse_lock_name(body),
        lock_ttl_seconds: parse_lock_ttl_seconds(body),
        handler_response: parse_handler_response(body),
        body: body.to_string(),
    })
}

fn parse_function(header: &str, body: &str) -> FunctionDecl {
    let name = header
        .split_whitespace()
        .nth(1)
        .and_then(|name| name.split_once('(').map(|(name, _)| name).or(Some(name)))
        .unwrap_or("unknown")
        .to_string();
    FunctionDecl {
        name,
        lock_name: parse_lock_name(body),
        lock_ttl_seconds: parse_lock_ttl_seconds(body),
        body: body.to_string(),
    }
}

fn parse_rate_limit(body: &str) -> Option<RateLimitDecl> {
    let re = Regex::new(
        r"rate_limit:\s*(\d+)/([A-Za-z0-9_]+)\s+by\s+([A-Za-z_][A-Za-z0-9_]*)(?:\s+when\s+([^\n]+))?",
    )
    .unwrap();
    let caps = re.captures(body)?;
    Some(RateLimitDecl {
        limit: caps[1].parse().ok()?,
        window: caps[2].to_string(),
        key: caps[3].to_string(),
        condition: caps.get(4).map(|m| m.as_str().trim().to_string()),
    })
}

fn parse_handler_response(body: &str) -> Option<HandlerResponse> {
    let handle_start = body.find("handle")?;
    let handle = &body[handle_start..];
    let response_re = Regex::new(r"\{\s*\{([^{}]+)\}\s*\}").unwrap();
    let caps = response_re.captures(handle)?;
    let mut fields = BTreeMap::new();
    for part in caps[1].split(',') {
        let Some((name, value)) = part.split_once(':') else {
            continue;
        };
        fields.insert(name.trim().to_string(), parse_handler_value(value.trim()));
    }
    Some(HandlerResponse { fields })
}

fn parse_handler_value(value: &str) -> HandlerValue {
    let value = value.trim();
    if value == "uuid()" {
        HandlerValue::GeneratedUuid
    } else if value == "true" {
        HandlerValue::Bool(true)
    } else if value == "false" {
        HandlerValue::Bool(false)
    } else if let Some(unquoted) = value.strip_prefix('"').and_then(|v| v.strip_suffix('"')) {
        HandlerValue::String(unquoted.to_string())
    } else if let Some(param) = value.strip_prefix("params.") {
        HandlerValue::Param(param.to_string())
    } else if let Ok(number) = value.parse() {
        HandlerValue::Number(number)
    } else {
        HandlerValue::String(value.to_string())
    }
}

fn parse_cron(header: &str, body: &str) -> CronDecl {
    let name = header
        .split_whitespace()
        .nth(1)
        .unwrap_or("unknown")
        .to_string();
    CronDecl {
        name,
        schedule: quoted_after(header, "schedule"),
        has_healthcheck: body.contains("healthcheck:"),
        has_lock: body.contains("with lock"),
        lock_name: parse_lock_name(body),
        lock_ttl_seconds: parse_lock_ttl_seconds(body),
        has_error_handler: body.contains("on error"),
    }
}

fn parse_daemon(header: &str, body: &str) -> DaemonDecl {
    let name = header
        .split_whitespace()
        .nth(1)
        .unwrap_or("unknown")
        .to_string();
    let runtime = Regex::new(r"runtime\s+([^\s{]+)")
        .unwrap()
        .captures(header)
        .map(|caps| caps[1].to_string());
    DaemonDecl {
        name,
        schedule: quoted_after(header, "schedule"),
        runtime,
        has_healthcheck: body.contains("healthcheck:"),
        has_lock: body.contains("with lock"),
        lock_name: parse_lock_name(body),
        lock_ttl_seconds: parse_lock_ttl_seconds(body),
        has_error_handler: body.contains("on error"),
    }
}

fn parse_lock_name(body: &str) -> Option<String> {
    Regex::new(r"with\s+lock\s+([A-Za-z][A-Za-z0-9_\-.\[\]]*)")
        .unwrap()
        .captures(body)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

fn parse_lock_ttl_seconds(body: &str) -> Option<u64> {
    let re = Regex::new(r"timeout\s+(\d+)\s*([smhd])").unwrap();
    let caps = re.captures(body)?;
    duration_seconds(caps.get(1)?.as_str().parse().ok()?, caps.get(2)?.as_str())
}

fn quoted_after(text: &str, key: &str) -> Option<String> {
    let re = Regex::new(&format!(r#"{}\s+"([^"]+)""#, key)).unwrap();
    re.captures(text).map(|caps| caps[1].to_string())
}

fn parse_services(body: &str) -> Vec<ServiceDecl> {
    let endpoints_re = Regex::new(r"endpoints:\s*\[(.*?)\]").unwrap();
    let instances_re = Regex::new(r"instances:\s*(\d+)\.\.(\d+)").unwrap();
    let mut services = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();
        if line.is_empty() || line == "}" {
            i += 1;
            continue;
        }
        if let (Some(open), Some(close)) = (line.find('{'), line.rfind('}')) {
            if close > open {
                let name = line[..open].trim();
                if name
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
                {
                    services.push(parse_service_parts(
                        name,
                        &line[open + 1..close],
                        &endpoints_re,
                        &instances_re,
                    ));
                    i += 1;
                    continue;
                }
            }
        }
        if let Some(name) = line.strip_suffix('{') {
            let name = name.trim();
            let mut service_body = String::new();
            i += 1;
            while i < lines.len() && lines[i].trim() != "}" {
                service_body.push_str(lines[i]);
                service_body.push('\n');
                i += 1;
            }
            services.push(parse_service_parts(
                name,
                &service_body,
                &endpoints_re,
                &instances_re,
            ));
        }
        i += 1;
    }

    services
}

fn parse_service_parts(
    name: &str,
    service_body: &str,
    endpoints_re: &Regex,
    instances_re: &Regex,
) -> ServiceDecl {
    let endpoints = endpoints_re
        .captures(service_body)
        .map(|endpoint_caps| {
            endpoint_caps[1]
                .split(',')
                .map(|part| part.trim().trim_matches('"').to_string())
                .filter(|part| !part.is_empty())
                .collect()
        })
        .unwrap_or_default();
    let (min_instances, max_instances) = instances_re
        .captures(service_body)
        .map(|c| (c[1].parse().ok(), c[2].parse().ok()))
        .unwrap_or((None, None));
    ServiceDecl {
        name: name.to_string(),
        endpoints,
        min_instances,
        max_instances,
    }
}

fn parse_cdn(body: &str) -> CdnDecl {
    let domain_re = Regex::new(r#"domain:\s*"([^"]+)""#).unwrap();
    let api_domain_re = Regex::new(r#"api_domain:\s*"([^"]+)""#).unwrap();
    let bypass_re = Regex::new(r#"bypass:\s*\[(.*?)\]"#).unwrap();
    CdnDecl {
        domain: domain_re.captures(body).map(|caps| caps[1].to_string()),
        api_domain: api_domain_re.captures(body).map(|caps| caps[1].to_string()),
        bypass: bypass_re
            .captures(body)
            .map(|caps| {
                caps[1]
                    .split(',')
                    .map(|part| part.trim().trim_matches('"').to_string())
                    .filter(|part| !part.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn parse_model_config(config: &str, body: &str) -> Vec<ModelSpec> {
    let mut models = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_body = String::new();

    for line in body.lines().map(str::trim) {
        if line.starts_with('"') && line.contains('{') && line.ends_with('}') {
            if let Some(name) = line.split('"').nth(1) {
                let inline_body = line
                    .split_once('{')
                    .and_then(|(_, rest)| rest.rsplit_once('}').map(|(inner, _)| inner))
                    .unwrap_or_default();
                models.push(parse_model_spec(config, name, inline_body));
            }
            continue;
        }
        if line.starts_with('"') && line.ends_with('{') {
            current_name = line.split('"').nth(1).map(std::string::ToString::to_string);
            current_body.clear();
            continue;
        }
        if line == "}" {
            if let Some(name) = current_name.take() {
                models.push(parse_model_spec(config, &name, &current_body));
            }
            continue;
        }
        if current_name.is_some() {
            current_body.push_str(line);
            current_body.push('\n');
        }
    }

    models
}

fn parse_model_spec(config: &str, name: &str, body: &str) -> ModelSpec {
    let providers_re = Regex::new(r"providers:\s*\[(.*?)\]").unwrap();
    let reasoning_re = Regex::new(r"reasoning_effort:\s*([A-Za-z0-9_-]+)").unwrap();
    let cost_re = Regex::new(r"cost:\s*([^\n}]+)").unwrap();
    let providers = providers_re
        .captures(body)
        .map(|caps| {
            caps[1]
                .split(',')
                .map(|provider| provider.trim().to_string())
                .filter(|provider| !provider.is_empty())
                .collect()
        })
        .unwrap_or_default();
    ModelSpec {
        config: config.to_string(),
        name: name.to_string(),
        providers,
        reasoning_effort: reasoning_re.captures(body).map(|caps| caps[1].to_string()),
        cost: cost_re
            .captures(body)
            .map(|caps| caps[1].trim().to_string()),
    }
}

fn parse_pricing(body: &str) -> Vec<PricingRule> {
    let price_re =
        Regex::new(r#"^("[^"]+"|[A-Za-z_][A-Za-z0-9_]*):\s*\$([0-9]+)\.([0-9]{2})"#).unwrap();
    let mut rules = Vec::new();
    let mut category: Option<String> = None;

    for line in body.lines().map(str::trim) {
        if line.is_empty() || line == "}" {
            if category.is_some() && line == "}" {
                category = None;
            }
            continue;
        }
        if line.ends_with('{') {
            category = Some(line.trim_end_matches('{').trim().to_string());
            continue;
        }
        let Some(category) = &category else {
            continue;
        };
        if let Some(caps) = price_re.captures(line) {
            let model = caps[1].trim_matches('"').to_string();
            let dollars = caps[2].parse::<i64>().unwrap_or_default();
            let cents = caps[3].parse::<i64>().unwrap_or_default();
            rules.push(PricingRule {
                category: category.clone(),
                model,
                cents: dollars * 100 + cents,
            });
        }
    }

    rules
}

fn parse_deploy_rules(body: &str) -> DeployRules {
    let health_re = Regex::new(r#"health_check:\s*([^\n]+)"#).unwrap();
    let rollback_re = Regex::new(r#"rollback:\s*([A-Za-z_]+)"#).unwrap();
    let cdn_re = Regex::new(r#"cdn_refresh_on:\s*\[(.*?)\]"#).unwrap();
    DeployRules {
        migration_before_deploy: body.contains("migration_before_deploy: true"),
        cdn_refresh_on: cdn_re
            .captures(body)
            .map(|caps| {
                caps[1]
                    .split(',')
                    .map(|part| part.trim().trim_matches('"').to_string())
                    .filter(|part| !part.is_empty())
                    .collect()
            })
            .unwrap_or_default(),
        health_check: health_re
            .captures(body)
            .map(|caps| caps[1].trim().to_string()),
        rollback: rollback_re.captures(body).map(|caps| caps[1].to_string()),
    }
}

fn parse_notification_channels(body: &str) -> BTreeSet<String> {
    body.lines()
        .filter_map(|line| {
            line.trim()
                .split_once(':')
                .map(|(name, _)| name.trim().into())
        })
        .collect()
}

fn parse_redis_primitive(kind: &str, header: &str, body: &str) -> RedisPrimitiveSpec {
    let name = match kind {
        "cached_fn" => Regex::new(r"^cached\s+fn\s+([A-Za-z][A-Za-z0-9_]*)")
            .unwrap()
            .captures(header)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string()),
        _ => Regex::new(&format!(r"^{}\s+([A-Za-z][A-Za-z0-9_]*)", kind))
            .unwrap()
            .captures(header)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str().to_string()),
    }
    .unwrap_or_else(|| kind.to_string());
    let combined = format!("{header}\n{body}");
    RedisPrimitiveSpec {
        kind: kind.into(),
        name,
        ttl_seconds: parse_ttl_seconds(&combined),
        queue_order: if kind == "queue" {
            Regex::new(r"sorted_by\s+([A-Za-z][A-Za-z0-9_]*)")
                .unwrap()
                .captures(&combined)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
        } else {
            None
        },
        backend: match kind {
            "topic" => "redis_pubsub",
            "queue" if combined.contains("sorted_by") => "redis_sorted_set",
            "queue" => "redis_list",
            "counter" => "redis_counter",
            _ => "redis_string",
        }
        .into(),
    }
}

fn parse_ttl_seconds(source: &str) -> Option<u64> {
    let re = Regex::new(r"ttl:?\s*(\d+)\s*([smhd])").unwrap();
    let caps = re.captures(source)?;
    let amount: u64 = caps.get(1)?.as_str().parse().ok()?;
    duration_seconds(amount, caps.get(2)?.as_str())
}

fn duration_seconds(amount: u64, unit: &str) -> Option<u64> {
    Some(match unit {
        "s" => amount,
        "m" => amount * 60,
        "h" => amount * 60 * 60,
        "d" => amount * 24 * 60 * 60,
        _ => return None,
    })
}

fn parse_language_decl(line: &str) -> Option<LanguageDecl> {
    let patterns = [
        ("cached fn", "cached_fn"),
        ("model_config", "model_config"),
        ("identity", "identity"),
        ("pricing", "pricing"),
        ("storage", "storage"),
        ("worker", "worker"),
        ("queue", "queue"),
        ("cache", "cache"),
        ("lock", "lock"),
        ("counter", "counter"),
        ("topic", "topic"),
        ("type", "type"),
        ("fn", "function"),
    ];
    for (prefix, kind) in patterns {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = rest
                .trim()
                .trim_start_matches('[')
                .split(|ch: char| ch.is_whitespace() || matches!(ch, ':' | '[' | '(' | '{' | '='))
                .find(|part| !part.is_empty())
                .unwrap_or(kind);
            return Some(LanguageDecl {
                kind: kind.to_string(),
                name: name.to_string(),
            });
        }
    }
    None
}

pub fn analyze(program: &Program) -> Result<(), Vec<DeepError>> {
    let mut errors = Vec::new();
    check_queries(program, &mut errors);
    check_sensitive_flows(program, &mut errors);
    check_public_pages(program, &mut errors);
    check_notifications(program, &mut errors);
    check_jobs(program, &mut errors);
    check_deploy_rules(program, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn all_bodies(program: &Program) -> String {
    let mut body = String::new();
    for page in &program.pages {
        body.push_str(&page.body);
    }
    for endpoint in &program.endpoints {
        body.push_str(&endpoint.body);
    }
    body
}

fn check_queries(program: &Program, errors: &mut Vec<DeepError>) {
    let source = all_bodies(program);
    for data in program.data.values() {
        for field in data.fields.values() {
            let direct_field = format!("_.{}", field.name);
            let indexed =
                (data.indexes.contains(&field.name) || field.raw_type == "pk") && !field.no_index;
            let query_on_field = source.lines().any(|line| {
                line.contains(&format!("{} |>", data.name))
                    && line.contains("where(")
                    && line.contains(&direct_field)
            });
            if !indexed && query_on_field {
                errors.push(DeepError::QueryOnUnindexedField {
                    data: data.name.clone(),
                    field: field.name.clone(),
                });
            }
            let in_query_on_field = source.lines().any(|line| {
                line.contains(&format!("{} |>", data.name))
                    && line.contains("where(")
                    && line.contains(&format!("{} in ", direct_field))
            });
            if in_query_on_field {
                errors.push(DeepError::QueryUsesInList {
                    data: data.name.clone(),
                    field: field.name.clone(),
                });
            }
        }
    }
}

fn compile_sql_plan(program: &Program) -> SqlPlan {
    SqlPlan {
        dialect: "mysql".into(),
        tables: program.data.values().map(compile_sql_table).collect(),
        queries: compile_sql_queries(program),
    }
}

fn compile_migrations(sql_plan: &SqlPlan) -> Vec<MigrationStep> {
    sql_plan
        .tables
        .iter()
        .enumerate()
        .map(|(idx, table)| {
            let mut ddl = vec![table.create_table.clone()];
            ddl.extend(table.indexes.iter().cloned());
            let additive_ddl = mysql_safe_column_adds(table);
            MigrationStep {
                table: table.table.clone(),
                action: "create_or_reconcile".into(),
                order: (idx + 1) as u32,
                phase: "pre_deploy".into(),
                online: true,
                resumable: true,
                lock_risk: "metadata_lock_only_for_new_table_or_online_additive_column_changes"
                    .into(),
                ddl,
                safe_column_add: true,
                additive_ddl,
                checkpoint_key: format!("migration:{}:create_or_reconcile", table.table),
                resume_policy: "checkpoint_before_each_ddl_statement".into(),
                bail_out: "stop_before_next_statement_and_keep_checkpoint".into(),
            }
        })
        .collect()
}

fn mysql_safe_column_adds(table: &SqlTablePlan) -> Vec<String> {
    table
        .create_table
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim().trim_end_matches(',');
            if !trimmed.starts_with('`') || trimmed.contains(" PRIMARY KEY ") {
                return None;
            }
            Some(format!(
                "ALTER TABLE `{}` ADD COLUMN {}, ALGORITHM=INPLACE, LOCK=NONE;",
                table.table, trimmed
            ))
        })
        .collect()
}

fn compile_sql_table(data: &DataDecl) -> SqlTablePlan {
    let mut columns = Vec::new();
    for field in data.fields.values() {
        columns.push(format!(
            "  `{}` {}{}",
            field.name,
            mysql_type(field),
            if field.raw_type == "pk" {
                " PRIMARY KEY AUTO_INCREMENT"
            } else {
                ""
            }
        ));
    }
    let create_table = format!(
        "CREATE TABLE `{}` (\n{}\n) ENGINE=InnoDB;",
        data.name,
        columns.join(",\n")
    );
    let indexes = data
        .indexes
        .iter()
        .filter(|field| field.as_str() != "id")
        .map(|field| {
            let unique = data
                .fields
                .get(field)
                .map(|decl| decl.unique)
                .unwrap_or(false);
            format!(
                "CREATE {}INDEX `idx_{}_{}` ON `{}` (`{}`);",
                if unique { "UNIQUE " } else { "" },
                data.name,
                field,
                data.name,
                field
            )
        })
        .collect();
    SqlTablePlan {
        table: data.name.clone(),
        create_table,
        indexes,
    }
}

fn mysql_type(field: &FieldDecl) -> &'static str {
    match field.raw_type.trim_end_matches('?') {
        "pk" => "BIGINT UNSIGNED",
        "int" => "BIGINT",
        "bool" => "BOOLEAN",
        "datetime" => "DATETIME",
        "uuid" => "CHAR(36)",
        raw if raw.starts_with('"') => "VARCHAR(64)",
        _ => "TEXT",
    }
}

fn compile_sql_queries(program: &Program) -> Vec<SqlQueryPlan> {
    let comparison_re =
        Regex::new(r"([A-Za-z][A-Za-z0-9_]*)\s*\|>\s*where\(_\.([A-Za-z][A-Za-z0-9_]*)\s*(==|>=|<=|>|<)\s*([^)]+)\)").unwrap();
    let between_re =
        Regex::new(r"([A-Za-z][A-Za-z0-9_]*)\s*\|>\s*where\(_\.([A-Za-z][A-Za-z0-9_]*)\s+between\s+(.+?)\s+and\s+(.+?)\)").unwrap();
    let source = all_bodies(program);
    let mut plans = Vec::new();
    for line in source.lines().map(str::trim) {
        if let Some(caps) = comparison_re.captures(line) {
            let table = caps[1].to_string();
            let field = caps[2].to_string();
            if !field_has_queryable_index(program, &table, &field) {
                continue;
            }
            let op = mysql_operator(&caps[3]);
            plans.push(SqlQueryPlan {
                source: line.to_string(),
                table: table.clone(),
                field: field.clone(),
                index: mysql_index_name(&table, &field),
                strategy: query_strategy(&field, op),
                sql: format!("SELECT * FROM `{table}` WHERE `{field}` {op} ?;"),
            });
            continue;
        }
        if let Some(caps) = between_re.captures(line) {
            let table = caps[1].to_string();
            let field = caps[2].to_string();
            if !field_has_queryable_index(program, &table, &field) {
                continue;
            }
            plans.push(SqlQueryPlan {
                source: line.to_string(),
                table: table.clone(),
                field: field.clone(),
                index: mysql_index_name(&table, &field),
                strategy: query_strategy(&field, "BETWEEN"),
                sql: format!("SELECT * FROM `{table}` WHERE `{field}` BETWEEN ? AND ?;"),
            });
        }
    }
    plans
}

fn field_has_queryable_index(program: &Program, table: &str, field: &str) -> bool {
    let Some(data) = program.data.get(table) else {
        return false;
    };
    let Some(field_decl) = data.fields.get(field) else {
        return false;
    };
    (data.indexes.contains(field) || field_decl.raw_type == "pk") && !field_decl.no_index
}

fn mysql_index_name(table: &str, field: &str) -> String {
    if field == "id" {
        "PRIMARY".into()
    } else {
        format!("idx_{table}_{field}")
    }
}

fn mysql_operator(op: &str) -> &'static str {
    match op {
        "==" => "=",
        ">=" => ">=",
        "<=" => "<=",
        ">" => ">",
        "<" => "<",
        _ => "=",
    }
}

fn query_strategy(field: &str, op: &str) -> String {
    if field == "id" && op != "=" {
        "pk_range_scan".into()
    } else if op == "=" {
        "indexed_lookup".into()
    } else {
        "indexed_range_scan".into()
    }
}

fn check_sensitive_flows(program: &Program, errors: &mut Vec<DeepError>) {
    let source = all_bodies(program);
    for data in program.data.values() {
        let aliases = aliases_for(&data.name, &source);
        for field in data.fields.values() {
            for alias in &aliases {
                let needle = format!("{{{alias}.{}}}", field.name);
                let log_or_notify = source
                    .lines()
                    .filter(|line| {
                        line.trim_start().starts_with("log ")
                            || line.trim_start().starts_with("notify ")
                    })
                    .any(|line| line.contains(&needle));
                if log_or_notify && field.secret {
                    errors.push(DeepError::SecretFlow {
                        data: data.name.clone(),
                        field: field.name.clone(),
                    });
                }
                if log_or_notify && field.pii {
                    errors.push(DeepError::PiiLog {
                        data: data.name.clone(),
                        field: field.name.clone(),
                    });
                }
            }
        }
    }
}

fn aliases_for(data_name: &str, source: &str) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    aliases.insert(data_name.to_lowercase());
    let re = Regex::new(&format!(r"([A-Za-z][A-Za-z0-9_]*)\s*=\s*{}\[", data_name)).unwrap();
    for caps in re.captures_iter(source) {
        aliases.insert(caps[1].to_string());
    }
    let param_re = Regex::new(&format!(r"([A-Za-z][A-Za-z0-9_]*)\s*:\s*{}\b", data_name)).unwrap();
    for caps in param_re.captures_iter(source) {
        aliases.insert(caps[1].to_string());
    }
    aliases
}

fn check_public_pages(program: &Program, errors: &mut Vec<DeepError>) {
    for page in &program.pages {
        if page.cache != CacheMode::Public {
            continue;
        }
        for data in program.data.keys() {
            if page.body.contains(&format!("{data}[")) || page.body.contains(&format!("{data} |>"))
            {
                errors.push(DeepError::PublicPageReadsPrivateData {
                    page: page.name.clone(),
                    data: data.clone(),
                });
            }
        }
    }
}

fn check_notifications(program: &Program, errors: &mut Vec<DeepError>) {
    let source = all_bodies(program);
    for line in source.lines().map(str::trim) {
        if let Some(rest) = line.strip_prefix("notify ") {
            let channel = rest.split_whitespace().next().unwrap_or_default();
            if !program.notification_channels.contains(channel) {
                errors.push(DeepError::UnknownNotificationChannel {
                    channel: channel.to_string(),
                });
            }
        }
    }
}

fn check_jobs(program: &Program, errors: &mut Vec<DeepError>) {
    for cron in &program.crons {
        if !(cron.has_healthcheck && cron.has_lock && cron.has_error_handler) {
            errors.push(DeepError::InvalidCron {
                name: cron.name.clone(),
            });
        }
    }
    for daemon in &program.daemons {
        if !(daemon.has_healthcheck
            && daemon.has_lock
            && daemon.has_error_handler
            && daemon.schedule.is_some()
            && daemon.runtime.is_some())
        {
            errors.push(DeepError::InvalidDaemon {
                name: daemon.name.clone(),
            });
        }
    }
}

fn check_deploy_rules(program: &Program, errors: &mut Vec<DeepError>) {
    if let Some(rules) = &program.deploy_rules {
        if !(rules.migration_before_deploy
            && rules.health_check.is_some()
            && rules.rollback.as_deref() == Some("automatic"))
        {
            errors.push(DeepError::InvalidDeployRules);
        }
    }
}

pub fn generate(program: &Program) -> BuildArtifacts {
    let services = if program.services.is_empty() {
        vec![ServicePlan {
            name: "web".into(),
            endpoints: program.endpoints.iter().map(|e| e.path.clone()).collect(),
            min_instances: 1,
            max_instances: 1,
        }]
    } else {
        program
            .services
            .iter()
            .map(|service| ServicePlan {
                name: service.name.clone(),
                endpoints: service.endpoints.clone(),
                min_instances: service.min_instances.unwrap_or(1),
                max_instances: service.max_instances.unwrap_or(1),
            })
            .collect()
    };
    let routes = program
        .pages
        .iter()
        .map(|page| RoutePlan {
            path: page.path.clone(),
            target: "services.web".into(),
            cache: page.cache.clone(),
            method: Some("GET".into()),
            kind: RouteKind::Page,
            identity: None,
            rate_limit: None,
            response_stream: false,
            lock_name: None,
            lock_ttl_seconds: None,
            handler_response: None,
        })
        .chain(program.endpoints.iter().map(|endpoint| {
            let function_lock = endpoint_function_lock(endpoint, &program.functions);
            RoutePlan {
                path: endpoint.path.clone(),
                target: route_target(&services, &endpoint.path),
                cache: CacheMode::Private,
                method: Some(endpoint.method.clone()),
                kind: RouteKind::Endpoint,
                identity: endpoint.identity.clone(),
                rate_limit: endpoint.rate_limit.clone(),
                response_stream: endpoint.response_stream,
                lock_name: endpoint.lock_name.clone().or_else(|| {
                    function_lock
                        .as_ref()
                        .and_then(|function| function.lock_name.clone())
                }),
                lock_ttl_seconds: endpoint
                    .lock_ttl_seconds
                    .or_else(|| function_lock.and_then(|function| function.lock_ttl_seconds)),
                handler_response: endpoint.handler_response.clone(),
            }
        }))
        .collect();
    let sql_plan = compile_sql_plan(program);
    let migrations = compile_migrations(&sql_plan);

    BuildArtifacts {
        manifest: DeployManifest {
            module: program.module.clone(),
            services,
            routes,
            cdn: program.cdn.clone(),
            health_check: "GET /ping expect 200".into(),
            rollback: "automatic".into(),
        },
        report: StaticReport {
            data_types: program.data.len(),
            endpoints: program.endpoints.len(),
            pages: program.pages.len(),
            crons: program.crons.len(),
            daemons: program.daemons.len(),
            language_units: language_unit_counts(program),
            invariants: program.invariants.clone(),
            checked_rules: vec![
                "indexed queries".into(),
                "secret and pii log flows".into(),
                "public page data access".into(),
                "notification channels".into(),
                "cron and daemon operational contracts".into(),
                "service routing manifest".into(),
                "endpoint identity and rate limit policies".into(),
                "deploy rule ordering".into(),
                "v2 construct inventory".into(),
            ],
        },
        runtime: RuntimeBundle {
            contains_http_server: !program.endpoints.is_empty() || !program.pages.is_empty(),
            contains_storage_engine: !program.data.is_empty(),
            contains_cache_queue_engine: true,
            contains_task_runner: !program.crons.is_empty() || !program.daemons.is_empty(),
            contains_frontend_assets: !program.pages.is_empty(),
            contains_worker_support: true,
        },
        migrations,
        sql_plan,
        storage_catalog: program.data.values().cloned().collect(),
        frontend_assets: compile_frontend_assets(&program.pages, program.cdn.as_ref()),
        task_catalog: compile_task_catalog(program),
        model_catalog: program.model_catalog.clone(),
        pricing_catalog: program.pricing_catalog.clone(),
        redis_catalog: program.redis_primitives.clone(),
    }
}

fn compile_task_catalog(program: &Program) -> Vec<TaskSpec> {
    let mut tasks = Vec::new();
    tasks.extend(program.crons.iter().map(|cron| TaskSpec {
        name: cron.name.clone(),
        kind: TaskKind::Cron,
        schedule: cron.schedule.clone(),
        runtime: None,
        healthcheck: cron.has_healthcheck,
        lock: cron.has_lock,
        lock_name: cron.lock_name.clone(),
        lock_ttl_seconds: cron.lock_ttl_seconds,
        on_error: cron.has_error_handler,
    }));
    tasks.extend(program.daemons.iter().map(|daemon| TaskSpec {
        name: daemon.name.clone(),
        kind: TaskKind::Daemon,
        schedule: daemon.schedule.clone(),
        runtime: daemon.runtime.clone(),
        healthcheck: daemon.has_healthcheck,
        lock: daemon.has_lock,
        lock_name: daemon.lock_name.clone(),
        lock_ttl_seconds: daemon.lock_ttl_seconds,
        on_error: daemon.has_error_handler,
    }));
    tasks.extend(
        program
            .declarations
            .iter()
            .filter(|decl| decl.kind == "worker")
            .map(|worker| TaskSpec {
                name: worker.name.clone(),
                kind: TaskKind::Worker,
                schedule: None,
                runtime: None,
                healthcheck: false,
                lock: true,
                lock_name: Some(worker.name.clone()),
                lock_ttl_seconds: Some(300),
                on_error: true,
            }),
    );
    tasks
}

fn endpoint_function_lock<'a>(
    endpoint: &EndpointDecl,
    functions: &'a [FunctionDecl],
) -> Option<&'a FunctionDecl> {
    functions.iter().find(|function| {
        function.lock_name.is_some() && endpoint.body.contains(&format!("{}(", function.name))
    })
}

fn compile_frontend_assets(pages: &[PageDecl], cdn: Option<&CdnDecl>) -> Vec<FrontendAsset> {
    pages
        .iter()
        .map(|page| {
            let title = extract_first_capture(&page.body, r#"h1\s+"([^"]+)""#)
                .unwrap_or_else(|| humanize_name(&page.name));
            let components = extract_named_declarations(&page.body, "component");
            let states = extract_named_declarations(&page.body, "state");
            let api_base_url = cdn.and_then(|cdn| {
                cdn.api_domain
                    .as_ref()
                    .map(|domain| format!("https://{domain}"))
            });
            let data_loading = match page.cache {
                CacheMode::Public | CacheMode::Private => "client_fetch".into(),
                CacheMode::Unspecified => "runtime_default".into(),
            };
            FrontendAsset {
                page: page.name.clone(),
                path: page.path.clone(),
                cache: page.cache.clone(),
                api_base_url,
                data_loading,
                title: title.clone(),
                components: components.clone(),
                states: states.clone(),
                html: render_frontend_html(page, &title, &components, &states, cdn),
            }
        })
        .collect()
}

fn extract_first_capture(source: &str, pattern: &str) -> Option<String> {
    Regex::new(pattern)
        .ok()?
        .captures(source)
        .and_then(|caps| caps.get(1))
        .map(|match_| match_.as_str().to_string())
}

fn extract_named_declarations(source: &str, keyword: &str) -> Vec<String> {
    let re = Regex::new(&format!(r"\b{}\s+([A-Za-z][A-Za-z0-9_]*)", keyword)).unwrap();
    let mut names = BTreeSet::new();
    for caps in re.captures_iter(source) {
        names.insert(caps[1].to_string());
    }
    names.into_iter().collect()
}

fn render_frontend_html(
    page: &PageDecl,
    title: &str,
    components: &[String],
    states: &[String],
    cdn: Option<&CdnDecl>,
) -> String {
    let component_attrs = components
        .iter()
        .map(|component| html_escape(component))
        .collect::<Vec<_>>()
        .join(",");
    let state_attrs = states
        .iter()
        .map(|state| html_escape(state))
        .collect::<Vec<_>>()
        .join(",");
    let api_base_url = cdn
        .and_then(|cdn| cdn.api_domain.as_ref())
        .map(|domain| format!("https://{domain}"))
        .unwrap_or_default();
    let cache_mode = match page.cache {
        CacheMode::Public => "public",
        CacheMode::Private => "private",
        CacheMode::Unspecified => "runtime_default",
    };
    let data_loading = match page.cache {
        CacheMode::Public | CacheMode::Private => "client_fetch",
        CacheMode::Unspecified => "runtime_default",
    };
    let data_url = format!("{}/__deep/page-data/{}", api_base_url, page.name);
    format!(
        "<!doctype html><html><head><title>{title}</title><meta name=\"deepapp-page\" content=\"{page_name}\"><meta name=\"deepapp-cache\" content=\"{cache_mode}\"><meta name=\"deepapp-data-loading\" content=\"{data_loading}\"></head><body><main data-route=\"{path}\" data-api-base=\"{api_base}\" data-cache=\"{cache_mode}\" data-data-loading=\"{data_loading}\" data-components=\"{components}\" data-states=\"{states}\"><h1>{title}</h1></main><script>window.__DEEPAPP__={{apiBase:\"{api_base}\",cache:\"{cache_mode}\",dataLoading:\"{data_loading}\",dataUrl:\"{data_url}\"}};if(window.__DEEPAPP__.dataLoading===\"client_fetch\"){{fetch(window.__DEEPAPP__.dataUrl,{{credentials:\"include\"}}).then(r=>r.json()).then(data=>{{window.__DEEPAPP__.data=data;document.dispatchEvent(new CustomEvent(\"deepapp:data\",{{detail:data}}));}});}}</script></body></html>",
        title = html_escape(title),
        page_name = html_escape(&page.name),
        path = html_escape(&page.path),
        api_base = html_escape(&api_base_url),
        data_url = html_escape(&data_url),
        cache_mode = cache_mode,
        data_loading = data_loading,
        components = component_attrs,
        states = state_attrs
    )
}

fn humanize_name(name: &str) -> String {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => "DeepApp".into(),
    }
}

fn language_unit_counts(program: &Program) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for decl in &program.declarations {
        *counts.entry(decl.kind.clone()).or_insert(0) += 1;
    }
    counts
}

fn route_target(services: &[ServicePlan], path: &str) -> String {
    let service = services
        .iter()
        .find(|service| {
            service.endpoints.iter().any(|pattern| {
                pattern == path
                    || pattern.ends_with('*') && path.starts_with(pattern.trim_end_matches('*'))
            })
        })
        .map(|service| service.name.as_str())
        .unwrap_or("web");
    format!("services.{service}")
}

impl RuntimeApp {
    pub fn new(artifacts: BuildArtifacts) -> Self {
        let store = MemoryStore::new_with_primitives(
            artifacts.storage_catalog.clone(),
            artifacts.redis_catalog.clone(),
        );
        Self { artifacts, store }
    }

    pub fn with_redis(artifacts: BuildArtifacts, redis_url: &str) -> anyhow::Result<Self> {
        let store = MemoryStore::with_redis(
            artifacts.storage_catalog.clone(),
            artifacts.redis_catalog.clone(),
            redis_url,
        )?;
        Ok(Self { artifacts, store })
    }

    pub fn store(&self) -> MemoryStore {
        self.store.clone()
    }

    pub fn handle(&self, method: &str, path: &str) -> RuntimeResponse {
        self.handle_as(method, path, RuntimeIdentity::Anonymous, "127.0.0.1")
    }

    pub fn handle_as(
        &self,
        method: &str,
        path: &str,
        identity: RuntimeIdentity,
        ip: &str,
    ) -> RuntimeResponse {
        let clean_path = path.split('?').next().unwrap_or(path);
        if method == "GET" && clean_path == "/ping" {
            return text_response(200, "ok\n");
        }
        if method == "GET" && clean_path == "/__deep/manifest" {
            return json_response(200, &self.artifacts.manifest);
        }
        if method == "GET" && clean_path == "/__deep/static-report" {
            return json_response(200, &self.artifacts.report);
        }
        if method == "GET" && clean_path == "/__deep/tasks" {
            return json_response(200, &self.artifacts.task_catalog);
        }
        if method == "GET" && clean_path == "/__deep/models" {
            return json_response(200, &self.artifacts.model_catalog);
        }
        if method == "GET" && clean_path == "/__deep/pricing" {
            return json_response(200, &self.artifacts.pricing_catalog);
        }
        if method == "GET" && clean_path.starts_with("/__deep/price/") {
            let parts: Vec<&str> = clean_path
                .trim_start_matches("/__deep/price/")
                .split('/')
                .collect();
            if parts.len() == 2 {
                return json_response(
                    200,
                    &price_for(&self.artifacts.pricing_catalog, parts[0], parts[1]),
                );
            }
        }
        if method == "GET" && clean_path.starts_with("/__deep/resolve-model/") {
            let requested = clean_path.trim_start_matches("/__deep/resolve-model/");
            return json_response(200, &self.resolve_model(requested, &BTreeSet::new()));
        }
        if method == "GET" && clean_path == "/__deep/snapshot" {
            return json_response(200, &self.store.snapshot());
        }
        if method == "GET" && clean_path.starts_with("/__deep/page-data/") {
            let page = clean_path.trim_start_matches("/__deep/page-data/");
            return page_data_response(page, &self.artifacts.frontend_assets);
        }
        if method == "POST" && clean_path == "/__deep/task-tick" {
            return json_response(200, &self.run_task_tick());
        }
        if method == "POST" && clean_path.starts_with("/__deep/publish/") {
            let topic = clean_path.trim_start_matches("/__deep/publish/");
            let subscribers = self.store.topic_publish(
                topic,
                serde_json::json!({
                    "topic": topic,
                    "source": "__deep/publish"
                }),
            );
            return json_response(
                200,
                &serde_json::json!({
                    "ok": true,
                    "topic": topic,
                    "subscribers": subscribers
                }),
            );
        }

        let Some(route) = self
            .artifacts
            .manifest
            .routes
            .iter()
            .find(|route| route_matches(&route.path, clean_path))
        else {
            return json_error(404, "route not found");
        };

        if route.method.as_deref() != Some(method) {
            return json_error(405, "method not allowed");
        }

        if let Some(error) = enforce_identity(route, identity) {
            return error;
        }
        if let Some(error) = enforce_rate_limit(route, &self.store, identity, ip) {
            return error;
        }

        match route.kind {
            RouteKind::Page => page_response(route, &self.artifacts.frontend_assets),
            RouteKind::Endpoint => endpoint_response(
                route,
                clean_path,
                &self.store,
                &self.artifacts.model_catalog,
                &self.artifacts.pricing_catalog,
            ),
        }
    }

    pub fn serve(&self, addr: &str) -> anyhow::Result<()> {
        self.serve_inner(addr, None)
    }

    pub fn serve_with_snapshot(&self, addr: &str, snapshot_path: &Path) -> anyhow::Result<()> {
        self.load_snapshot_if_exists(snapshot_path)?;
        self.serve_inner(addr, Some(snapshot_path))
    }

    pub fn load_snapshot_if_exists(&self, path: &Path) -> anyhow::Result<bool> {
        if path.exists() {
            self.store.load_snapshot(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn save_snapshot(&self, path: &Path) -> anyhow::Result<()> {
        self.store.save_snapshot(path)
    }

    fn serve_inner(&self, addr: &str, snapshot_path: Option<&Path>) -> anyhow::Result<()> {
        let listener = TcpListener::bind(addr)?;
        println!("DeepApp runtime listening on http://{addr}");
        for stream in listener.incoming() {
            let stream = stream?;
            self.handle_stream(stream)?;
            if let Some(path) = snapshot_path {
                self.save_snapshot(path)?;
            }
        }
        Ok(())
    }

    fn handle_stream(&self, mut stream: TcpStream) -> anyhow::Result<()> {
        let mut buffer = [0; 8192];
        let bytes = stream.read(&mut buffer)?;
        let request = String::from_utf8_lossy(&buffer[..bytes]);
        let mut request_line = request
            .lines()
            .next()
            .unwrap_or_default()
            .split_whitespace();
        let method = request_line.next().unwrap_or("GET");
        let path = request_line.next().unwrap_or("/");
        let ip = header_value(&request, "x-forwarded-for")
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("127.0.0.1");
        let identity = identity_from_http(&request, &self.store, ip);
        let response = self.handle_as(method, path, identity, ip);
        write_http_response(&mut stream, response)?;
        Ok(())
    }

    pub fn run_task_tick(&self) -> Vec<TaskRun> {
        self.artifacts
            .task_catalog
            .iter()
            .map(|task| run_task(task, &self.store))
            .collect()
    }

    pub fn resolve_model(
        &self,
        requested: &str,
        unavailable_providers: &BTreeSet<String>,
    ) -> ModelResolution {
        resolve_model_from_catalog(
            requested,
            &self.artifacts.model_catalog,
            unavailable_providers,
        )
    }

    pub fn charge_for_usage(&self, category: &str, model: &str) -> ChargeResult {
        charge_for_usage(
            &self.store,
            &self.artifacts.pricing_catalog,
            category,
            model,
        )
    }
}

fn price_for(catalog: &[PricingRule], category: &str, model: &str) -> Option<PricingRule> {
    catalog
        .iter()
        .find(|rule| rule.category == category && rule.model == model)
        .or_else(|| {
            catalog
                .iter()
                .find(|rule| rule.category == category && rule.model == "default")
        })
        .cloned()
}

fn charge_for_usage(
    store: &MemoryStore,
    catalog: &[PricingRule],
    category: &str,
    model: &str,
) -> ChargeResult {
    let price = price_for(catalog, category, model).unwrap_or(PricingRule {
        category: category.to_string(),
        model: "default".into(),
        cents: 0,
    });
    let usage_key = format!("usage:{category}:{model}:count");
    let total_key = format!("usage:{category}:{model}:cents");
    let usage_count = store.counter_add(usage_key, 1);
    let total_cents = store.counter_add(total_key, price.cents);
    ChargeResult {
        category: category.to_string(),
        model: model.to_string(),
        cents: price.cents,
        usage_count,
        total_cents,
    }
}

fn resolve_model_from_catalog(
    requested: &str,
    catalog: &[ModelSpec],
    unavailable_providers: &BTreeSet<String>,
) -> ModelResolution {
    let fallback = catalog
        .iter()
        .find(|model| model.name == "gpt-4.1-nano")
        .or_else(|| catalog.first());
    let selected = catalog
        .iter()
        .find(|model| model.name == requested)
        .or(fallback);
    let selected_provider = selected.and_then(|model| {
        model
            .providers
            .iter()
            .find(|provider| !unavailable_providers.contains(*provider))
            .cloned()
    });
    ModelResolution {
        requested: requested.to_string(),
        selected_model: selected
            .map(|model| model.name.clone())
            .unwrap_or_else(|| requested.to_string()),
        selected_provider,
        fallback_used: selected.map(|model| model.name.as_str()) != Some(requested),
    }
}

fn run_task(task: &TaskSpec, store: &MemoryStore) -> TaskRun {
    let lock_token = format!("task:{}:{}", task.name, current_epoch_seconds());
    let lock_name = task
        .lock_name
        .as_deref()
        .filter(|name| !name.is_empty())
        .or_else(|| task.lock.then_some(task.name.as_str()));
    if let Some(lock_name) = lock_name {
        let ttl_seconds = task.lock_ttl_seconds.unwrap_or(300);
        if !store.lock_acquire(lock_name, &lock_token, ttl_seconds) {
            return TaskRun {
                name: task.name.clone(),
                kind: task.kind.clone(),
                status: "locked".into(),
                detail: format!(
                    "task {} skipped because lock {lock_name} is held",
                    task.name
                ),
            };
        }
    }

    store.counter_add(format!("task:{}:runs", task.name), 1);
    let detail = match task.kind {
        TaskKind::Cron => {
            let message = format!("cron {} completed", task.name);
            store.queue_push(
                "task_events",
                serde_json::json!({ "task": task.name, "kind": "cron", "event": message }),
            );
            message
        }
        TaskKind::Daemon => {
            let message = format!("daemon {} tick completed", task.name);
            store.queue_push(
                "task_events",
                serde_json::json!({ "task": task.name, "kind": "daemon", "event": message }),
            );
            message
        }
        TaskKind::Worker => {
            let job = store.queue_pop(&task.name);
            let message = match job {
                Some(job) => {
                    let output = serde_json::json!({
                        "worker": task.name,
                        "job": job,
                        "status": "done"
                    });
                    store.queue_push(format!("{}:results", task.name), output);
                    "worker processed one job".to_string()
                }
                None => "worker idle".to_string(),
            };
            store.queue_push(
                "task_events",
                serde_json::json!({ "task": task.name, "kind": "worker", "event": message }),
            );
            message
        }
    };
    if let Some(lock_name) = lock_name {
        store.lock_release(lock_name, &lock_token);
    }
    TaskRun {
        name: task.name.clone(),
        kind: task.kind.clone(),
        status: "ok".into(),
        detail,
    }
}

impl MemoryStore {
    pub fn new(catalog: Vec<DataDecl>) -> Self {
        Self::new_with_primitives(catalog, Vec::new())
    }

    pub fn new_with_primitives(
        catalog: Vec<DataDecl>,
        primitives: Vec<RedisPrimitiveSpec>,
    ) -> Self {
        let schemas = catalog
            .into_iter()
            .map(|schema| (schema.name.clone(), schema))
            .collect::<BTreeMap<_, _>>();
        let rows = schemas
            .keys()
            .map(|name| (name.clone(), BTreeMap::new()))
            .collect();
        let next_pk = schemas.keys().map(|name| (name.clone(), 1)).collect();
        Self {
            inner: Arc::new(Mutex::new(StoreInner {
                schemas,
                rows,
                next_pk,
                cache: BTreeMap::new(),
                queues: BTreeMap::new(),
                counters: BTreeMap::new(),
                locks: BTreeMap::new(),
                topics: BTreeMap::new(),
            })),
            redis: None,
            primitives,
        }
    }

    pub fn with_redis(
        catalog: Vec<DataDecl>,
        primitives: Vec<RedisPrimitiveSpec>,
        redis_url: &str,
    ) -> anyhow::Result<Self> {
        let mut store = Self::new_with_primitives(catalog, primitives);
        let redis = RedisPrimitiveStore::new(redis_url, "runtime")?;
        redis.ping()?;
        store.redis = Some(redis);
        Ok(store)
    }

    pub fn primitive_backend(&self) -> &'static str {
        if self.redis.is_some() {
            "redis"
        } else {
            "memory"
        }
    }

    fn primitive_spec(&self, kind: &str, name: &str) -> Option<&RedisPrimitiveSpec> {
        self.primitives
            .iter()
            .find(|spec| spec.kind == kind && spec.name == name)
    }

    fn cache_ttl_seconds(&self, key: &str) -> Option<u64> {
        self.primitives.iter().find_map(|spec| {
            let cache_like = spec.kind == "cache" || spec.kind == "cached_fn";
            if cache_like && (key == spec.name || key.starts_with(&format!("{}:", spec.name))) {
                spec.ttl_seconds
            } else {
                None
            }
        })
    }

    pub fn create(&self, data: &str, mut record: JsonRecord) -> Result<u64, StoreError> {
        let mut inner = self.inner.lock().expect("memory store lock poisoned");
        let schema = inner
            .schemas
            .get(data)
            .ok_or_else(|| StoreError::UnknownData { data: data.into() })?
            .clone();

        for field in record.keys() {
            if !schema.fields.contains_key(field) {
                return Err(StoreError::UnknownField {
                    data: data.into(),
                    field: field.clone(),
                });
            }
        }

        for field in schema.fields.values().filter(|field| field.unique) {
            if let Some(value) = record.get(&field.name) {
                let duplicate = inner.rows[data]
                    .values()
                    .any(|existing| existing.get(&field.name) == Some(value));
                if duplicate {
                    return Err(StoreError::DuplicateUnique {
                        data: data.into(),
                        field: field.name.clone(),
                    });
                }
            }
        }

        let id = record
            .get("id")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| {
                let id = inner.next_pk[data];
                inner.next_pk.insert(data.into(), id + 1);
                id
            });
        record.insert("id".into(), serde_json::json!(id));
        inner.rows.get_mut(data).unwrap().insert(id, record);
        Ok(id)
    }

    pub fn get(&self, data: &str, id: u64) -> Result<Option<JsonRecord>, StoreError> {
        let inner = self.inner.lock().expect("memory store lock poisoned");
        if !inner.schemas.contains_key(data) {
            return Err(StoreError::UnknownData { data: data.into() });
        }
        Ok(inner.rows[data].get(&id).cloned())
    }

    pub fn where_eq(
        &self,
        data: &str,
        field: &str,
        expected: &serde_json::Value,
    ) -> Result<Vec<JsonRecord>, StoreError> {
        let inner = self.inner.lock().expect("memory store lock poisoned");
        let schema = inner
            .schemas
            .get(data)
            .ok_or_else(|| StoreError::UnknownData { data: data.into() })?;
        let Some(field_decl) = schema.fields.get(field) else {
            return Err(StoreError::UnknownField {
                data: data.into(),
                field: field.into(),
            });
        };
        if !schema.indexes.contains(field) || field_decl.no_index {
            return Err(StoreError::UnindexedField {
                data: data.into(),
                field: field.into(),
            });
        }
        Ok(inner.rows[data]
            .values()
            .filter(|record| record.get(field) == Some(expected))
            .cloned()
            .collect())
    }

    pub fn cache_set(&self, key: impl Into<String>, value: serde_json::Value) {
        let key = key.into();
        if let Some(redis) = &self.redis {
            let ttl_seconds = self.cache_ttl_seconds(&key);
            redis
                .cache_set(&key, &value, ttl_seconds)
                .expect("redis cache_set failed");
            return;
        }
        self.inner
            .lock()
            .expect("memory store lock poisoned")
            .cache
            .insert(key, value);
    }

    pub fn cache_get(&self, key: &str) -> Option<serde_json::Value> {
        if let Some(redis) = &self.redis {
            return redis.cache_get(key).expect("redis cache_get failed");
        }
        self.inner
            .lock()
            .expect("memory store lock poisoned")
            .cache
            .get(key)
            .cloned()
    }

    pub fn queue_push(&self, queue: impl Into<String>, value: serde_json::Value) {
        let queue = queue.into();
        if let Some(redis) = &self.redis {
            let spec = self.primitive_spec("queue", &queue);
            redis
                .queue_push(&queue, &value, spec)
                .expect("redis queue_push failed");
            return;
        }
        self.inner
            .lock()
            .expect("memory store lock poisoned")
            .queues
            .entry(queue)
            .or_default()
            .push_back(value);
    }

    pub fn queue_pop(&self, queue: &str) -> Option<serde_json::Value> {
        if let Some(redis) = &self.redis {
            let spec = self.primitive_spec("queue", queue);
            return redis
                .queue_pop(queue, spec)
                .expect("redis queue_pop failed");
        }
        self.inner
            .lock()
            .expect("memory store lock poisoned")
            .queues
            .get_mut(queue)
            .and_then(VecDeque::pop_front)
    }

    pub fn counter_add(&self, key: impl Into<String>, amount: i64) -> i64 {
        let key = key.into();
        if let Some(redis) = &self.redis {
            return redis
                .counter_add(&key, amount)
                .expect("redis counter_add failed");
        }
        let mut inner = self.inner.lock().expect("memory store lock poisoned");
        let value = inner.counters.entry(key).or_insert(0);
        *value += amount;
        *value
    }

    pub fn counter_get(&self, key: &str) -> i64 {
        if let Some(redis) = &self.redis {
            return redis.counter_get(key).expect("redis counter_get failed");
        }
        *self
            .inner
            .lock()
            .expect("memory store lock poisoned")
            .counters
            .get(key)
            .unwrap_or(&0)
    }

    pub fn lock_acquire(&self, key: &str, token: &str, ttl_seconds: u64) -> bool {
        if let Some(redis) = &self.redis {
            return redis
                .lock_acquire(key, token, ttl_seconds)
                .expect("redis lock_acquire failed");
        }
        let mut inner = self.inner.lock().expect("memory store lock poisoned");
        let now = current_epoch_seconds();
        if inner
            .locks
            .get(key)
            .is_some_and(|entry| entry.expires_at <= now)
        {
            inner.locks.remove(key);
        }
        if inner.locks.contains_key(key) {
            return false;
        }
        inner.locks.insert(
            key.into(),
            LockEntry {
                token: token.into(),
                expires_at: now + ttl_seconds,
            },
        );
        true
    }

    pub fn lock_release(&self, key: &str, token: &str) -> bool {
        if let Some(redis) = &self.redis {
            return redis
                .lock_release(key, token)
                .expect("redis lock_release failed");
        }
        let mut inner = self.inner.lock().expect("memory store lock poisoned");
        if inner
            .locks
            .get(key)
            .is_some_and(|entry| entry.token == token)
        {
            inner.locks.remove(key);
            return true;
        }
        false
    }

    pub fn topic_publish(&self, topic: impl Into<String>, value: serde_json::Value) -> i64 {
        let topic = topic.into();
        if let Some(redis) = &self.redis {
            return redis
                .topic_publish(&topic, &value)
                .expect("redis topic_publish failed");
        }
        let mut inner = self.inner.lock().expect("memory store lock poisoned");
        let messages = inner.topics.entry(topic).or_default();
        messages.push(value);
        messages.len() as i64
    }

    pub fn snapshot(&self) -> StoreSnapshot {
        let inner = self.inner.lock().expect("memory store lock poisoned");
        StoreSnapshot {
            rows: inner.rows.clone(),
            next_pk: inner.next_pk.clone(),
            cache: inner.cache.clone(),
            queues: inner
                .queues
                .iter()
                .map(|(name, queue)| (name.clone(), queue.iter().cloned().collect()))
                .collect(),
            counters: inner.counters.clone(),
            topics: inner.topics.clone(),
        }
    }

    pub fn restore(&self, snapshot: StoreSnapshot) {
        let mut inner = self.inner.lock().expect("memory store lock poisoned");
        inner.rows = snapshot.rows;
        inner.next_pk = snapshot.next_pk;
        inner.cache = snapshot.cache;
        inner.queues = snapshot
            .queues
            .into_iter()
            .map(|(name, queue)| (name, VecDeque::from(queue)))
            .collect();
        inner.counters = snapshot.counters;
        inner.topics = snapshot.topics;
    }

    pub fn save_snapshot(&self, path: &Path) -> anyhow::Result<()> {
        fs::write(path, serde_json::to_string_pretty(&self.snapshot())?)?;
        Ok(())
    }

    pub fn load_snapshot(&self, path: &Path) -> anyhow::Result<()> {
        let snapshot = serde_json::from_str(&fs::read_to_string(path)?)?;
        self.restore(snapshot);
        Ok(())
    }
}

fn route_matches(pattern: &str, path: &str) -> bool {
    if pattern == path {
        return true;
    }
    let pattern_parts: Vec<&str> = pattern.trim_matches('/').split('/').collect();
    let path_parts: Vec<&str> = path.trim_matches('/').split('/').collect();
    pattern_parts.len() == path_parts.len()
        && pattern_parts
            .iter()
            .zip(path_parts)
            .all(|(pattern, value)| {
                pattern.starts_with('{') && pattern.ends_with('}') || *pattern == value
            })
}

fn page_response(route: &RoutePlan, assets: &[FrontendAsset]) -> RuntimeResponse {
    if let Some(asset) = assets.iter().find(|asset| asset.path == route.path) {
        return RuntimeResponse {
            status: 200,
            content_type: "text/html; charset=utf-8".into(),
            body: asset.html.clone(),
            chunked: false,
        };
    }
    RuntimeResponse {
        status: 200,
        content_type: "text/html; charset=utf-8".into(),
        body: format!(
            "<!doctype html><html><head><title>DeepApp</title></head><body><main data-route=\"{}\" data-target=\"{}\"><h1>DeepApp page {}</h1></main></body></html>",
            html_escape(&route.path),
            html_escape(&route.target),
            html_escape(&route.path)
        ),
        chunked: false,
    }
}

fn page_data_response(page: &str, assets: &[FrontendAsset]) -> RuntimeResponse {
    let Some(asset) = assets.iter().find(|asset| asset.page == page) else {
        return json_error(404, "page data not found");
    };
    let states = asset
        .states
        .iter()
        .map(|state| (state.clone(), serde_json::json!([])))
        .collect::<serde_json::Map<_, _>>();
    json_response(
        200,
        &serde_json::json!({
            "page": asset.page,
            "path": asset.path,
            "cache": asset.cache,
            "data_loading": asset.data_loading,
            "states": states
        }),
    )
}

fn enforce_identity(route: &RoutePlan, identity: RuntimeIdentity) -> Option<RuntimeResponse> {
    let Some(policy) = route.identity.as_deref() else {
        return None;
    };
    if route.kind == RouteKind::Page || policy_allows_identity(policy, identity) {
        return None;
    }
    Some(json_error(401, "authentication required"))
}

fn policy_allows_identity(policy: &str, identity: RuntimeIdentity) -> bool {
    let normalized = policy.replace(' ', "");
    normalized == "any"
        || match identity {
            RuntimeIdentity::Anonymous => normalized.contains("anonymous"),
            RuntimeIdentity::LoggedIn => normalized.contains("logged_in"),
            RuntimeIdentity::ApiKey => normalized.contains("api_key"),
        }
}

fn enforce_rate_limit(
    route: &RoutePlan,
    store: &MemoryStore,
    identity: RuntimeIdentity,
    ip: &str,
) -> Option<RuntimeResponse> {
    enforce_rate_limit_at(route, store, identity, ip, current_epoch_seconds())
}

fn enforce_rate_limit_at(
    route: &RoutePlan,
    store: &MemoryStore,
    identity: RuntimeIdentity,
    ip: &str,
    now_seconds: u64,
) -> Option<RuntimeResponse> {
    let Some(limit) = &route.rate_limit else {
        return None;
    };
    if !rate_limit_applies(limit, identity) {
        return None;
    }
    let subject = match limit.key.as_str() {
        "ip" => ip.to_string(),
        other => format!("{identity:?}:{other}"),
    };
    let bucket = rate_limit_bucket(limit, now_seconds);
    let key = format!(
        "rate_limit:{}:{}:{}:{}:{}",
        route.path, limit.window, bucket, limit.key, subject
    );
    let observed = store.counter_add(key, 1);
    if observed > i64::from(limit.limit) {
        Some(json_error(429, "rate limit exceeded"))
    } else {
        None
    }
}

fn current_epoch_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn rate_limit_bucket(limit: &RateLimitDecl, now_seconds: u64) -> u64 {
    now_seconds / window_seconds(&limit.window)
}

fn window_seconds(window: &str) -> u64 {
    match window {
        "s" | "sec" | "second" | "seconds" => 1,
        "m" | "min" | "minute" | "minutes" => 60,
        "h" | "hr" | "hour" | "hours" => 60 * 60,
        "d" | "day" | "days" => 24 * 60 * 60,
        _ => 60,
    }
}

fn rate_limit_applies(limit: &RateLimitDecl, identity: RuntimeIdentity) -> bool {
    match limit.condition.as_deref() {
        Some("anonymous") => identity == RuntimeIdentity::Anonymous,
        Some("logged_in") => identity == RuntimeIdentity::LoggedIn,
        Some("api_key") => identity == RuntimeIdentity::ApiKey,
        Some(_) => false,
        None => true,
    }
}

fn endpoint_response(
    route: &RoutePlan,
    path: &str,
    store: &MemoryStore,
    models: &[ModelSpec],
    pricing: &[PricingRule],
) -> RuntimeResponse {
    let lock_token = format!("endpoint:{}:{}", route.path, current_epoch_seconds());
    if let Some(lock_name) = route.lock_name.as_deref() {
        let ttl_seconds = route.lock_ttl_seconds.unwrap_or(300);
        if !store.lock_acquire(lock_name, &lock_token, ttl_seconds) {
            return json_error(423, &format!("endpoint lock {lock_name} is held"));
        }
        let response = endpoint_response_unlocked(route, path, store, models, pricing);
        store.lock_release(lock_name, &lock_token);
        return response;
    }
    endpoint_response_unlocked(route, path, store, models, pricing)
}

fn endpoint_response_unlocked(
    route: &RoutePlan,
    path: &str,
    store: &MemoryStore,
    models: &[ModelSpec],
    pricing: &[PricingRule],
) -> RuntimeResponse {
    if route.path == "/start_deep_research" {
        let next = store.counter_add("research_tasks:sequence", 1);
        let task_id = format!("research-{next}");
        store.cache_set(
            format!("research_status:{task_id}"),
            serde_json::json!("pending"),
        );
        store.queue_push(
            "research_tasks",
            serde_json::json!({
                "task_id": task_id,
                "status": "pending"
            }),
        );
        return RuntimeResponse {
            status: 200,
            content_type: "application/json".into(),
            body: serde_json::json!({
                "ok": true,
                "task_id": task_id
            })
            .to_string(),
            chunked: false,
        };
    }

    if route.path.starts_with("/check_research_status/") {
        let params = route_params(&route.path, path);
        let Some(task_id) = params.get("task_id") else {
            return json_error(404, "task not found");
        };
        if let Some(status) = store.cache_get(&format!("research_status:{task_id}")) {
            return RuntimeResponse {
                status: 200,
                content_type: "application/json".into(),
                body: serde_json::json!({
                    "ok": true,
                    "task_id": task_id,
                    "status": status
                })
                .to_string(),
                chunked: false,
            };
        }
    }

    if route.path == "/hacking_is_a_serious_crime" {
        store.counter_add("endpoint:/hacking_is_a_serious_crime", 1);
        let user_id = find_or_create_record(
            store,
            "User",
            "email",
            serde_json::json!("demo@deepai.org"),
            JsonRecord::from([
                ("email".into(), serde_json::json!("demo@deepai.org")),
                ("password_hash".into(), serde_json::json!("runtime-managed")),
            ]),
        );
        let session_id = find_or_create_record(
            store,
            "ChatSession",
            "owner",
            serde_json::json!(user_id),
            JsonRecord::from([("owner".into(), serde_json::json!(user_id))]),
        );
        let sessions = store
            .where_eq("ChatSession", "owner", &serde_json::json!(user_id))
            .unwrap_or_default();
        let messages = store
            .where_eq("ChatMessage", "session", &serde_json::json!(session_id))
            .unwrap_or_default();
        let resolution = resolve_model_from_catalog("gpt-4.1", models, &BTreeSet::new());
        let charge = charge_for_usage(store, pricing, "chat", &resolution.selected_model);
        let payload = serde_json::json!({
            "ok": true,
            "route": route.path,
            "target": route.target,
            "kind": "endpoint",
            "model": resolution.selected_model,
            "provider": resolution.selected_provider,
            "fallback_used": resolution.fallback_used,
            "charge_cents": charge.cents,
            "usage_count": charge.usage_count,
            "handler_queries": {
                "sessions": sessions.len(),
                "messages": messages.len()
            }
        });
        return if route.response_stream {
            sse_response(vec![
                ("message", payload),
                (
                    "done",
                    serde_json::json!({
                        "ok": true,
                        "done": true
                    }),
                ),
            ])
        } else {
            RuntimeResponse {
                status: 200,
                content_type: "application/json".into(),
                body: payload.to_string(),
                chunked: false,
            }
        };
    }

    if let Some(handler_response) = &route.handler_response {
        return RuntimeResponse {
            status: 200,
            content_type: "application/json".into(),
            body: evaluate_handler_response(handler_response, &route_params(&route.path, path))
                .to_string(),
            chunked: false,
        };
    }

    RuntimeResponse {
        status: 200,
        content_type: "application/json".into(),
        body: serde_json::json!({
            "ok": true,
            "route": route.path,
            "target": route.target,
            "kind": "endpoint"
        })
        .to_string(),
        chunked: false,
    }
}

fn find_or_create_record(
    store: &MemoryStore,
    data: &str,
    field: &str,
    value: serde_json::Value,
    defaults: JsonRecord,
) -> u64 {
    try_find_or_create_record(store, data, field, value, defaults)
        .expect("native handler record create failed")
}

fn try_find_or_create_record(
    store: &MemoryStore,
    data: &str,
    field: &str,
    value: serde_json::Value,
    defaults: JsonRecord,
) -> Result<u64, StoreError> {
    if let Ok(records) = store.where_eq(data, field, &value) {
        if let Some(id) = records
            .first()
            .and_then(|record| record.get("id"))
            .and_then(serde_json::Value::as_u64)
        {
            return Ok(id);
        }
    }
    store.create(data, defaults)
}

fn evaluate_handler_response(
    response: &HandlerResponse,
    params: &BTreeMap<String, String>,
) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    for (name, value) in &response.fields {
        let value = match value {
            HandlerValue::String(value) => serde_json::json!(value),
            HandlerValue::Number(value) => serde_json::json!(value),
            HandlerValue::Bool(value) => serde_json::json!(value),
            HandlerValue::Param(name) => serde_json::json!(params.get(name).cloned()),
            HandlerValue::GeneratedUuid => serde_json::json!("generated-uuid"),
        };
        object.insert(name.clone(), value);
    }
    serde_json::Value::Object(object)
}

fn route_params(pattern: &str, path: &str) -> BTreeMap<String, String> {
    let mut params = BTreeMap::new();
    let pattern_parts = pattern.trim_matches('/').split('/');
    let path_parts = path.trim_matches('/').split('/');
    for (pattern, value) in pattern_parts.zip(path_parts) {
        if pattern.starts_with('{') && pattern.ends_with('}') {
            let name = pattern
                .trim_start_matches('{')
                .trim_end_matches('}')
                .split(':')
                .next()
                .unwrap_or(pattern);
            params.insert(name.to_string(), value.to_string());
        }
    }
    params
}

fn identity_from_http(request: &str, store: &MemoryStore, ip: &str) -> RuntimeIdentity {
    if header_value(request, "api-key")
        .and_then(|key| resolve_api_key(store, key))
        .is_some()
    {
        return RuntimeIdentity::ApiKey;
    }
    if header_value(request, "cookie")
        .and_then(session_cookie)
        .and_then(|session_id| store.cache_get(&format!("user_session:{session_id}")))
        .is_some()
    {
        return RuntimeIdentity::LoggedIn;
    }
    resolve_anonymous_client(store, request, ip);
    RuntimeIdentity::Anonymous
}

fn resolve_api_key(store: &MemoryStore, key: &str) -> Option<u64> {
    store
        .where_eq("UserApiKey", "key_hash", &serde_json::json!(key))
        .ok()?
        .first()
        .and_then(|record| record.get("user"))
        .and_then(serde_json::Value::as_u64)
}

fn session_cookie(cookie: &str) -> Option<&str> {
    cookie.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        (name == "session" && !value.is_empty()).then_some(value)
    })
}

fn resolve_anonymous_client(store: &MemoryStore, request: &str, ip: &str) {
    let fingerprint = header_value(request, "x-client-fingerprint").unwrap_or(ip);
    let _ = try_find_or_create_record(
        store,
        "ClientInfo",
        "fingerprint",
        serde_json::json!(fingerprint),
        JsonRecord::from([
            ("fingerprint".into(), serde_json::json!(fingerprint)),
            ("ip_address".into(), serde_json::json!(ip)),
        ]),
    );
}

fn header_value<'a>(request: &'a str, name: &str) -> Option<&'a str> {
    request.lines().skip(1).find_map(|line| {
        let (key, value) = line.split_once(':')?;
        if key.trim().eq_ignore_ascii_case(name) {
            Some(value.trim())
        } else {
            None
        }
    })
}

fn sse_response(events: Vec<(&str, serde_json::Value)>) -> RuntimeResponse {
    let body = events
        .into_iter()
        .map(|(event, data)| format!("event: {event}\ndata: {data}\n\n"))
        .collect::<String>();
    RuntimeResponse {
        status: 200,
        content_type: "text/event-stream; charset=utf-8".into(),
        body,
        chunked: true,
    }
}

fn text_response(status: u16, body: impl Into<String>) -> RuntimeResponse {
    RuntimeResponse {
        status,
        content_type: "text/plain; charset=utf-8".into(),
        body: body.into(),
        chunked: false,
    }
}

fn json_response(status: u16, value: &impl Serialize) -> RuntimeResponse {
    RuntimeResponse {
        status,
        content_type: "application/json".into(),
        body: serde_json::to_string(value).unwrap_or_else(|_| "{}".into()),
        chunked: false,
    }
}

fn json_error(status: u16, message: &str) -> RuntimeResponse {
    RuntimeResponse {
        status,
        content_type: "application/json".into(),
        body: serde_json::json!({ "ok": false, "error": message }).to_string(),
        chunked: false,
    }
}

fn write_http_response(stream: &mut TcpStream, response: RuntimeResponse) -> anyhow::Result<()> {
    let reason = match response.status {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Error",
    };
    if response.chunked {
        write!(
            stream,
            "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nTransfer-Encoding: chunked\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
            response.status, reason, response.content_type
        )?;
        for chunk in response.body.split_inclusive("\n\n") {
            write!(stream, "{:X}\r\n{}\r\n", chunk.len(), chunk)?;
        }
        write!(stream, "0\r\n\r\n")?;
        return Ok(());
    }
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        response.status,
        reason,
        response.content_type,
        response.body.len(),
        response.body
    )?;
    Ok(())
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_program() -> &'static str {
        r#"
module chat

type Email = string @pii format email
identity RequestIdentity = logged_in(user: User) | anonymous
model_config chat_models {
  "gpt-4.1-nano" { providers: [openai.chat] cost: 1 credit }
}

notification_channels {
  errors: log when dev
  notifications: log when dev
}

data User {
  id: pk
  email: Email unique @pii
  password_hash: string @secret
  locked: bool default false
  index email
}

data UserApiKey {
  id: pk
  key_hash: string unique @secret
  user: User
  index key_hash
  index user
}

data ClientInfo {
  id: pk
  fingerprint: string unique
  ip_address: string @pii
  index fingerprint
}

data ChatSession {
  id: pk
  owner: User?
  created_at: datetime default now() @no_index
  index owner
}

data ChatMessage {
  id: pk
  session: ChatSession
  content: string
  index session
}

queue research_tasks: ResearchTask sorted_by created_at { ttl: 1h }
cache user_session[session_id: uuid] ttl 1h { type: SessionData }
cached fn research_status[task_id: uuid] -> ResearchStatus ttl 1h
counter api_calls[ip: string, model: string] window 60s
topic model_updates: ModelEvent
pricing {
  chat {
    default: $0.01
  }
}
storage images { backend: s3 bucket: "deepai-images" }
worker stable_diffusion { image: "sdxl-worker:latest" }
fn resolve_model(name: string) -> ChatModel { chat_models[name] }
fn charge_for_usage(user: User, category: string, model: string) {
  with lock billing[user.id] { notify notifications "usage charged" }
}

endpoint POST /hacking_is_a_serious_crime {
  identity: any
  rate_limit: 30/min by ip when anonymous
  request { messages: [{ role: string, content: string }] }
  response: stream { content: string done: bool }
  handle {
    user = User[email: "foo@bar.com"]
    by_owner = ChatSession |> where(_.owner == user)
    recent_messages = ChatMessage |> where(_.id between request.after_id and request.before_id)
    charge_for_usage(user, "chat", "gpt-4.1")
    notify errors "request accepted"
  }
}

endpoint POST /start_deep_research {
  identity: logged_in | api_key
  request { messages: [{ role: string, content: string }] session_uuid: uuid }
  handle { { task_id: uuid() } }
}

endpoint GET /check_research_status/{task_id:uuid} {
  identity: any
  handle { { status: "pending" } }
}

page chat at / cache private {
  view { h1 "Chat" }
}

cron daily_billing schedule "0 3 * * *" {
  healthcheck: "https://hc-ping.com/key"
  with lock daily_billing { notify notifications "ok" }
  on error(e) { notify errors "failed" }
}

daemon chat_task_checker schedule "*/5 * * * *" runtime 280s {
  healthcheck: "https://hc-ping.com/key"
  with lock chat_task_checker { loop { sleep 5s } }
  on error(e) { notify errors "failed" }
}

services {
  web { endpoints: [pages, static_assets] instances: 2..10 }
  chat { endpoints: [/hacking_is_a_serious_crime, /start_deep_research, /check_research_status/{task_id:uuid}] instances: 2..8 }
}

cdn {
  domain: "deepai.org"
  rules { bypass: ["/dashboard*", "/api/*"] api_domain: "api.deepai.org" }
}

deploy_rules {
  migration_before_deploy: true
  cdn_refresh_on: [pages, static_assets]
  health_check: GET /ping expect 200
  rollback: automatic
}

invariant "cron jobs have healthchecks"
"#
    }

    #[test]
    fn parses_data_indexes_and_generates_manifest() {
        let program = parse(valid_program()).unwrap();
        assert!(program.data["User"].indexes.contains("email"));
        assert_eq!(program.endpoints[0].path, "/hacking_is_a_serious_crime");
        assert_eq!(
            program.endpoints[0].rate_limit,
            Some(RateLimitDecl {
                limit: 30,
                window: "min".into(),
                key: "ip".into(),
                condition: Some("anonymous".into())
            })
        );
        assert_eq!(
            program.endpoints[2].handler_response,
            Some(HandlerResponse {
                fields: BTreeMap::from([("status".into(), HandlerValue::String("pending".into()))])
            })
        );

        let artifacts = compile_source(valid_program()).unwrap();
        assert!(artifacts.runtime.contains_http_server);
        assert!(artifacts.runtime.contains_storage_engine);
        assert_eq!(artifacts.manifest.services.len(), 2);
        assert_eq!(artifacts.manifest.routes[1].identity, Some("any".into()));
        assert_eq!(
            artifacts.manifest.routes[1]
                .rate_limit
                .as_ref()
                .map(|limit| limit.limit),
            Some(30)
        );
        assert_eq!(artifacts.manifest.routes[1].response_stream, true);
        assert_eq!(
            artifacts.manifest.routes[1].lock_name.as_deref(),
            Some("billing[user.id]")
        );
        assert_eq!(artifacts.report.language_units["queue"], 1);
        assert_eq!(artifacts.report.language_units["worker"], 1);
        assert_eq!(artifacts.report.language_units["model_config"], 1);
        assert!(artifacts.redis_catalog.iter().any(|primitive| {
            primitive.kind == "queue"
                && primitive.name == "research_tasks"
                && primitive.ttl_seconds == Some(3600)
                && primitive.queue_order.as_deref() == Some("created_at")
                && primitive.backend == "redis_sorted_set"
        }));
        assert!(artifacts.redis_catalog.iter().any(|primitive| {
            primitive.kind == "cached_fn"
                && primitive.name == "research_status"
                && primitive.ttl_seconds == Some(3600)
        }));
        assert!(artifacts.redis_catalog.iter().any(|primitive| {
            primitive.kind == "topic"
                && primitive.name == "model_updates"
                && primitive.backend == "redis_pubsub"
        }));
        assert_eq!(artifacts.model_catalog.len(), 1);
        assert_eq!(artifacts.model_catalog[0].name, "gpt-4.1-nano");
        assert_eq!(artifacts.model_catalog[0].providers, vec!["openai.chat"]);
        assert_eq!(artifacts.pricing_catalog.len(), 1);
        assert_eq!(
            artifacts.pricing_catalog[0],
            PricingRule {
                category: "chat".into(),
                model: "default".into(),
                cents: 1
            }
        );
        assert_eq!(artifacts.sql_plan.dialect, "mysql");
        assert!(artifacts.sql_plan.tables.iter().any(|table| {
            table.table == "ChatSession"
                && table.create_table.contains("CREATE TABLE `ChatSession`")
                && table
                    .indexes
                    .iter()
                    .any(|index| index.contains("idx_ChatSession_owner"))
        }));
        assert!(artifacts.sql_plan.queries.iter().any(|query| {
            query.table == "ChatSession"
                && query.field == "owner"
                && query.strategy == "indexed_lookup"
                && query.sql == "SELECT * FROM `ChatSession` WHERE `owner` = ?;"
        }));
        assert!(artifacts.sql_plan.queries.iter().any(|query| {
            query.table == "ChatMessage"
                && query.field == "id"
                && query.index == "PRIMARY"
                && query.strategy == "pk_range_scan"
                && query.sql == "SELECT * FROM `ChatMessage` WHERE `id` BETWEEN ? AND ?;"
        }));
        assert!(artifacts
            .migrations
            .windows(2)
            .all(|steps| steps[0].order < steps[1].order));
        assert!(artifacts.migrations.iter().all(|step| {
            step.phase == "pre_deploy"
                && step.online
                && step.resumable
                && step.checkpoint_key.starts_with("migration:")
                && step.safe_column_add
                && step
                    .additive_ddl
                    .iter()
                    .all(|ddl| ddl.contains("ALGORITHM=INPLACE, LOCK=NONE"))
                && step.resume_policy == "checkpoint_before_each_ddl_statement"
                && step.bail_out == "stop_before_next_statement_and_keep_checkpoint"
                && !step.ddl.is_empty()
        }));
        assert_eq!(artifacts.frontend_assets[0].title, "Chat");
        assert_eq!(artifacts.frontend_assets[0].cache, CacheMode::Private);
        assert_eq!(
            artifacts.frontend_assets[0].api_base_url,
            Some("https://api.deepai.org".into())
        );
        assert_eq!(artifacts.frontend_assets[0].data_loading, "client_fetch");
        assert_eq!(
            artifacts.frontend_assets[0].html.contains("<h1>Chat</h1>"),
            true
        );
        assert!(artifacts.frontend_assets[0]
            .html
            .contains("data-api-base=\"https://api.deepai.org\""));
        assert!(artifacts.frontend_assets[0]
            .html
            .contains("data-data-loading=\"client_fetch\""));
        assert_eq!(artifacts.task_catalog.len(), 3);
        assert!(artifacts
            .task_catalog
            .iter()
            .any(|task| task.name == "daily_billing"
                && task.kind == TaskKind::Cron
                && task.lock_name.as_deref() == Some("daily_billing")
                && task.lock_ttl_seconds.is_none()));
        assert!(artifacts
            .task_catalog
            .iter()
            .any(|task| task.name == "chat_task_checker"
                && task.kind == TaskKind::Daemon
                && task.lock_name.as_deref() == Some("chat_task_checker")
                && task.lock_ttl_seconds.is_none()));
        assert!(artifacts
            .task_catalog
            .iter()
            .any(|task| task.name == "stable_diffusion" && task.kind == TaskKind::Worker));
        assert_eq!(
            artifacts
                .manifest
                .routes
                .iter()
                .find(|route| route.path == "/hacking_is_a_serious_crime")
                .unwrap()
                .target,
            "services.chat"
        );
    }

    #[test]
    fn rejects_queries_on_no_index_fields() {
        let source = valid_program().replace(
            "by_owner = ChatSession |> where(_.owner == user)",
            "recent = ChatSession |> where(_.created_at > now() - 1d)",
        );
        let errors = compile_source(&source).unwrap_err();
        assert!(errors.contains(&DeepError::QueryOnUnindexedField {
            data: "ChatSession".into(),
            field: "created_at".into()
        }));
    }

    #[test]
    fn rejects_in_queries_for_production_planning() {
        let source = valid_program().replace(
            "by_owner = ChatSession |> where(_.owner == user)",
            "by_ids = ChatSession |> where(_.id in request.session_ids)",
        );
        let errors = compile_source(&source).unwrap_err();
        assert!(errors.contains(&DeepError::QueryUsesInList {
            data: "ChatSession".into(),
            field: "id".into()
        }));
    }

    #[test]
    fn rejects_secret_and_pii_flows_to_logs_or_notifications() {
        let source = valid_program().replace(
            "notify errors \"request accepted\"",
            "notify errors \"Hash: {user.password_hash}\"\n    log \"Email: {user.email}\"",
        );
        let errors = compile_source(&source).unwrap_err();
        assert!(errors.contains(&DeepError::SecretFlow {
            data: "User".into(),
            field: "password_hash".into()
        }));
        assert!(errors.contains(&DeepError::PiiLog {
            data: "User".into(),
            field: "email".into()
        }));
    }

    #[test]
    fn rejects_public_pages_that_read_private_data() {
        let source = valid_program().replace(
            "page chat at / cache private {\n  view { h1 \"Chat\" }",
            "page chat at / cache public {\n  view { user = User[id: 1] }",
        );
        let errors = compile_source(&source).unwrap_err();
        assert!(errors.contains(&DeepError::PublicPageReadsPrivateData {
            page: "chat".into(),
            data: "User".into()
        }));
    }

    #[test]
    fn rejects_unknown_notification_channels() {
        let source = valid_program().replace(
            "notify errors \"request accepted\"",
            "notify billing_alerts \"request accepted\"",
        );
        let errors = compile_source(&source).unwrap_err();
        assert!(errors.contains(&DeepError::UnknownNotificationChannel {
            channel: "billing_alerts".into()
        }));
    }

    #[test]
    fn rejects_crons_without_operational_contracts() {
        let source = valid_program().replace(
            "cron daily_billing schedule \"0 3 * * *\" {\n  healthcheck: \"https://hc-ping.com/key\"\n  with lock daily_billing { notify notifications \"ok\" }\n  on error(e) { notify errors \"failed\" }\n}",
            "cron daily_billing schedule \"0 3 * * *\" {\n  notify notifications \"ok\"\n}",
        );
        let errors = compile_source(&source).unwrap_err();
        assert!(errors.contains(&DeepError::InvalidCron {
            name: "daily_billing".into()
        }));
    }

    #[test]
    fn rejects_incomplete_deploy_rules() {
        let source = valid_program().replace(
            "deploy_rules {\n  migration_before_deploy: true\n  cdn_refresh_on: [pages, static_assets]\n  health_check: GET /ping expect 200\n  rollback: automatic\n}",
            "deploy_rules {\n  migration_before_deploy: false\n  rollback: manual\n}",
        );
        let errors = compile_source(&source).unwrap_err();
        assert!(errors.contains(&DeepError::InvalidDeployRules));
    }

    #[test]
    fn runtime_serves_health_page_endpoint_and_metadata() {
        let artifacts = compile_source(valid_program()).unwrap();
        let runtime = RuntimeApp::new(artifacts);

        let ping = runtime.handle("GET", "/ping");
        assert_eq!(ping.status, 200);
        assert_eq!(ping.body, "ok\n");

        let page = runtime.handle("GET", "/");
        assert_eq!(page.status, 200);
        assert_eq!(page.content_type, "text/html; charset=utf-8");
        assert!(page.body.contains("<h1>Chat</h1>"));
        assert!(page.body.contains("data-route=\"/\""));
        assert!(page.body.contains("/__deep/page-data/chat"));
        assert!(page.body.contains("credentials:\"include\""));

        let page_data = runtime.handle("GET", "/__deep/page-data/chat");
        assert_eq!(page_data.status, 200);
        assert!(page_data.body.contains("\"data_loading\":\"client_fetch\""));
        assert!(page_data.body.contains("\"states\""));

        let endpoint = runtime.handle("POST", "/hacking_is_a_serious_crime");
        assert_eq!(endpoint.status, 200);
        assert_eq!(endpoint.content_type, "text/event-stream; charset=utf-8");
        assert!(endpoint.chunked);
        assert!(endpoint.body.contains("event: message"));
        assert!(endpoint.body.contains("event: done"));
        assert!(endpoint.body.contains("\"kind\":\"endpoint\""));

        let report = runtime.handle("GET", "/__deep/static-report");
        assert_eq!(report.status, 200);
        assert!(report.body.contains("checked_rules"));

        let tasks = runtime.handle("GET", "/__deep/tasks");
        assert_eq!(tasks.status, 200);
        assert!(tasks.body.contains("daily_billing"));

        let models = runtime.handle("GET", "/__deep/models");
        assert_eq!(models.status, 200);
        assert!(models.body.contains("gpt-4.1-nano"));

        let pricing = runtime.handle("GET", "/__deep/pricing");
        assert_eq!(pricing.status, 200);
        assert!(pricing.body.contains("\"category\":\"chat\""));

        let price = runtime.handle("GET", "/__deep/price/chat/gpt-4.1-nano");
        assert_eq!(price.status, 200);
        assert!(price.body.contains("\"cents\":1"));

        let model_resolution = runtime.handle("GET", "/__deep/resolve-model/gpt-unknown");
        assert_eq!(model_resolution.status, 200);
        assert!(model_resolution.body.contains("\"fallback_used\":true"));

        let task_tick = runtime.handle("POST", "/__deep/task-tick");
        assert_eq!(task_tick.status, 200);
        assert!(task_tick.body.contains("daily_billing"));

        let snapshot = runtime.handle("GET", "/__deep/snapshot");
        assert_eq!(snapshot.status, 200);
        assert!(snapshot.body.contains("counters"));
    }

    #[test]
    fn runtime_rejects_unknown_routes_and_wrong_methods() {
        let artifacts = compile_source(valid_program()).unwrap();
        let runtime = RuntimeApp::new(artifacts);

        assert_eq!(runtime.handle("GET", "/missing").status, 404);
        assert_eq!(
            runtime.handle("GET", "/hacking_is_a_serious_crime").status,
            405
        );
    }

    #[test]
    fn runtime_enforces_endpoint_identity_policy() {
        let runtime = RuntimeApp::new(compile_source(valid_program()).unwrap());

        let anonymous = runtime.handle("POST", "/start_deep_research");
        assert_eq!(anonymous.status, 401);

        let logged_in = runtime.handle_as(
            "POST",
            "/start_deep_research",
            RuntimeIdentity::LoggedIn,
            "203.0.113.10",
        );
        assert_eq!(logged_in.status, 200);
        assert!(logged_in.body.contains("research-1"));

        let api_key = runtime.handle_as(
            "POST",
            "/start_deep_research",
            RuntimeIdentity::ApiKey,
            "203.0.113.10",
        );
        assert_eq!(api_key.status, 200);
        assert!(api_key.body.contains("research-2"));
    }

    #[test]
    fn http_identity_resolves_sessions_api_keys_and_anonymous_clients() {
        let runtime = RuntimeApp::new(compile_source(valid_program()).unwrap());
        let store = runtime.store();
        let user_id = store
            .create(
                "User",
                JsonRecord::from([
                    ("email".into(), serde_json::json!("auth@deep.test")),
                    ("password_hash".into(), serde_json::json!("hashed")),
                ]),
            )
            .unwrap();
        store
            .create(
                "UserApiKey",
                JsonRecord::from([
                    ("key_hash".into(), serde_json::json!("dev-key")),
                    ("user".into(), serde_json::json!(user_id)),
                ]),
            )
            .unwrap();
        store.cache_set(
            "user_session:session-1",
            serde_json::json!({ "user_id": user_id }),
        );

        assert_eq!(
            identity_from_http(
                "GET / HTTP/1.1\r\napi-key: dev-key\r\n\r\n",
                &store,
                "127.0.0.1"
            ),
            RuntimeIdentity::ApiKey
        );
        assert_eq!(
            identity_from_http(
                "GET / HTTP/1.1\r\ncookie: session=session-1\r\n\r\n",
                &store,
                "127.0.0.1"
            ),
            RuntimeIdentity::LoggedIn
        );
        assert_eq!(
            identity_from_http(
                "GET / HTTP/1.1\r\napi-key: missing\r\nx-client-fingerprint: fp-1\r\n\r\n",
                &store,
                "203.0.113.9"
            ),
            RuntimeIdentity::Anonymous
        );
        assert_eq!(
            store
                .where_eq("ClientInfo", "fingerprint", &serde_json::json!("fp-1"))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn runtime_enforces_anonymous_rate_limits() {
        let source = valid_program().replace(
            "rate_limit: 30/min by ip when anonymous",
            "rate_limit: 1/min by ip when anonymous",
        );
        let runtime = RuntimeApp::new(compile_source(&source).unwrap());

        let first = runtime.handle_as(
            "POST",
            "/hacking_is_a_serious_crime",
            RuntimeIdentity::Anonymous,
            "198.51.100.9",
        );
        assert_eq!(first.status, 200);

        let second = runtime.handle_as(
            "POST",
            "/hacking_is_a_serious_crime",
            RuntimeIdentity::Anonymous,
            "198.51.100.9",
        );
        assert_eq!(second.status, 429);

        let logged_in = runtime.handle_as(
            "POST",
            "/hacking_is_a_serious_crime",
            RuntimeIdentity::LoggedIn,
            "198.51.100.9",
        );
        assert_eq!(logged_in.status, 200);
    }

    #[test]
    fn rate_limits_roll_over_by_declared_window() {
        let source = valid_program().replace(
            "rate_limit: 30/min by ip when anonymous",
            "rate_limit: 1/min by ip when anonymous",
        );
        let artifacts = compile_source(&source).unwrap();
        let route = artifacts
            .manifest
            .routes
            .iter()
            .find(|route| route.path == "/hacking_is_a_serious_crime")
            .unwrap();
        let store = MemoryStore::new(artifacts.storage_catalog);

        assert_eq!(rate_limit_bucket(route.rate_limit.as_ref().unwrap(), 59), 0);
        assert_eq!(rate_limit_bucket(route.rate_limit.as_ref().unwrap(), 60), 1);
        assert!(enforce_rate_limit_at(
            route,
            &store,
            RuntimeIdentity::Anonymous,
            "198.51.100.10",
            59
        )
        .is_none());
        assert_eq!(
            enforce_rate_limit_at(
                route,
                &store,
                RuntimeIdentity::Anonymous,
                "198.51.100.10",
                59
            )
            .unwrap()
            .status,
            429
        );
        assert!(enforce_rate_limit_at(
            route,
            &store,
            RuntimeIdentity::Anonymous,
            "198.51.100.10",
            60
        )
        .is_none());
    }

    #[test]
    fn memory_store_creates_reads_and_enforces_unique_indexes() {
        let artifacts = compile_source(valid_program()).unwrap();
        let store = RuntimeApp::new(artifacts).store();

        let mut user = JsonRecord::new();
        user.insert("email".into(), serde_json::json!("foo@bar.com"));
        user.insert("password_hash".into(), serde_json::json!("hashed"));
        let id = store.create("User", user.clone()).unwrap();

        assert_eq!(id, 1);
        assert_eq!(
            store.get("User", id).unwrap().unwrap()["email"],
            serde_json::json!("foo@bar.com")
        );
        assert_eq!(
            store
                .where_eq("User", "email", &serde_json::json!("foo@bar.com"))
                .unwrap()
                .len(),
            1
        );

        let duplicate = store.create("User", user).unwrap_err();
        assert_eq!(
            duplicate,
            StoreError::DuplicateUnique {
                data: "User".into(),
                field: "email".into()
            }
        );
    }

    #[test]
    fn memory_store_rejects_queries_without_indexes() {
        let artifacts = compile_source(valid_program()).unwrap();
        let store = RuntimeApp::new(artifacts).store();

        let error = store
            .where_eq(
                "ChatSession",
                "created_at",
                &serde_json::json!("2026-05-17"),
            )
            .unwrap_err();

        assert_eq!(
            error,
            StoreError::UnindexedField {
                data: "ChatSession".into(),
                field: "created_at".into()
            }
        );
    }

    #[test]
    fn memory_store_supports_cache_queues_and_counters() {
        let artifacts = compile_source(valid_program()).unwrap();
        let store = RuntimeApp::new(artifacts).store();
        assert_eq!(store.primitive_backend(), "memory");

        store.cache_set("session:1", serde_json::json!({ "user": 1 }));
        assert_eq!(
            store.cache_get("session:1"),
            Some(serde_json::json!({ "user": 1 }))
        );

        store.queue_push("research_tasks", serde_json::json!({ "task": 1 }));
        store.queue_push("research_tasks", serde_json::json!({ "task": 2 }));
        assert_eq!(
            store.queue_pop("research_tasks"),
            Some(serde_json::json!({ "task": 1 }))
        );
        assert_eq!(
            store.queue_pop("research_tasks"),
            Some(serde_json::json!({ "task": 2 }))
        );
        assert_eq!(store.queue_pop("research_tasks"), None);

        assert_eq!(store.counter_add("api:127.0.0.1:gpt-5", 1), 1);
        assert_eq!(store.counter_add("api:127.0.0.1:gpt-5", 4), 5);
        assert_eq!(store.counter_get("api:127.0.0.1:gpt-5"), 5);

        assert!(store.lock_acquire("billing:user-1", "worker-a", 30));
        assert!(!store.lock_acquire("billing:user-1", "worker-b", 30));
        assert!(!store.lock_release("billing:user-1", "worker-b"));
        assert!(store.lock_release("billing:user-1", "worker-a"));
        assert!(store.lock_acquire("billing:user-1", "worker-b", 30));

        assert_eq!(
            store.topic_publish("model_updates", serde_json::json!({ "model": "gpt-5" })),
            1
        );
        assert_eq!(
            store.snapshot().topics["model_updates"][0],
            serde_json::json!({ "model": "gpt-5" })
        );
    }

    #[test]
    fn redis_store_supports_cache_queues_counters_and_locks_when_configured() {
        let Ok(redis_url) = std::env::var("REDIS_URL") else {
            return;
        };
        use std::sync::mpsc;
        use std::time::Duration;

        let suffix = format!("test_{}", std::process::id());
        let queue_name = format!("{suffix}_jobs");
        let cache_name = format!("{suffix}_cache");
        let topic_name = format!("{suffix}_events");
        let source = format!(
            "module redis.test\nqueue {queue_name}: Job sorted_by score {{ ttl: 1h }}\ncache {cache_name}[id: string] ttl 1h {{ type: Value }}\ncounter {suffix}_counter[id: string] window 60s\ntopic {topic_name}: Event\ndata User {{ id: pk }}\n"
        );
        let artifacts = compile_source(&source).unwrap();
        let store = RuntimeApp::with_redis(artifacts, &redis_url)
            .unwrap()
            .store();
        assert_eq!(store.primitive_backend(), "redis");

        store.cache_set(
            format!("{cache_name}:session"),
            serde_json::json!({ "user": 42 }),
        );
        assert_eq!(
            store.cache_get(&format!("{cache_name}:session")),
            Some(serde_json::json!({ "user": 42 }))
        );

        store.queue_push(&queue_name, serde_json::json!({ "job": 2, "score": 2 }));
        store.queue_push(&queue_name, serde_json::json!({ "job": 1, "score": 1 }));
        assert_eq!(
            store.queue_pop(&queue_name),
            Some(serde_json::json!({ "job": 1, "score": 1 }))
        );
        assert_eq!(
            store.queue_pop(&queue_name),
            Some(serde_json::json!({ "job": 2, "score": 2 }))
        );

        assert_eq!(store.counter_add(format!("{suffix}:counter"), 2), 2);
        assert_eq!(store.counter_add(format!("{suffix}:counter"), 3), 5);
        assert_eq!(store.counter_get(&format!("{suffix}:counter")), 5);

        assert!(store.lock_acquire(&format!("{suffix}:lock"), "worker-a", 30));
        assert!(!store.lock_acquire(&format!("{suffix}:lock"), "worker-b", 30));
        assert!(!store.lock_release(&format!("{suffix}:lock"), "worker-b"));
        assert!(store.lock_release(&format!("{suffix}:lock"), "worker-a"));
        assert!(store.lock_acquire(&format!("{suffix}:lock"), "worker-b", 30));

        let redis_url_for_thread = redis_url.clone();
        let redis_topic = format!("deepapp:runtime:topic:{topic_name}");
        let (ready_tx, ready_rx) = mpsc::channel();
        let (message_tx, message_rx) = mpsc::channel();
        std::thread::spawn(move || {
            let client = redis::Client::open(redis_url_for_thread).unwrap();
            let mut connection = client.get_connection().unwrap();
            let mut pubsub = connection.as_pubsub();
            pubsub.subscribe(&redis_topic).unwrap();
            ready_tx.send(()).unwrap();
            let message = pubsub.get_message().unwrap();
            let payload: String = message.get_payload().unwrap();
            message_tx.send(payload).unwrap();
        });
        ready_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert_eq!(
            store.topic_publish(&topic_name, serde_json::json!({ "event": "published" })),
            1
        );
        let payload = message_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        assert!(payload.contains("\"event\":\"published\""));
    }

    #[test]
    fn memory_store_saves_and_restores_durable_snapshots() {
        let artifacts = compile_source(valid_program()).unwrap();
        let store = RuntimeApp::new(artifacts.clone()).store();

        let mut user = JsonRecord::new();
        user.insert("email".into(), serde_json::json!("snapshot@deep.test"));
        user.insert("password_hash".into(), serde_json::json!("hashed"));
        let id = store.create("User", user).unwrap();
        store.cache_set("session:snapshot", serde_json::json!({ "user": id }));
        store.queue_push("research_tasks", serde_json::json!({ "task": "snapshot" }));
        store.counter_add("api:snapshot", 7);

        let path =
            std::env::temp_dir().join(format!("deepapp-snapshot-{}.json", std::process::id()));
        let _ = fs::remove_file(&path);
        store.save_snapshot(&path).unwrap();

        let restored = RuntimeApp::new(artifacts).store();
        restored.load_snapshot(&path).unwrap();

        assert_eq!(
            restored.get("User", id).unwrap().unwrap()["email"],
            serde_json::json!("snapshot@deep.test")
        );
        assert_eq!(
            restored.cache_get("session:snapshot"),
            Some(serde_json::json!({ "user": id }))
        );
        assert_eq!(
            restored.queue_pop("research_tasks"),
            Some(serde_json::json!({ "task": "snapshot" }))
        );
        assert_eq!(restored.counter_get("api:snapshot"), 7);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn runtime_research_endpoints_use_queue_and_cache_engine() {
        let artifacts = compile_source(valid_program()).unwrap();
        let runtime = RuntimeApp::new(artifacts);

        let started = runtime.handle_as(
            "POST",
            "/start_deep_research",
            RuntimeIdentity::LoggedIn,
            "203.0.113.20",
        );
        assert_eq!(started.status, 200);
        let payload: serde_json::Value = serde_json::from_str(&started.body).unwrap();
        let task_id = payload["task_id"].as_str().unwrap();
        assert_eq!(task_id, "research-1");

        let queued = runtime.store().queue_pop("research_tasks").unwrap();
        assert_eq!(queued["task_id"], serde_json::json!("research-1"));

        let status = runtime.handle("GET", "/check_research_status/research-1");
        assert_eq!(status.status, 200);
        assert!(status.body.contains("\"status\":\"pending\""));
    }

    #[test]
    fn runtime_chat_endpoint_records_usage_counter() {
        let artifacts = compile_source(valid_program()).unwrap();
        let runtime = RuntimeApp::new(artifacts);

        let response = runtime.handle("POST", "/hacking_is_a_serious_crime");
        assert_eq!(response.status, 200);
        assert!(response.chunked);
        assert!(response.body.contains("\"model\":\"gpt-4.1-nano\""));
        assert!(response.body.contains("\"charge_cents\":1"));
        assert!(response.body.contains("\"handler_queries\""));
        assert!(response.body.contains("\"sessions\":1"));
        assert!(response.body.contains("\"messages\":0"));
        assert_eq!(
            runtime
                .store()
                .counter_get("endpoint:/hacking_is_a_serious_crime"),
            1
        );
        assert_eq!(
            runtime.store().counter_get("usage:chat:gpt-4.1-nano:cents"),
            1
        );

        runtime
            .store()
            .create(
                "ChatMessage",
                JsonRecord::from([
                    ("session".into(), serde_json::json!(1)),
                    ("content".into(), serde_json::json!("hello")),
                ]),
            )
            .unwrap();
        let response = runtime.handle("POST", "/hacking_is_a_serious_crime");
        assert!(response.body.contains("\"messages\":1"));
    }

    #[test]
    fn runtime_endpoint_uses_called_function_lock() {
        let artifacts = compile_source(valid_program()).unwrap();
        let runtime = RuntimeApp::new(artifacts);
        assert!(runtime
            .store()
            .lock_acquire("billing[user.id]", "external-billing", 60));

        let response = runtime.handle("POST", "/hacking_is_a_serious_crime");
        assert_eq!(response.status, 423);
        assert!(response
            .body
            .contains("endpoint lock billing[user.id] is held"));
        assert_eq!(
            runtime
                .store()
                .counter_get("endpoint:/hacking_is_a_serious_crime"),
            0
        );
        assert!(runtime
            .store()
            .lock_release("billing[user.id]", "external-billing"));
    }

    #[test]
    fn runtime_charges_usage_from_pricing_catalog() {
        let runtime = RuntimeApp::new(compile_source(valid_program()).unwrap());

        let first = runtime.charge_for_usage("chat", "gpt-4.1-nano");
        let second = runtime.charge_for_usage("chat", "gpt-5");

        assert_eq!(first.cents, 1);
        assert_eq!(first.usage_count, 1);
        assert_eq!(second.cents, 1);
        assert_eq!(second.total_cents, 1);
        assert_eq!(
            runtime.store().counter_get("usage:chat:gpt-4.1-nano:cents"),
            1
        );
        assert_eq!(runtime.store().counter_get("usage:chat:gpt-5:cents"), 1);
    }

    #[test]
    fn runtime_resolves_models_with_provider_failover() {
        let source = valid_program().replace(
            "\"gpt-4.1-nano\" { providers: [openai.chat] cost: 1 credit }",
            "\"gpt-5\" { providers: [openai.chat, novita.chat] reasoning_effort: medium cost: 3 credits }\n  \"gpt-4.1-nano\" { providers: [openai.chat] cost: 1 credit }",
        );
        let runtime = RuntimeApp::new(compile_source(&source).unwrap());
        let mut unavailable = BTreeSet::new();
        unavailable.insert("openai.chat".to_string());

        let resolution = runtime.resolve_model("gpt-5", &unavailable);
        assert_eq!(resolution.selected_model, "gpt-5");
        assert_eq!(resolution.selected_provider, Some("novita.chat".into()));
        assert!(!resolution.fallback_used);

        let fallback = runtime.resolve_model("missing-model", &BTreeSet::new());
        assert_eq!(fallback.selected_model, "gpt-4.1-nano");
        assert!(fallback.fallback_used);
    }

    #[test]
    fn runtime_interprets_simple_endpoint_return_objects() {
        let source = valid_program().replace(
            "endpoint GET /check_research_status/{task_id:uuid} {\n  identity: any\n  handle { { status: \"pending\" } }\n}",
            "endpoint GET /check_research_status/{task_id:uuid} {\n  identity: any\n  handle { { task_id: params.task_id, status: \"pending\", cached: true } }\n}",
        );
        let runtime = RuntimeApp::new(compile_source(&source).unwrap());

        let response = runtime.handle("GET", "/check_research_status/interpreted-1");
        assert_eq!(response.status, 200);
        let body: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(body["task_id"], serde_json::json!("interpreted-1"));
        assert_eq!(body["status"], serde_json::json!("pending"));
        assert_eq!(body["cached"], serde_json::json!(true));
    }

    #[test]
    fn runtime_task_tick_runs_crons_daemons_and_workers() {
        let artifacts = compile_source(valid_program()).unwrap();
        let runtime = RuntimeApp::new(artifacts);
        runtime.store().queue_push(
            "stable_diffusion",
            serde_json::json!({ "prompt": "sunset" }),
        );

        let runs = runtime.run_task_tick();
        assert_eq!(runs.len(), 3);
        assert!(runs.iter().any(|run| run.name == "daily_billing"));
        assert!(runs.iter().any(|run| run.name == "chat_task_checker"));
        assert!(runs
            .iter()
            .any(|run| run.name == "stable_diffusion" && run.detail == "worker processed one job"));
        assert_eq!(runtime.store().counter_get("task:daily_billing:runs"), 1);
        assert!(runtime.store().queue_pop("task_events").is_some());
        assert_eq!(
            runtime
                .store()
                .queue_pop("stable_diffusion:results")
                .unwrap()["status"],
            serde_json::json!("done")
        );
    }

    #[test]
    fn runtime_task_tick_skips_jobs_when_declared_lock_is_held() {
        let artifacts = compile_source(valid_program()).unwrap();
        let runtime = RuntimeApp::new(artifacts);
        assert!(runtime
            .store()
            .lock_acquire("daily_billing", "external-runner", 1800));

        let runs = runtime.run_task_tick();
        let daily_billing = runs.iter().find(|run| run.name == "daily_billing").unwrap();
        assert_eq!(daily_billing.status, "locked");
        assert!(daily_billing.detail.contains("lock daily_billing is held"));
        assert_eq!(runtime.store().counter_get("task:daily_billing:runs"), 0);
        assert!(runtime
            .store()
            .lock_release("daily_billing", "external-runner"));
    }
}
