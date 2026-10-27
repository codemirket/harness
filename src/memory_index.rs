//! Rebuildable, revision-bound semantic data. No inference or model loading.
//!
//! Selection holds only a read transaction. Publication rechecks the exact head
//! and content digest under an immediate transaction; computation happens outside
//! both. Ordinary tables keep filtering in canonical SQL before cosine scoring.

use super::*;
use crate::embedding::{self, ChunkEmbedding, ModelIdentity, PassageEmbedding};

pub const MAX_INDEX_HEADS: usize = 32;
pub const MAX_INDEX_BYTES: usize = 256 * 1024;
const VECTOR_BYTES: usize = embedding::DIMENSIONS * 4;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndexCandidate {
    pub reference: RecordRef,
    pub source_sha256: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct IndexBatch {
    pub model_identity: ModelIdentity,
    pub items: Vec<IndexCandidate>,
    /// Includes already indexed heads inspected within this bounded page.
    pub scanned_heads: usize,
    pub scanned_bytes: usize,
    pub next_after_row_id: Option<i64>,
    pub has_more: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct IndexCoverage {
    pub model_identity: ModelIdentity,
    pub current_heads: u64,
    /// Structurally complete expected-model rows, not a full vector-integrity
    /// audit or a semantic-quality measurement. Recall must use validated chunks.
    pub indexed_heads: u64,
    pub missing_heads: u64,
}

/// Call only from the parent's authorized writable schema transaction.
pub(super) fn initialize(tx: &Transaction<'_>) -> Result<()> {
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS semantic_revisions(
            revision_row_id INTEGER PRIMARY KEY REFERENCES revisions(row_id) ON DELETE CASCADE,
            model_identity TEXT NOT NULL,
            source_sha256 TEXT NOT NULL CHECK(length(source_sha256)=64 AND source_sha256 NOT GLOB '*[^0-9a-f]*'),
            input_bytes INTEGER NOT NULL CHECK(input_bytes BETWEEN 1 AND 16384),
            content_tokens INTEGER NOT NULL CHECK(content_tokens BETWEEN 1 AND 65536),
            chunk_count INTEGER NOT NULL CHECK(chunk_count BETWEEN 1 AND 128)
        ) STRICT;
        CREATE TABLE IF NOT EXISTS semantic_chunks(
            revision_row_id INTEGER NOT NULL REFERENCES revisions(row_id) ON DELETE CASCADE
                REFERENCES semantic_revisions(revision_row_id) ON DELETE CASCADE,
            ordinal INTEGER NOT NULL CHECK(ordinal BETWEEN 0 AND 127),
            start_byte INTEGER NOT NULL CHECK(start_byte>=0),
            end_byte INTEGER NOT NULL CHECK(end_byte>start_byte AND end_byte<=16384),
            input_tokens INTEGER NOT NULL CHECK(input_tokens BETWEEN 1 AND 512),
            vector BLOB NOT NULL CHECK(length(vector)=1536),
            PRIMARY KEY(revision_row_id,ordinal)
        ) STRICT;
        CREATE TRIGGER IF NOT EXISTS semantic_retired AFTER UPDATE OF is_current,content ON revisions
        WHEN new.is_current=0 OR new.content<>old.content BEGIN
            DELETE FROM semantic_revisions WHERE revision_row_id=old.row_id;
        END;",
    )?;
    let identity = model_identity()?;
    tx.execute(
        "DELETE FROM semantic_revisions WHERE model_identity<>?1 OR revision_row_id NOT IN
        (SELECT r.row_id FROM revisions r JOIN records h ON h.record_id=r.record_id
         AND h.head_revision=r.revision WHERE r.is_current=1)",
        [serde_json::to_string(&identity)?],
    )?;
    Ok(())
}

fn model_identity() -> Result<ModelIdentity> {
    embedding::expected_model_identity()
        .map_err(|_| MemoryError::InvalidInput("compiled embedding identity is invalid"))
}

pub fn source_digest(content: &str) -> String {
    Sha256::digest(content.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

impl MemoryStore {
    /// Scan at most `limit` current heads and `max_bytes` of their content.
    /// An empty items page can still have a continuation: already indexed heads
    /// also consume the scan budget. Restart at None on a new indexing pass;
    /// changes behind the cursor require that new pass. Separate pages do not
    /// claim one stable snapshot across concurrent canonical mutations.
    /// No schema, repair, model, filesystem or lifecycle mutation occurs here.
    pub fn select_index_batch(
        &mut self,
        after_row_id: Option<i64>,
        limit: usize,
        max_bytes: usize,
    ) -> Result<IndexBatch> {
        if !(1..=MAX_INDEX_HEADS).contains(&limit)
            || !(MAX_CONTENT_BYTES..=MAX_INDEX_BYTES).contains(&max_bytes)
            || after_row_id.is_some_and(|id| id <= 0)
        {
            return Err(MemoryError::InvalidInput(
                "invalid bounded index page request",
            ));
        }
        let identity = model_identity()?;
        let tx = self.connection.transaction()?;
        let mut statement = tx.prepare(
            "SELECT r.row_id,r.record_id,r.revision,
             CASE WHEN length(CAST(r.content AS BLOB)) BETWEEN 1 AND 16384 THEN r.content ELSE '' END
             FROM revisions r
             JOIN records h ON h.record_id=r.record_id AND h.head_revision=r.revision
             WHERE r.is_current=1 AND r.row_id>?1 ORDER BY r.row_id LIMIT ?2",
        )?;
        let mut rows = statement.query(params![after_row_id.unwrap_or(0), limit as i64 + 1])?;
        let mut batch = IndexBatch {
            model_identity: identity.clone(),
            items: Vec::new(),
            scanned_heads: 0,
            scanned_bytes: 0,
            next_after_row_id: None,
            has_more: false,
        };
        let mut last = after_row_id;
        while let Some(row) = rows.next()? {
            if batch.scanned_heads == limit {
                batch.has_more = true;
                break;
            }
            let row_id: i64 = row.get(0)?;
            let content: String = row.get(3)?;
            if content.is_empty() || content.len() > MAX_CONTENT_BYTES {
                return Err(MemoryError::InvalidInput(
                    "canonical content is outside index bounds",
                ));
            }
            if batch.scanned_bytes + content.len() > max_bytes {
                batch.has_more = true;
                break;
            }
            batch.scanned_heads += 1;
            batch.scanned_bytes += content.len();
            last = Some(row_id);
            let digest = source_digest(&content);
            // Corrupt/obsolete derived values are a cache miss, never a reason
            // to repair canonical data or silently consume an invalid vector.
            if read_chunks(&tx, row_id, &content, &identity)?.is_none() {
                batch.items.push(IndexCandidate {
                    reference: RecordRef {
                        record_id: row.get(1)?,
                        revision: unsigned(row, 2)?,
                    },
                    source_sha256: digest,
                    content,
                });
            }
        }
        batch.next_after_row_id = batch.has_more.then_some(last).flatten();
        drop(rows);
        drop(statement);
        tx.commit()?;
        Ok(batch)
    }

    /// Publish only after recomputing the exact source digest under a write lock.
    /// The scope access guard excludes forgetting/restore throughout this store's
    /// lifetime. A reopened store rejects exclusions before reaching this method.
    pub fn apply_index(
        &mut self,
        reference: &RecordRef,
        source_sha256: &str,
        passage: &PassageEmbedding,
    ) -> Result<()> {
        validate_reference(reference)?;
        let identity = model_identity()?;
        if passage.model_identity != identity {
            return Err(MemoryError::InvalidInput(
                "embedding model identity differs from compiled pins",
            ));
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let source: Option<(i64, String)> = tx.query_row(
            "SELECT r.row_id,
             CASE WHEN length(CAST(r.content AS BLOB)) BETWEEN 1 AND 16384 THEN r.content ELSE '' END
             FROM revisions r JOIN records h ON h.record_id=r.record_id
             AND h.head_revision=r.revision WHERE r.record_id=?1 AND r.revision=?2 AND r.is_current=1",
            params![reference.record_id, reference.revision as i64],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).optional()?;
        let (row_id, content) = source.ok_or(MemoryError::Conflict)?;
        if source_digest(&content) != source_sha256 {
            return Err(MemoryError::Conflict);
        }
        validate_passage(&content, passage, &identity)?;
        // Validation precedes deletion, and every chunk shares one summary row.
        tx.execute(
            "DELETE FROM semantic_revisions WHERE revision_row_id=?1",
            [row_id],
        )?;
        tx.execute(
            "INSERT INTO semantic_revisions(revision_row_id,model_identity,source_sha256,input_bytes,content_tokens,chunk_count)
             VALUES(?1,?2,?3,?4,?5,?6)",
            params![row_id, serde_json::to_string(&identity)?, source_sha256,
                passage.input_bytes as i64, passage.content_tokens as i64, passage.chunks.len() as i64],
        )?;
        for (ordinal, chunk) in passage.chunks.iter().enumerate() {
            tx.execute(
                "INSERT INTO semantic_chunks(revision_row_id,ordinal,start_byte,end_byte,input_tokens,vector)
                 VALUES(?1,?2,?3,?4,?5,?6)",
                params![row_id, ordinal as i64, chunk.start_byte as i64, chunk.end_byte as i64,
                    chunk.input_tokens as i64, vector_blob(&chunk.vector)],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn index_coverage(&mut self) -> Result<IndexCoverage> {
        let identity = model_identity()?;
        let tx = self.connection.transaction()?;
        let current_heads: u64 = tx.query_row(
            "SELECT count(*) FROM revisions r JOIN records h ON h.record_id=r.record_id
             AND h.head_revision=r.revision WHERE r.is_current=1",
            [],
            |row| unsigned(row, 0),
        )?;
        let indexed_heads: u64 = tx.query_row(
            "SELECT count(*) FROM semantic_revisions s JOIN revisions r ON r.row_id=s.revision_row_id
             JOIN records h ON h.record_id=r.record_id AND h.head_revision=r.revision
             WHERE r.is_current=1 AND s.model_identity=?1 AND s.input_bytes=length(CAST(r.content AS BLOB))
             AND (SELECT count(*) FROM semantic_chunks c WHERE c.revision_row_id=s.revision_row_id)=s.chunk_count
             AND (SELECT min(ordinal) FROM semantic_chunks c WHERE c.revision_row_id=s.revision_row_id)=0
             AND (SELECT max(ordinal) FROM semantic_chunks c WHERE c.revision_row_id=s.revision_row_id)=s.chunk_count-1
             AND (SELECT start_byte FROM semantic_chunks c WHERE c.revision_row_id=s.revision_row_id AND ordinal=0)=0
             AND (SELECT end_byte FROM semantic_chunks c WHERE c.revision_row_id=s.revision_row_id AND ordinal=s.chunk_count-1)=s.input_bytes
             AND NOT EXISTS(SELECT 1 FROM semantic_chunks c JOIN semantic_chunks p
                 ON p.revision_row_id=c.revision_row_id AND p.ordinal=c.ordinal-1
                 WHERE c.revision_row_id=s.revision_row_id AND
                 (c.start_byte>p.end_byte OR c.start_byte<p.start_byte OR c.end_byte<=p.end_byte))",
            [serde_json::to_string(&identity)?], |row| unsigned(row, 0),
        )?;
        tx.commit()?;
        Ok(IndexCoverage {
            model_identity: identity,
            current_heads,
            indexed_heads,
            missing_heads: current_heads - indexed_heads,
        })
    }
}

/// Used by filtered Rust cosine recall after canonical SQLite eligibility.
/// Missing, mismatched or corrupt derived values are an explicit cache miss.
pub(super) fn read_chunks(
    tx: &Transaction<'_>,
    row_id: i64,
    content: &str,
    identity: &ModelIdentity,
) -> Result<Option<Vec<ChunkEmbedding>>> {
    ChunkReader::new(tx, identity)?.read(row_id, content)
}

/// Reuse the two index lookups across an eligible recall scan. The statements
/// stay inside the caller's canonical read transaction and share its snapshot.
pub(super) struct ChunkReader<'tx> {
    header: rusqlite::Statement<'tx>,
    chunks: rusqlite::Statement<'tx>,
    identity: ModelIdentity,
    identity_json: String,
}

impl<'tx> ChunkReader<'tx> {
    pub(super) fn new(tx: &'tx Transaction<'_>, identity: &ModelIdentity) -> Result<Self> {
        Ok(Self {
            header: tx.prepare(
                "SELECT substr(model_identity,1,1024),substr(source_sha256,1,65),input_bytes,content_tokens,chunk_count
                 FROM semantic_revisions WHERE revision_row_id=?1",
            )?,
            chunks: tx.prepare(
                "SELECT ordinal,start_byte,end_byte,input_tokens,
                 CASE WHEN length(vector)=1536 THEN vector ELSE X'' END FROM semantic_chunks
                 WHERE revision_row_id=?1 ORDER BY ordinal LIMIT 129",
            )?,
            identity: identity.clone(),
            identity_json: serde_json::to_string(identity)?,
        })
    }

    pub(super) fn read(
        &mut self,
        row_id: i64,
        content: &str,
    ) -> Result<Option<Vec<ChunkEmbedding>>> {
        let header: Option<(String, String, i64, i64, i64)> = self
            .header
            .query_row([row_id], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })
            .optional()?;
        let Some((stored_identity, digest, bytes, tokens, count)) = header else {
            return Ok(None);
        };
        if stored_identity != self.identity_json
            || digest != source_digest(content)
            || bytes != content.len() as i64
            || !(1..=embedding::MAX_CHUNKS as i64).contains(&count)
            || !(1..=(embedding::MAX_CHUNKS * embedding::MAX_INPUT_TOKENS) as i64).contains(&tokens)
        {
            return Ok(None);
        }
        let mut rows = self.chunks.query([row_id])?;
        let mut chunks = Vec::new();
        while let Some(row) = rows.next()? {
            let ordinal: i64 = row.get(0)?;
            let start: i64 = row.get(1)?;
            let end: i64 = row.get(2)?;
            let input_tokens: i64 = row.get(3)?;
            let blob: Vec<u8> = row.get(4)?;
            if ordinal != chunks.len() as i64
                || start < 0
                || end < 0
                || input_tokens < 0
                || blob.len() != VECTOR_BYTES
                || chunks.len() == embedding::MAX_CHUNKS
            {
                return Ok(None);
            }
            chunks.push(ChunkEmbedding {
                start_byte: start as usize,
                end_byte: end as usize,
                input_tokens: input_tokens as usize,
                vector: blob
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|bytes| f32::from_le_bytes(*bytes))
                    .collect(),
            });
        }
        if chunks.len() != count as usize {
            return Ok(None);
        }
        let passage = PassageEmbedding {
            model_identity: self.identity.clone(),
            input_bytes: bytes as usize,
            content_tokens: tokens as usize,
            covered_bytes: bytes as usize,
            complete: true,
            chunks,
        };
        if validate_passage(content, &passage, &self.identity).is_err() {
            return Ok(None);
        }
        Ok(Some(passage.chunks))
    }
}

pub(super) fn vector_blob(vector: &[f32]) -> Vec<u8> {
    // All admitted targets are little endian, as is the persisted f32 contract.
    vector
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn validate_passage(
    content: &str,
    passage: &PassageEmbedding,
    identity: &ModelIdentity,
) -> Result<()> {
    let invalid = || {
        MemoryError::InvalidInput(
            "embedding has invalid identity, vector or complete source coverage",
        )
    };
    if content.is_empty()
        || content.len() > MAX_CONTENT_BYTES
        || passage.model_identity != *identity
        || passage.input_bytes != content.len()
        || passage.covered_bytes != content.len()
        || !passage.complete
        || passage.chunks.is_empty()
        || passage.chunks.len() > embedding::MAX_CHUNKS
    {
        return Err(invalid());
    }
    let mut end = 0;
    let mut last_start = 0;
    let mut token_sum = 0;
    for chunk in &passage.chunks {
        if chunk.start_byte > end
            || chunk.end_byte <= end
            || chunk.start_byte < last_start
            || chunk.start_byte >= chunk.end_byte
            || chunk.end_byte > content.len()
            || !content.is_char_boundary(chunk.start_byte)
            || !content.is_char_boundary(chunk.end_byte)
            || !(1..=embedding::MAX_INPUT_TOKENS).contains(&chunk.input_tokens)
            || chunk.vector.len() != embedding::DIMENSIONS
            || chunk.vector.iter().any(|x| !x.is_finite())
        {
            return Err(invalid());
        }
        let norm: f64 = chunk.vector.iter().map(|x| f64::from(*x).powi(2)).sum();
        if (norm - 1.0).abs() > 0.002 {
            return Err(invalid());
        }
        token_sum += chunk.input_tokens;
        last_start = chunk.start_byte;
        end = chunk.end_byte;
    }
    if end != content.len() || passage.content_tokens == 0 || passage.content_tokens > token_sum {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "mirket-index-{}-{}",
                std::process::id(),
                NEXT_ID.fetch_add(1, Ordering::Relaxed)
            ));
            private_directory(&root, true).unwrap();
            Self(root.canonicalize().unwrap())
        }
        fn open(&self) -> MemoryStore {
            MemoryStore::open(&self.0, Scope::Project("index-test".into())).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    fn operation() -> String {
        format!("op-index-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
    fn input(content: &str) -> RecordInput {
        RecordInput {
            kind: RecordKind::Fact,
            basis: ClaimBasis::Observed,
            content: content.into(),
            sources: vec![Source {
                source_id: "fixture:exact".into(),
                revision: "v1".into(),
                locator: "line:1".into(),
                captured_at: 1,
            }],
            status: RecordStatus::Current,
            critical: false,
            applicability: Applicability::default(),
            valid_from: None,
            valid_until: None,
        }
    }
    fn reference(record: &Record) -> RecordRef {
        RecordRef {
            record_id: record.record_id.clone(),
            revision: record.revision,
        }
    }
    fn unit(axis: usize) -> Vec<f32> {
        let mut result = vec![0.0; embedding::DIMENSIONS];
        result[axis] = 1.0;
        result
    }
    fn passage(content: &str) -> PassageEmbedding {
        PassageEmbedding {
            model_identity: model_identity().unwrap(),
            input_bytes: content.len(),
            content_tokens: 1,
            covered_bytes: content.len(),
            complete: true,
            chunks: vec![ChunkEmbedding {
                start_byte: 0,
                end_byte: content.len(),
                input_tokens: 4,
                vector: unit(0),
            }],
        }
    }
    fn two_chunks(content: &str, first_end: usize, second_start: usize) -> PassageEmbedding {
        let mut output = passage(content);
        output.chunks[0].end_byte = first_end;
        output.chunks.push(ChunkEmbedding {
            start_byte: second_start,
            end_byte: content.len(),
            input_tokens: 4,
            vector: unit(1),
        });
        output
    }
    fn publish(store: &mut MemoryStore, record: &Record) {
        store
            .apply_index(
                &reference(record),
                &source_digest(&record.data.content),
                &passage(&record.data.content),
            )
            .unwrap();
    }
    fn count(store: &MemoryStore, table: &str) -> u64 {
        assert!(matches!(table, "semantic_revisions" | "semantic_chunks"));
        store
            .connection
            .query_row(&format!("SELECT count(*) FROM {table}"), [], |row| {
                unsigned(row, 0)
            })
            .unwrap()
    }

    #[test]
    fn sqlite_omits_extension_loader() {
        let connection = Connection::open_in_memory().unwrap();
        verify_runtime(&connection).unwrap();
        let functions: u64 = connection
            .query_row(
                "SELECT count(*) FROM pragma_function_list WHERE name='load_extension'",
                [],
                |row| unsigned(row, 0),
            )
            .unwrap();
        assert_eq!(functions, 0);
    }

    #[test]
    fn exact_utf8_chunks_replace_atomically_and_selection_is_read_only() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        let record = store
            .create(&operation(), input("İstanbul 中文 tail"), None)
            .unwrap();
        let batch = store.select_index_batch(None, 32, MAX_INDEX_BYTES).unwrap();
        assert_eq!(batch.items.len(), 1);
        assert_eq!(batch.items[0].reference, reference(&record));
        let output = two_chunks(
            &record.data.content,
            "İstanbul 中文".len(),
            "İstanbul ".len(),
        );
        store
            .apply_index(&reference(&record), &batch.items[0].source_sha256, &output)
            .unwrap();
        assert_eq!(count(&store, "semantic_chunks"), 2);
        assert_eq!(store.index_coverage().unwrap().indexed_heads, 1);
        let changes = store.connection.total_changes();
        assert!(
            store
                .select_index_batch(None, 32, MAX_INDEX_BYTES)
                .unwrap()
                .items
                .is_empty()
        );
        assert_eq!(store.connection.total_changes(), changes);
        publish(&mut store, &record);
        assert_eq!(count(&store, "semantic_chunks"), 1);
    }

    #[test]
    fn invalid_results_cannot_destroy_a_previous_index() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        let record = store
            .create(&operation(), input("İstanbul 中文 tail"), None)
            .unwrap();
        publish(&mut store, &record);
        let good = passage(&record.data.content);
        let mut variants = vec![];
        let mut altered = good.clone();
        altered.model_identity.id.push('x');
        variants.push(altered);
        let mut altered = good.clone();
        altered.complete = false;
        variants.push(altered);
        let mut altered = good.clone();
        altered.content_tokens = usize::MAX;
        variants.push(altered);
        let mut altered = good.clone();
        altered.chunks[0].vector[0] = f32::NAN;
        variants.push(altered);
        let mut altered = good.clone();
        altered.chunks[0].vector[0] = 0.0;
        variants.push(altered);
        let mut altered = good.clone();
        altered.chunks[0].start_byte = 1;
        variants.push(altered);
        let mut altered = good.clone();
        altered.chunks[0].end_byte -= 1;
        variants.push(altered);
        let mut altered = good.clone();
        altered.chunks[0].input_tokens = 513;
        variants.push(altered);
        // A byte inside the two-byte Turkish capital I is not a UTF-8 boundary.
        variants.push(two_chunks(&record.data.content, 1, 0));
        // Valid boundaries but a gap between chunks must also fail.
        variants.push(two_chunks(&record.data.content, 2, 3));
        for output in variants {
            assert!(
                store
                    .apply_index(
                        &reference(&record),
                        &source_digest(&record.data.content),
                        &output
                    )
                    .is_err()
            );
            assert_eq!(count(&store, "semantic_chunks"), 1);
            assert_eq!(store.index_coverage().unwrap().indexed_heads, 1);
        }
        assert!(matches!(
            store.apply_index(&reference(&record), &"0".repeat(64), &good),
            Err(MemoryError::Conflict)
        ));
        assert_eq!(count(&store, "semantic_chunks"), 1);
    }

    #[test]
    fn concurrent_revision_and_cross_record_retirement_reject_late_results() {
        let fixture = Fixture::new();
        let mut first = fixture.open();
        let mut second = fixture.open();
        let old = first
            .create(&operation(), input("old source"), None)
            .unwrap();
        publish(&mut first, &old);
        let selected = first.select_index_batch(None, 32, MAX_INDEX_BYTES).unwrap();
        assert!(selected.items.is_empty());
        let new = second
            .revise(&operation(), &old.record_id, 1, input("new source"))
            .unwrap();
        assert_eq!(count(&first, "semantic_chunks"), 0);
        assert!(matches!(
            first.apply_index(
                &reference(&old),
                &source_digest(&old.data.content),
                &passage(&old.data.content)
            ),
            Err(MemoryError::Conflict)
        ));
        publish(&mut first, &new);
        second
            .create(
                &operation(),
                input("superseding fact"),
                Some(reference(&new)),
            )
            .unwrap();
        assert_eq!(count(&first, "semantic_chunks"), 0);
        assert!(matches!(
            first.apply_index(
                &reference(&new),
                &source_digest(&new.data.content),
                &passage(&new.data.content)
            ),
            Err(MemoryError::Conflict)
        ));
    }

    #[test]
    fn forgetting_cascades_vectors_and_late_result_cannot_recreate_them() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        let record = store
            .create(&operation(), input("forgotten source"), None)
            .unwrap();
        publish(&mut store, &record);
        drop(store);
        crate::memory_lifecycle::forget(
            &fixture.0,
            Scope::Project("index-test".into()),
            &record.record_id,
            &operation(),
            1,
        )
        .unwrap();
        let mut reopened = fixture.open();
        assert_eq!(count(&reopened, "semantic_revisions"), 0);
        assert_eq!(count(&reopened, "semantic_chunks"), 0);
        assert!(matches!(
            reopened.apply_index(
                &reference(&record),
                &source_digest(&record.data.content),
                &passage(&record.data.content)
            ),
            Err(MemoryError::Conflict)
        ));
        assert_eq!(reopened.index_coverage().unwrap().current_heads, 0);
    }

    #[test]
    fn keyset_page_bounds_include_already_indexed_heads_and_byte_budget() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        let content = "x".repeat(MAX_CONTENT_BYTES);
        for _ in 0..3 {
            let record = store.create(&operation(), input(&content), None).unwrap();
            publish(&mut store, &record);
        }
        let last = store
            .create(&operation(), input("needs indexing"), None)
            .unwrap();
        let mut cursor = None;
        for _ in 0..3 {
            let batch = store
                .select_index_batch(cursor, 32, MAX_CONTENT_BYTES)
                .unwrap();
            assert!(batch.items.is_empty());
            assert_eq!(batch.scanned_heads, 1);
            assert_eq!(batch.scanned_bytes, MAX_CONTENT_BYTES);
            assert!(batch.has_more);
            assert!(batch.next_after_row_id > cursor);
            cursor = batch.next_after_row_id;
        }
        let batch = store
            .select_index_batch(cursor, 1, MAX_CONTENT_BYTES)
            .unwrap();
        assert_eq!(batch.items[0].reference, reference(&last));
        assert!(!batch.has_more);
        assert!(batch.next_after_row_id.is_none());
        assert!(store.select_index_batch(None, 33, MAX_INDEX_BYTES).is_err());
        assert!(
            store
                .select_index_batch(None, 1, MAX_INDEX_BYTES + 1)
                .is_err()
        );
        assert!(
            store
                .select_index_batch(Some(-1), 1, MAX_INDEX_BYTES)
                .is_err()
        );
    }

    #[test]
    fn corrupt_vectors_digest_spans_and_missing_chunks_are_cache_misses() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        let record = store
            .create(&operation(), input("source text"), None)
            .unwrap();
        publish(&mut store, &record);
        store
            .connection
            .execute(
                "UPDATE semantic_chunks SET vector=?1",
                [vec![0u8; VECTOR_BYTES]],
            )
            .unwrap();
        assert_eq!(
            store
                .select_index_batch(None, 32, MAX_INDEX_BYTES)
                .unwrap()
                .items
                .len(),
            1
        );
        publish(&mut store, &record);
        store
            .connection
            .execute(
                "UPDATE semantic_revisions SET source_sha256=?1",
                ["0".repeat(64)],
            )
            .unwrap();
        assert_eq!(
            store
                .select_index_batch(None, 32, MAX_INDEX_BYTES)
                .unwrap()
                .items
                .len(),
            1
        );
        publish(&mut store, &record);
        store
            .connection
            .execute("UPDATE semantic_chunks SET start_byte=1", [])
            .unwrap();
        assert_eq!(
            store
                .select_index_batch(None, 32, MAX_INDEX_BYTES)
                .unwrap()
                .items
                .len(),
            1
        );
        assert_eq!(store.index_coverage().unwrap().indexed_heads, 0);
        publish(&mut store, &record);
        store
            .connection
            .execute("DELETE FROM semantic_chunks", [])
            .unwrap();
        assert_eq!(
            store
                .select_index_batch(None, 32, MAX_INDEX_BYTES)
                .unwrap()
                .items
                .len(),
            1
        );
        assert_eq!(store.index_coverage().unwrap().indexed_heads, 0);
    }

    #[test]
    fn obsolete_model_is_discarded_and_canonical_content_change_invalidates_cache() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        let record = store
            .create(&operation(), input("source text"), None)
            .unwrap();
        publish(&mut store, &record);
        store
            .connection
            .execute(
                "UPDATE semantic_revisions SET model_identity='obsolete'",
                [],
            )
            .unwrap();
        assert_eq!(store.index_coverage().unwrap().indexed_heads, 0);
        let tx = store.connection.transaction().unwrap();
        initialize(&tx).unwrap();
        tx.commit().unwrap();
        assert_eq!(count(&store, "semantic_chunks"), 0);
        publish(&mut store, &record);
        store
            .connection
            .execute("UPDATE revisions SET content='altered source'", [])
            .unwrap();
        assert_eq!(count(&store, "semantic_revisions"), 0);
    }

    #[test]
    fn failed_chunk_insert_rolls_back_entire_replacement() {
        let fixture = Fixture::new();
        let mut store = fixture.open();
        let record = store
            .create(&operation(), input("source text"), None)
            .unwrap();
        publish(&mut store, &record);
        store
            .connection
            .execute_batch(
                "CREATE TRIGGER reject_second BEFORE INSERT ON semantic_chunks
            WHEN new.ordinal=1 BEGIN SELECT RAISE(ABORT,'injected failure'); END;",
            )
            .unwrap();
        assert!(
            store
                .apply_index(
                    &reference(&record),
                    &source_digest(&record.data.content),
                    &two_chunks(&record.data.content, 7, 6)
                )
                .is_err()
        );
        assert_eq!(count(&store, "semantic_chunks"), 1);
        assert!(
            store
                .select_index_batch(None, 32, MAX_INDEX_BYTES)
                .unwrap()
                .items
                .is_empty()
        );
    }

    #[test]
    fn existing_read_only_store_can_inspect_but_cannot_publish() {
        let fixture = Fixture::new();
        let mut writable = fixture.open();
        let record = writable
            .create(&operation(), input("read-only inspection"), None)
            .unwrap();
        drop(writable);
        let mut store =
            MemoryStore::open_existing(&fixture.0, Scope::Project("index-test".into())).unwrap();
        let changes = store.connection.total_changes();
        assert_eq!(
            store
                .select_index_batch(None, 32, MAX_INDEX_BYTES)
                .unwrap()
                .items
                .len(),
            1
        );
        assert_eq!(store.index_coverage().unwrap().missing_heads, 1);
        assert_eq!(store.connection.total_changes(), changes);
        assert!(
            store
                .apply_index(
                    &reference(&record),
                    &source_digest(&record.data.content),
                    &passage(&record.data.content)
                )
                .is_err()
        );
        assert_eq!(count(&store, "semantic_chunks"), 0);
    }
}
