use mirket::role_eval;
use std::path::Path;

#[test]
fn illustration_suite_binds_every_brief_rubric_and_configuration() {
    let project = Path::new(env!("CARGO_MANIFEST_DIR"));
    let result =
        role_eval::suite(project, Path::new("evals/illustration-quality/suite.json")).unwrap();
    assert_eq!(result["execution"], "not_dispatched");
    assert_eq!(result["expected_cells"], 32);
    assert_eq!(result["suite"]["cases"].as_array().unwrap().len(), 8);
}
