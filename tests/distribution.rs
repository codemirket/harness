use serde_json::Value;
use std::process::Command;

#[test]
fn cli_identity_reports_the_executed_build() {
    let output = Command::new(env!("CARGO_BIN_EXE_mirket"))
        .env("MIRKET_DIRECT", "1")
        .args(["info", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let info: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(info["name"], "mirket");
    assert_eq!(info["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(info["target"], env!("MIRKET_TARGET"));
}
