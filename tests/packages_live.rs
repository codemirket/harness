//! Opt-in tests of reviewed package binaries, using an explicitly disposable home.
use anyhow::{Context, Result, ensure};
use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequestParams, CallToolResult},
    service::RunningService,
    transport::TokioChildProcess,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

fn home() -> Result<PathBuf> {
    let path =
        PathBuf::from(std::env::var_os("MIRKET_PACKAGE_TEST_HOME").context(
            "set MIRKET_PACKAGE_TEST_HOME to the explicit disposable package installation",
        )?)
        .canonicalize()?;
    ensure!(
        fs::read(path.join(".mirket-package-test-home"))? == b"disposable package test home\n",
        "test-home marker is required"
    );
    Ok(path)
}
fn project() -> Result<tempfile::TempDir> {
    Ok(tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?)
}
async fn session(home: &Path, project: &Path, id: &str) -> Result<RunningService<RoleClient, ()>> {
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_mirket"));
    command
        .arg("--home")
        .arg(home)
        .args(["package", "serve", id, "--project"])
        .arg(project)
        .stderr(Stdio::null());
    Ok(tokio::time::timeout(
        Duration::from_secs(60),
        ().serve(TokioChildProcess::new(command)?),
    )
    .await??)
}
async fn call(
    service: &RunningService<RoleClient, ()>,
    name: &str,
    args: Value,
) -> Result<CallToolResult> {
    Ok(tokio::time::timeout(
        Duration::from_secs(120),
        service.peer().call_tool(
            CallToolRequestParams::new(name.to_string())
                .with_arguments(args.as_object().context("object arguments")?.clone()),
        ),
    )
    .await??)
}
fn text(result: &CallToolResult) -> Result<String> {
    ensure!(
        result.is_error != Some(true),
        "tool failed: {}",
        serde_json::to_string(result)?
    );
    Ok(result
        .content
        .iter()
        .filter_map(|c| c.as_text().map(|t| t.text.as_str()))
        .collect::<Vec<_>>()
        .join("\n"))
}

#[tokio::test]
#[ignore = "uses explicitly installed pinned browser and disposable package home"]
async fn packages_live_browser_changes_real_page_and_blocks_node_execution() -> Result<()> {
    let home = home()?;
    let project = project()?;
    let service = session(&home, project.path(), "playwright").await?;
    let names = service
        .peer()
        .list_all_tools()
        .await?
        .into_iter()
        .map(|t| t.name.to_string())
        .collect::<Vec<_>>();
    ensure!(
        !names.contains(&"browser_run_code_unsafe".into())
            && !names.contains(&"browser_install".into()),
        "forbidden browser tools advertised"
    );
    let page = "data:text/html,<title>Mirket fixture</title><h1>Inventory</h1><button onclick=\"this.textContent=String(42)\">Check inventory</button>";
    text(&call(&service, "browser_navigate", json!({"url":page})).await?)?;
    let diagnostics=text(&call(&service,"browser_evaluate",json!({"function":"() => ({state: document.visibilityState, button: document.querySelector('button').getBoundingClientRect().toJSON(), style: getComputedStyle(document.querySelector('button')).visibility})"})).await?)?;
    eprintln!("browser pre-click state: {diagnostics}");
    text(
        &call(
            &service,
            "browser_click",
            json!({"target":"button","element":"Check inventory"}),
        )
        .await?,
    )?;
    let value = text(
        &call(
            &service,
            "browser_evaluate",
            json!({"function":"() => document.querySelector('button').textContent"}),
        )
        .await?,
    )?;
    ensure!(value.contains("42"), "real page did not change: {value}");
    ensure!(
        call(
            &service,
            "browser_run_code_unsafe",
            json!({"code":"return 42"})
        )
        .await?
        .is_error
            == Some(true),
        "unsafe Node execution was exposed"
    );
    text(&call(&service, "browser_close", json!({})).await?)?;
    service.cancel().await?;
    Ok(())
}

#[tokio::test]
#[ignore = "uses explicitly installed DBHub and disposable package home"]
async fn packages_live_sqlite_queries_and_rejects_writes() -> Result<()> {
    let home = home()?;
    let project = project()?;
    let database = project.path().join("inventory.sqlite");
    let connection = rusqlite::Connection::open(&database)?;
    connection.execute_batch("CREATE TABLE products(name TEXT, units INTEGER); INSERT INTO products VALUES('pens',3),('books',7);")?;
    drop(connection);
    let before = fs::read(&database)?;
    let supplied_database = if cfg!(unix) {
        PathBuf::from(format!("/{}", database.display()))
    } else {
        database.clone()
    };
    let output = tokio::process::Command::new(env!("CARGO_BIN_EXE_mirket"))
        .arg("--home")
        .arg(&home)
        .args(["package", "configure", "dbhub", "--database"])
        .arg(&supplied_database)
        .output()
        .await?;
    ensure!(
        output.status.success(),
        "DBHub config failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&fs::read(home.join(".mirket/setup.json"))?)?;
    ensure!(
        receipt["options"]["package_settings"]["dbhub"]["database"] == database.to_str().unwrap(),
        "database alias was not normalized"
    );
    let service = session(&home, project.path(), "dbhub").await?;
    let result = text(
        &call(
            &service,
            "execute_sql",
            json!({"sql":"SELECT SUM(units) AS total FROM products"}),
        )
        .await?,
    )?;
    ensure!(result.contains("10"), "wrong aggregate: {result}");
    for sql in [
        "INSERT INTO products VALUES('bad',99)",
        "DROP TABLE products",
        "ATTACH DATABASE 'escape.sqlite' AS extra",
        "PRAGMA query_only=OFF",
        "PRAGMA \"query_only\"(OFF)",
        "SELECT 1; DELETE FROM products",
    ] {
        let result = call(&service, "execute_sql", json!({"sql":sql})).await?;
        ensure!(
            result.is_error == Some(true),
            "readonly probe unexpectedly accepted {sql}: {}",
            serde_json::to_string(&result)?
        );
    }
    let result = text(
        &call(
            &service,
            "execute_sql",
            json!({"sql":"SELECT SUM(units) AS total FROM products"}),
        )
        .await?,
    )?;
    ensure!(
        result.contains("10"),
        "database changed after failure probes"
    );
    service.cancel().await?;
    ensure!(fs::read(&database)? == before, "SQLite file bytes changed");
    ensure!(
        !project.path().join("escape.sqlite").exists(),
        "ATTACH created an extra database"
    );
    Ok(())
}
