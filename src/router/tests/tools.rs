#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unknown_design_tools_fail_before_runtime_activation() {
        let dir = tempfile::tempdir().unwrap();
        let paths = Paths::new(dir.path().to_path_buf()).unwrap();
        let error = invoke(&paths, dir.path(), "design/execute_code", json!({}))
            .await
            .err()
            .unwrap();
        assert!(error.to_string().contains("unknown owned tool"));
        assert!(!paths.root.exists());
    }

    #[test]
    fn openpencil_exports_verify_only_the_exact_consumed_source_revision() {
        let root = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        std::fs::write(root.path().join("design.fig"), b"editable").unwrap();
        let digest = util::sha256(b"editable");
        let value =
            json!({"source":"design.fig","source_sha256":digest,"artifact":{"path":"preview.png"}});
        let receipt = openpencil_source_receipt(root.path(), &value)
            .unwrap()
            .unwrap();
        assert_eq!(receipt.sha256, digest);
        std::fs::write(root.path().join("design.fig"), b"changed").unwrap();
        assert!(openpencil_source_receipt(root.path(), &value).is_err());
        assert!(openpencil_source_receipt(root.path(), &json!({"source":"../other.fig","source_sha256":digest,"artifact":{"path":"preview.png"}})).is_err());
        assert!(openpencil_source_receipt(root.path(), &json!({"source":"design.fig","source_sha256":digest,"artifact":{"path":"design.fig"}})).unwrap().is_none());
    }

    #[test]
    fn interrupted_write_reconciles_bytes_without_replay() {
        let project = tempfile::tempdir().unwrap();
        let root = project.path().canonicalize().unwrap();
        let args = json!({"path":"result.txt","content":"new bytes"});
        let intent = write_intent(&root, &args).unwrap();
        assert!(!reconcile_write(&root, &intent).unwrap().0);
        assert!(!root.join("result.txt").exists());
        std::fs::write(root.join("result.txt"), "new bytes").unwrap();
        assert!(reconcile_write(&root, &intent).unwrap().0);
        let edits = json!({"path":"result.txt","edits":[{"old":"new","new":"changed"}],"expected_sha256":util::sha256(b"new bytes")});
        let edit_intent = write_intent(&root, &edits).unwrap();
        assert!(!reconcile_write(&root, &edit_intent).unwrap().0);
        std::fs::write(root.join("result.txt"), "changed bytes").unwrap();
        assert!(reconcile_write(&root, &edit_intent).unwrap().0);
        std::fs::write(root.join("result.txt"), "unrelated user edit").unwrap();
        assert!(reconcile_write(&root, &edit_intent).is_err());
        let mut escaped = intent.clone();
        escaped["path"] = json!("../escape");
        assert!(reconcile_write(&root, &escaped).is_err());
        assert!(write_intent(&root, &edits).is_err());
    }

    #[test]
    fn disabled_assigned_design_tools_fail_preflight_without_blocking_other_work() {
        let raw = json!({"name":"vector_render","_meta":{"mirket/availability":{"enabled":false,"group":"core","start":"mirket package start design"}}});
        assert!(require_available(&raw, &["design/d2_render".into()]).is_ok());
        let error = require_available(&raw, &["design/vector_render".into()])
            .unwrap_err()
            .to_string();
        assert!(error.contains("no model was dispatched") && error.contains("package start design"));
        let mut ready = raw.clone();
        ready["_meta"]["mirket/availability"]["enabled"] = json!(true);
        assert!(require_available(&ready, &["design/vector_render".into()]).is_ok());
    }

    #[test]
    fn completed_negative_qa_keeps_artifacts_without_claiming_success() {
        use rmcp::model::CallToolResult;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        std::fs::write(root.join("render.svg"), "saved render").unwrap();
        let value = json!({"execution":{"tool":"vector_render","state":"completed"},
            "qa":{"status":"changed_pixels"},
            "artifact":{"path":"render.svg","sha256":util::sha256(b"saved render")}});
        let result = CallToolResult::structured_error(value.clone());
        assert!(design_completed("design/vector_render", &result));
        assert!(design_mutates("design/vector_render", &result));
        assert!(!design_check_passed("design/vector_render", &result));
        let artifacts = design_artifacts(&root, "design/vector_render", &value).unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(
            current_digest(&artifacts[0].path).unwrap(),
            artifacts[0].sha256
        );
        assert!(!design_completed("design/browser_screenshot", &result));
        let mut unknown = value.clone();
        unknown["execution"]["tool"] = json!("execute_code");
        assert!(!design_completed(
            "design/execute_code",
            &CallToolResult::structured_error(unknown)
        ));
        assert!(!design_completed(
            "design/vector_render",
            &CallToolResult::structured_error(json!({"message":"completed"}))
        ));
        let mut uncertain = value;
        uncertain["execution"]["state"] = json!("started");
        assert!(!design_completed(
            "design/vector_render",
            &CallToolResult::structured_error(uncertain)
        ));
    }

    #[test]
    fn only_bound_owned_predispatch_rejections_can_clear_mutation_uncertainty() {
        use rmcp::model::CallToolResult;
        let receipt = json!({"execution":{"tool":"vector_render","state":"not_started","code":"invalid_arguments"}});
        assert!(!design_mutates(
            "design/vector_render",
            &CallToolResult::structured_error(receipt.clone())
        ));
        let mut conflict = receipt.clone();
        conflict["execution"]["code"] = json!("output_preflight");
        assert!(!design_mutates(
            "design/vector_render",
            &CallToolResult::structured_error(conflict)
        ));
        assert!(design_mutates(
            "design/vector_render",
            &CallToolResult::structured(receipt.clone())
        ));
        assert!(design_mutates(
            "design/browser_screenshot",
            &CallToolResult::structured_error(receipt.clone())
        ));
        let mut remote = receipt.clone();
        remote["execution"]["tool"] = json!("untrusted_tool");
        assert!(design_mutates(
            "design/untrusted_tool",
            &CallToolResult::structured_error(remote)
        ));
        for (key, value) in [("state", "started"), ("code", "renderer_failure")] {
            let mut altered = receipt.clone();
            altered["execution"][key] = json!(value);
            assert!(design_mutates(
                "design/vector_render",
                &CallToolResult::structured_error(altered)
            ));
        }
        assert!(design_mutates(
            "design/vector_render",
            &CallToolResult::structured_error(json!({"message":"operation did not start"}))
        ));
    }
    #[tokio::test]
    async fn paged_reads_preserve_utf8_reject_drift_and_never_verify_fragments() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let project = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let original = "α".repeat(100_000);
        std::fs::write(project.path().join("large"), &original).unwrap();
        let first = invoke(
            &paths,
            project.path(),
            "workspace_read",
            json!({"path":"large","limit":131071}),
        )
        .await
        .unwrap();
        assert!(first.verifies.is_empty());
        let next = first.value["next_offset"].as_u64().unwrap();
        assert_eq!(next, 131070);
        let second = invoke(
            &paths,
            project.path(),
            "workspace_read",
            json!({"path":"large","offset":next,"expected_sha256":first.value["sha256"]}),
        )
        .await
        .unwrap();
        assert!(second.verifies.is_empty());
        assert!(second.value["next_offset"].is_null());
        assert_eq!(
            format!(
                "{}{}",
                first.value["content"].as_str().unwrap(),
                second.value["content"].as_str().unwrap()
            ),
            original
        );
        assert!(
            invoke(
                &paths,
                project.path(),
                "workspace_read",
                json!({"path":"large","offset":next})
            )
            .await
            .is_err()
        );
        std::fs::write(project.path().join("large"), "changed").unwrap();
        assert!(
            invoke(
                &paths,
                project.path(),
                "workspace_read",
                json!({"path":"large","expected_sha256":first.value["sha256"]})
            )
            .await
            .is_err()
        );
        let complete = invoke(
            &paths,
            project.path(),
            "workspace_read",
            json!({"path":"large"}),
        )
        .await
        .unwrap();
        assert_eq!(complete.verifies.len(), 1);
    }
    #[tokio::test]
    async fn directory_pagination_is_stable_and_detects_added_entries() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let project = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        for index in 0..1001 {
            std::fs::write(project.path().join(format!("file-{index:04}")), "").unwrap();
        }
        let first = invoke(
            &paths,
            project.path(),
            "workspace_list",
            json!({"path":"."}),
        )
        .await
        .unwrap();
        assert_eq!(first.value["entries"].as_array().unwrap().len(), 1000);
        let second = invoke(
            &paths,
            project.path(),
            "workspace_list",
            json!({"path":".","offset":1000,"expected_sha256":first.value["sha256"]}),
        )
        .await
        .unwrap();
        assert_eq!(second.value["entries"], json!(["file-1000"]));
        std::fs::write(project.path().join("added"), "").unwrap();
        assert!(
            invoke(
                &paths,
                project.path(),
                "workspace_list",
                json!({"path":".","offset":1000,"expected_sha256":first.value["sha256"]})
            )
            .await
            .is_err()
        );
    }
    #[tokio::test]
    async fn exact_edits_are_digest_bound_and_transactional() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let project = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let original = "first α line\nsecond line\n";
        let file = project.path().join("source");
        std::fs::write(&file, original).unwrap();
        for args in [
            json!({"path":"source","edits":[{"old":"first","new":"changed"}],"expected_sha256":null}),
            json!({"path":"source","edits":[{"old":"first","new":"changed"},{"old":"absent","new":"bad"}],"expected_sha256":util::sha256(original.as_bytes())}),
            json!({"path":"source","edits":[{"old":"line","new":"ambiguous"}],"expected_sha256":util::sha256(original.as_bytes())}),
            json!({"path":"source","content":"wrong","edits":[{"old":"first","new":"changed"}],"expected_sha256":util::sha256(original.as_bytes())}),
        ] {
            assert!(
                invoke(&paths, project.path(), "workspace_write", args)
                    .await
                    .is_err()
            );
            assert_eq!(std::fs::read_to_string(&file).unwrap(), original);
        }
        let result = invoke(&paths, project.path(), "workspace_write", json!({"path":"source","edits":[{"old":"α","new":"β"},{"old":"second","new":"last"}],"expected_sha256":util::sha256(original.as_bytes())})).await.unwrap();
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "first β line\nlast line\n"
        );
        assert!(result.mutates && result.artifacts.len() == 1);
        assert!(
            edited_content(
                b"aaa",
                &[TextEdit {
                    old: "aa".into(),
                    new: "b".into()
                }]
            )
            .is_err()
        );
        assert!(
            edited_content(
                b"small",
                &[TextEdit {
                    old: "small".into(),
                    new: "x".repeat(1024 * 1024 + 1)
                }]
            )
            .is_err()
        );
    }
    #[test]
    fn odiff_reports_differences_and_only_fails_explicit_equality_gates() {
        use rmcp::model::CallToolResult;
        assert!(!design_check_passed(
            "design/odiff_compare",
            &CallToolResult::structured(
                json!({"match": false, "expect_match":true, "exit_code": 21})
            )
        ));
        assert!(!design_check_passed(
            "design/odiff_compare",
            &CallToolResult::structured(json!({"exit_code": 0}))
        ));
        assert!(design_check_passed(
            "design/odiff_compare",
            &CallToolResult::structured(
                json!({"match": true, "expect_match":true, "exit_code": 0})
            )
        ));
        assert!(design_check_passed(
            "design/odiff_compare",
            &CallToolResult::structured(json!({"match":false,"expect_match":false,"exit_code":21}))
        ));
        assert!(!design_check_passed(
            "design/odiff_compare",
            &CallToolResult::structured_error(json!({"match": true}))
        ));
    }

    #[test]
    fn tiled_evidence_keeps_original_identity_and_requires_complete_nonoverlapping_coverage() {
        let dir = tempfile::tempdir_in(std::env::temp_dir().canonicalize().unwrap()).unwrap();
        let root = dir.path();
        let image = b"\x89PNG\r\n\x1a\nfixture";
        for name in ["original.png", "overview.png", "left.png", "right.png"] {
            std::fs::write(root.join(name), image).unwrap();
        }
        let receipt = |name: &str| json!({"path":name,"sha256":util::sha256(image)});
        let original = artifact_from_json(root, &receipt("original.png"))
            .unwrap()
            .unwrap();
        let plan = json!({"inspection":[{"source":receipt("original.png"),"width":4,"height":2,
        "overview":receipt("overview.png"),"tiles":[
            {"path":"left.png","sha256":util::sha256(image),"left":0,"top":0,"width":2,"height":2},
            {"path":"right.png","sha256":util::sha256(image),"left":2,"top":0,"width":2,"height":2}
        ]}]});
        let mut pending = vec![original.clone()];
        let stable = inspection_artifacts(root, &plan, &mut pending).unwrap();
        assert_eq!(stable, vec![original.clone()]);
        assert_eq!(pending.len(), 3);
        assert!(!pending.contains(&original));
        let mut overlap = plan.clone();
        overlap["inspection"][0]["tiles"][1]["left"] = json!(1);
        assert!(inspection_artifacts(root, &overlap, &mut vec![original.clone()]).is_err());
        let mut missing = plan.clone();
        missing["inspection"][0]["tiles"]
            .as_array_mut()
            .unwrap()
            .pop();
        assert!(inspection_artifacts(root, &missing, &mut vec![original.clone()]).is_err());
        std::fs::write(&original.path, b"changed source").unwrap();
        assert_ne!(current_digest(&stable[0].path).unwrap(), stable[0].sha256);
        assert!(inspection_artifacts(root, &plan, &mut vec![original]).is_err());
    }

    #[test]
    fn odiff_published_diagnostics_are_mutations_with_artifact_receipts() {
        use rmcp::model::CallToolResult;
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        std::fs::create_dir_all(design_root(&paths)).unwrap();
        let value = json!({"match":false,"diff":{"path":"diff.png","sha256":"a".repeat(64)}});
        assert!(may_mutate("design/odiff_compare"));
        assert!(design_mutates(
            "design/odiff_compare",
            &CallToolResult::structured(value.clone())
        ));
        let artifacts =
            design_artifacts(&design_root(&paths), "design/odiff_compare", &value).unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].path, design_root(&paths).join("diff.png"));
        assert!(!design_mutates(
            "design/odiff_compare",
            &CallToolResult::structured(json!({"match":true,"diff":null}))
        ));
        assert!(design_mutates(
            "design/odiff_compare",
            &CallToolResult::structured_error(json!({"match":true,"diff":null}))
        ));
    }

    #[test]
    fn repository_internals_reject_case_aliases_on_every_path_surface() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().canonicalize().unwrap();
        let paths = crate::paths::Paths::new(project.clone()).unwrap();
        for relative in [".git/config", ".GIT/config", ".GiT/config"] {
            assert!(
                file(&project, relative)
                    .unwrap_err()
                    .to_string()
                    .contains("repository internals")
            );
            assert!(
                readable_file(&paths, &project, project.join(relative).to_str().unwrap())
                    .unwrap_err()
                    .to_string()
                    .contains("repository internals")
            );
        }
    }

    #[test]
    fn paths_cannot_escape_project() {
        let root = tempfile::tempdir().unwrap();
        let paths = Paths::new(root.path().into()).unwrap();
        for p in ["../private", "/tmp/file", ".git/config"] {
            assert!(file(root.path(), p).is_err());
        }
        std::fs::create_dir_all(root.path().join(".git")).unwrap();
        std::fs::write(root.path().join(".git/config"), "private").unwrap();
        assert!(
            readable_file(
                &paths,
                root.path(),
                root.path().join(".git/config").to_str().unwrap()
            )
            .is_err()
        );
    }
    #[tokio::test]
    async fn writes_require_observed_identity() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let project = tempfile::tempdir().unwrap();
        std::fs::write(project.path().join("a"), "original").unwrap();
        assert!(
            invoke(
                &paths,
                project.path(),
                "workspace_write",
                json!({"path":"a","content":"changed","expected_sha256":null})
            )
            .await
            .is_err()
        );
        assert_eq!(
            std::fs::read_to_string(project.path().join("a")).unwrap(),
            "original"
        );
    }

    #[test]
    fn design_artifact_refs_stay_in_assigned_workspace() {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().into()).unwrap();
        let project = home.path().canonicalize().unwrap().join("project");
        std::fs::create_dir(&project).unwrap();
        let root = design_scope(&paths, &project).unwrap();
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("render.png"), b"image").unwrap();
        let digest = util::sha256(b"image");
        let refs = design_artifacts(
            &root,
            "design/static_render",
            &json!({"artifact":{"path":"render.png","sha256":digest}}),
        )
        .unwrap();
        assert_eq!(
            refs,
            vec![ArtifactRef {
                path: root.join("render.png"),
                sha256: util::sha256(b"image")
            }]
        );
        assert!(
            design_artifacts(
                &root,
                "design/static_render",
                &json!({"artifact":{"path":"../outside.png","sha256":util::sha256(b"image")}})
            )
            .is_err()
        );
        assert_eq!(
            readable_file(&paths, &project, root.join("render.png").to_str().unwrap()).unwrap(),
            root.join("render.png")
        );
    }

    #[test]
    fn primary_artifact_digest_rejects_changed_or_deleted_output() {
        let root = tempfile::tempdir().unwrap();
        let pdf = root.path().canonicalize().unwrap().join("document.pdf");
        std::fs::write(&pdf, b"%PDF-original").unwrap();
        let expected = current_digest(&pdf).unwrap();
        assert_eq!(expected, util::sha256(b"%PDF-original"));
        std::fs::write(&pdf, b"%PDF-changed").unwrap();
        assert_ne!(current_digest(&pdf).unwrap(), expected);
        std::fs::remove_file(&pdf).unwrap();
        assert!(current_digest(&pdf).is_err());
    }
}

