use rmcp::{
    RoleClient, ServiceExt,
    model::{CallToolRequestParams, CallToolResult},
    service::RunningService,
    transport::TokioChildProcess,
};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};

type Client = RunningService<RoleClient, ()>;

fn fixture_home() -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os("MIRKET_PACKAGE_TEST_HOME")
            .expect("set MIRKET_PACKAGE_TEST_HOME to an existing disposable home"),
    );
    assert!(path.is_absolute(), "the disposable home must be absolute");
    let home = path.canonicalize().unwrap();
    assert!(home.is_dir());
    assert_eq!(
        fs::read(home.join(".mirket-package-test-home")).unwrap(),
        b"disposable package test home\n",
        "test-home marker is required"
    );
    home
}

fn retain(name: &str, bytes: &[u8]) {
    if let Some(path) = std::env::var_os("MIRKET_PACKAGE_TEST_OUTPUT") {
        let path = PathBuf::from(path);
        assert!(
            path.is_absolute(),
            "the optional evidence directory must be absolute"
        );
        let path = path.canonicalize().unwrap();
        assert!(path.is_dir());
        fs::write(path.join(name), bytes).unwrap();
    }
}

async fn start(home: &Path, project: &Path, package: &str) -> Client {
    assert!(home.is_absolute() && project.is_absolute());
    assert_eq!(project, project.canonicalize().unwrap());
    let mut command = tokio::process::Command::new(env!("CARGO_BIN_EXE_mirket"));
    command
        .arg("--home")
        .arg(home)
        .args(["package", "serve", package, "--project"])
        .arg(project)
        .kill_on_drop(true);
    tokio::time::timeout(
        Duration::from_secs(30),
        ().serve(TokioChildProcess::new(command).unwrap()),
    )
    .await
    .expect("package bridge startup deadline")
    .expect("selected package must start through the real Mirket CLI")
}

async fn call(client: &Client, name: &str, args: Value) -> CallToolResult {
    tokio::time::timeout(
        Duration::from_secs(30),
        client.call_tool(
            CallToolRequestParams::new(name.to_owned())
                .with_arguments(args.as_object().unwrap().clone()),
        ),
    )
    .await
    .expect("package tool deadline")
    .unwrap()
}

fn response_json(result: &CallToolResult) -> Value {
    if let Some(value) = &result.structured_content {
        return value.clone();
    }
    let response = serde_json::to_value(result).unwrap();
    response["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["text"].as_str())
        .find_map(|text| serde_json::from_str(text).ok())
        .expect("tool must return its documented JSON result")
}

fn docx_success(result: &CallToolResult) -> Value {
    assert_ne!(result.is_error, Some(true), "{result:?}");
    let value = response_json(result);
    assert_eq!(value["success"], true, "{value}");
    value
}

fn docx_failure(result: &CallToolResult) {
    if result.is_error != Some(true) {
        assert_eq!(response_json(result)["success"], false, "{result:?}");
    }
}

fn write_docx(path: &Path) {
    let mut archive = zip::ZipWriter::new(fs::File::create(path).unwrap());
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    let parts = [
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>"#,
        ),
        (
            "word/document.xml",
            r#"<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body><w:p><w:r><w:t>Quarterly planning note.</w:t></w:r></w:p><w:p><w:r><w:t>Revenue target is 100 units.</w:t></w:r></w:p><w:p><w:r><w:t>Keep this paragraph unchanged.</w:t></w:r></w:p><w:sectPr><w:pgSz w:w="11906" w:h="16838"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>"#,
        ),
    ];
    for (name, content) in parts {
        archive.start_file(name, options).unwrap();
        archive.write_all(content.as_bytes()).unwrap();
    }
    archive.finish().unwrap();
}

fn document_xml(path: &Path) -> String {
    let mut archive = zip::ZipArchive::new(fs::File::open(path).unwrap()).unwrap();
    let mut text = String::new();
    archive
        .by_name("word/document.xml")
        .unwrap()
        .take(1024 * 1024)
        .read_to_string(&mut text)
        .unwrap();
    text
}

fn paragraph_text(value: &Value) -> Vec<String> {
    let rows: Value = serde_json::from_str(value["content"].as_str().unwrap()).unwrap();
    rows.as_array()
        .unwrap()
        .iter()
        .map(|row| row["text"].as_str().unwrap().to_owned())
        .collect()
}

