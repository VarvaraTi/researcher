//! First-run scaffolding only; existing learning records are never rewritten.
use serde_json::{json, Value};
use std::{fs, io::Write, path::Path, process::Command};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn create_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = tempfile::NamedTempFile::new_in(path.parent().ok_or("Missing parent")?)?;
    file.write_all(bytes)?;
    file.persist_noclobber(path)?;
    Ok(())
}

fn okf(executable: &Path, cwd: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new(executable)
        .current_dir(cwd)
        .args(args)
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "OKF failed: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    Ok(())
}

fn setup_library(workspace: &Path, executable: &Path) -> Result<Value> {
    let catalog_path = workspace.join("okf-catalog.yaml");
    if catalog_path.is_symlink()
        || workspace.join("okf-catalog.local.yaml").exists()
        || workspace.join("okf.toml").exists()
    {
        return Err("Catalog overrides or symlink require explicit review".into());
    }
    let original = fs::read(&catalog_path)?;
    let mut catalog: Value = serde_yaml::from_slice(&original)?;
    if catalog["catalog_version"] != 2 || !catalog["bundles"].is_object() {
        return Err("Expected catalog version 2".into());
    }
    let path = workspace.join("courses/library");
    if path.is_symlink() {
        return Err("Refusing symlinked library".into());
    }
    if let Some(entry) = catalog["bundles"].get("learning.library") {
        if entry["location"]["type"] != "directory"
            || entry["location"]["path"] != "courses/library"
            || !path.join("ontology.yaml").is_file()
        {
            return Err("Existing library registration conflicts or bundle is missing".into());
        }
        okf(
            executable,
            workspace,
            &["validate", path.to_str().ok_or("Non-UTF8 path")?],
        )?;
        return Ok(json!({"library":"learning.library","status":"ready"}));
    }
    if path.exists() {
        return Err("Unregistered library directory requires review".into());
    }
    fs::create_dir_all(workspace.join("courses"))?;
    let stage = tempfile::tempdir_in(workspace.join("courses"))?;
    let staged = stage.path().join("library");
    let staged_str = staged.to_str().ok_or("Non-UTF8 path")?;
    okf(
        executable,
        stage.path(),
        &["init", staged_str, "--title", "Shared book library"],
    )?;
    let schema = stage.path().join("schema.yaml");
    fs::write(
        &schema,
        include_str!("../../../../schemas/library-ontology.yaml"),
    )?;
    okf(
        executable,
        stage.path(),
        &[
            "ontology",
            "apply",
            staged_str,
            "--from",
            schema.to_str().ok_or("Non-UTF8 path")?,
        ],
    )?;
    fs::write(
        staged.join("okf-bundle.yaml"),
        include_str!("../../../../templates/library-bundle.yaml"),
    )?;
    okf(executable, stage.path(), &["validate", staged_str])?;
    if fs::read(&catalog_path)? != original {
        return Err("Catalog changed during setup; retry".into());
    }
    fs::rename(&staged, &path)?;
    catalog["bundles"]["learning.library"] =
        json!({"location":{"type":"directory","path":"courses/library"}});
    let mut replacement = tempfile::NamedTempFile::new_in(workspace)?;
    replacement.write_all(serde_yaml::to_string(&catalog)?.as_bytes())?;
    replacement.persist(&catalog_path)?;
    Ok(json!({"library":"learning.library","status":"created"}))
}

