//! Explicit facts: lifecycle publication and retrieval eligibility share one transaction owner.
use crate::util;
use anyhow::{Context, Result, ensure};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::Path};

const APPLICATION_ID: i64 = 0x4d464143;
const DIMENSIONS: usize = 384;
const AUTHORITY: &str = "retrieved_data_not_instructions_or_verification";

pub struct FactStore {
    connection: Connection,
    scope: String,
    model: String,
}

fn digest(value: &Value) -> Result<String> {
    Ok(util::sha256(&serde_json::to_vec(value)?))
}

fn vector_bytes(vector: &[f32]) -> Result<Vec<u8>> {
    ensure!(vector.len() == DIMENSIONS, "embedding dimension mismatch");
    ensure!(vector.iter().all(|x| x.is_finite()), "non-finite embedding");
    let norm = vector
        .iter()
        .map(|x| f64::from(*x).powi(2))
        .sum::<f64>()
        .sqrt();
    ensure!(norm > 0.0, "zero embedding");
    Ok(vector
        .iter()
        .flat_map(|x| ((f64::from(*x) / norm) as f32).to_le_bytes())
        .collect())
}

fn presented(payload: &str) -> Result<Value> {
    let mut value: Value = serde_json::from_str(payload)?;
    value["sha256"] = json!(digest(
        &json!({"memory":value["memory"],"metadata":value["metadata"]})
    )?);
    value["authority"] = json!(AUTHORITY);
    Ok(value)
}

// Closed function-word filter prevents lexical OR matches on "the/is" from
// manufacturing evidence for an otherwise unanswerable query. This is not an
// English NLP pipeline: Unicode terms and identifiers remain verbatim for FTS5.
fn lexical_term(term: &str) -> bool {
    !term.is_empty()
        && !matches!(
            term.to_lowercase().as_str(),
            "a" | "an"
                | "the"
                | "is"
                | "are"
                | "was"
                | "were"
                | "be"
                | "been"
                | "being"
                | "what"
                | "which"
                | "who"
                | "whom"
                | "whose"
                | "where"
                | "when"
                | "why"
                | "how"
                | "do"
                | "does"
                | "did"
                | "of"
                | "to"
                | "in"
                | "on"
                | "at"
                | "for"
                | "from"
                | "with"
                | "and"
                | "or"
                | "not"
                | "this"
                | "that"
                | "it"
                | "its"
                | "my"
                | "our"
                | "your"
                | "hangi"
                | "bir"
                | "bu"
                | "ve"
                | "ile"
                | "için"
                | "ne"
                | "nedir"
                | "nasıl"
                | "mi"
                | "mı"
                | "mu"
                | "mü"
                | "olan"
        )
}

