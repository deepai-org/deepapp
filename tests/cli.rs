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

    let manifest = fs::read_to_string(temp.join("manifest.json")).unwrap();
    assert!(manifest.contains("\"rollback\": \"automatic\""));
    assert!(manifest.contains("\"identity\": \"logged_in | api_key\""));
    assert!(manifest.contains("\"rate_limit\""));
    assert!(manifest.contains("\"limit\": 30"));
    let migrations = fs::read_to_string(temp.join("migrations.json")).unwrap();
    assert!(migrations.contains("\"phase\": \"pre_deploy\""));
    assert!(migrations.contains("\"resumable\": true"));
    assert!(migrations.contains("\"checkpoint_key\""));
    let storage_catalog = fs::read_to_string(temp.join("storage-catalog.json")).unwrap();
    assert!(storage_catalog.contains("\"name\": \"User\""));
    let sql_plan = fs::read_to_string(temp.join("sql-plan.json")).unwrap();
    assert!(sql_plan.contains("\"dialect\": \"mysql\""));
    assert!(sql_plan.contains("idx_ChatSession_owner"));
    let frontend_assets = fs::read_to_string(temp.join("frontend-assets.json")).unwrap();
    assert!(frontend_assets.contains("\"page\": \"chat\""));
    let task_catalog = fs::read_to_string(temp.join("task-catalog.json")).unwrap();
    assert!(task_catalog.contains("\"name\": \"daily_billing\""));
    let model_catalog = fs::read_to_string(temp.join("model-catalog.json")).unwrap();
    assert!(model_catalog.contains("\"name\": \"gpt-4.1-nano\""));
    let pricing_catalog = fs::read_to_string(temp.join("pricing-catalog.json")).unwrap();
    assert!(pricing_catalog.contains("\"category\": \"chat\""));

    let _ = fs::remove_dir_all(temp);
}
