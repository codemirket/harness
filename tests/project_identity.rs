use mirket::{backup, maintenance, paths::Paths, state::State};
use rusqlite::{Connection, params};
use serde_json::Value;
use std::{fs, path::PathBuf};

struct Fixture {
    _home: tempfile::TempDir,
    paths: Paths,
    roots: Vec<PathBuf>,
    ids: Vec<String>,
}

impl Fixture {
    fn new(count: usize) -> Self {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        let state = State::open(&paths).unwrap();
        let mut roots = Vec::new();
        let mut ids = Vec::new();
        for n in 0..count {
            let root = paths.home.join(format!("project-{n}"));
            fs::create_dir(&root).unwrap();
            let id = state.register_project(&root).unwrap()["id"]
                .as_str()
                .unwrap()
                .to_owned();
            roots.push(root);
            ids.push(id);
        }
        Self {
            _home: home,
            paths,
            roots,
            ids,
        }
    }

    // This is an isolated disposable fixture only. Production recovery never
    // edits SQLite outside the supported Mirket command.
    fn set_identity(&self, n: usize, identity: &str) {
        let conn = Connection::open(self.paths.database()).unwrap();
        conn.execute(
            "UPDATE projects SET identity=?1 WHERE id=?2",
            params![identity, self.ids[n]],
        )
        .unwrap();
        conn.execute(
            "UPDATE project_scopes SET identity=?1 WHERE project_id=?2",
            params![identity, self.ids[n]],
        )
        .unwrap();
    }

    fn legacy(&self, n: usize) -> String {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let m = fs::metadata(&self.roots[n]).unwrap();
            format!("{}:{}", m.dev(), m.ino())
        }
        #[cfg(not(unix))]
        {
            use std::time::UNIX_EPOCH;
            fs::metadata(&self.roots[n])
                .unwrap()
                .created()
                .unwrap()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
                .to_string()
        }
    }

    fn backup(&self) -> (PathBuf, String) {
        let archive = self.paths.home.join("identity-backup.tar");
        let result = backup::create(&self.paths, &archive).unwrap();
        let sha = result["sha256"].as_str().unwrap().to_owned();
        assert_eq!(
            backup::inspect(&self.paths, &archive, &sha).unwrap()["valid"],
            true
        );
        (archive, sha)
    }

    fn save_plan(&self, plan: &Value) -> PathBuf {
        let file = self.paths.home.join("reviewed-plan.json");
        fs::write(&file, serde_json::to_vec_pretty(plan).unwrap()).unwrap();
        file
    }

    fn save_attestations(&self, plan: &Value) -> PathBuf {
        let file = self.paths.home.join("reviewed-evidence.json");
        let projects: Vec<_> = plan["projects"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|p| p["status"] == "legacy_device_transition_review_required")
            .map(|p| {
                serde_json::json!({
                    "id":p["id"],
                    "root":p["root"],
                    "saved_identity":p["saved_identity"],
                    "current_identity":p["current_identity"],
                    "evidence":"Disposable test directory created before registration; only its simulated device field changed."
                })
            })
            .collect();
        fs::write(
            &file,
            serde_json::to_vec_pretty(&serde_json::json!({"schema":1,"projects":projects}))
                .unwrap(),
        )
        .unwrap();
        file
    }
}

#[test]
fn new_registration_uses_versioned_identity_and_replacement_fails_closed() {
    let f = Fixture::new(1);
    let state = State::open(&f.paths).unwrap();
    let preview = state.identity_preview(0, 10).unwrap();
    assert_eq!(preview["projects"][0]["status"], "valid");
    assert!(
        preview["projects"][0]["saved_identity"]
            .as_str()
            .unwrap()
            .starts_with("v2:")
    );
    let replacement = f.paths.home.join("replacement");
    fs::rename(&f.roots[0], &replacement).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(20));
    fs::create_dir(&f.roots[0]).unwrap();
    assert_eq!(
        state.identity_preview(0, 10).unwrap()["projects"][0]["status"],
        "replaced_or_changed"
    );
    assert!(state.tasks(&f.ids[0], None, 1).is_err());
}

