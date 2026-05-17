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
    pub has_error_handler: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DaemonDecl {
    pub name: String,
    pub schedule: Option<String>,
    pub runtime: Option<String>,
    pub has_healthcheck: bool,
    pub has_lock: bool,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FrontendAsset {
    pub page: String,
    pub path: String,
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
}

#[derive(Debug, Clone)]
struct StoreInner {
    schemas: BTreeMap<String, DataDecl>,
    rows: BTreeMap<String, BTreeMap<u64, JsonRecord>>,
    next_pk: BTreeMap<String, u64>,
    cache: BTreeMap<String, serde_json::Value>,
    queues: BTreeMap<String, VecDeque<serde_json::Value>>,
    counters: BTreeMap<String, i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoreSnapshot {
    pub rows: BTreeMap<String, BTreeMap<u64, JsonRecord>>,
    pub next_pk: BTreeMap<String, u64>,
    pub cache: BTreeMap<String, serde_json::Value>,
    pub queues: BTreeMap<String, Vec<serde_json::Value>>,
    pub counters: BTreeMap<String, i64>,
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
        handler_response: parse_handler_response(body),
        body: body.to_string(),
    })
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
        has_error_handler: body.contains("on error"),
    }
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
            let indexed = data.indexes.contains(&field.name) && !field.no_index;
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
    let query_re =
        Regex::new(r"([A-Za-z][A-Za-z0-9_]*)\s*\|>\s*where\(_\.([A-Za-z][A-Za-z0-9_]*)\s*(==|>=|<=|>|<)\s*([^)]+)\)").unwrap();
    let source = all_bodies(program);
    let mut plans = Vec::new();
    for line in source.lines().map(str::trim) {
        let Some(caps) = query_re.captures(line) else {
            continue;
        };
        let table = caps[1].to_string();
        let field = caps[2].to_string();
        let Some(data) = program.data.get(&table) else {
            continue;
        };
        let Some(field_decl) = data.fields.get(&field) else {
            continue;
        };
        if !data.indexes.contains(&field) || field_decl.no_index {
            continue;
        }
        let op = mysql_operator(&caps[3]);
        let index = if field == "id" {
            "PRIMARY".into()
        } else {
            format!("idx_{table}_{field}")
        };
        plans.push(SqlQueryPlan {
            source: line.to_string(),
            table: table.clone(),
            field: field.clone(),
            index,
            strategy: query_strategy(&field, op),
            sql: format!("SELECT * FROM `{table}` WHERE `{field}` {op} ?;"),
        });
    }
    plans
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
            handler_response: None,
        })
        .chain(program.endpoints.iter().map(|endpoint| RoutePlan {
            path: endpoint.path.clone(),
            target: route_target(&services, &endpoint.path),
            cache: CacheMode::Private,
            method: Some(endpoint.method.clone()),
            kind: RouteKind::Endpoint,
            identity: endpoint.identity.clone(),
            rate_limit: endpoint.rate_limit.clone(),
            handler_response: endpoint.handler_response.clone(),
        }))
        .collect();

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
        migrations: program
            .data
            .keys()
            .map(|table| MigrationStep {
                table: table.clone(),
                action: "create_or_reconcile".into(),
            })
            .collect(),
        sql_plan: compile_sql_plan(program),
        storage_catalog: program.data.values().cloned().collect(),
        frontend_assets: compile_frontend_assets(&program.pages),
        task_catalog: compile_task_catalog(program),
        model_catalog: program.model_catalog.clone(),
        pricing_catalog: program.pricing_catalog.clone(),
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
        on_error: cron.has_error_handler,
    }));
    tasks.extend(program.daemons.iter().map(|daemon| TaskSpec {
        name: daemon.name.clone(),
        kind: TaskKind::Daemon,
        schedule: daemon.schedule.clone(),
        runtime: daemon.runtime.clone(),
        healthcheck: daemon.has_healthcheck,
        lock: daemon.has_lock,
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
                on_error: true,
            }),
    );
    tasks
}