fn run() -> Result<Value> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 && !(args.len() == 3 && args[2] == "--library") {
        return Err("Usage: learning-workspace WORKSPACE OKF_EXECUTABLE [--library]".into());
    }
    let executable = fs::canonicalize(&args[1])?;
    let version = Command::new(&executable).arg("version").output()?;
    if !version.status.success()
        || String::from_utf8_lossy(&version.stdout).trim() != "okf 0.6.0 (OKF spec 0.2)"
    {
        return Err("Expected OKF 0.6.0".into());
    }
    fs::create_dir_all(&args[0])?;
    let workspace = fs::canonicalize(&args[0])?;
    if args.len() == 3 {
        return setup_library(&workspace, &executable);
    }
    let config_path = workspace.join("learning.local.json");
    let catalog_path = workspace.join("okf-catalog.yaml");
    let ignore_path = workspace.join(".gitignore");
    for path in [&config_path, &catalog_path, &ignore_path] {
        if path.is_symlink() {
            return Err(format!("Refusing a symlink: {}", path.display()).into());
        }
    }
    let config: Value = if config_path.exists() {
        serde_json::from_slice(&fs::read(&config_path)?)?
    } else {
        json!({"schema_version":1,"learner_bundle":"learning.learner","selected_course":null})
    };
    if config["schema_version"] != 1 {
        return Err("Unsupported learning.local.json schema_version".into());
    }
    let learner_id = config["learner_bundle"]
        .as_str()
        .filter(|id| !id.is_empty())
        .ok_or("Missing learner_bundle in learning.local.json")?;
    let catalog_bytes = if catalog_path.exists() {
        Some(fs::read(&catalog_path)?)
    } else {
        None
    };
    let mut catalog: Value = match &catalog_bytes {
        Some(bytes) => serde_yaml::from_slice(bytes)?,
        None => json!({"catalog_version":2,"default_bundle":learner_id,"bundles":{}}),
    };
    if catalog["catalog_version"] != 2 || !catalog["bundles"].is_object() {
        return Err("Expected catalog_version 2 and a bundles mapping; migrate explicitly".into());
    }
    if workspace.join("okf-catalog.local.yaml").exists() || workspace.join("okf.toml").exists() {
        return Err("Existing OKF overrides require explicit review before workspace setup".into());
    }
    let registration = catalog["bundles"].get(learner_id);
    let registered = registration.is_some();
    let learner_path = match registration {
        Some(entry) => {
            if entry["location"]["type"] != "directory" {
                return Err("Learner bundle must be a local directory".into());
            }
            workspace.join(
                entry["location"]["path"]
                    .as_str()
                    .ok_or("Missing learner path")?,
            )
        }
        None => workspace.join("learner"),
    };
    if let Some(selected) = config["selected_course"].as_str() {
        if catalog["bundles"].get(selected).is_none() {
            return Err(
                "Selected course is not registered; resolve the existing selection first".into(),
            );
        }
    }
    if learner_path.is_symlink() {
        return Err("Refusing a symlinked learner directory".into());
    }
    if registered && !learner_path.exists() {
        return Err("Registered learner bundle is missing; restore it instead of creating an empty replacement".into());
    }
    if learner_path.exists() {
        if !learner_path.join("index.md").is_file() || !learner_path.join("ontology.yaml").is_file()
        {
            return Err(
                "Existing learner directory is incomplete; inspect it before retrying".into(),
            );
        }
        if !registered
            && catalog["bundles"]
                .as_object()
                .is_some_and(|m| !m.is_empty())
        {
            return Err("Unregistered learner directory conflicts with an existing catalog; reconcile explicitly".into());
        }
        okf(
            &executable,
            &workspace,
            &[
                "validate",
                learner_path.to_str().ok_or("Non-UTF8 learner path")?,
            ],
        )?;
    } else {
        // Prepare the bundle in isolation before exposing it in the workspace.
        let stage = tempfile::tempdir_in(&workspace)?;
        let staged = stage.path().join("learner");
        let staged_str = staged.to_str().ok_or("Non-UTF8 staging path")?;
        okf(
            &executable,
            stage.path(),
            &["init", staged_str, "--title", "Learner"],
        )?;
        let ontology = stage.path().join("learner-schema.yaml");
        fs::write(
            &ontology,
            include_str!("../../../../schemas/learner-ontology.yaml"),
        )?;
        okf(
            &executable,
            stage.path(),
            &[
                "ontology",
                "apply",
                staged_str,
                "--from",
                ontology.to_str().ok_or("Non-UTF8 schema path")?,
            ],
        )?;
        fs::write(
            staged.join("okf-bundle.yaml"),
            include_str!("../../../../templates/learner-bundle.yaml"),
        )?;
        okf(&executable, stage.path(), &["validate", staged_str])?;
        fs::rename(staged, &learner_path)?;
    }
    fs::create_dir_all(workspace.join("courses"))?;
    if !registered {
        catalog["bundles"][learner_id] = json!({"location":{"type":"directory","path":"learner"}});
    }
    match &catalog_bytes {
        None => create_file(&catalog_path, serde_yaml::to_string(&catalog)?.as_bytes())?,
        Some(bytes) if !registered => {
            // Preserve the original including comments before adding one registration.
            let backup = workspace.join("okf-catalog.before-researcher.yaml");
            create_file(&backup, bytes)?;
            let mut replacement = tempfile::NamedTempFile::new_in(&workspace)?;
            replacement.write_all(serde_yaml::to_string(&catalog)?.as_bytes())?;
            replacement.persist(&catalog_path)?;
        }
        Some(_) => {}
    }
    if !config_path.exists() {
        create_file(
            &config_path,
            serde_json::to_string_pretty(&config)?.as_bytes(),
        )?;
    }
    let mut ignore = if ignore_path.exists() {
        fs::read_to_string(&ignore_path)?
    } else {
        String::new()
    };
    let original_ignore = ignore.clone();
    for pattern in [
        "/.local/",
        "/learner/",
        "/courses/",
        "/learning.local.json",
        "/okf-catalog.yaml",
        "/okf-catalog.local.yaml",
        "/okf.toml",
        "/okf-catalog.before-researcher.yaml",
        "/exports/",
        "/backups/",
        "/.learning-stage-*/",
    ] {
        if !ignore.lines().any(|line| line == pattern) {
            if !ignore.is_empty() && !ignore.ends_with('\n') {
                ignore.push('\n');
            }
            ignore.push_str(pattern);
            ignore.push('\n');
        }
    }
    if ignore != original_ignore {
        let mut replacement = tempfile::NamedTempFile::new_in(&workspace)?;
        replacement.write_all(ignore.as_bytes())?;
        replacement.persist(&ignore_path)?;
    }
    Ok(
        json!({"workspace":workspace,"learner_bundle":learner_id,"selected_course":config["selected_course"],"status":"ready"}),
    )
}

fn main() {
    match run() {
        Ok(value) => println!("{value}"),
        Err(error) => {
            eprintln!("Workspace setup failed: {error}");
            std::process::exit(1);
        }
    }
}
