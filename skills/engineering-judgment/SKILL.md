---
name: engineering-judgment
description: Use for coding changes that require judgment about interfaces, behavior, debugging, tests, or review. Helps choose useful engineering techniques in an existing project. Simple mechanical edits need no additional workflow.
---

# Engineering judgment

Use the sections relevant to the decision at hand. Follow the project's
architecture, tests, and vocabulary. Scale the work to the consequences of
being wrong; this skill adds no required approval stages or agent roles.

## Understand the behavior

Identify the entry point, the caller's expectation, and the observable result.
Trace the affected path through its actual implementation and configuration.
An exported symbol or configured dependency does not establish that it runs.
For state changes, find the write and the conditions that permit it.

Distinguish a requirement from an implementation choice. When a requirement
is missing, use established behavior and nearby consumers to narrow the gap.
Ask only when the remaining choice changes the outcome materially.

## Choose an interface

An interface includes inputs, outputs, errors, ordering, configuration, and
invariants callers must understand. Prefer one that hides useful complexity
and keeps changes local to the implementation.

Before adding an abstraction, name the complexity it will contain and its
present callers. Imagine removing it: does difficult behavior spread across
callers, or does a pass-through simply disappear? Use that result as evidence,
not a formula. Preserve useful boundaries for security, ownership, or change.

When alternatives matter, compare a small number against real caller examples,
compatibility, failure handling, and migration cost. Introduce new variation
points when a current need justifies them.

## Diagnose a failure

Capture the actual symptom and find the smallest available feedback loop
that distinguishes the failure from success. Reproduce when practical; if
access or intermittency prevents it, use traces and name the remaining gap.

Trace suspect values and control flow back to their origin. Compare a working
case or recent known-good revision. Form a falsifiable explanation, then make
a small experiment that distinguishes it from credible alternatives.
For performance, establish a measured baseline before changing the code.
Remove temporary instrumentation after it has answered the question.

## Choose tests that matter

Name the realistic failure each test catches. Check observable behavior at
the narrowest boundary that still exposes that failure. Derive expected values
from requirements, hand-checked examples, or independent fixtures.

Keep the relevant code real; substitute slow, external, or nondeterministic
operations where necessary. Assert calls or ordering when those are part of
the contract. A regression check should fail on the known broken behavior.
Use a test-first cycle when it clarifies the change or proves a regression.
Preserve meaningful existing coverage when changing module boundaries.

## Review the result

Check both the requested behavior and the project's constraints. Trace likely
failure cases and affected consumers; distinguish a defect from a style
preference. For a finding, give a concrete trigger, location, and consequence.
Resolve conflicting feedback against evidence and established requirements.
Report what the checks establish and what remains uncertain.