#[test]
fn matching_legacy_promotes_without_changing_other_valid_registration() {
    let f = Fixture::new(2);
    let state = State::open(&f.paths).unwrap();
    let unchanged = state.identity_preview(0, 10).unwrap()["projects"][1]["saved_identity"]
        .as_str()
        .unwrap()
        .to_owned();
    f.set_identity(0, &f.legacy(0));
    let plan = state.identity_preview(0, 10).unwrap();
    assert_eq!(plan["projects"][0]["status"], "valid_legacy");
    assert_eq!(plan["projects"][1]["status"], "valid");
    let file = f.save_plan(&plan);
    let (archive, sha) = f.backup();
    let outcome = state.identity_apply(&file, &archive, &sha, None).unwrap();
    assert_eq!(outcome["changed"].as_array().unwrap().len(), 1);
    let after = state.identity_preview(0, 10).unwrap();
    assert_eq!(after["projects"][0]["status"], "valid");
    assert_eq!(after["projects"][1]["saved_identity"], unchanged);
    assert_eq!(
        state.identity_apply(&file, &archive, &sha, None).unwrap()["replayed"],
        true
    );
}

#[test]
fn backup_must_contain_reviewed_registration_preimage() {
    let f = Fixture::new(1);
    let state = State::open(&f.paths).unwrap();
    let (archive, sha) = f.backup();
    f.set_identity(0, &f.legacy(0));
    let plan = state.identity_preview(0, 10).unwrap();
    let file = f.save_plan(&plan);
    assert!(state.identity_apply(&file, &archive, &sha, None).is_err());
    assert_eq!(state.identity_preview(0, 10).unwrap(), plan);
}

#[test]
fn identical_path_and_inode_with_changed_creation_identity_is_rejected() {
    let f = Fixture::new(1);
    let state = State::open(&f.paths).unwrap();
    let current = state.identity_preview(0, 10).unwrap()["projects"][0]["current_identity"]
        .as_str()
        .unwrap()
        .to_owned();
    let (prefix, birth) = current.rsplit_once(':').unwrap();
    let forged = format!("{prefix}:{}", birth.parse::<u128>().unwrap() + 1);
    f.set_identity(0, &forged);
    assert_eq!(
        state.identity_preview(0, 10).unwrap()["projects"][0]["status"],
        "replaced_or_changed"
    );
    assert!(state.tasks(&f.ids[0], None, 1).is_err());
}

#[test]
fn legacy_inode_match_with_directory_born_after_registration_is_rejected() {
    let f = Fixture::new(1);
    let state = State::open(&f.paths).unwrap();
    f.set_identity(0, &f.legacy(0));
    Connection::open(f.paths.database())
        .unwrap()
        .execute("UPDATE projects SET created=0 WHERE id=?1", [&f.ids[0]])
        .unwrap();
    assert_eq!(
        state.identity_preview(0, 10).unwrap()["projects"][0]["status"],
        "replaced_or_ambiguous"
    );
    assert!(state.tasks(&f.ids[0], None, 1).is_err());
}

#[test]
fn missing_root_requires_separate_relocation_evidence() {
    let f = Fixture::new(1);
    let state = State::open(&f.paths).unwrap();
    fs::rename(&f.roots[0], f.paths.home.join("moved")).unwrap();
    assert_eq!(
        state.identity_preview(0, 10).unwrap()["projects"][0]["status"],
        "missing_or_moved"
    );
}

#[test]
fn maintenance_lease_blocks_identity_recovery_without_mutation() {
    let f = Fixture::new(1);
    let state = State::open(&f.paths).unwrap();
    f.set_identity(0, &f.legacy(0));
    let plan = state.identity_preview(0, 10).unwrap();
    let file = f.save_plan(&plan);
    let (archive, sha) = f.backup();
    let lease = maintenance::shared(&f.paths).unwrap();
    assert!(state.identity_apply(&file, &archive, &sha, None).is_err());
    drop(lease);
    assert_eq!(state.identity_preview(0, 10).unwrap(), plan);
}

