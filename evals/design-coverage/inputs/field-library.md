# Larch Field Library — reservation desk

Build a polished, self-contained `index.html` for a fictional equipment lending
desk. A volunteer needs to choose a suitable field sampling kit, understand what
is due and reserve one kit. This is a local prototype: no network services,
downloads, build dependencies or real reservations. Use original visual choices
and local/system fonts. The working screen is the primary deliverable.

Use these exact fictional facts:

| ID | Name | Daily fee | Weight | All-weather | Available |
| --- | --- | ---: | ---: | --- | ---: |
| moss | Moss pocket kit | €18 | 6 kg | No | 1 |
| reed | Reed field kit | €26 | 9 kg | Yes | 2 |
| stone | Stone survey kit | €41 | 15 kg | Yes | 0 |

One kit per reservation. Duration is a whole number from 1 to 3 days, initially
2. The refundable deposit is €40 per reservation, separate from the hire fee;
due now is the fee plus the deposit. Initially select Reed. Show the important
comparison facts before commitment. Show unavailable equipment truthfully.

An `All-weather only` filter must affect the actual options. If it hides the
selected kit, clear the selection and summary and require a new selection.
Changing duration must update the selected kit's summary. Invalid duration must
block reservation with an understandable error.

The first reservation attempt deliberately fails before a save: communicate that
failure, retain the user's choices and offer Retry. Retrying succeeds locally.
The failed attempt must not create a reservation; repeating the successful action
must not create another one. Make the simulated/local nature clear to the user.

Use native, named, keyboard-operable controls, visible focus and meaningful status
announcements. The comparison and summary must remain readable at 390 px width
without hiding required facts. Respect reduced motion if motion is used.

For reliable inspection, expose the following public production contract. These
hooks supplement accessible semantics and are not product copy:

- Kit controls: `input[name="bundle"][value="moss|reed|stone"]`.
- `#weather-only`: checkbox; `#days`: numeric duration input.
- `#summary`: `data-fee-eur`, `data-deposit-eur`, `data-due-eur` containing decimal
  numbers when selected and empty strings when no kit is selected.
- `#reserve` and `#retry`: reservation and retry buttons; `#status`: visible
  status; `#reservation-count`: an output containing the actual saved count.
- `#summary` also displays the fee, refundable deposit and total in readable copy.

Also save `decision.md` (at most 700 words): explain the content hierarchy and the
relationship transferred from any reference actually read, what changed for this
brief, the states you exercised and any verification gaps. If a reference does
not cover this task, state that limitation and use ordinary design judgment.
Do not put teaching notes or reference names into the product interface.