impl FactStore {
    pub fn open(path: &Path, scope: &str, model: &str) -> Result<Self> {
        ensure!(
            scope.len() == 64 && scope.bytes().all(|b| b.is_ascii_hexdigit()),
            "invalid project scope"
        );
        ensure!(
            !model.is_empty() && model.len() <= 256,
            "invalid embedding identity"
        );
        let connection = crate::agent_runtime::database::open(path, APPLICATION_ID, |tx| {
            tx.execute_batch("CREATE TABLE identity(scope TEXT PRIMARY KEY, model TEXT NOT NULL, revision INTEGER NOT NULL DEFAULT 0);
                CREATE TABLE facts(id TEXT PRIMARY KEY, payload TEXT NOT NULL, text TEXT NOT NULL,
                    expires INTEGER, superseded INTEGER NOT NULL DEFAULT 0 CHECK(superseded IN(0,1)),
                    vector BLOB NOT NULL);
                CREATE TABLE requests(key TEXT PRIMARY KEY, fingerprint TEXT NOT NULL,
                    fact_id TEXT NOT NULL REFERENCES facts(id) ON DELETE CASCADE);
                CREATE TABLE supersessions(old_id TEXT PRIMARY KEY, new_id TEXT NOT NULL);
                CREATE TABLE forgotten(id TEXT PRIMARY KEY);
                CREATE VIRTUAL TABLE fact_text USING fts5(id UNINDEXED, text, tokenize='unicode61');
                CREATE TRIGGER fact_insert AFTER INSERT ON facts BEGIN
                    INSERT INTO fact_text(id,text) VALUES(new.id,new.text); END;
                CREATE TRIGGER fact_delete AFTER DELETE ON facts BEGIN
                    DELETE FROM fact_text WHERE id=old.id; END;")?;
            tx.execute(
                "INSERT INTO identity(scope,model) VALUES(?1,?2)",
                params![scope, model],
            )?;
            Ok(())
        })?;
        let identity_rows: i64 =
            connection.query_row("SELECT count(*) FROM identity", [], |r| r.get(0))?;
        ensure!(
            identity_rows == 1,
            "fact store identity cardinality invalid"
        );
        let identity: (String, String) =
            connection.query_row("SELECT scope,model FROM identity", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?;
        ensure!(
            identity == (scope.to_owned(), model.to_owned()),
            "fact store project/model identity mismatch"
        );
        Ok(Self {
            connection,
            scope: scope.to_owned(),
            model: model.to_owned(),
        })
    }

    /// Full same-format validation for backup/restore and explicit doctor, never the hot path.
    pub(crate) fn validate_backup(db: &Connection, scope: &str) -> Result<()> {
        let identity_rows: i64 = db.query_row("SELECT count(*) FROM identity", [], |r| r.get(0))?;
        ensure!(
            identity_rows == 1,
            "fact store identity cardinality invalid"
        );
        let stored_scope: String = db.query_row("SELECT scope FROM identity", [], |r| r.get(0))?;
        ensure!(stored_scope == scope, "fact store scope identity mismatch");
        let mut statement=db.prepare("SELECT f.id,f.payload,f.text,f.expires,f.vector,r.key,r.fingerprint FROM facts f LEFT JOIN requests r ON r.fact_id=f.id ORDER BY f.id")?;
        let mut rows = statement.query([])?;
        while let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let payload: String = row.get(1)?;
            let text: String = row.get(2)?;
            ensure!(payload.len() <= 16 * 1024, "oversized saved fact");
            let value: Value = serde_json::from_str(&payload)?;
            let key: String = row.get(5)?;
            let fingerprint: String = row.get(6)?;
            let hash = util::sha256(format!("{scope}:{key}:{fingerprint}").as_bytes());
            let expected = format!(
                "{}-{}-{}-{}-{}",
                &hash[..8],
                &hash[8..12],
                &hash[12..16],
                &hash[16..20],
                &hash[20..32]
            );
            ensure!(
                id == expected
                    && value["id"] == id
                    && value["memory"] == text
                    && !text.trim().is_empty()
                    && text.len() <= 2048,
                "fact identity/content mismatch"
            );
            let metadata = &value["metadata"];
            ensure!(
                metadata["request_key"] == key
                    && metadata["request_sha256"] == fingerprint
                    && fingerprint.len() == 64
                    && fingerprint.bytes().all(|b| b.is_ascii_hexdigit()),
                "fact retry identity mismatch"
            );
            ensure!(
                metadata["provenance"]
                    .as_str()
                    .is_some_and(|p| !p.is_empty() && p.len() <= 1024)
                    && matches!(
                        metadata["kind"].as_str(),
                        Some("fact" | "decision" | "preference" | "constraint")
                    ),
                "fact provenance/kind invalid"
            );
            let expires: Option<u64> = row.get(3)?;
            ensure!(
                metadata["expires_at"] == json!(expires),
                "fact expiry mismatch"
            );
            ensure!(
                value["created_at"].as_u64().is_some() && value["updated_at"].as_u64().is_some(),
                "fact timestamp invalid"
            );
            let vector: Vec<u8> = row.get(4)?;
            ensure!(
                vector.len() == DIMENSIONS * 4,
                "saved embedding dimension mismatch"
            );
            let norm = vector
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| f64::from(f32::from_le_bytes(*b)).powi(2))
                .sum::<f64>();
            ensure!(
                (0.99..=1.01).contains(&norm),
                "saved embedding normalization invalid"
            );
        }
        for query in [
            "SELECT EXISTS(SELECT 1 FROM forgotten JOIN facts USING(id))",
            "SELECT EXISTS(SELECT 1 FROM facts f LEFT JOIN fact_text t ON f.id=t.id WHERE t.id IS NULL OR t.text!=f.text)",
            "SELECT EXISTS(SELECT 1 FROM fact_text t LEFT JOIN facts f ON f.id=t.id WHERE f.id IS NULL)",
            "SELECT EXISTS(SELECT 1 FROM facts GROUP BY id HAVING count(*)!=1)",
            "SELECT EXISTS(SELECT 1 FROM requests GROUP BY fact_id HAVING count(*)!=1)",
            "SELECT EXISTS(SELECT 1 FROM fact_text GROUP BY id HAVING count(*)!=1)",
            "SELECT EXISTS(SELECT 1 FROM supersessions s JOIN facts f ON s.old_id=f.id WHERE f.superseded!=1)",
            "SELECT EXISTS(SELECT 1 FROM facts f WHERE f.superseded=1 AND NOT EXISTS(SELECT 1 FROM supersessions s WHERE s.old_id=f.id))",
        ] {
            let invalid: bool = db.query_row(query, [], |r| r.get(0))?;
            ensure!(!invalid, "fact lifecycle/index binding invalid");
        }
        Ok(())
    }

