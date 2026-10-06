# Technical documentation evidence and runnable examples

Read for a README, API guide, migration note, runbook, or ADR whose correctness
depends on code and configuration. For Word, PDF, slides, or spreadsheet layout,
use the relevant artifact workflow instead. Preserve the repository's owning
documents, generator, templates, and review conventions.

## Trace a material claim to what consumers use

Start from the public entry point, installed version, and affected caller. An
internal helper does not establish the exported API; a dependency range does not
establish the resolved version; a sample configuration does not establish the
deployment setting. Use code, tests, schemas, configuration, and version-matched
official documentation for the claims they can actually support.

For a changed lookup API, a compact working ledger might be:

| Documentation claim | Owning evidence to inspect | Required check |
| --- | --- | --- |
| Missing records raise `NotFound` | Public lookup path and exception type | Exercise a missing record through that entry point. |
| Existing records keep the same return shape | Schema/type and actual result | Compare a representative result including null/zero fields. |
| Network failure is a separate error | Connector translation and caller path | Use a controlled transport failure; do not imply every exception means missing. |
| Users can migrate their old `None` check | Existing caller and final example | Run both found and missing examples with the resolved package. |

Names above are illustrative; substitute the actual project interface. Record
unknown behavior as a gap to resolve, not a plausible statement to publish.
If docs and code disagree, determine the intended contract from project owners
and consumers before deciding which one to change.

## Make examples executable as published

Use the reader's documented working directory, environment, entry point, and
fixtures. State required configuration without exposing secrets. Run the exact
final command or code block where practical; checking a different scratch script
does not prove the README example runs. Include imports, arguments, meaningful
output, and error recovery when they are needed to use the interface.

For the illustrative lookup above, a migration example should catch the specific
documented exception while allowing transport/authentication errors to remain
visible:

```python
from inventory import NotFound, lookup

try:
    item = lookup("sample-item")
except NotFound:
    print("Item not found")
else:
    print(item["name"])
```

Verify against the actual public package with a known item and a missing item.
Check output, exit status, and any side effects; import/syntax checks alone cannot
prove behavior. Do not wrap the example in a broad `except Exception` just to make
it appear successful. If the project exposes an async interface, its final example
must use the supported event-loop/context pattern rather than this sync sketch.

Use an existing test/example harness or safe local fixtures. A documented remote
write still needs authorization and a suitable environment to execute. When a
required account, runtime, or service is unavailable, verify what is possible
and state which example remains unexecuted; do not label it tested. Do not add
dependencies or silently connect a service merely to remove that qualification.

## Preserve the reason for a decision

An ADR explains the decision, relevant constraints, credible alternatives actually
considered, consequences, and conditions for revisiting it. A README explains how
to use the result. For the lookup change, the ADR may record why callers need to
distinguish missing inventory from unavailable inventory, while the migration
guide carries the runnable catch example. Do not invent historical alternatives,
benchmarks, approvals, or a rationale absent from the evidence.

Follow the existing ADR numbering and supersession policy. Update or link the
owning API/setup reference rather than copying its full contents into the ADR.
Check links, anchors, generated documentation, and example consumers affected by
the change. A narrative diff can be small while its setup command reaches many
readers, so select verification from that consumer impact.

## Completion evidence

Compare the final documentation with the intended behavior and inspect it in
the repository's normal reading/build surface when relevant. Run affected doc
build/link checks and examples through the existing toolchain. State what was
verified against which version/environment and which meaningful gaps remain.
An accurate draft still needs the requested save/update; a saved file still needs
content and example checks. Do not claim publication or service behavior without
separate evidence for those steps.