fn fixture_xml_text(xml: &str) -> String {
    // This ASCII fixture needs run concatenation, not a raw substring match:
    // tracked replacements may split a number across several text nodes.
    xml.split("<w:t")
        .skip(1)
        .filter(|run| run.starts_with('>') || run.starts_with(' '))
        .map(|run| {
            run.split_once('>')
                .unwrap()
                .1
                .split_once("</w:t>")
                .unwrap()
                .0
        })
        .collect()
}

#[tokio::test]
#[ignore = "requires safe-docx selected in an explicit MIRKET_PACKAGE_TEST_HOME"]
async fn packages_documents_live_docx_edit_save_reopen_and_refuse_escape() {
    let home = fixture_home();
    let project = tempfile::tempdir().unwrap();
    let project = project.path().canonicalize().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let input = project.join("input.docx");
    let saved = project.join("edited.docx");
    let tracked = project.join("tracked.docx");
    write_docx(&input);
    let original = fs::read(&input).unwrap();
    let client = start(&home, &project, "safe-docx").await;
    let tools = client.list_all_tools().await.unwrap();
    for name in ["read_file", "replace_text", "save"] {
        assert!(tools.iter().any(|tool| tool.name == name));
    }
    let read = docx_success(
        &call(
            &client,
            "read_file",
            json!({"file_path":input,"format":"json","show_formatting":false}),
        )
        .await,
    );
    assert_eq!(paragraph_text(&read).len(), 3);
    let rows: Value = serde_json::from_str(read["content"].as_str().unwrap()).unwrap();
    let anchor = rows[1]["id"].as_str().unwrap();
    assert_eq!(rows[1]["text"], "Revenue target is 100 units.");

    let invalid = call(&client, "replace_text", json!({"file_path":input,"target_paragraph_id":anchor,"old_string":"999 units","new_string":"120 units","instruction":"Update the planning target"})).await;
    docx_failure(&invalid);
    docx_success(&call(&client, "replace_text", json!({"file_path":input,"target_paragraph_id":anchor,"old_string":"100 units","new_string":"120 units","instruction":"Update the planning target"})).await);
    let escape = outside.path().join("escaped.docx");
    let denied = call(
        &client,
        "save",
        json!({"file_path":input,"save_to_local_path":escape,"save_format":"clean"}),
    )
    .await;
    docx_failure(&denied);
    assert!(
        !escape.exists(),
        "a denied save must not create an outside file"
    );
    let save = docx_success(&call(&client, "save", json!({"file_path":input,"save_to_local_path":saved,"tracked_save_to_local_path":tracked,"save_format":"both"})).await);
    retain("docx-save.json", &serde_json::to_vec_pretty(&save).unwrap());
    assert_eq!(
        fs::read(&input).unwrap(),
        original,
        "source bytes must be preserved"
    );
    let clean_xml = document_xml(&saved);
    let tracked_xml = document_xml(&tracked);
    retain("edited.docx", &fs::read(&saved).unwrap());
    retain("tracked.docx", &fs::read(&tracked).unwrap());
    retain("edited-document.xml", clean_xml.as_bytes());
    retain("tracked-document.xml", tracked_xml.as_bytes());
    assert_eq!(
        fixture_xml_text(&clean_xml),
        "Quarterly planning note.Revenue target is 120 units.Keep this paragraph unchanged."
    );
    assert!(!clean_xml.contains("w:ins") && !clean_xml.contains("w:del"));
    assert!(tracked_xml.contains("w:ins") && tracked_xml.contains("w:del"));
    let saved_bytes = fs::read(&saved).unwrap();
    let protected = project.join("protected.docx");
    fs::write(&protected, &original).unwrap();
    let protected_result = call(&client, "save", json!({"file_path":input,"save_to_local_path":protected,"save_format":"clean","allow_overwrite":false})).await;
    let protected_after = fs::read(&protected).unwrap();
    assert_eq!(fs::read(&saved).unwrap(), saved_bytes);
    client.cancel().await.unwrap();

    let reopened = start(&home, &project, "safe-docx").await;
    let read = docx_success(
        &call(
            &reopened,
            "read_file",
            json!({"file_path":saved,"format":"json","show_formatting":false}),
        )
        .await,
    );
    assert_eq!(
        paragraph_text(&read),
        [
            "Quarterly planning note.",
            "Revenue target is 120 units.",
            "Keep this paragraph unchanged."
        ]
    );
    reopened.cancel().await.unwrap();
    retain("edited.docx", &saved_bytes);
    retain("tracked.docx", &fs::read(&tracked).unwrap());
    retain("edited-document.xml", clean_xml.as_bytes());
    retain(
        "docx-reopened.json",
        &serde_json::to_vec_pretty(&read).unwrap(),
    );
    retain(
        "docx-overwrite-probe.json",
        &serde_json::to_vec_pretty(
            &json!({"bytes_preserved":protected_after == original,"response":protected_result}),
        )
        .unwrap(),
    );
    assert!(
        protected_after == original,
        "a refused save must preserve a preexisting document"
    );
    docx_failure(&protected_result);
}

