#[cfg(test)]
mod binding_tests {
    use super::*;
    #[test]
    fn checkpoint_references_reduce_ipc_and_detect_archive_tampering() {
        let dir = tempfile::tempdir().unwrap(); let root = dir.path().canonicalize().unwrap();
        let original = json!({"turn":5,"deadline":100,"transcript":[{"result":"x".repeat(128*1024)}]});
        let compact = compact_snapshot(&root, original.clone(), &mut Default::default()).unwrap();
        assert!(serde_json::to_vec(&compact).unwrap().len() < 512);
        assert_eq!(expand_snapshot(&root, compact.clone()).unwrap(), original);
        let sha = compact["observation_archive"][0]["sha256"].as_str().unwrap();
        std::fs::write(root.join("observations").join(format!("{sha}.json")), "{}").unwrap();
        assert!(expand_snapshot(&root, compact).is_err());
    }
    #[test]
    fn runtime_diagnostics_retain_measurements_without_fact_bodies() {
        let mut driver = Driver { run: PathBuf::new(), recorded: Default::default(), session: None,
            project: PathBuf::new(), run_id: "routed-fixture".into(), observed: Value::Null,
            runtime_diagnostics: Vec::new() };
        driver.record_runtime("workflow_resume", &json!({"timing_ms":{"lock_wait":1.5},
            "result":{"diagnostics":{"checkpoint_serialized_bytes_appended":42},
            "state":{"private_fact":"must not enter diagnostics"}}}));
        assert_eq!(driver.runtime_diagnostics[0]["checkpoint"]["checkpoint_serialized_bytes_appended"],42);
        assert_eq!(driver.runtime_diagnostics[0]["timing_ms"]["lock_wait"],1.5);
        assert!(!serde_json::to_string(&driver.runtime_diagnostics).unwrap().contains("private_fact"));
        let summary=memory_summary(&json!({"results":[{"text":"private fact"}],"status":"available",
            "timing_ms":{"retrieval":2.5},"queries":["private query"]}));
        assert_eq!(summary["timing_ms"]["retrieval"],2.5);
        assert!(!summary.to_string().contains("private"));
    }
    #[test]
    fn shared_pause_contract_matches_rust() {
        let contract: Value =
            serde_json::from_str(include_str!("../../../runtime/agent/recovery-contract.json"))
                .unwrap();
        for case in contract["cases"].as_array().unwrap() {
            assert_eq!(
                effective_deadline(&case["state"], case["now"].as_u64().unwrap()),
                case["effective"].as_u64()
            );
        }
    }
    #[test]
    fn recall_summary_discloses_failure_and_coverage_without_private_payloads() {
        let summary = memory_summary(&json!({"results":[{"text":"private fact"}],
            "queries":["private query"],"retrieval":{"query_count":4,"incomplete":true}}));
        assert_eq!(summary["status"], "available");
        assert_eq!(summary["result_count"], 1);
        assert_eq!(summary["query_count"], 4);
        assert_eq!(summary["incomplete"], true);
        assert!(!summary.to_string().contains("private"));
        for status in ["disabled", "unavailable", "empty"] {
            let summary = memory_summary(
                &json!({"status":status,"results":[],"warning":"private exception"}),
            );
            assert_eq!(summary["status"], status);
            assert!(!summary.to_string().contains("private"));
        }
        assert_eq!(memory_summary(&json!({"results":[]}))["status"], "empty");
        assert_eq!(memory_summary(&Value::Null)["status"], "unknown");
        assert!(memory_summary(&Value::Null)["result_count"].is_null());
    }
    #[test]
    fn pause_status_matches_checkpoint_budget_and_uncertain_work_expires() {
        assert_eq!(
            effective_deadline(
                &json!({"deadline":1120,"paused_at":1010,"pause_used":0}),
                2010
            ),
            Some(2120)
        );
        assert_eq!(effective_deadline(&json!({"deadline":1120}), 2010), None);
        assert_eq!(
            effective_deadline(
                &json!({"deadline":1120,"paused_at":1010,"pause_used":86400}),
                1011
            ),
            None
        );
        assert_eq!(
            effective_deadline(&json!({"deadline":1120,"paused_at":1010}), 1009),
            None
        );
        assert_eq!(
            effective_deadline(&json!({"deadline":1120,"paused_at":1120}), 1121),
            None
        );
    }

    #[test]
    fn relocated_or_old_generation_checkpoints_are_not_advertised_as_resumable() {
        let run = json!({"state":{"envelope":{"checkpoint_format_version":1,"project":"/original","execution_sha256":env!("MIRKET_EXECUTION_SHA256")}}});
        assert!(bindings_match(&run, Path::new("/original")));
        let mut audit_only = run.clone();
        audit_only["state"]["envelope"]["source_sha256"] = json!("different audit source");
        assert!(bindings_match(&audit_only, Path::new("/original")));
        audit_only["state"]["envelope"]["execution_sha256"] = json!("different runtime");
        assert!(!bindings_match(&audit_only, Path::new("/original")));
        assert!(!bindings_match(&run, Path::new("/relocated")));
        let old = json!({"state":{"envelope":{"project":"/original","source_sha256":"old"}}});
        assert!(!bindings_match(&old, Path::new("/original")));
    }
}