    /// Embedding runs before the write transaction; publication is all-or-nothing.
    pub fn retry(&self, args: &Value) -> Result<Option<Value>> {
        let key = args["key"].as_str().context("request key required")?;
        let prior:Option<(String,String)>=self.connection.query_row("SELECT r.fingerprint,f.payload FROM requests r JOIN facts f ON f.id=r.fact_id WHERE r.key=?1",[key],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        prior
            .map(|(fingerprint, payload)| {
                ensure!(
                    fingerprint == digest(args)?,
                    "memory idempotency key was used for different input"
                );
                Ok(json!({"memory":presented(&payload)?,"reused":true}))
            })
            .transpose()
    }

    pub fn put(&mut self, args: &Value, vector: &[f32], now: u64) -> Result<Value> {
        let text = args["text"].as_str().context("fact text required")?.trim();
        let key = args["key"].as_str().context("request key required")?;
        let provenance = args["provenance"].as_str().context("provenance required")?;
        ensure!(
            !text.is_empty() && text.len() <= 2048,
            "fact requires 1..2048 bytes"
        );
        ensure!(
            !key.is_empty() && key.len() <= 128,
            "key requires 1..128 bytes"
        );
        ensure!(
            !provenance.is_empty() && provenance.len() <= 1024,
            "provenance requires 1..1024 bytes"
        );
        ensure!(
            matches!(
                args["kind"].as_str(),
                Some("fact" | "decision" | "preference" | "constraint")
            ),
            "invalid fact kind"
        );
        let fingerprint = digest(args)?;
        let bytes = vector_bytes(vector)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let prior: Option<(String,String)> = tx.query_row("SELECT r.fingerprint,f.payload FROM requests r JOIN facts f ON f.id=r.fact_id WHERE r.key=?1", [key], |r| Ok((r.get(0)?,r.get(1)?))).optional()?;
        if let Some((previous, payload)) = prior {
            ensure!(
                previous == fingerprint,
                "memory idempotency key was used for different input"
            );
            return Ok(json!({"memory":presented(&payload)?,"reused":true}));
        }
        let expiry = if args["expires_at"].is_null() {
            None
        } else {
            Some(args["expires_at"].as_u64().context("invalid expiry")?)
        };
        ensure!(
            expiry.is_none_or(|x| x > now && x <= i64::MAX as u64),
            "expiry must be a future timestamp"
        );
        let replacement = !args["supersedes"].is_null();
        if replacement {
            let old_id = args["supersedes"]["id"]
                .as_str()
                .context("superseded ID required")?;
            let old: Option<String> = tx.query_row("SELECT payload FROM facts WHERE id=?1 AND superseded=0 AND (expires IS NULL OR expires>?2)", params![old_id,now], |r|r.get(0)).optional()?;
            let old = presented(&old.context("superseded fact changed or is inactive")?)?;
            ensure!(
                old["sha256"] == args["supersedes"]["sha256"],
                "superseded fact changed"
            );
        }
        let mut metadata = json!({"kind":args["kind"],"provenance":provenance,"request_key":key,"request_sha256":fingerprint,"expires_at":expiry,"assurance":"host_supplied_fact"});
        if replacement {
            metadata["supersedes"] = args["supersedes"].clone();
        }
        // Stable within this project/request. IDs carry no authorization.
        let hash = util::sha256(format!("{}:{key}:{fingerprint}", self.scope).as_bytes());
        let id = format!(
            "{}-{}-{}-{}-{}",
            &hash[..8],
            &hash[8..12],
            &hash[12..16],
            &hash[16..20],
            &hash[20..32]
        );
        let forgotten: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM forgotten WHERE id=?1)",
            [&id],
            |r| r.get(0),
        )?;
        ensure!(
            !forgotten,
            "forgotten request cannot be replayed; use a new key for an explicit new fact"
        );
        let payload =
            json!({"id":id,"memory":text,"metadata":metadata,"created_at":now,"updated_at":now});
        tx.execute(
            "INSERT INTO facts(id,payload,text,expires,vector) VALUES(?1,?2,?3,?4,?5)",
            params![id, serde_json::to_string(&payload)?, text, expiry, bytes],
        )?;
        tx.execute(
            "INSERT INTO requests VALUES(?1,?2,?3)",
            params![key, fingerprint, id],
        )?;
        if replacement {
            let old = args["supersedes"]["id"].as_str().unwrap();
            tx.execute("UPDATE facts SET superseded=1 WHERE id=?1", [old])?;
            tx.execute("INSERT INTO supersessions VALUES(?1,?2)", params![old, id])?;
        }
        tx.execute("UPDATE identity SET revision=revision+1", [])?;
        tx.commit()?;
        Ok(json!({"memory":presented(&serde_json::to_string(&payload)?)?,"reused":false}))
    }

    pub fn read(
        &self,
        id: Option<&str>,
        cursor: Option<&str>,
        limit: usize,
        now: u64,
    ) -> Result<Value> {
        ensure!((1..=20).contains(&limit), "memory limit must be 1..20");
        let mut statement = self.connection.prepare("SELECT id,payload FROM facts WHERE superseded=0 AND (expires IS NULL OR expires>?1) AND (?2 IS NULL OR id=?2) AND (?3 IS NULL OR id>?3) ORDER BY id LIMIT ?4")?;
        let rows = statement
            .query_map(params![now, id, cursor, limit + 1], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let more = rows.len() > limit;
        let results = rows
            .iter()
            .take(limit)
            .map(|(_, payload)| presented(payload))
            .collect::<Result<Vec<_>>>()?;
        Ok(
            json!({"results":results,"scope":self.scope,"engine":"sqlite","limit":limit,"next_cursor":if more {rows.get(limit-1).map(|r|r.0.clone())}else{None},"authority":AUTHORITY}),
        )
    }

    pub fn forget(&mut self, id: &str, expected: &str) -> Result<Value> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let payload: Option<String> = tx
            .query_row("SELECT payload FROM facts WHERE id=?1", [id], |r| r.get(0))
            .optional()?;
        if let Some(payload) = payload {
            ensure!(
                presented(&payload)?["sha256"] == expected,
                "memory changed; inspect before deletion"
            );
            tx.execute("DELETE FROM facts WHERE id=?1", [id])?;
            tx.execute("INSERT OR IGNORE INTO forgotten VALUES(?1)", [id])?;
            tx.execute("UPDATE identity SET revision=revision+1", [])?;
        }
        // Supersession edges contain identities only; retain old inactive state even if the successor is forgotten.
        tx.commit()?;
        Ok(
            json!({"deleted":true,"id":id,"retention":"Fact content and retry input removed. An ID-only tombstone prevents replay. Existing run traces and backups remain; not forensic erasure."}),
        )
    }

    pub fn revision(&self) -> Result<u64> {
        Ok(self
            .connection
            .query_row("SELECT revision FROM identity", [], |r| r.get(0))?)
    }
    pub fn count(&self) -> Result<u64> {
        Ok(self
            .connection
            .query_row("SELECT count(*) FROM facts", [], |r| r.get(0))?)
    }
    pub fn all_after(&self, after: &str, limit: usize) -> Result<Vec<Value>> {
        ensure!((1..=100).contains(&limit), "invalid fact export limit");
        let mut stmt = self
            .connection
            .prepare("SELECT payload,superseded FROM facts WHERE id>?1 ORDER BY id LIMIT ?2")?;
        let rows = stmt.query_map(params![after, limit], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, bool>(1)?))
        })?;
        rows.map(|r| {
            let (payload, superseded) = r?;
            let mut value = presented(&payload)?;
            value["superseded"] = json!(superseded);
            Ok(value)
        })
        .collect()
    }
    pub fn get_any(&self, id: &str) -> Result<Option<Value>> {
        self.connection
            .query_row("SELECT payload FROM facts WHERE id=?1", [id], |r| {
                r.get::<_, String>(0)
            })
            .optional()?
            .map(|p| presented(&p))
            .transpose()
    }

    pub fn search(
        &mut self,
        query: &str,
        vector: Option<&[f32]>,
        limit: usize,
        now: u64,
    ) -> Result<Value> {
        ensure!((1..=20).contains(&limit), "memory limit must be 1..20");
        ensure!(
            !query.trim().is_empty() && query.len() <= 512,
            "invalid query: query must be 1..512 bytes"
        );
        let tokens: Vec<_> = query
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| lexical_term(s))
            .map(|s| format!("\"{s}\""))
            .collect();
        ensure!(
            tokens.len() <= 128,
            "invalid query: at most 128 lexical terms"
        );
        let started = std::time::Instant::now();
        let query_vector = vector.map(vector_bytes).transpose()?;
        let tx = self.connection.transaction()?;
        let mut ranks: BTreeMap<String, f64> = BTreeMap::new();
        if !tokens.is_empty() {
            let mut stmt = tx.prepare("SELECT f.id FROM fact_text JOIN facts f ON f.id=fact_text.id WHERE fact_text MATCH ?1 AND f.superseded=0 AND (f.expires IS NULL OR f.expires>?2) ORDER BY bm25(fact_text),f.id LIMIT 100")?;
            let ids = stmt
                .query_map(params![tokens.join(" OR "), now], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?;
            for (rank, id) in ids.into_iter().enumerate() {
                *ranks.entry(id).or_default() += 1.0 / (61 + rank) as f64;
            }
        }
        let lexical_ms = started.elapsed().as_secs_f64() * 1000.;
        let scoring_started = std::time::Instant::now();
        let mut examined = 0;
        if let Some(query) = query_vector {
            let q: Vec<f64> = query
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| f64::from(f32::from_le_bytes(*b)))
                .collect();
            let mut stmt = tx.prepare("SELECT id,vector FROM facts WHERE superseded=0 AND (expires IS NULL OR expires>?1)")?;
            let mut rows = stmt.query([now])?;
            let mut best: Vec<(String, f64)> = Vec::new();
            while let Some(row) = rows.next()? {
                let id: String = row.get(0)?;
                let bytes: Vec<u8> = row.get(1)?;
                ensure!(bytes.len() == DIMENSIONS * 4, "corrupt embedding size");
                let (score, norm) = bytes.as_chunks::<4>().0.iter().zip(&q).fold(
                    (0., 0.),
                    |(score, norm), (b, q)| {
                        let v = f64::from(f32::from_le_bytes(*b));
                        (score + v * q, norm + v * v)
                    },
                );
                ensure!(
                    score.is_finite() && (0.99..=1.01).contains(&norm),
                    "corrupt embedding values or normalization"
                );
                examined += 1;
                if score >= 0.25 {
                    let position = best.partition_point(|(prior_id, prior_score)| {
                        prior_score.total_cmp(&score).is_gt()
                            || (prior_score.total_cmp(&score).is_eq() && prior_id < &id)
                    });
                    if position < 100 {
                        best.insert(position, (id, score));
                        best.truncate(100);
                    }
                }
            }
            for (rank, (id, _)) in best.into_iter().enumerate() {
                *ranks.entry(id).or_default() += 1.0 / (61 + rank) as f64;
            }
        }
        let scoring_ms = scoring_started.elapsed().as_secs_f64() * 1000.;
        let mut ranked: Vec<_> = ranks.into_iter().collect();
        ranked.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        let mut results = Vec::new();
        for (id, score) in ranked.into_iter().take(limit) {
            let payload: String =
                tx.query_row("SELECT payload FROM facts WHERE id=?1", [id], |r| r.get(0))?;
            let mut item = presented(&payload)?;
            item["score"] = json!(score);
            results.push(item);
        }
        tx.commit()?;
        Ok(
            json!({"results":results,"scope":self.scope,"engine":"sqlite","limit":limit,"retrieval":{"mode":if vector.is_some(){"hybrid"}else{"lexical"},"model":self.model,"eligible_vectors_examined":examined,"supported_active_envelope":10000,"stress_envelope":examined>10000,"incomplete":false},"timing_ms":{"lexical":lexical_ms,"dense_scoring":scoring_ms,"total":started.elapsed().as_secs_f64()*1000.},"authority":AUTHORITY}),
        )
    }
}

