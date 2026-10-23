//! Isolated package-owner gate, usable while unrelated Rust integration continues.
#[test]
#[ignore = "explicit prepared engine; builds and exercises candidate design tools"]
fn contained_design_runtime_suite() {
    let home = std::env::var_os("MIRKET_TEST_ENGINE_HOME").expect("explicit engine home");
    let paths = mirket::paths::Paths::new(std::path::PathBuf::from(home)).unwrap();
    let report = mirket::packages::test_design_tools(
        &paths,
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")),
    )
    .unwrap();
    assert_eq!(report["ok"], true);
    println!("{report}");
}
