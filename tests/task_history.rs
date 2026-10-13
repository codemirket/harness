use mirket::{paths::Paths, state::State};
use serde_json::Value;
use std::fs;

struct Fixture {
    _home: tempfile::TempDir,
    paths: Paths,
    state: State,
    project_id: String,
}

impl Fixture {
    fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        let paths = Paths::new(home.path().canonicalize().unwrap()).unwrap();
        let project = paths.home.join("project");
        fs::create_dir(&project).unwrap();
        let state = State::open(&paths).unwrap();
        let registered = state.register_project(&project).unwrap();
        Self {
            _home: home,
            paths,
            state,
            project_id: registered["id"].as_str().unwrap().into(),
        }
    }

    fn begin(&self, key: &str) -> String {
        self.state
            .begin(
                &self.project_id,
                "Recover the original constraint after an interruption",
                &["engineering-judgment".into()],
                "Every checkpoint is retrievable in order",
                "An append between pages cannot hide an earlier constraint",
                key,
            )
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .into()
    }

    fn append(&self, task: &str, revision: i64, note: &str) {
        self.state
            .checkpoint(task, revision, note, &format!("checkpoint-{revision}"))
            .unwrap();
    }
}

fn checkpoints(page: &Value) -> &[Value] {
    page["checkpoints"].as_array().unwrap()
}

#[test]
fn oldest_constraint_survives_restart_and_append_between_history_pages() {
    let f = Fixture::new();
    let task = f.begin("start");
    f.append(
        &task,
        1,
        "Original constraint: preserve the customer's settings",
    );
    for revision in 2..=25 {
        f.append(&task, revision, &format!("Progress {revision}"));
    }
    // A newly opened State models a fresh client resuming after interruption.
    let resumed = State::open(&f.paths).unwrap();
    let status = resumed.status(&task).unwrap();
    assert_eq!(status["checkpoint_count"], 25);
    assert_eq!(status["history_has_more"], true);
    assert_eq!(status["recent_checkpoints"].as_array().unwrap().len(), 8);
    let first = resumed.task_history(&task, None, 10).unwrap();
    assert_eq!(checkpoints(&first).len(), 10);
    assert_eq!(checkpoints(&first)[0]["revision"], 26);
    assert_eq!(checkpoints(&first)[9]["revision"], 17);
    // Another writer appends after page one. Offset paging would duplicate or
    // skip rows; immutable revision paging must still return 16 down to 2.
    std::thread::scope(|scope| {
        scope.spawn(|| f.append(&task, 26, "New work after the recovery began"));
    });
    let second = resumed
        .task_history(&task, first["next_cursor"].as_str(), 10)
        .unwrap();
    assert_eq!(checkpoints(&second).len(), 10);
    assert_eq!(checkpoints(&second)[0]["revision"], 16);
    assert_eq!(checkpoints(&second)[9]["revision"], 7);
    let third = resumed
        .task_history(&task, second["next_cursor"].as_str(), 10)
        .unwrap();
    assert_eq!(checkpoints(&third).len(), 5);
    assert_eq!(checkpoints(&third)[0]["revision"], 6);
    assert_eq!(checkpoints(&third)[4]["revision"], 2);
    assert_eq!(
        checkpoints(&third)[4]["note"],
        "Original constraint: preserve the customer's settings"
    );
    assert!(third["next_cursor"].is_null());
    assert_eq!(resumed.status(&task).unwrap()["checkpoint_count"], 26);
}

#[test]
fn history_bounds_cursors_to_the_task_and_rejects_malformed_input() {
    let f = Fixture::new();
    let task = f.begin("first");
    let other = f.begin("other");
    f.append(&task, 1, "first constraint");
    f.append(&task, 2, "second constraint");
    let page = f.state.task_history(&task, None, 1).unwrap();
    let cursor = page["next_cursor"].as_str().unwrap();
    assert!(f.state.task_history(&other, Some(cursor), 1).is_err());
    for cursor in [
        String::new(),
        "garbage".into(),
        format!("history2:{task}:3"),
        format!("history1:{task}:03"),
        format!("history1:{task}:+3"),
        format!("history1:{task}:0"),
        format!("history1:{task}:-1"),
        format!("history1:{task}:9223372036854775808"),
        format!("history1:{task}:3:extra"),
        format!("history1:{task}:3\0"),
        "🦀".repeat(33),
    ] {
        assert!(
            f.state.task_history(&task, Some(&cursor), 1).is_err(),
            "accepted malformed cursor {cursor:?}"
        );
    }
    for limit in [0, 21, usize::MAX] {
        assert!(f.state.task_history(&task, None, limit).is_err());
    }
    assert!(f.state.task_history("t_missing", None, 10).is_err());
    assert_eq!(f.state.status(&other).unwrap()["checkpoint_count"], 0);
    assert_eq!(f.state.status(&other).unwrap()["history_has_more"], false);
    let empty = f.state.task_history(&other, None, 20).unwrap();
    assert!(checkpoints(&empty).is_empty());
    assert!(empty["next_cursor"].is_null());
}

#[test]
fn history_preserves_unicode_and_bounds_serialized_pages_without_losing_notes() {
    let f = Fixture::new();
    let task = f.begin("unicode");
    let unicode = "🦀".repeat(512);
    f.append(&task, 1, &unicode);
    assert!(
        f.state
            .checkpoint(&task, 2, &"🦀".repeat(513), "too-many-bytes")
            .is_err()
    );
    for revision in 2..=25 {
        // Four maximally escaped notes per status window fit its existing
        // budget, but a 20-item history page must cut at the byte boundary.
        let note = if revision % 2 == 0 {
            "\u{0001}".repeat(2048)
        } else {
            format!("Plain {revision}")
        };
        f.append(&task, revision, &note);
    }
    let mut cursor = None;
    let mut seen = Vec::new();
    loop {
        let page = f.state.task_history(&task, cursor.as_deref(), 20).unwrap();
        assert!(serde_json::to_vec(&page).unwrap().len() <= 60 * 1024);
        assert!(!checkpoints(&page).is_empty());
        seen.extend_from_slice(checkpoints(&page));
        cursor = page["next_cursor"].as_str().map(str::to_owned);
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(seen.len(), 25);
    assert_eq!(seen.last().unwrap()["note"], unicode);
    assert_eq!(
        seen.iter()
            .map(|item| item["revision"].as_i64().unwrap())
            .collect::<Vec<_>>(),
        (2..=26).rev().collect::<Vec<_>>()
    );
}