#[tokio::test]
#[ignore = "requires semiotic selected in an explicit MIRKET_PACKAGE_TEST_HOME"]
async fn packages_documents_live_svg_chart_preserves_data_and_refuses_png() {
    let home = fixture_home();
    let project = tempfile::tempdir().unwrap();
    let project = project.path().canonicalize().unwrap();
    let client = start(&home, &project, "semiotic").await;
    let tools = client.list_all_tools().await.unwrap();
    let names: std::collections::BTreeSet<_> =
        tools.iter().map(|tool| tool.name.as_ref()).collect();
    assert_eq!(
        names,
        std::collections::BTreeSet::from([
            "suggestCharts",
            "getSchema",
            "diagnoseConfig",
            "renderChart",
            "auditAccessibility",
            "groundChart"
        ])
    );
    let data = json!([{"region":"North","revenue":120},{"region":"South","revenue":80},{"region":"West","revenue":100}]);
    let props = json!({"data":data,"categoryAccessor":"region","valueAccessor":"revenue","title":"Revenue by region","description":"North 120, South 80, West 100 units","width":640,"height":420});
    let suggestion = call(&client, "suggestCharts", json!({"data":data,"intent":"compare-categories","allow":["BarChart"],"fieldRoles":{"region":"category","revenue":"y"}})).await;
    assert_ne!(suggestion.is_error, Some(true));
    assert_eq!(
        response_json(&suggestion)["suggestions"][0]["component"],
        "BarChart"
    );
    let chart = call(
        &client,
        "renderChart",
        json!({"component":"BarChart","props":props,"format":"svg"}),
    )
    .await;
    assert_ne!(chart.is_error, Some(true), "{chart:?}");
    let value = serde_json::to_value(&chart).unwrap();
    retain(
        "semiotic-render.json",
        &serde_json::to_vec_pretty(&value).unwrap(),
    );
    let svg = value["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["text"].as_str())
        .find(|text| text.trim_start().starts_with("<svg"))
        .expect("static chart tool must return actual SVG, not a widget reference");
    for label in ["North", "South", "West"] {
        assert!(
            svg.contains(label),
            "saved SVG must contain category {label}"
        );
    }
    assert_eq!(
        svg.matches("<rect ").count(),
        3,
        "saved SVG must contain exactly three bar geometries"
    );
    let heights: Vec<f64> = svg
        .split("<rect ")
        .skip(1)
        .map(|rect| {
            rect.split("height=\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap()
                .parse()
                .unwrap()
        })
        .collect();
    assert!((heights[0] / heights[1] - 1.5).abs() < 1e-10);
    assert!((heights[2] / heights[1] - 1.25).abs() < 1e-10);
    let evidence = value["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|item| item["text"].as_str())
        .find_map(|text| text.strip_prefix("Render evidence:\n"))
        .expect("renderer must supply scene evidence");
    let evidence: Value = serde_json::from_str(evidence).unwrap();
    assert_eq!(evidence["markCount"], 3);
    assert_eq!(evidence["markCountByType"]["rect"], 3);
    assert_eq!(evidence["yDomain"][0], 0);
    assert!(evidence["yDomain"][1].as_f64().unwrap() >= 120.0);
    let grounding = call(
        &client,
        "groundChart",
        json!({"component":"BarChart","props":props}),
    )
    .await;
    assert_ne!(grounding.is_error, Some(true));
    let grounding = response_json(&grounding);
    let description = grounding["description"]["text"].as_str().unwrap();
    assert!(
        description.contains("80 (South)")
            && description.contains("120 (North)")
            && description.contains("mean of 100"),
        "{description}"
    );
    let denied = call(
        &client,
        "renderChart",
        json!({"component":"BarChart","props":props,"format":"png"}),
    )
    .await;
    assert_eq!(
        denied.is_error,
        Some(true),
        "PNG must be rejected instead of offering npm installation or silently returning SVG"
    );
    let invalid = call(
        &client,
        "renderChart",
        json!({"component":"UnknownFixtureChart","props":props}),
    )
    .await;
    assert_eq!(invalid.is_error, Some(true));
    retain("revenue.svg", svg.as_bytes());
    retain(
        "semiotic-grounding.json",
        &serde_json::to_vec_pretty(&grounding).unwrap(),
    );
    client.cancel().await.unwrap();
}
