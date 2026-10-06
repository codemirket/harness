# Application composition

Use for a working application surface: shell/navigation, collection, form,
detail/edit screen or a combination. Start with the user's next decision, the
real data and the project's established components. The examples below describe
illustrative choices, not product facts or requirements to add features.

## Frame the screen around a job

Write a short composition intent before a substantial change: what the user is
trying to do, which information enables it, what should be noticed first, and
what can stay secondary. For example: "Compare unsettled items, open the unusual
one, then resolve it; amount, state and owner should be scannable before notes."
This is a design decision, not a new approval stage or mandatory artifact.

| Surface | First question to resolve | Useful arrangement | Tradeoff to inspect |
| --- | --- | --- | --- |
| Shell | Where am I, and where can I go? | Stable navigation, current section, page identity, then local actions | Chrome should not consume the working area or duplicate the page toolbar |
| Collection | Which record needs attention? | Relevant filters, a legible comparison surface, contextual row actions | Cards help distinct entities; tables help comparing repeated fields |
| Detail | What is this, what is its state, what can I do? | Identity and state, important facts, related context, actions | A wall of equally weighted facts hides the decision |
| Form | What must I supply, and what happens on submission? | Meaningful groups in task order, labels/helper text, validation, clear submit action | Dense alignment can help comparison; narrow fields can harm entry |
| Overview | What changed or needs action? | A few useful summaries followed by underlying work | An extra metric or chart earns space only if it changes a decision |

Keep a stable destination or page title in the content even when the shell already
names the section. Place page-wide actions near that identity; place local actions
with the affected content. Avoid repeating the same command in several equally
prominent toolbars unless the long workflow justifies it.

## Build hierarchy through relationships

Use a small number of distinct roles: page identity, section heading, field or
column label, primary value, supporting explanation. Map these onto existing
tokens rather than assigning a different size and weight to each widget.

- Group related content with proximity first. A border, tinted surface or card is
  useful when it marks a separate task, object or interactive boundary. A card
  around every paragraph can make equal content look unrelated.
- Make the main action easy to find at the moment it is useful. A primary fill,
  stronger text or a clear position can establish emphasis. Destructive actions
  need accurate labels and the project's established treatment; they need not
  compete visually with the everyday action.
- Use muted text for secondary explanation, not for information needed to decide.
  Check actual contrast and selected/disabled states rather than trusting the
  token name.
- Treat status colors as semantic roles. Pair an important state with a readable
  label or icon. Do not repurpose a warning hue as arbitrary decoration.
- Align labels, value starts and action edges where users compare them. Optical
  alignment of an icon with its label can differ from the geometric box center;
  judge the rendered pair.

Density follows frequency and device. An expert's repeated comparison task may
need compact rows; a first-time setup may need more explanation and breathing
room. Increase spacing between different groups more than within one group. Use
the existing spacing scale and keep control heights coherent within the same row.
Do not shrink critical text or touch targets simply to fit more records.

For a new system without useful tokens, an initial desktop data treatment might
use 14–16px values, about 1.3–1.5 line height and 8–12px vertical cell padding;
entry-heavy surfaces may need a roomier treatment. Derive row height from the
rendered text and controls, not a fixed target. Prefer the existing product's
scale when available, and tune against real labels, target sizes and zoom.

## Typography for reading and comparison

Keep the product's font stack unless changing it is in scope. Tune its roles before
introducing another family: size, weight, line height, measure and contrast can
provide a useful hierarchy within one family.

- Compare the longest realistic label and a short one in the actual container.
  Heading wrapping should preserve meaning; do not insert line breaks to repair
  only one fixture. A narrower heading measure may clarify a section, while a
  dense toolbar may need a shorter accurate label.
- Use tabular numerals when aligned amounts or counters need stable comparison.
  Right-align numeric columns where magnitude is the comparison task; align text
  columns to the reading direction. Include units/currency and preserve locale.
- Keep identifiers recognizable and recoverable. A compact truncated identifier
  can work when a detail view or copy action exposes the full value. A hidden
  overflow rule by itself does not provide that access.
- Let explanatory paragraphs have a comfortable measure rather than filling an
  ultra-wide screen. Roughly 55–75 characters is a useful initial range for prose,
  not a restriction on tables, code, translations or the user's chosen layout.
- Use font weights actually available in the project. Check fallback/loading
  behavior before concluding that the type hierarchy looks correct.

## Forms: support accurate entry

Order fields by the user's reasoning and dependencies. Group related inputs under
an informative heading; separate unrelated decisions rather than distributing
fields into equal cards for symmetry.

- Give each field a persistent accessible label. Put format examples and short
  explanations where they are needed; do not use a placeholder as the only label.
- Match width to the expected input. A long entity name needs room; a short code
  does not need the full page. Adjacent fields belong on one row when the user
  thinks about them together and they remain usable at narrow widths.
