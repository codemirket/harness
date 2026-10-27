//! Stateless canonical enumeration, independent of retrieval ranking or models.
//!
//! Each page observes current heads in one SQLite transaction. Pages do not form
//! a stable snapshot: corrections behind the keyset boundary and new lower IDs
//! require a fresh scan. A deleted boundary remains a valid ordering key. Every
//! summary names its exact observed revision; callers re-read that revision and
//! its currentness before relying on it. A cursor is data, never authority.

use super::{
    APPLIES, Applicability, MAX_RESPONSE_BYTES, MAX_RESULTS, MAY_APPLY, MISSING_CONTEXT,
    MemoryError, MemoryStore, RecordKind, RecordRef, RecordStatus, Result, Scope, SearchContext,
    StatusFilter, VALID_AT, now, row_ids, valid_time, validate_budget, validate_context,
    validate_id,
};
use rusqlite::{Transaction, named_params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const EXCERPT_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApplicabilityFilter {
    #[default]
    Matching,
    /// Potentially matching heads with missing caller context. These are not
    /// established applicable; explicit conflicting selectors remain excluded.
    Unclassified,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListCursor {
    pub schema: u8,
    pub after_record_id: String,
    pub valid_at: u64,
    pub query_sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ListRequest {
    pub context: SearchContext,
    pub kind: Option<RecordKind>,
    pub status: StatusFilter,
    pub critical_only: bool,
    pub applicability: ApplicabilityFilter,
    pub limit: usize,
    pub max_bytes: usize,
    pub cursor: Option<ListCursor>,
}
impl Default for ListRequest {
    fn default() -> Self {
        Self {
            context: SearchContext::default(),
            kind: None,
            status: StatusFilter::Both,
            critical_only: false,
            applicability: ApplicabilityFilter::Matching,
            limit: MAX_RESULTS,
            max_bytes: MAX_RESPONSE_BYTES,
            cursor: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct HeadSummary {
    pub reference: RecordRef,
    pub kind: RecordKind,
    pub status: RecordStatus,
    pub critical: bool,
    pub applicability: Applicability,
    pub recorded_at: u64,
    pub valid_from: Option<u64>,
    pub valid_until: Option<u64>,
    pub excerpt: String,
    pub content_bytes: usize,
    pub content_truncated: bool,
    pub source_count: usize,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SummaryOmission {
    ByteBudget,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "detail", rename_all = "snake_case")]
pub enum HeadEntry {
    Summary(HeadSummary),
    ReferenceOnly {
        reference: RecordRef,
        reason: SummaryOmission,
    },
}
impl HeadEntry {
    pub fn reference(&self) -> &RecordRef {
        match self {
            Self::Summary(summary) => &summary.reference,
            Self::ReferenceOnly { reference, .. } => reference,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ListPage {
    pub scope: Scope,
    pub entries: Vec<HeadEntry>,
    pub valid_at: u64,
    pub observed_at: u64,
    pub stable_snapshot: bool,
    pub has_more: bool,
    pub next_cursor: Option<ListCursor>,
}

/// Pure validation before the service opens existing state. The service must
/// additionally bind checkout identity from the explicitly selected project.
pub fn validate_list_request(request: &ListRequest) -> Result<()> {
    validate_budget(request.limit, request.max_bytes)?;
    validate_context(&request.context)?;
    if let Some(cursor) = &request.cursor {
        validate_id(&cursor.after_record_id)?;
        valid_time(cursor.valid_at)?;
        if cursor.schema != 1
            || cursor.query_sha256.len() != 64
            || !cursor
                .query_sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || request
                .context
                .at_time
                .is_some_and(|at| at != cursor.valid_at)
        {
            return Err(MemoryError::InvalidInput("memory list cursor is invalid"));
        }
    }
    Ok(())
}

fn query_hash(scope: &Scope, request: &ListRequest, valid_at: u64) -> Result<String> {
    // Neither page size nor byte budget changes eligibility. No connection-local
    // SQLite data_version or invented global generation is a snapshot token.
    let bytes = serde_json::to_vec(&(
        1u8,
        scope,
        &request.context.task_key,
        &request.context.checkout_id,
        &request.context.operation,
        valid_at,
        request.kind,
        request.status,
        request.critical_only,
        request.applicability,
    ))?;
    Ok(Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

impl MemoryStore {
    /// Enumerate current canonical heads, including unresolved heads when
    /// selected. Open through open_existing at the service boundary: no schema,
    /// registration, vector index or operation receipt is created by this call.
    pub fn list_heads(&mut self, request: ListRequest) -> Result<ListPage> {
        validate_list_request(&request)?;
        let observed_at = now()?;
        let valid_at = request
            .cursor
            .as_ref()
            .map(|cursor| cursor.valid_at)
            .or(request.context.at_time)
            .unwrap_or(observed_at);
        let fingerprint = query_hash(&self.scope, &request, valid_at)?;
        if request
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.query_sha256 != fingerprint)
        {
            return Err(MemoryError::InvalidInput(
                "memory list cursor belongs to different scope or filters",
            ));
        }
        let after = request
            .cursor
            .as_ref()
            .map_or("", |cursor| cursor.after_record_id.as_str());
        let applicability = match request.applicability {
            ApplicabilityFilter::Matching => APPLIES.to_owned(),
            ApplicabilityFilter::Unclassified => {
                format!("({MAY_APPLY}) AND ({MISSING_CONTEXT})")
            }
        };
        let tx = self.connection.transaction()?;
        let ids = row_ids(
            &tx,
            &format!(
                "SELECT row_id FROM revisions WHERE is_current=1 AND {VALID_AT}
                 AND {applicability} AND (:kind IS NULL OR kind=:kind)
                 AND (:status='both' OR status=:status) AND (:critical=0 OR critical=1)
                 AND record_id>:after ORDER BY record_id LIMIT :limit"
            ),
            named_params! {
                ":at": valid_at as i64,
                ":task": request.context.task_key,
                ":checkout": request.context.checkout_id,
                ":operation": request.context.operation,
                ":kind": request.kind.map(RecordKind::as_str),
                ":status": request.status.as_str(),
                ":critical": request.critical_only,
                ":after": after,
                ":limit": (request.limit + 1) as i64,
            },
        )?;
        let mut page = ListPage {
            scope: self.scope.clone(),
            entries: Vec::new(),
            valid_at,
            observed_at,
            stable_snapshot: false,
            has_more: false,
            next_cursor: None,
        };
        if !fits(&page, request.max_bytes)? {
            return Err(MemoryError::InvalidInput(
                "memory list budget cannot fit page metadata",
            ));
        }
        for (position, id) in ids.iter().take(request.limit).enumerate() {
            let summary = read_summary(&tx, *id)?;
            let reference = summary.reference.clone();
            page.entries.push(HeadEntry::Summary(summary));
            set_continuation(&mut page, position + 1 < ids.len(), &fingerprint);
            if fits(&page, request.max_bytes)? {
                continue;
            }
            page.entries.pop();
            if page.entries.is_empty() {
                // Canonical records, selectors and their JSON escaping can be
                // larger than a page. Preserve discoverability and progress.
                page.entries.push(HeadEntry::ReferenceOnly {
                    reference,
                    reason: SummaryOmission::ByteBudget,
                });
                set_continuation(&mut page, position + 1 < ids.len(), &fingerprint);
                if !fits(&page, request.max_bytes)? {
                    return Err(MemoryError::InvalidInput(
                        "memory list budget cannot fit one reference and continuation; increase it",
                    ));
                }
            } else {
                set_continuation(&mut page, true, &fingerprint);
                break;
            }
        }
        tx.commit()?;
        Ok(page)
    }
}

fn fits(page: &ListPage, budget: usize) -> Result<bool> {
    Ok(serde_json::to_vec(page)?.len() <= budget)
}
fn set_continuation(page: &mut ListPage, has_more: bool, fingerprint: &str) {
    page.has_more = has_more;
    page.next_cursor = if has_more {
        page.entries.last().map(|entry| ListCursor {
            schema: 1,
            after_record_id: entry.reference().record_id.clone(),
            valid_at: page.valid_at,
            query_sha256: fingerprint.to_owned(),
        })
    } else {
        None
    };
}

fn read_summary(tx: &Transaction<'_>, row_id: i64) -> Result<HeadSummary> {
    #[derive(Deserialize)]
    struct Stored {
        reference: RecordRef,
        kind: RecordKind,
        status: RecordStatus,
        critical: u8,
        applicability: Applicability,
        recorded_at: u64,
        valid_from: Option<u64>,
        valid_until: Option<u64>,
        excerpt: String,
        content_bytes: usize,
        source_count: usize,
    }
    // Only a bounded prefix enters Rust. Source count uses the existing keyed
    // source index; no provenance array or supersession graph scan is needed.
    let json: String = tx.query_row(
        "SELECT json_object(
           'reference',json_object('record_id',record_id,'revision',revision),
           'kind',kind,'status',status,'critical',critical,
           'applicability',json_object('task_key',task_key,'checkout_id',checkout_id,'operation',operation),
           'recorded_at',recorded_at,'valid_from',valid_from,'valid_until',valid_until,
           'excerpt',substr(content,1,256),'content_bytes',length(CAST(content AS BLOB)),
           'source_count',(SELECT count(*) FROM sources WHERE revision_row_id=revisions.row_id))
         FROM revisions WHERE row_id=?1 AND is_current=1",
        [row_id],
        |row| row.get(0),
    )?;
    let mut stored: Stored = serde_json::from_str(&json)?;
    let mut end = EXCERPT_BYTES.min(stored.excerpt.len());
    while !stored.excerpt.is_char_boundary(end) {
        end -= 1;
    }
    stored.excerpt.truncate(end);
    Ok(HeadSummary {
        reference: stored.reference,
        kind: stored.kind,
        status: stored.status,
        critical: stored.critical == 1,
        applicability: stored.applicability,
        recorded_at: stored.recorded_at,
        valid_from: stored.valid_from,
        valid_until: stored.valid_until,
        content_truncated: stored.excerpt.len() < stored.content_bytes,
        excerpt: stored.excerpt,
        content_bytes: stored.content_bytes,
        source_count: stored.source_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{ClaimBasis, Record, RecordInput, Source};
    use std::{
        collections::BTreeMap,
        fs,
        path::{Path, PathBuf},
        sync::atomic::Ordering,
    };

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "mirket-list-test-{}-{}",
                std::process::id(),
                super::super::NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            super::super::private_directory(&path, true).unwrap();
            Self(path.canonicalize().unwrap())
        }
        fn scope(&self) -> Scope {
            Scope::Project("list-test".into())
        }
        fn open(&self) -> MemoryStore {
            MemoryStore::open(&self.0, self.scope()).unwrap()
        }
        fn read(&self) -> MemoryStore {
            MemoryStore::open_existing(&self.0, self.scope()).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn operation() -> String {
        format!(
            "op-list-{}",
            super::super::NEXT_ID.fetch_add(1, Ordering::Relaxed)
        )
    }
    fn input(content: &str) -> RecordInput {
        RecordInput {
            kind: RecordKind::Fact,
            basis: ClaimBasis::Observed,
            content: content.into(),
            sources: vec![Source {
                source_id: "fixture".into(),
                revision: "revision-1".into(),
                locator: "line:1".into(),
                captured_at: 10,
            }],
            status: RecordStatus::Current,
            critical: false,
            applicability: Applicability::default(),
            valid_from: None,
            valid_until: None,
        }
    }
    fn seed(store: &mut MemoryStore, id: &str, data: RecordInput) -> Record {
        super::super::validate_input(&store.scope, &data).unwrap();
        validate_id(id).unwrap();
        let tx = store.connection.transaction().unwrap();
        tx.execute(
            "INSERT INTO records(record_id,head_revision) VALUES(?1,1)",
            [id],
        )
        .unwrap();
        super::super::insert_revision(&tx, id, 1, &data, 10, None).unwrap();
        let record = super::super::read_record(&tx, id, 1).unwrap();
        tx.commit().unwrap();
        record
    }
    fn request() -> ListRequest {
        ListRequest {
            context: SearchContext {
                at_time: Some(100),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    fn ids(page: &ListPage) -> Vec<&str> {
        page.entries
            .iter()
            .map(|e| e.reference().record_id.as_str())
            .collect()
    }
    fn summary(entry: &HeadEntry) -> &HeadSummary {
        match entry {
            HeadEntry::Summary(s) => s,
            _ => panic!("expected full summary"),
        }
    }

    #[test]
    fn critical_overflow_has_complete_bounded_enumeration_and_exact_tail_read() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        for n in 0..53 {
            let mut data = input(&format!("constraint {n}: decisive exception"));
            data.critical = true;
            seed(&mut store, &format!("constraint-{n:03}"), data);
        }
        let search = store
            .search(super::super::SearchRequest::default())
            .unwrap();
        assert!(search.coverage.critical_overflow);
        assert_eq!(search.coverage.applicable_current_critical, 53);
        let mut request = ListRequest {
            critical_only: true,
            status: StatusFilter::Current,
            ..request()
        };
        let mut references = Vec::new();
        let mut pages = 0;
        loop {
            let page = store.list_heads(request.clone()).unwrap();
            assert!(!page.stable_snapshot);
            assert!(page.entries.len() <= 20);
            assert!(serde_json::to_vec(&page).unwrap().len() <= request.max_bytes);
            references.extend(page.entries.iter().map(|e| e.reference().clone()));
            pages += 1;
            if !page.has_more {
                assert!(page.next_cursor.is_none());
                break;
            }
            request.cursor = page.next_cursor;
            assert!(pages < 4);
        }
        assert_eq!(pages, 3);
        assert_eq!(references.len(), 53);
        assert!(
            references
                .windows(2)
                .all(|w| w[0].record_id < w[1].record_id)
        );
        let tail = references.last().unwrap();
        let record = store.read(&tail.record_id, Some(tail.revision)).unwrap();
        assert_eq!(record.data.content, "constraint 52: decisive exception");
        assert_eq!(record.data.sources[0].locator, "line:1");
    }

    #[test]
    fn maximum_escaped_record_uses_explicit_reference_fallback_and_progresses() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        let mut data = input(&"\u{1}".repeat(super::super::MAX_CONTENT_BYTES));
        data.sources = (0..8)
            .map(|_| Source {
                source_id: "\u{1}".repeat(512),
                revision: "\u{1}".repeat(256),
                locator: "\u{1}".repeat(512),
                captured_at: 100,
            })
            .collect();
        data.applicability = Applicability {
            task_key: Some("\u{1}".repeat(256)),
            checkout_id: Some("\u{1}".repeat(256)),
            operation: Some("\u{1}".repeat(256)),
        };
        let context = SearchContext {
            task_key: data.applicability.task_key.clone(),
            checkout_id: data.applicability.checkout_id.clone(),
            operation: data.applicability.operation.clone(),
            at_time: Some(100),
        };
        let record = seed(&mut store, "a", data);
        assert!(serde_json::to_vec(&record).unwrap().len() > MAX_RESPONSE_BYTES);
        seed(&mut store, "b", input("second record remains discoverable"));
        let mut req = ListRequest {
            context,
            max_bytes: 1024,
            limit: 1,
            ..Default::default()
        };
        let first = store.list_heads(req.clone()).unwrap();
        assert_eq!(ids(&first), ["a"]);
        assert!(matches!(
            &first.entries[0],
            HeadEntry::ReferenceOnly {
                reason: SummaryOmission::ByteBudget,
                ..
            }
        ));
        assert!(first.has_more);
        assert!(serde_json::to_vec(&first).unwrap().len() <= 1024);
        req.cursor = first.next_cursor;
        req.limit = 20;
        req.max_bytes = MAX_RESPONSE_BYTES;
        let second = store.list_heads(req).unwrap();
        assert_eq!(ids(&second), ["b"]);
        assert!(!second.has_more);
        assert!(second.next_cursor.is_none());
        assert_eq!(store.read("a", Some(1)).unwrap().data, record.data);
    }

    #[test]
    fn utf8_excerpt_is_bounded_and_full_short_content_is_explicit() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        seed(
            &mut store,
            "a",
            input(&format!("{}tail", "界ç".repeat(100))),
        );
        seed(&mut store, "b", input("kısa"));
        let page = store.list_heads(request()).unwrap();
        let long = summary(&page.entries[0]);
        assert!(long.excerpt.len() <= EXCERPT_BYTES);
        assert!(format!("{}tail", "界ç".repeat(100)).starts_with(&long.excerpt));
        assert_eq!(long.content_bytes, 504);
        assert!(long.content_truncated);
        assert_eq!(long.source_count, 1);
        let short = summary(&page.entries[1]);
        assert_eq!(short.excerpt, "kısa");
        assert!(!short.content_truncated);
    }

    #[test]
    fn filters_preserve_currentness_validity_and_matching_vs_unclassified_context() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        seed(&mut store, "global", input("global"));
        let mut scoped = input("task checkpoint");
        scoped.kind = RecordKind::TaskState;
        scoped.status = RecordStatus::Unresolved;
        scoped.critical = true;
        scoped.applicability = Applicability {
            task_key: Some("task-a".into()),
            checkout_id: Some("checkout-a".into()),
            operation: Some("read".into()),
        };
        seed(&mut store, "task", scoped.clone());
        scoped.applicability.checkout_id = Some("checkout-b".into());
        seed(&mut store, "other-checkout", scoped);
        let mut expired = input("expired");
        expired.valid_until = Some(100);
        seed(&mut store, "expired", expired);
        let mut future = input("future");
        future.valid_from = Some(101);
        seed(&mut store, "future", future);
        let mut req = request();
        req.context.checkout_id = Some("checkout-a".into());
        assert_eq!(ids(&store.list_heads(req.clone()).unwrap()), ["global"]);
        req.applicability = ApplicabilityFilter::Unclassified;
        let page = store.list_heads(req.clone()).unwrap();
        assert_eq!(ids(&page), ["task"]);
        assert_eq!(
            summary(&page.entries[0]).applicability.task_key.as_deref(),
            Some("task-a")
        );
        req.context.task_key = Some("task-a".into());
        req.context.operation = Some("read".into());
        assert!(store.list_heads(req.clone()).unwrap().entries.is_empty());
        req.applicability = ApplicabilityFilter::Matching;
        req.kind = Some(RecordKind::TaskState);
        req.status = StatusFilter::Unresolved;
        req.critical_only = true;
        assert_eq!(ids(&store.list_heads(req.clone()).unwrap()), ["task"]);
        req.status = StatusFilter::Current;
        assert!(store.list_heads(req).unwrap().entries.is_empty());
    }

    #[test]
    fn cursor_binds_scope_filters_checkout_and_time_but_not_page_budgets() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        seed(&mut store, "a", input("one"));
        seed(&mut store, "b", input("two"));
        let mut original = ListRequest {
            limit: 1,
            ..request()
        };
        original.cursor = store.list_heads(original.clone()).unwrap().next_cursor;
        let mut changed = Vec::new();
        let mut req = original.clone();
        req.context.task_key = Some("other".into());
        changed.push(req);
        let mut req = original.clone();
        req.context.checkout_id = Some("other".into());
        changed.push(req);
        let mut req = original.clone();
        req.context.operation = Some("other".into());
        changed.push(req);
        let mut req = original.clone();
        req.context.at_time = Some(101);
        changed.push(req);
        let mut req = original.clone();
        req.kind = Some(RecordKind::Decision);
        changed.push(req);
        let mut req = original.clone();
        req.status = StatusFilter::Current;
        changed.push(req);
        let mut req = original.clone();
        req.critical_only = true;
        changed.push(req);
        let mut req = original.clone();
        req.applicability = ApplicabilityFilter::Unclassified;
        changed.push(req);
        for request in changed {
            assert!(matches!(
                store.list_heads(request),
                Err(MemoryError::InvalidInput(_))
            ));
        }
        let mut other = MemoryStore::open(&fixture.0, Scope::Project("other".into())).unwrap();
        assert!(matches!(
            other.list_heads(original.clone()),
            Err(MemoryError::InvalidInput(_))
        ));
        original.limit = 20;
        original.max_bytes = 1024;
        assert_eq!(ids(&store.list_heads(original).unwrap()), ["b"]);
    }

    #[test]
    fn pages_reobserve_heads_without_claiming_snapshot_or_requiring_live_boundary() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        for id in ["a", "b", "c", "d", "e"] {
            seed(&mut store, id, input(id));
        }
        let mut req = ListRequest {
            limit: 2,
            ..request()
        };
        let first = store.list_heads(req.clone()).unwrap();
        assert_eq!(ids(&first), ["a", "b"]);
        req.cursor = first.next_cursor;
        store
            .revise(&operation(), "a", 1, input("behind boundary correction"))
            .unwrap();
        store
            .revise(&operation(), "c", 1, input("ahead correction"))
            .unwrap();
        let successor = store
            .create(
                &operation(),
                input("successor"),
                Some(RecordRef {
                    record_id: "e".into(),
                    revision: 1,
                }),
            )
            .unwrap();
        seed(&mut store, "0", input("inserted behind boundary"));
        seed(&mut store, "f", input("inserted ahead"));
        drop(store);
        for id in ["b", "d"] {
            crate::memory_lifecycle::forget(&fixture.0, fixture.scope(), id, &operation(), 1)
                .unwrap();
        }
        let mut store = fixture.read();
        req.limit = 20;
        let next = store.list_heads(req).unwrap();
        assert_eq!(ids(&next), ["c", "f", successor.record_id.as_str()]);
        assert_eq!(next.entries[0].reference().revision, 2);
        assert!(!next.stable_snapshot);
        assert!(!next.has_more);
        assert!(!store.read("a", Some(1)).unwrap().is_current_revision);
        assert!(matches!(
            store.read("b", Some(1)),
            Err(MemoryError::NotFound)
        ));
        let restarted = store.list_heads(request()).unwrap();
        assert_eq!(
            ids(&restarted),
            ["0", "a", "c", "f", successor.record_id.as_str()]
        );
        assert_eq!(restarted.entries[1].reference().revision, 2);
    }

    #[test]
    fn cursor_freezes_validity_instant_without_freezing_canonical_heads() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        let mut data = input("valid at requested instant");
        data.valid_until = Some(101);
        seed(&mut store, "a", data.clone());
        seed(&mut store, "b", data);
        let first = store
            .list_heads(ListRequest {
                limit: 1,
                ..request()
            })
            .unwrap();
        assert_eq!(first.valid_at, 100);
        let second = store
            .list_heads(ListRequest {
                cursor: first.next_cursor,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(ids(&second), ["b"]);
        assert_eq!(second.valid_at, 100);
        assert!(!second.stable_snapshot);
        assert!(
            store
                .list_heads(ListRequest::default())
                .unwrap()
                .entries
                .is_empty()
        );
    }

    fn tree(path: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
        fn walk(root: &Path, at: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
            for entry in fs::read_dir(at).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    walk(root, &path, files);
                } else {
                    files.insert(
                        path.strip_prefix(root).unwrap().to_owned(),
                        fs::read(path).unwrap(),
                    );
                }
            }
        }
        let mut files = BTreeMap::new();
        walk(path, path, &mut files);
        files
    }

    #[test]
    fn existing_read_only_enumeration_never_creates_or_changes_canonical_state() {
        let fixture = Fixture::new();
        assert!(MemoryStore::open_existing(&fixture.0, fixture.scope()).is_err());
        assert_eq!(fs::read_dir(&fixture.0).unwrap().count(), 0);
        let mut store = fixture.open();
        seed(&mut store, "a", input("read-only"));
        drop(store);
        let before = tree(&fixture.0);
        let mut store = fixture.read();
        assert_eq!(ids(&store.list_heads(request()).unwrap()), ["a"]);
        assert!(store.connection.execute("DELETE FROM records", []).is_err());
        drop(store);
        assert_eq!(tree(&fixture.0), before);
    }

    #[test]
    fn validation_rejects_unknown_fields_and_bad_bounds_before_state_access() {
        for (limit, max_bytes) in [
            (0, 1024),
            (21, 1024),
            (1, 1023),
            (1, MAX_RESPONSE_BYTES + 1),
        ] {
            assert!(
                validate_list_request(&ListRequest {
                    limit,
                    max_bytes,
                    ..request()
                })
                .is_err()
            );
        }
        let mut value = serde_json::to_value(request()).unwrap();
        value["unknown"] = true.into();
        assert!(serde_json::from_value::<ListRequest>(value).is_err());
        let mut req = request();
        req.cursor = Some(ListCursor {
            schema: 1,
            after_record_id: "a".into(),
            valid_at: 100,
            query_sha256: "a".repeat(64),
        });
        validate_list_request(&req).unwrap();
        req.cursor.as_mut().unwrap().schema = 2;
        assert!(validate_list_request(&req).is_err());
        req.cursor.as_mut().unwrap().schema = 1;
        req.cursor.as_mut().unwrap().query_sha256 = "A".repeat(64);
        assert!(validate_list_request(&req).is_err());
        req.cursor.as_mut().unwrap().query_sha256 = "a".repeat(64);
        req.cursor.as_mut().unwrap().after_record_id = "../a".into();
        assert!(validate_list_request(&req).is_err());
    }
}
