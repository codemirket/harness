# Evaluate guidance through delivered work

Registration proves availability; it cannot establish that an agent chose useful
advice, applied it correctly or produced a better result. Use
[feedback controls](feedback-controls.md) to connect an observed failure to
focused guidance and an independently useful check.

## Define a discriminating task

Use realistic input, an observable deliverable, a failure probe and fixed
acceptance criteria. Record the exact source, model, settings, tool access and
budget actually used. Keep task inputs and evaluation rules outside the worker's
write scope. Do not change fixtures or thresholds to make an output pass.

Include implicit selection, explicitly requested guidance, irrelevant negative
cases and multi-role composition where appropriate. A worker naming a skill is
not proof that it read its content. Observe actual skill retrieval and examine
where the resulting artifact follows or violates that guidance.

## Capture the work

Register an isolated project through `mirket project register`, start a task for
the intended capability and read its selected guidance. Use MCP tools or
`mirket task` to checkpoint meaningful decisions, attach current acceptance and
failure-probe artifacts, and record completion when the actual checks pass.
Keep the CLI's state checks separate from the application's own verification.

The task store validates registered roots, selected skill delivery, revisions and
artifact hashes. It does not execute arbitrary checks, attest reviewer identity,
prove model usage or certify a semantic result. A caller's summary remains an
attestation. Never label an agent review as human acceptance.

## Judge the actual result

Execute the project's relevant checks under the host's permissions. For visual
work inspect actual renders and interactions; for research trace claims to read
sources; for data reconcile units, grain and known totals. Include failures,
recovery and an adversarial case that could expose a convincing fake.

Keep correctness, craft review and user acceptance distinct. Compare conditions
with the same inputs, model, tools and budgets; repeat representative trials
before claiming general quality or efficiency gains. A single passing repair
shows that case worked, not that the harness is universally better.