- Keep input geometry steady across focus, validation and async states. Use an
  outline or reserved icon slot when a changing border/icon would move text.
- Validation should follow the form's actual contract. Describe what is wrong and
  how to recover. Preserve entered values after a rejected submission and move or
  announce attention through the project's accessible form behavior.
- Place the submit action after the information needed to understand its effect.
  A sticky action area can help a long form if it does not cover fields, errors,
  focused controls or mobile keyboard content. Reuse the application's pattern.
- A multi-step flow earns its complexity when grouping genuinely reduces effort
  or later steps depend on earlier choices. Do not split a short form to create a
  more impressive-looking interface.

## Tables and collections: support comparison

Select columns from the task. Put identity, relevant state and decisive values in
the initial scan; place supporting metadata later or in an existing detail view.
Preserve information the workflow actually requires.

Headers should describe the values and make sort state clear when sorting exists.
Use row separators, restrained striping or whitespace to help track across a row;
test the actual density before adding all three. Hover styling can aid tracking
but must not be the sole way to reveal a required row action.

Filters should explain their scope and current effect. Keep query state and result
count consistent with the real response. For bulk actions, show the selection
scope and affected count accurately; selecting visible rows must not imply every
matching record is selected unless that behavior is implemented.

At narrow widths, choose deliberately:

- Keep a horizontally scrollable table when cross-column comparison matters.
  Make the scrolling region discoverable and usable by keyboard/touch.
- Move secondary columns into details when the data contract and task permit it.
  Do not silently hide the value needed to distinguish two records.
- Use stacked records when users inspect one entity at a time. Preserve field
  labels and meaningful order; changing table cells to blocks is not enough.

Keep semantic table/header relationships when the surface is a table. Respect
the existing grid's navigation model if it is an interactive data grid. Appearance
alone is not a reason to replace a mature accessible primitive.

## Detail, edit and shell continuity

Anchor a detail view with identity, meaningful state and the next valid actions.
Group facts into task-relevant sections; allow long narrative content a different
measure from key/value comparisons. An activity/history area should distinguish
events from current facts rather than becoming a second competing summary.

For edit flows, preserve the user's location and entered work through the real
navigation/save contract. Make view mode, edit mode and unsaved state legible where
they exist. A new visual panel does not justify inventing autosave or optimistic
behavior; coordinate such behavior with frontend/API engineering.

Keep shell landmarks and familiar action positions stable across sibling screens.
Use a shared typography/color vocabulary while letting a reading-heavy detail
screen be less dense than its comparison table. Consistency means shared meaning,
not forcing every screen into identical card counts and column ratios.

## Responsive behavior and real states

Choose breakpoints where content stops working in its actual container, including
sidebars and split panes. Let controls wrap or reorganize in a meaningful order;
check focus/reading order after visual reordering. Test text enlargement, translated
labels, empty results, missing media, long names and the supported themes.

Design only states the feature supports, but make their differences clear:

| Actual condition | Useful treatment |
| --- | --- |
| First load | Keep page identity and layout anchors stable; indicate the region waiting for data |
| Refresh with usable data | Preserve the usable content and indicate refresh where relevant |
| No records yet | Explain what belongs here and the real available next action |
| No filter matches | Explain the current filter effect and offer a real way to adjust it |
| Failed request | Keep recoverable context, identify the failed region and expose supported retry |
| Partial/stale data | Label the limitation and its consequences; do not imply a confirmed total |
| Pending write | Give local feedback and honor duplicate-submission/unknown-outcome rules |

## Worked composition decisions

**Operations table.** A screen has large summary cards, several equal toolbar
buttons and a narrow amount column. Users mainly compare records and open an
exception. Keep only summaries that change that decision; group filters above the
results, make identity/state/amount easy to scan, and keep the row's detail action
available without hover. Preserve bulk selection semantics. Inspect the same
records before/after, including a long name, negative amount and no filter matches.

**Supplier edit form.** The existing form mixes contact, payment and address
inputs in one wide grid. Group by those concepts in the user's entry order; give
long names and addresses sufficient width, align small related fields, and keep
the save action after the groups. Reuse existing labels, validators and components.
Check a server-rejected save and a narrow viewport with the keyboard open; a clean
default screenshot alone does not establish a better edit experience.

**Record detail.** Equal cards make a reference number, a state and a paragraph of
notes look equally important. Put identity and state together, use a compact
facts group for comparable values, and give notes a reading measure. Keep the
existing valid actions adjacent to the relevant decision. Verify that secondary
data remains accessible and that the return path preserves the collection context.

An illustrative wide-screen arrangement is identity/state/actions above a main
facts column and a quieter contextual column. At a narrow width, keep identity
first, actions near the decision, then facts and context in meaningful reading
order. This is preferable to retaining two squeezed columns solely because the
desktop screenshot used two.
