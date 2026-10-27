# Browser session contract and qualification

The Rust MCP owner exposes `browser_open`, `browser_navigate`,
`browser_inspect`, `browser_article`, `browser_click`, `browser_fill`, `browser_scroll`,
`browser_resize`, `browser_close` and the bounded capture
operations verified in the [installed browser qualification](m3-browser-setup-acquisition.md).
Chromium is the sole private browser. `browser_open` accepts the release ID,
optional `visible` flag and optional `profile_name`; it has no engine selector.
An obsolete `engine` field is rejected, and the private adapter explicitly
rejects `--engine` before launch. Development qualification requires both
`--browser-adapter` and `--browser-chromium` with explicit existing absolute
file paths. Installed operation has no system-browser lookup, downloader,
public CDP port or arbitrary page-evaluation operation.

A connection owns each session and its installation-wide heavy slot. Its
private Node child owns one Playwright persistent Chromium context and its
browser descendants. The supervisor verifies exact release and process paths
before launch, limits the installation to two heavy browser sessions, defaults
to a disposable profile and headless mode, and reaps its process group before
releasing the profile and slot. Named profiles live at
`browser-profiles/<name>` and are locked across clients. Their marker binds
the exact release; opening one under a different release is refused. No
personal browser state is imported and no previous selector-based profile
format is migrated implicitly.

The idle window is 15 minutes, or a renewed 60-minute window for visible
interaction. A request has a 60-second outer deadline; the adapter uses a
shorter internal deadline. A cancelled launch finishes in its owner task and
disposes a late session. A cancelled in-flight request retires its session
rather than accepting a stale response on a later call. The private ready
handshake binds protocol version, release ID, frame limit and page limit;
there is no browser selector in the launch, session, profile or capture
transport. Capture intents retain exact connection/session/page/revision,
project and destination bindings and issuance checks.

An adapter error that reports a possible page effect, stage output or terminal
adapter failure retires the session and reaps its owned child before a later
call can assume page state. A definite, nonterminal pre-effect refusal leaves
the session usable. The owner reports uncertain effects and never retries an
interaction automatically. The adapter's terminal error is carried by its
structured reply; a zero child exit after that reply means context cleanup
succeeded, while a close failure remains nonzero and retains the ownership
marker.

Interactions use finite inputs: a unique CSS target for click and fill,
UTF-8 fill values of at most 4 KiB without NUL, nonzero scroll deltas of at
most 10,000 pixels per axis, and viewport dimensions of 1–4,096 pixels with
at most 16 million pixels in total. Fill replies omit the entered value.
Each interaction requires the current session revision and a stale revision
refuses before dispatch. Successful interactions advance the revision;
captures must use the current one. This binds operations to Mirket's observed
call order, not to unrelated asynchronous DOM changes, so inspect changing
pages again before selecting an element. The browser adapter does not accept
caller-supplied scripts.

`browser_article` accepts exactly the same release/session/optional page identity
as inspection. It observes one live main document and returns its `source_url`,
`status`, explicit nullable `article` and `truncated`; it advances the serialized
session revision like other observations. An article contains `title`, `text`,
`excerpt`, `byline`, `published_time`, `site_name`, `lang` and `dir`. Missing
metadata is explicitly null. Status is `article`, `no_article` or `dom_limit`;
the latter returns a null article and `truncated: true`, without presenting
partial DOM contents as a complete article. Article text is plain text, and
the private IPC refuses extra fields, HTML/content payloads, changed page
identity, inconsistent status and oversized text/metadata. Text remains
untrusted source material and can itself contain angle brackets or instructions.

The exact-source `@mozilla/readability` 0.6.0 pair runs in a Chromium isolated
world. A fixed preflight admits at most 10,000 nodes, 20,000 attributes,
1,000,000 UTF-16 code units across all CharacterData (including XML CDATA and
processing instructions), element and attribute names, their namespace URIs,
doctype and attribute values, and depth 256. It has a 500 ms preflight clock
budget; Chromium's
evaluation deadline is 5 seconds, within the existing 15-second adapter and
60-second owner deadlines. On an extraction execution failure the owner retires
the session to stop remaining work. These are enforced limits, not measured
latency promises. Subframe documents, template contents and shadow trees are not
traversed or copied. Reconstructing the admitted main tree has a separate
1,000 ms total elapsed cap from the start of preflight.

Readability and its readerability heuristic receive a detached document rebuilt
from the admitted main tree, because parsing mutates its input. Native deep and
shallow element clones are avoided: they can copy hidden clonable shadow roots
or invoke page custom-element constructors. HTML custom elements become inert
generic wrappers, retaining their child text. No script is inserted into the live document
or its globals, and the serializer produces text rather than HTML. A failed
readerability heuristic, null parse, or fewer than 500 trimmed text characters
returns `no_article`. This heuristic can miss short or unusual articles and
can classify some long nonarticle pages as articles. It does not establish
source truth, publication-date validity, authorship or completeness.

The complete JSON string representation of text is capped at 20,000 bytes,
title at 1,024, excerpt at 2,048, byline/site name at 512 each, publication time
at 256, language at 128 and direction at 32. The source URL has a 4,096-byte
budget. Code-point clipping preserves valid Unicode; isolated UTF-16 surrogates
become replacement characters. Any clipped field sets `truncated: true`, so a
clipped source URL also cannot be mistaken for a complete provenance URL.
The result stays under the existing 65,536-byte private frame limit even with
escaped controls and maximal protocol identifiers. No jsdom, network article
fetcher, caller-supplied scripts, selector or extraction-options bag is used.

The exact npm archive, locked integrity, unchanged source, license and package
file inventory are recorded in
[the Readability receipt](../packaging/admission/m3-browser-readability.runtime.json).
Release assembly must include `browser-article.js` beside the two existing
browser adapter modules, its exact package files and the retained Apache license
and attribution. Article extraction reads the package only from that adapter's
local `node_modules/@mozilla/readability` in an installed release (or the
compiled adapter's sibling `node_modules` during development); missing local
files refuse rather than resolving an ancestor host package. This npm
qualification does not change Chromium admission.

On 2026-10-25, macOS 26.5.2 ARM64 source checks passed 18 Rust browser tests
and two development CLI tests, including removed-selector refusal, exact
profile markers, exclusive locks, release binding, cancellation, malformed
capture refusal, saved-output verification and cleanup. The focused adapter
suite compiled and passed all 15 tests with private Node 24.22.1 and the
explicit development Chromium candidate, including native navigation, bounded
inspection, a unique CSS click, persisted page state and a 1,280 × 720 PNG stage.
The opt-in Rust native round trip also passed. The public stdio capture smoke
verified the exact `browser_open` schema, refusal of the removed selector,
revision-bound publication, lost-start-response recovery, reconnect receipt
inspection and temporary-profile cleanup. All nine observed owned process
identities were gone after graceful EOF. The saved 21,642-byte PNG was hashed,
decoded by the publication path and visually inspected for the clicked fixture
and Turkish text. These observations used the explicitly supplied candidate;
they did not change release admission.

A disposable Mac release with the setup-acquired browser passed the
[installed browser qualification](m3-browser-setup-acquisition.md). The older
development-candidate tests alone did not establish that route. Windows native
execution, a visible user journey and actual Codex Desktop activation remain
unverified.
