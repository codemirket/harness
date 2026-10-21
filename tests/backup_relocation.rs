use mirket::{backup, maintenance, paths::Paths, state::State, util};
use std::{fs, path::PathBuf};

struct Fixture {
    _home: tempfile::TempDir,
    paths: Paths,
    project: PathBuf,
    id: String,
    task: String,
}
impl Fixture {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        let project = paths.home.join("original");
        fs::create_dir(&project).unwrap();
        let state = State::open(&paths).unwrap();
        let id = state.register_project(&project).unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned();
        let task = state
            .begin(
                &id,
                "Keep durable history",
                &["engineering-judgment".into()],
                "Records recover",
                "Wrong archive fails",
                "begin",
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_owned();
        Self {
            _home: home,
            paths,
            project,
            id,
            task,
        }
    }
    fn erase_durable_fixture(&self) {
        for name in ["state.db", "state.db-wal", "state.db-shm"] {
            let p = self.paths.root.join(name);
            if p.exists() {
                fs::remove_file(p).unwrap();
            }
        }
        for p in [
            self.paths.root.join("runs"),
            mirket::packages::agent::data(&self.paths),
        ] {
            if p.exists() {
                fs::remove_dir_all(p).unwrap();
            }
        }
    }
}
#[test]
fn backup_round_trip_preserves_wal_tasks_runtime_bytes_and_traces() {
    let f = Fixture::new();
    let state = State::open(&f.paths).unwrap();
    // Keep an open reader so the backup must account for committed WAL pages.
    let conn = rusqlite::Connection::open(f.paths.database()).unwrap();
    conn.execute_batch("PRAGMA wal_autocheckpoint=0").unwrap();
    state
        .checkpoint(&f.task, 1, "Recovery checkpoint", "checkpoint")
        .unwrap();
    let expected = state.status(&f.task).unwrap();
    let scope = state.runtime_scope(&f.project).unwrap();
    let runtime = mirket::packages::agent::data(&f.paths)
        .join(&scope)
        .join("nested");
    fs::create_dir_all(&runtime).unwrap();
    fs::write(runtime.join("memory-history"), b"preserve every byte").unwrap();
    let run = f.paths.root.join("runs/routed-test");
    fs::create_dir_all(&run).unwrap();
    fs::write(run.join("trace.json"), b"historical evidence").unwrap();
    let archive = f.paths.home.join("backup.tar");
    let result = backup::create(&f.paths, &archive).unwrap();
    let hash = result["sha256"].as_str().unwrap();
    assert!(
        backup::inspect(&f.paths, &archive, hash).unwrap()["valid"]
            .as_bool()
            .unwrap()
    );
    assert!(
        backup::restore(&f.paths, &archive, hash)
            .unwrap_err()
            .to_string()
            .contains("empty durable")
    );
    assert_eq!(state.status(&f.task).unwrap(), expected);
    drop(conn);
    f.erase_durable_fixture();
    backup::restore(&f.paths, &archive, hash).unwrap();
    assert_eq!(
        State::open(&f.paths).unwrap().status(&f.task).unwrap(),
        expected
    );
    assert_eq!(
        fs::read(runtime.join("memory-history")).unwrap(),
        b"preserve every byte"
    );
    assert_eq!(
        fs::read(run.join("trace.json")).unwrap(),
        b"historical evidence"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(archive).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}
#[test]
fn backup_corruption_wrong_home_and_busy_state_fail_without_publication() {
    let f = Fixture::new();
    let archive = f.paths.home.join("backup.tar");
    let lease = maintenance::shared(&f.paths).unwrap();
    assert!(backup::create(&f.paths, &archive).is_err());
    assert!(!archive.exists());
    drop(lease);
    let backup = backup::create(&f.paths, &archive).unwrap();
    let hash = backup["sha256"].as_str().unwrap();
    let other = tempfile::tempdir().unwrap();
    let other = Paths::new(other.path().canonicalize().unwrap()).unwrap();
    assert!(
        backup::restore(&other, &archive, hash)
            .unwrap_err()
            .to_string()
            .contains("different home")
    );
    assert!(!other.database().exists());
    assert!(backup::inspect(&f.paths, &archive, &"0".repeat(64)).is_err());
    let mut bytes = fs::read(&archive).unwrap();
    bytes[600] ^= 1;
    fs::write(&archive, bytes).unwrap();
    assert!(backup::inspect(&f.paths, &archive, hash).is_err());
    assert!(backup::create(&f.paths, &archive).is_err());
}
#[test]
fn relocation_preserves_task_id_namespace_and_rejects_replacement_or_old_path_reuse() {
    let f = Fixture::new();
    let state = State::open(&f.paths).unwrap();
    let scope = state.runtime_scope(&f.project).unwrap();
    let copy = f.paths.home.join("copy");
    fs::create_dir(&copy).unwrap();
    assert!(state.relocate(&f.id, &f.project, &copy).is_err());
    let new = f.paths.home.join("renamed");
    fs::rename(&f.project, &new).unwrap();
    let lease = maintenance::shared(&f.paths).unwrap();
    assert!(state.relocate(&f.id, &f.project, &new).is_err());
    drop(lease);
    state.relocate(&f.id, &f.project, &new).unwrap();
    assert_eq!(state.project_id(&new).unwrap(), f.id);
    assert_eq!(state.runtime_scope(&new).unwrap(), scope);
    assert_eq!(state.register_project(&new).unwrap()["id"], f.id);
    assert_eq!(
        state.tasks(&f.id, None, 20).unwrap()["tasks"][0]["id"],
        f.task
    );
    assert_eq!(
        state.relocate(&f.id, &f.project, &new).unwrap()["replayed"],
        true
    );
    state.unregister_project(&f.id).unwrap();
    assert!(state.project_id(&new).is_err());
    assert_eq!(state.runtime_scope(&new).unwrap(), scope);
    assert_eq!(state.register_project(&new).unwrap()["id"], f.id);
    fs::create_dir(&f.project).unwrap();
    assert!(state.runtime_scope(&f.project).is_err());
    assert!(state.register_project(&f.project).is_err());
}
#[test]
fn maintenance_blocks_connections_without_changing_current_namespace() {
    let f = Fixture::new();
    let state = State::open(&f.paths).unwrap();
    assert_eq!(
        state.runtime_scope(&f.project).unwrap(),
        util::sha256(f.project.to_str().unwrap().as_bytes())
    );
    let lease = maintenance::exclusive(&f.paths).unwrap();
    assert!(state.status(&f.task).is_err());
    assert!(State::open(&f.paths).is_err());
    drop(lease);
    fs::write(f.paths.root.join("restore-pending.json"), b"{}").unwrap();
    assert!(state.status(&f.task).is_err());
}
#[cfg(unix)]
#[test]
fn backup_rejects_symlink_data_and_relocation_aliases() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let state = State::open(&f.paths).unwrap();
    let alias = f.paths.home.join("alias");
    symlink(&f.project, &alias).unwrap();
    assert!(state.relocate(&f.id, &f.project, &alias).is_err());
    let data = mirket::packages::agent::data(&f.paths);
    fs::create_dir_all(&data).unwrap();
    symlink(&f.project, data.join("escaped")).unwrap();
    let archive = f.paths.home.join("bad.tar");
    assert!(backup::create(&f.paths, &archive).is_err());
    assert!(!archive.exists());
}
