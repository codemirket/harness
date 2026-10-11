//! Opt-in conversion checks through the actual pinned Python MCP package.
use anyhow::{Context, Result, ensure};
use rmcp::{ServiceExt, model::CallToolRequestParams, transport::TokioChildProcess};
use serde_json::json;
use std::{fs, io::Write, path::PathBuf, time::Duration};

#[tokio::test]
#[ignore = "uses the explicitly installed pinned Python package and disposable home"]
async fn packages_python_live_converts_local_documents_and_refuses_network_and_escape() -> Result<()>
{
    let home =
        PathBuf::from(std::env::var_os("MIRKET_PACKAGE_TEST_HOME").context(
            "set MIRKET_PACKAGE_TEST_HOME to an explicit disposable package installation",
        )?)
        .canonicalize()?;
    ensure!(
        fs::read(home.join(".mirket-package-test-home"))? == b"disposable package test home\n",
        "test-home marker is required"
    );
    let project = tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let outside = tempfile::tempdir_in(std::env::temp_dir().canonicalize()?)?;
    let html = project.path().join("inventory.html");
    fs::write(
        &html,
        "<!doctype html><html><body><h1>Inventory</h1><table><tr><th>Item</th><th>Units</th></tr><tr><td>Pens</td><td>3</td></tr><tr><td>Books</td><td>7</td></tr></table></body></html>",
    )?;
    let docx = project.path().join("brief.docx");
    let mut archive = zip::ZipWriter::new(fs::File::create(&docx)?);
    let options = zip::write::SimpleFileOptions::default();
    for (path, text) in [
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Mirket conversion fixture</w:t></w:r></w:p><w:p><w:r><w:t>Quarterly revenue: 120.</w:t></w:r></w:p></w:body></w:document>"#,
        ),
    ] {
        archive.start_file(path, options)?;
        archive.write_all(text.as_bytes())?;
    }
    archive.finish()?;
    let foreign = outside.path().join("private.txt");
    fs::write(&foreign, "must not be read")?;
    let audio = project.path().join("renamed.txt");
    fs::write(&audio, "RIFF audio content")?;

    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_mirket"));
    command
        .arg("--home")
        .arg(&home)
        .args(["package", "serve", "markitdown", "--project"])
        .arg(project.path());
    let service = tokio::time::timeout(
        Duration::from_secs(90),
        ().serve(TokioChildProcess::new(command)?),
    )
    .await??;
    let tools = service.peer().list_all_tools().await?;
    ensure!(
        tools.len() == 1 && tools[0].name == "convert_to_markdown",
        "unexpected conversion tools"
    );
    for (path, expected) in [
        (&html, vec!["Inventory", "Pens", "Books", "3", "7"]),
        (
            &docx,
            vec!["Mirket conversion fixture", "Quarterly revenue: 120."],
        ),
    ] {
        let uri = reqwest::Url::from_file_path(path).map_err(|()| anyhow::anyhow!("file URL"))?;
        let result = tokio::time::timeout(
            Duration::from_secs(120),
            service.peer().call_tool(
                CallToolRequestParams::new("convert_to_markdown")
                    .with_arguments(json!({"uri":uri.as_str()}).as_object().unwrap().clone()),
            ),
        )
        .await??;
        ensure!(
            result.is_error != Some(true),
            "conversion failed: {}",
            serde_json::to_string(&result)?
        );
        let text = result
            .content
            .iter()
            .filter_map(|item| item.as_text().map(|value| value.text.as_str()))
            .collect::<Vec<_>>()
            .join("\n");
        for value in expected {
            ensure!(text.contains(value), "conversion omitted {value}: {text}");
        }
    }
    for uri in [
        "https://example.com/private.pdf".into(),
        reqwest::Url::from_file_path(&foreign).unwrap().to_string(),
        reqwest::Url::from_file_path(&audio).unwrap().to_string(),
    ] {
        let result = service
            .peer()
            .call_tool(
                CallToolRequestParams::new("convert_to_markdown")
                    .with_arguments(json!({"uri":uri}).as_object().unwrap().clone()),
            )
            .await?;
        ensure!(
            result.is_error == Some(true),
            "disallowed conversion was accepted"
        );
    }
    service.cancel().await?;
    Ok(())
}
