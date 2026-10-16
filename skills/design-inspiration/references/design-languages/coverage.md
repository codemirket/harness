# Ask what is represented

A reference library cannot cover every interface. This optional lookup describes
its inventory by product family, target platform, interactions and constraints;
it does not delimit the model's sources, methods or possible designs. Exact IDs
and their meanings are in [coverage.json](coverage.json). A browser analogy is useful
evidence for a relationship; native behavior still requires native work.

Use the existing catalog search with explicit requirements:

```sh
mirket catalog search --family commerce --platform web --interaction cart-checkout --constraint keyboard --json
mirket catalog search --family collaboration --platform native-mobile --interaction offline-sync --constraint rtl --json
```

The same MCP request is:

```json
{"query":"","design_context":{"family":"collaboration","platform":"native-mobile","interactions":["offline-sync"],"constraints":["rtl"]}}
```

Call `catalog_search` with this object. Omitted axes are unspecified; they are
not silently satisfied. Values are exact registry IDs. Unknown or repeated IDs
are rejected so a typo cannot become a confident recommendation. Ordinary
keyword search remains useful for source names and conceptual relationships.

Read the top-level `design_coverage`, which is independent of the keyword page:

- `represented`: at least one scoped **worked example** contains the complete
  requested combination. Read its limits; this establishes inventory coverage.
- `partial`: relevant cells exist, but each has an explicit missing requirement
  or source guidance without a worked example on the requested platform.
- `unrepresented`: no cell falls within the requested family/platform boundary.

Each candidate supplies the exact lesson, source record and example routes,
scope, limits and its own missing axes. The lookup never combines one lesson's
platform with another lesson's access constraint to claim full coverage.
`combination_represented` is explicit; `current_task_validated` is always false.
Automated markup checks do not establish a screen-reader journey. Directional
markup does not establish language quality or cultural appropriateness.

Partial or absent inventory is not a refusal, a quality verdict or a requirement
to adopt another lesson. Existing product knowledge, current primary guidance and
other references remain available. Investigate or build a working slice when an
actual uncertainty warrants it, and verify meaningful failure and recovery on the
target platform. Keep material evidence gaps visible. Do not substitute a web
rendering for native/device evidence.

The registry includes deliberately unrepresented platforms and constraints:
kiosk, TV/remote input, wearable, spatial and embedded devices; actual offline
sync/concurrent editing; actual screen-reader, forced-colors, touch and constrained
network journeys; human language/locale acceptance, privacy, safety-critical and
long-running operational behavior. These need appropriate runtime and specialist
evidence. A family match does not imply industry-specific domain coverage.

The library's original good/misleading examples teach relationships. Evaluate
transfer with new content and fixed task criteria: see
`evals/design-coverage/README.md` in a Mirket source checkout. Preserve every arm
and failure, keep grader assertions outside worker ownership, and distinguish
correct behavior, visual craft and human acceptance. A small pilot does not
measure universal competence or justify a general quality-gain claim.