#[cfg(unix)]
#[test]
fn rejected_second_registration_rolls_back_first_update() {
    let f = Fixture::new(2);
    let state = State::open(&f.paths).unwrap();
    f.set_identity(0, &f.legacy(0));
    f.set_identity(1, "999999:999999");
    let plan = state.identity_preview(0, 10).unwrap();
    assert_eq!(plan["projects"][0]["status"], "valid_legacy");
    assert_eq!(plan["projects"][1]["status"], "replaced_or_ambiguous");
    let file = f.save_plan(&plan);
    let (archive, sha) = f.backup();
    assert!(state.identity_apply(&file, &archive, &sha, None).is_err());
    assert_eq!(state.identity_preview(0, 10).unwrap(), plan);
}

#[cfg(unix)]
#[test]
fn symlink_substitution_cannot_recover() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new(1);
    let state = State::open(&f.paths).unwrap();
    let real = f.paths.home.join("original-moved");
    fs::rename(&f.roots[0], &real).unwrap();
    symlink(&real, &f.roots[0]).unwrap();
    let plan = state.identity_preview(0, 10).unwrap();
    assert_eq!(
        plan["projects"][0]["status"],
        "unsafe_path_or_unknown_identity"
    );
    let file = f.save_plan(&plan);
    let (archive, sha) = f.backup();
    assert!(state.identity_apply(&file, &archive, &sha, None).is_err());
}

#[cfg(unix)]
#[test]
fn legacy_device_transition_requires_reviewed_backup_and_preserves_durable_state() {
    let f = Fixture::new(2);
    let state = State::open(&f.paths).unwrap();
    let task = state
        .begin(
            &f.ids[0],
            "Preserve history",
            &["engineering-judgment".into()],
            "Status retained",
            "Wrong project rejected",
            "begin-identity-test",
        )
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let original_status = state.status(&task).unwrap();
    let original_scope = state.runtime_scope(&f.roots[0]).unwrap();
    for n in 0..2 {
        let legacy = f.legacy(n);
        let (dev, ino) = legacy.split_once(':').unwrap();
        let old_dev = dev.parse::<u64>().unwrap() + 1;
        f.set_identity(n, &format!("{old_dev}:{ino}"));
    }
    let plan = state.identity_preview(0, 10).unwrap();
    assert_eq!(plan["total"], 2);
    assert!(
        plan["projects"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| { p["status"] == "legacy_device_transition_review_required" })
    );
    let file = f.save_plan(&plan);
    let evidence = f.save_attestations(&plan);
    let (archive, sha) = f.backup();
    assert!(state.identity_apply(&file, &archive, &sha, None).is_err());
    assert_eq!(state.identity_preview(0, 10).unwrap(), plan);
    let result = state
        .identity_apply(&file, &archive, &sha, Some(&evidence))
        .unwrap();
    assert_eq!(result["changed"].as_array().unwrap().len(), 2);
    assert_eq!(
        state
            .identity_apply(&file, &archive, &sha, Some(&evidence))
            .unwrap()["replayed"],
        true
    );
    assert_eq!(state.status(&task).unwrap(), original_status);
    assert_eq!(state.runtime_scope(&f.roots[0]).unwrap(), original_scope);
    assert!(
        state.identity_preview(0, 10).unwrap()["projects"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["status"] == "valid")
    );
}

#[cfg(unix)]
#[test]
fn stale_page_or_ambiguous_legacy_never_partially_recovers() {
    let f = Fixture::new(2);
    let state = State::open(&f.paths).unwrap();
    for n in 0..2 {
        let legacy = f.legacy(n);
        let (dev, ino) = legacy.split_once(':').unwrap();
        f.set_identity(n, &format!("{}:{ino}", dev.parse::<u64>().unwrap() + 1));
    }
    let plan = state.identity_preview(0, 10).unwrap();
    let file = f.save_plan(&plan);
    let evidence = f.save_attestations(&plan);
    let (archive, sha) = f.backup();
    f.set_identity(1, "999999:999999");
    assert!(
        state
            .identity_apply(&file, &archive, &sha, Some(&evidence))
            .is_err()
    );
    let after = state.identity_preview(0, 10).unwrap();
    assert_eq!(
        after["projects"][0]["saved_identity"],
        plan["projects"][0]["saved_identity"]
    );
    assert!(
        after["projects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["status"] == "replaced_or_ambiguous")
    );
}
