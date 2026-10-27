# M0 activation review

Reviewed on 2026-10-24 against the local `release_store.rs`, `main.rs`,
distribution verification and maintenance/session consumers. This review covers
the four-file local qualification payload. It does not require the later full
tool distribution. No Cargo build, executable replacement, installer, activation
or power-failure experiment was run during this review; the owner was running
release integration fixtures concurrently.

## Findings and disposition

### P2 — Bound both retained releases and interrupted staging

At the reviewed `ReleaseStore::stage` implementation
(`src/release_store.rs:129`), each distinct release is copied into another
`releases/r-<hash>` directory. Activation updates the previous-release pointer
but does not remove or count older payloads (`src/release_store.rs:273`). A
failed copy also leaves a staging directory; the existence check applies only
to the next requested identity (`src/release_store.rs:150`). Staging another
identity can therefore accumulate further abandoned directories. A one-candidate
JSON pointer does not bound the actual retained bytes.

**Consequence:** repeated successful updates or interrupted copies consume disk
without a release-store bound. The per-copy free-space check eventually refuses
work but is not a retention policy.

**Minimal correction:** validate/count the finite managed inventory before a
new copy, preserve unknown or drifted entries, and bound retained state to the
active release, its predecessor and one candidate. Under exclusive maintenance,
post-commit cleanup may remove only verified retired releases. Keep the staged
record until cleanup completes so explicit retry is idempotent. Interrupted
staging must either block new growth or have an explicit, evidence-preserving
recovery action.

**Discriminating checks:** activate A, B, C and D and verify that retained
payloads remain bounded; interrupt a copy before its staged record exists and
then request a different release; inject an unknown or modified retired entry
and verify it is preserved and blocks unsafe reclamation. Kill activation after
the active commit but before cleanup and verify explicit retry preserves active
and predecessor identities.

**Status:** the owner accepted this finding and is implementing bounded
inventory admission plus maintenance-held verified cleanup. That implementation
was pending when this report was saved.

### P1 — Persist payload directory entries before the activation commit

The initial reviewed staging implementation synced copied file data before
applying executable permissions, and synced only the `releases` and `staging`
parent directories after the final rename. It did not flush the newly created
payload directories themselves. A power loss could therefore leave a durable
active record referring to a payload whose nested entries or executable mode
had not completed the intended persistence sequence.

**Correction observed in current source:** permission updates now precede
`output.sync_all()` (`src/release_store.rs:184–201`), and all declared parent
directories, including the staging root, are synced before publication
(`src/release_store.rs:210–229`). Reverse path order places descendants before
their own parents for this finite inventory.

**Remaining evidence:** add a discriminating ordering/fault-injection check for
file sync failure after mode changes, nested directory sync failure, and the
rename/activation boundary. Such tests can establish failure handling and
ordering; actual power-loss durability remains native qualification work.

**Status:** source-level correction verified during review. No power-failure
claim is made.

### P2 — Partial recovery records have preservation but no product recovery route

`write_activation` resumes only an exactly matching `.next` record
(`src/release_store.rs:326`). A crash during the write can leave a partial
record; explicit retry refuses because those bytes differ. Similarly, an
interrupted staging directory is preserved but cannot be resumed by the current
stage operation. The current CLI exposes initialization, staging and activation,
but no recovery inspection/resolution operation (`src/main.rs:24–27`).

Failing closed is correct. The missing part is a bounded supported route for
resolving the preserved state; deleting fixture residues in a test does not
establish that route.

**Minimal correction:** expose inspection of the active/staged/pending record
identities and verified candidate state. An explicit recovery action may finish
an exact matching intent or quarantine a confirmed task-owned partial record
before retrying the selected operation. It must preserve differing valid
records and unknown staging contents, never infer activation from mere presence,
and never run on ordinary server startup.

**Discriminating checks:** persist a prefix of `active.json.next`, reconnect and
verify the prior active release still serves, then exercise the explicit
recovery path. Repeat for a complete matching record, a differing valid record,
`staged.json.next`, and a partial staging directory. Check that unrelated bytes
remain unchanged in every refusal case.

**Status:** recovery limitation identified; exact-match retry already exists.
Broader recovery must remain reported as incomplete until its actual route is
implemented and exercised.

## Boundaries checked without new findings

- An unrecognized nonempty home is refused before managed files are written.
- Source copying uses the frozen verified manifest, bounded reads and a second
  verification of the saved candidate; changing the source cannot silently
  change the staged release identity.
- `open_active` takes its shared session guard before reading the active pointer.
  Activation holds exclusive maintenance across the commit, and worker sessions
  and unresolved ownership markers participate in that exclusion.
- Installed serving checks the current executable's on-disk bytes against the
  active manifest before constructing the engine. This is
  meaningful under the immutable-release assumption; it is not proof of loaded
  image identity if another actor replaces the executable pathname concurrently.
- Free-space checking is an actual observation before copying. It cannot reserve
  space against unrelated disk consumers; copy/sync errors preserve the old
  active pointer. The retention finding above is separate from that limitation.
- Ordinary reconnect reads the committed active pointer; it does not implicitly
  activate a candidate. Successful replay of a matching `.next` requires an
  explicit activation request.

The subsequent changed-core macOS fixture passes; see [current activation
evidence](m0-release-qualification.md). Windows behavior and actual power-loss
recovery remain unqualified.