#[cfg(test)]
mod disk_full_tests {
    use super::*;

    fn clamp_max_page_count(store: &FactStore) {
        let pages: i64 = store
            .connection
            .pragma_query_value(None, "page_count", |row| row.get(0))
            .unwrap();
        store
            .connection
            .pragma_update(None, "max_page_count", pages)
            .unwrap();
    }

    fn restore_max_page_count(store: &FactStore) {
        store
            .connection
            .pragma_update(None, "max_page_count", 131072_i64)
            .unwrap();
    }

    #[test]
    fn sqlite_page_limit_rolls_back_failed_fact_without_losing_acknowledged_data() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().canonicalize().unwrap().join("facts.sqlite");
        let scope = "d".repeat(64);
        let mut store = FactStore::open(&path, &scope, "disk-full-fixture").unwrap();
        let old = json!({"key":"acknowledged","text":"acknowledged fact survives disk pressure","kind":"fact","provenance":"disk-full fixture","expires_at":null});
        let old_item = store.put(&old, &vec![1.; 384], 100).unwrap()["memory"].clone();
        // A small page ceiling forces a real SQLite FULL transaction failure
        // while the existing fact remains readable after reopening.
        store
            .connection
            .pragma_update(None, "max_page_count", 24)
            .unwrap();
        let mut failed = false;
        for index in 0..128 {
            let args = json!({"key":format!("pressure-{index}"),"text":"x".repeat(2048),"kind":"fact","provenance":"disk-full fixture","expires_at":null});
            if store
                .put(
                    &args,
                    &vec![0.; 383]
                        .into_iter()
                        .chain(std::iter::once(1.))
                        .collect::<Vec<_>>(),
                    100,
                )
                .is_err()
            {
                failed = true;
                break;
            }
        }
        assert!(failed, "fixture must reach the SQLite page limit");
        drop(store);
        let mut reopened = FactStore::open(&path, &scope, "disk-full-fixture").unwrap();
        let results = reopened.search("acknowledged", None, 5, 101).unwrap()["results"].clone();
        assert!(
            results
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["id"] == old_item["id"])
        );
    }

    #[test]
    fn backup_validation_rejects_superseded_fact_without_edge() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().canonicalize().unwrap().join("facts.sqlite");
        let scope = "e".repeat(64);
        let mut store = FactStore::open(&path, &scope, "orphan-fixture").unwrap();
        let args = json!({"key":"orphan","text":"orphan superseded fact","kind":"fact","provenance":"orphan fixture","expires_at":null});
        let item = store.put(&args, &vec![1.; 384], 100).unwrap()["memory"].clone();
        store
            .connection
            .execute(
                "UPDATE facts SET superseded=1 WHERE id=?1",
                [item["id"].as_str().unwrap()],
            )
            .unwrap();
        assert!(FactStore::validate_backup(&store.connection, &scope).is_err());
    }

    #[test]
    fn sqlite_full_fact_write_preserves_acknowledged_fact_and_retries() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().canonicalize().unwrap().join("facts.sqlite");
        let scope = "f".repeat(64);
        let mut store = FactStore::open(&path, &scope, "disk-full-retry").unwrap();
        let old = json!({"key":"full-old","text":"acknowledged before SQLite FULL","kind":"fact","provenance":"disk-full fixture","expires_at":null});
        let old_item = store.put(&old, &vec![1.; 384], 100).unwrap()["memory"].clone();
        let before_revision = store.revision().unwrap();
        let failed = json!({"key":"full-new","text":format!("SQLite FULL retry payload {}", "x".repeat(1900)),"kind":"fact","provenance":"disk-full fixture","expires_at":null});
        clamp_max_page_count(&store);
        assert!(
            store
                .put(
                    &failed,
                    &vec![0.; 383]
                        .into_iter()
                        .chain(std::iter::once(1.))
                        .collect::<Vec<_>>(),
                    101
                )
                .is_err()
        );
        assert_eq!(store.revision().unwrap(), before_revision);
        assert!(
            store
                .get_any(old_item["id"].as_str().unwrap())
                .unwrap()
                .is_some()
        );
        restore_max_page_count(&store);
        assert_eq!(
            store
                .put(
                    &failed,
                    &vec![0.; 383]
                        .into_iter()
                        .chain(std::iter::once(1.))
                        .collect::<Vec<_>>(),
                    101
                )
                .unwrap()["reused"],
            false
        );
        assert_eq!(store.revision().unwrap(), before_revision + 1);
    }

    #[test]
    fn sqlite_full_supersession_keeps_old_fact_and_retries_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().canonicalize().unwrap().join("facts.sqlite");
        let scope = "b".repeat(64);
        let mut store = FactStore::open(&path, &scope, "disk-full-supersession").unwrap();
        let old = json!({"key":"full-old","text":"old value remains visible","kind":"fact","provenance":"disk-full fixture","expires_at":null});
        let old_item = store.put(&old, &vec![1.; 384], 100).unwrap()["memory"].clone();
        let before_revision = store.revision().unwrap();
        let mut replacement = json!({"key":"full-replacement","text":format!("replacement SQLite FULL payload {}", "y".repeat(1900)),"kind":"fact","provenance":"disk-full fixture","expires_at":null});
        replacement["supersedes"] = json!({"id":old_item["id"],"sha256":old_item["sha256"]});
        let replacement_vector = [0.; 383]
            .into_iter()
            .chain(std::iter::once(1.))
            .collect::<Vec<_>>();
        clamp_max_page_count(&store);
        assert!(store.put(&replacement, &replacement_vector, 101).is_err());
        assert_eq!(store.revision().unwrap(), before_revision);
        assert!(
            store
                .get_any(old_item["id"].as_str().unwrap())
                .unwrap()
                .is_some()
        );
        assert_eq!(
            store.read(None, None, 5, 101).unwrap()["results"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        restore_max_page_count(&store);
        assert_eq!(
            store.put(&replacement, &replacement_vector, 101).unwrap()["reused"],
            false
        );
        assert_eq!(store.revision().unwrap(), before_revision + 1);
        assert!(
            store
                .read(Some(old_item["id"].as_str().unwrap()), None, 1, 101)
                .unwrap()["results"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            store.read(None, None, 5, 101).unwrap()["results"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
    }
}