#[cfg(test)]
mod scoped_design_tests {
    use super::*;
    #[tokio::test]
    async fn project_design_reads_writes_and_recovery_reject_siblings() {
        let home = tempfile::tempdir().unwrap();
        let root = home.path().canonicalize().unwrap();
        let paths = Paths::new(root.clone()).unwrap();
        let a = root.join("a");
        let b = root.join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        let own = design_scope(&paths, &a).unwrap();
        let other = design_scope(&paths, &b).unwrap();
        std::fs::create_dir_all(&own).unwrap();
        std::fs::create_dir_all(&other).unwrap();
        std::fs::write(other.join("private.txt"), "other project").unwrap();
        assert!(readable_file(&paths, &a, other.join("private.txt").to_str().unwrap()).is_err());
        let args = json!({"path":own.join("source.txt"),"content":"owned"});
        let intent = write_intent_scoped(&paths, &a, &args).unwrap();
        invoke(&paths, &a, "workspace_write", args).await.unwrap();
        assert!(reconcile_write_scoped(&paths, &a, &intent).unwrap().0);
        assert!(reconcile_write_scoped(&paths, &b, &intent).is_err());
        assert!(invoke(&paths,&b,"workspace_write",json!({"path":own.join("source.txt"),"content":"bad","expected_sha256":util::sha256(b"owned")})).await.is_err());
    }
}