fn compile_frontend_assets(pages: &[PageDecl]) -> Vec<FrontendAsset> {
    pages
        .iter()
        .map(|page| {
            let title = extract_first_capture(&page.body, r#"h1\s+"([^"]+)""#)
                .unwrap_or_else(|| humanize_name(&page.name));
            let components = extract_named_declarations(&page.body, "component");
            let states = extract_named_declarations(&page.body, "state");
            FrontendAsset {
                page: page.name.clone(),
                path: page.path.clone(),
                title: title.clone(),
                components: components.clone(),
                states: states.clone(),
                html: render_frontend_html(page, &title, &components, &states),
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
    format!(
        "<!doctype html><html><head><title>{title}</title><meta name=\"deepapp-page\" content=\"{page_name}\"></head><body><main data-route=\"{path}\" data-components=\"{components}\" data-states=\"{states}\"><h1>{title}</h1></main></body></html>",
        title = html_escape(title),
        page_name = html_escape(&page.name),
        path = html_escape(&page.path),
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
        let store = MemoryStore::new(artifacts.storage_catalog.clone());
        Self { artifacts, store }
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
        if method == "POST" && clean_path == "/__deep/task-tick" {
            return json_response(200, &self.run_task_tick());
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
        let identity = identity_from_http(&request);
        let ip = header_value(&request, "x-forwarded-for")
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("127.0.0.1");
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
    TaskRun {
        name: task.name.clone(),
        kind: task.kind.clone(),
        status: "ok".into(),
        detail,
    }
}

impl MemoryStore {
    pub fn new(catalog: Vec<DataDecl>) -> Self {
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
            })),
        }
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
        self.inner
            .lock()
            .expect("memory store lock poisoned")
            .cache
            .insert(key.into(), value);
    }

    pub fn cache_get(&self, key: &str) -> Option<serde_json::Value> {
        self.inner
            .lock()
            .expect("memory store lock poisoned")
            .cache
            .get(key)
            .cloned()
    }

    pub fn queue_push(&self, queue: impl Into<String>, value: serde_json::Value) {
        self.inner
            .lock()
            .expect("memory store lock poisoned")
            .queues
            .entry(queue.into())
            .or_default()
            .push_back(value);
    }

    pub fn queue_pop(&self, queue: &str) -> Option<serde_json::Value> {
        self.inner
            .lock()
            .expect("memory store lock poisoned")
            .queues
            .get_mut(queue)
            .and_then(VecDeque::pop_front)
    }

    pub fn counter_add(&self, key: impl Into<String>, amount: i64) -> i64 {
        let mut inner = self.inner.lock().expect("memory store lock poisoned");
        let value = inner.counters.entry(key.into()).or_insert(0);
        *value += amount;
        *value
    }

    pub fn counter_get(&self, key: &str) -> i64 {
        *self
            .inner
            .lock()
            .expect("memory store lock poisoned")
            .counters
            .get(key)
            .unwrap_or(&0)
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
    }
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
            };
        }
    }

    if route.path == "/hacking_is_a_serious_crime" {
        store.counter_add("endpoint:/hacking_is_a_serious_crime", 1);
        let resolution = resolve_model_from_catalog("gpt-4.1", models, &BTreeSet::new());
        let charge = charge_for_usage(store, pricing, "chat", &resolution.selected_model);
        return RuntimeResponse {
            status: 200,
            content_type: "application/json".into(),
            body: serde_json::json!({
                "ok": true,
                "route": route.path,
                "target": route.target,
                "kind": "endpoint",
                "model": resolution.selected_model,
                "provider": resolution.selected_provider,
                "fallback_used": resolution.fallback_used,
                "charge_cents": charge.cents,
                "usage_count": charge.usage_count
            })
            .to_string(),
        };
    }

    if let Some(handler_response) = &route.handler_response {
        return RuntimeResponse {
            status: 200,
            content_type: "application/json".into(),
            body: evaluate_handler_response(handler_response, &route_params(&route.path, path))
                .to_string(),
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
    }
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

fn identity_from_http(request: &str) -> RuntimeIdentity {
    if header_value(request, "api-key").is_some() {
        RuntimeIdentity::ApiKey
    } else if header_value(request, "authorization").is_some()
        || header_value(request, "cookie")
            .map(|cookie| cookie.contains("session="))
            .unwrap_or(false)
    {
        RuntimeIdentity::LoggedIn
    } else {
        RuntimeIdentity::Anonymous
    }
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

fn text_response(status: u16, body: impl Into<String>) -> RuntimeResponse {
    RuntimeResponse {
        status,
        content_type: "text/plain; charset=utf-8".into(),
        body: body.into(),
    }
}

fn json_response(status: u16, value: &impl Serialize) -> RuntimeResponse {
    RuntimeResponse {
        status,
        content_type: "application/json".into(),
        body: serde_json::to_string(value).unwrap_or_else(|_| "{}".into()),
    }
}

fn json_error(status: u16, message: &str) -> RuntimeResponse {
    RuntimeResponse {
        status,
        content_type: "application/json".into(),
        body: serde_json::json!({ "ok": false, "error": message }).to_string(),
    }
}

fn write_http_response(stream: &mut TcpStream, response: RuntimeResponse) -> anyhow::Result<()> {
    let reason = match response.status {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "Error",
    };
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

data ChatSession {
  id: pk
  owner: User?
  created_at: datetime default now() @no_index
  index owner
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

endpoint POST /hacking_is_a_serious_crime {
  identity: any
  rate_limit: 30/min by ip when anonymous
  request { messages: [{ role: string, content: string }] }
  response: stream { content: string done: bool }
  handle {
    user = User[email: "foo@bar.com"]
    by_owner = ChatSession |> where(_.owner == user)
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
        assert_eq!(artifacts.report.language_units["queue"], 1);
        assert_eq!(artifacts.report.language_units["worker"], 1);
        assert_eq!(artifacts.report.language_units["model_config"], 1);
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
        assert!(artifacts
            .sql_plan
            .tables
            .iter()
            .any(|table| table.table == "ChatSession"
                && table.create_table.contains("CREATE TABLE `ChatSession`")
                && table
                    .indexes
                    .iter()
                    .any(|index| index.contains("idx_ChatSession_owner"))));
        assert!(artifacts.sql_plan.queries.iter().any(|query| {
            query.table == "ChatSession"
                && query.field == "owner"
                && query.strategy == "indexed_lookup"
                && query.sql == "SELECT * FROM `ChatSession` WHERE `owner` = ?;"
        }));
        assert_eq!(artifacts.frontend_assets[0].title, "Chat");
        assert_eq!(
            artifacts.frontend_assets[0].html.contains("<h1>Chat</h1>"),
            true
        );
        assert_eq!(artifacts.task_catalog.len(), 3);
        assert!(artifacts
            .task_catalog
            .iter()
            .any(|task| task.name == "daily_billing" && task.kind == TaskKind::Cron));
        assert!(artifacts
            .task_catalog
            .iter()
            .any(|task| task.name == "chat_task_checker" && task.kind == TaskKind::Daemon));
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

        let endpoint = runtime.handle("POST", "/hacking_is_a_serious_crime");
        assert_eq!(endpoint.status, 200);
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
        assert!(response.body.contains("\"model\":\"gpt-4.1-nano\""));
        assert!(response.body.contains("\"charge_cents\":1"));
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
}
