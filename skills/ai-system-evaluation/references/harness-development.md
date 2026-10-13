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

## Role workflow fixtures and reports

In a Mirket source checkout, `evals/role-workflow/README.md` documents six
development cases, two separate held-out cases and the version 1 report. Use:

```sh
mirket dev role-eval suite --project . --suite evals/role-workflow/suite.json
mirket dev role-eval check --project . --suite evals/role-workflow/suite.json --report path/to/report.json
```

These read-only commands check fixed task, checker, rubric and configuration
identities, complete trial coverage and saved evidence hashes. The checker
recomputes scores from explicit criterion verdicts and reports infrastructure
failures separately from task failures. Unknown costs remain unknown. A host
agent or human must perform the work and an independent grader must substantiate
the verdicts. Label synthetic checker examples as synthetic, never as model runs.

Fix actual host/model/tools/settings, authority and trial budget before dispatch.
Give workers only task inputs; keep grader assertions outside their write scope.
The public held-out fixtures are split separately but are not intrinsically blind.
Use fresh unseen cases before making a blind evaluation claim. Retain every
planned trial, including failures and rejected candidates. Source-bound records
prove identity and consistency, not execution authenticity or semantic quality.
