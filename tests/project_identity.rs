use mirket::{paths::Paths, state::State};
use rusqlite::{Connection, params};
use std::fs;

fn registered_project() -> (tempfile::TempDir, Paths, std::path::PathBuf, String) {
    let home = tempfile::tempdir().unwrap();
    let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
    let root = paths.home.join("project");
    fs::create_dir(&root).unwrap();
    let state = State::open(&paths).unwrap();
    let id = state.register_project(&root).unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned();
    (home, paths, root, id)
}

fn saved_identity(paths: &Paths, id: &str) -> String {
    Connection::open(paths.database())
        .unwrap()
        .query_row("SELECT identity FROM projects WHERE id=?1", [id], |r| {
            r.get(0)
        })
        .unwrap()
}

#[test]
fn new_registration_uses_persistent_identity_and_rejects_replaced_directory() {
    let (_home, paths, root, id) = registered_project();
    assert!(saved_identity(&paths, &id).starts_with("v2:"));
    let state = State::open(&paths).unwrap();
    assert!(state.tasks(&id, None, 1).is_ok());
    fs::rename(&root, paths.home.join("old-project")).unwrap();
    fs::create_dir(&root).unwrap();
    assert!(state.tasks(&id, None, 1).is_err());
    assert!(state.register_project(&root).is_err());
}

#[test]
fn changed_creation_identity_and_old_identity_format_fail_closed() {
    let (_home, paths, _root, id) = registered_project();
    let saved = saved_identity(&paths, &id);
    let (prefix, birth) = saved.rsplit_once(':').unwrap();
    let conn = Connection::open(paths.database()).unwrap();
    for forged in [
        format!("{prefix}:{}", birth.parse::<u128>().unwrap() + 1),
        "16777230:192684729".to_owned(),
    ] {
        conn.execute(
            "UPDATE projects SET identity=?1 WHERE id=?2",
            params![forged, id],
        )
        .unwrap();
        conn.execute(
            "UPDATE project_scopes SET identity=?1 WHERE project_id=?2",
            params![forged, id],
        )
        .unwrap();
        assert!(State::open(&paths).unwrap().tasks(&id, None, 1).is_err());
    }
}

#[cfg(unix)]
#[test]
fn symlink_substitution_cannot_claim_registered_project() {
    use std::os::unix::fs::symlink;
    let (_home, paths, root, id) = registered_project();
    let moved = paths.home.join("moved");
    fs::rename(&root, &moved).unwrap();
    symlink(&moved, &root).unwrap();
    assert!(State::open(&paths).unwrap().tasks(&id, None, 1).is_err());
}
