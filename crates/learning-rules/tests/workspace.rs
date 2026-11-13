use serde_json::{json, Value};
use std::{fs, path::Path, process::Command};

fn setup(workspace: &Path) -> std::process::Output {
    let okf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/bin/okf");
    Command::new(env!("CARGO_BIN_EXE_learning-workspace"))
        .arg(workspace)
        .arg(okf)
        .output()
        .unwrap()
}

#[test]
fn fresh_workspace_and_retry_preserve_personal_data_and_selection() {
    let temp = tempfile::tempdir().unwrap();
    let workspace = temp.path().join("learning space");
    let result = setup(&workspace);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(workspace.join("learner/ontology.yaml").is_file());
    assert!(workspace.join("learner/okf-bundle.yaml").is_file());
    assert!(workspace.join("courses").is_dir());
    let config: Value =
        serde_json::from_slice(&fs::read(workspace.join("learning.local.json")).unwrap()).unwrap();
    assert!(config["selected_course"].is_null());
    let catalog_path = workspace.join("okf-catalog.yaml");
    let mut catalog: Value = serde_yaml::from_slice(&fs::read(&catalog_path).unwrap()).unwrap();
    catalog["bundles"]["learning.book"] =
        json!({"location":{"type":"directory","path":"courses/book"}});
    fs::write(&catalog_path, serde_yaml::to_string(&catalog).unwrap()).unwrap();
    let config = br#"{"schema_version":1,"learner_bundle":"learning.learner","selected_course":"learning.book","custom":"preserve"}"#;
    fs::write(workspace.join("learning.local.json"), config).unwrap();
    fs::write(
        workspace.join("learner/answer.txt"),
        "Original learner submission",
    )
    .unwrap();
    let catalog_before = fs::read(&catalog_path).unwrap();
    let ontology_before = fs::read(workspace.join("learner/ontology.yaml")).unwrap();
    let result = setup(&workspace);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        fs::read(workspace.join("learning.local.json")).unwrap(),
        config
    );
    assert_eq!(fs::read(catalog_path).unwrap(), catalog_before);
    assert_eq!(
        fs::read(workspace.join("learner/ontology.yaml")).unwrap(),
        ontology_before
    );
    assert_eq!(
        fs::read_to_string(workspace.join("learner/answer.txt")).unwrap(),
        "Original learner submission"
    );
}

#[test]
fn extends_catalog_with_exact_backup_and_preserves_default() {
    let temp = tempfile::tempdir().unwrap();
    let original = "# Existing catalog\ncatalog_version: 2\ndefault_bundle: work.notes\nbundles:\n  work.notes:\n    location: {type: directory, path: notes}\n";
    fs::write(temp.path().join("okf-catalog.yaml"), original).unwrap();
    let result = setup(temp.path());
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("okf-catalog.before-researcher.yaml")).unwrap(),
        original
    );
    let catalog: Value =
        serde_yaml::from_slice(&fs::read(temp.path().join("okf-catalog.yaml")).unwrap()).unwrap();
    assert_eq!(catalog["default_bundle"], "work.notes");
    assert_eq!(catalog["bundles"].as_object().unwrap().len(), 2);
}

#[test]
fn missing_registered_learner_is_not_replaced_with_empty_data() {
    let temp = tempfile::tempdir().unwrap();
    let original = "catalog_version: 2\nbundles:\n  learning.learner:\n    location: {type: directory, path: missing}\n";
    fs::write(temp.path().join("okf-catalog.yaml"), original).unwrap();
    assert!(!setup(temp.path()).status.success());
    assert!(!temp.path().join("missing").exists());
    assert!(!temp.path().join("learning.local.json").exists());
    assert_eq!(
        fs::read_to_string(temp.path().join("okf-catalog.yaml")).unwrap(),
        original
    );
}

#[test]
fn malformed_selection_does_not_create_learning_records() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("learning.local.json"), "{broken").unwrap();
    assert!(!setup(temp.path()).status.success());
    assert!(!temp.path().join("learner").exists());
    assert!(!temp.path().join("okf-catalog.yaml").exists());
}

#[test]
fn library_setup_is_on_demand_and_idempotent() {
    let temp = tempfile::tempdir().unwrap();
    assert!(setup(temp.path()).status.success());
    assert!(!temp.path().join("courses/library").exists());
    let config = fs::read(temp.path().join("learning.local.json")).unwrap();
    let okf = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.local/bin/okf");
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_learning-workspace"))
            .arg(temp.path())
            .arg(&okf)
            .arg("--library")
            .output()
            .unwrap()
    };
    let first = run();
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let catalog = fs::read(temp.path().join("okf-catalog.yaml")).unwrap();
    let second = run();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert_eq!(
        catalog,
        fs::read(temp.path().join("okf-catalog.yaml")).unwrap()
    );
    assert_eq!(
        config,
        fs::read(temp.path().join("learning.local.json")).unwrap()
    );
    assert!(temp.path().join("courses/library/ontology.yaml").is_file());
}
