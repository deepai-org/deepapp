use std::fs;
use std::process::Command;

#[test]
fn cli_builds_example_artifacts() {
    let temp = std::env::temp_dir().join(format!("deepapp-test-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);

    let status = Command::new(env!("CARGO_BIN_EXE_deepapp"))
        .args(["build", "examples/chat.deep", "--out"])
        .arg(&temp)
        .status()
        .expect("run deepapp build");

    assert!(status.success());
    assert!(temp.join("manifest.json").exists());
    assert!(temp.join("static-report.json").exists());
    assert!(temp.join("runtime-bundle.json").exists());
    assert!(temp.join("migrations.json").exists());
    assert!(temp.join("sql-plan.json").exists());
    assert!(temp.join("storage-catalog.json").exists());
    assert!(temp.join("frontend-assets.json").exists());
    assert!(temp.join("task-catalog.json").exists());
    assert!(temp.join("model-catalog.json").exists());
    assert!(temp.join("pricing-catalog.json").exists());
    assert!(temp.join("redis-catalog.json").exists());

    let manifest = fs::read_to_string(temp.join("manifest.json")).unwrap();
    assert!(manifest.contains("\"rollback\": \"automatic\""));
    assert!(manifest.contains("\"identity\": \"logged_in | api_key\""));
    assert!(manifest.contains("\"rate_limit\""));
    assert!(manifest.contains("\"limit\": 30"));
    assert!(manifest.contains("\"response_stream\": true"));
    assert!(manifest.contains("\"lock_name\": \"billing[user.id]\""));
    let migrations = fs::read_to_string(temp.join("migrations.json")).unwrap();
    assert!(migrations.contains("\"phase\": \"pre_deploy\""));
    assert!(migrations.contains("\"resumable\": true"));
    assert!(migrations.contains("\"checkpoint_key\""));
    let storage_catalog = fs::read_to_string(temp.join("storage-catalog.json")).unwrap();
    assert!(storage_catalog.contains("\"name\": \"User\""));
    let sql_plan = fs::read_to_string(temp.join("sql-plan.json")).unwrap();
    assert!(sql_plan.contains("\"dialect\": \"mysql\""));
    assert!(sql_plan.contains("idx_ChatSession_owner"));
    assert!(sql_plan.contains("\"strategy\": \"pk_range_scan\""));
    assert!(sql_plan.contains("BETWEEN ? AND ?"));
    let frontend_assets = fs::read_to_string(temp.join("frontend-assets.json")).unwrap();
    assert!(frontend_assets.contains("\"page\": \"chat\""));
    assert!(frontend_assets.contains("\"api_base_url\": \"https://api.deepai.org\""));
    assert!(frontend_assets.contains("\"data_loading\": \"client_fetch\""));
    let task_catalog = fs::read_to_string(temp.join("task-catalog.json")).unwrap();
    assert!(task_catalog.contains("\"name\": \"daily_billing\""));
    assert!(task_catalog.contains("\"lock_name\": \"daily_billing\""));
    assert!(task_catalog.contains("\"lock_ttl_seconds\": 1800"));
    let model_catalog = fs::read_to_string(temp.join("model-catalog.json")).unwrap();
    assert!(model_catalog.contains("\"name\": \"gpt-4.1-nano\""));
    let pricing_catalog = fs::read_to_string(temp.join("pricing-catalog.json")).unwrap();
    assert!(pricing_catalog.contains("\"category\": \"chat\""));
    let redis_catalog = fs::read_to_string(temp.join("redis-catalog.json")).unwrap();
    assert!(redis_catalog.contains("\"name\": \"research_tasks\""));
    assert!(redis_catalog.contains("\"backend\": \"redis_sorted_set\""));
    assert!(redis_catalog.contains("\"ttl_seconds\": 3600"));

    let _ = fs::remove_dir_all(temp);
}
