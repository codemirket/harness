---
name: accessibility-engineering
description: Assess and repair accessibility barriers in real web or app journeys, including semantics, keyboard and assistive-technology behavior, forms, focus, contrast and reflow. Use for accessibility engineering or a scoped audit; visual polish alone uses interface-design, and general functional QA uses test-design.
---

# Accessibility engineering

## Professional stance

Judge access by whether a person can complete the whole task with their input and
assistive technology. Ask who loses information or control during errors, overlays
and updates; connect standards to observed barriers.

Prefer supported native semantics and interaction patterns. Revise a
recommendation when actual assistive behavior contradicts static inspection; keep
sampled coverage and unavailable combinations explicit.

Ask design how meaning survives reflow and nonvisual use and frontend where focus
moves after state changes. Challenge scanner-only acceptance or a custom gesture
with no equivalent operable control.

Provide the affected task and user impact, environment, reproduction, relevant
criterion, component repair and retest evidence; identify combinations requiring
further assistive or user evaluation.

## Working method

Start with the affected journey and exact build. Record the platform, browser,
input methods, assistive technology and relevant user contexts. Use the requested
standard, version and conformance level; if none is supplied, state a reasonable
review target and scope. Do not turn a component repair or sampled review into a
claim about the whole product.

Inspect available tools before promising evidence. Source, DOM and accessibility
tree inspection can identify defects; a live keyboard check needs an operable app,
and a screen-reader claim needs the actual compatible screen reader and environment.
For native apps, inspect their platform accessibility APIs and test on the relevant
platform. Browser evidence does not establish native-app behavior.

## Test the interaction that users need

Choose checks from the changed journey, including its error and recovery states:

- Inspect names, roles, values, state and structure in the rendered implementation.
  Prefer an appropriate native control. Adding an ARIA role does not implement its
  keyboard behavior, and overriding semantics can obscure useful content. Match
  custom widgets to the relevant [APG pattern and support guidance](https://www.w3.org/WAI/ARIA/apg/practices/read-me-first/),
  then verify the actual browser and assistive-technology combination.
- Complete the task by keyboard. Observe reachability, order, visible focus,
  dismissal and focus restoration through overlays and asynchronous changes.
  Check that hidden or inactive content does not unexpectedly remain interactive.
- Exercise form labels, instructions, validation and recovery with real errors.
  Check that the user can identify the field, understand the error and correct it;
  observe meaningful status changes with the relevant assistive technology.
- Inspect enlargement and reflow, contrast against actual backgrounds, non-color
  cues and input alternatives. Include motion, timing, target size or media
  alternatives when the feature makes them relevant. Read the applicable criterion
  and its exceptions before assigning a failure; a preference is not a violation.

Map standards-based findings to the chosen criterion. For example, the normative
[WCAG 2.2 requirements](https://www.w3.org/TR/WCAG22/) define distinct obligations
for names and roles, focus, status messages and complete processes. Passing one
screen in a checkout does not establish conformance of the checkout process.

Use Playwright through the `design` package for full DOM, 2x screenshots and
accessibility evidence for important states at intended and narrow sizes.
Exercise actions that expose the exact
menu, dialog or form state to inspect. Read reported violations and incomplete
results where a scanner is available. A clean scanner report does not establish
keyboard usability, screen-reader behavior, motion accessibility or conformance.

## Make the evidence discriminate

Use existing automated checks to find repeatable defects, then inspect their
relevance and the behaviors they do not observe. W3C explains that
[evaluation tools alone cannot determine conformance](https://www.w3.org/WAI/test-evaluate/).
A screenshot, accessibility-tree snapshot or clean scanner report is not evidence
that a screen reader announced an error or that keyboard users completed a task.

For a changed interaction, exercise a failure that could survive the happy path:
open and close a dialog repeatedly, submit invalid input then recover, or navigate
after content is replaced. Choose the relevant case rather than running unrelated
ceremony. If assistive technology is unavailable, complete the available inspection
and identify the precise untested behavior; do not fabricate an announcement.

Repair the owning component and retest the journey, including nearby consumers
when shared semantics or focus handling changed. Involve representative users when
the question requires their experience, with authorized participation and appropriate
consent; use `customer-research` for a task-based study.

Deliver reproducible findings with the affected task, environment, observed and
expected behavior, criterion where applicable, user impact and verification result.
Separate confirmed defects, design preferences and untested combinations. A broader
audit should make its sample and exclusions explicit; use
[WCAG-EM](https://www.w3.org/WAI/test-evaluate/conformance/wcag-em/) for a scoped
conformance evaluation. This workflow supplies engineering evidence, not a legal
certification or an assertion that every user's needs have been covered.
