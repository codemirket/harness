# Turkish booking interface translation QA

Locale: `tr-TR`. Audience: professional travel operations. Deliverable: `tr.json`, containing the same six keys, in the same order, as `en.json`.

## Terminology and register

- **Refund → para iadesi**: distinguishes repayment from cancellation of the booking.
- **Supplier → tedarikçi** and **booking → rezervasyon**: consistent travel operations terminology.
- **Request cancellation → İptal talebinde bulunun**: a formal request action; it does not claim that cancellation is complete.
- **Pending → beklemede** and **confirmed / not confirmed → onaylanmış / onaylanmış değil**: retain the distinction between a pending booking and a confirmed booking. The delivered preview uses the natural equivalent “henüz onaylanmadı.”
- **Seats available → koltuk müsait**: Turkish uses an uninflected noun after a number, so both numbered ICU branches use “# koltuk müsait.”

The imperatives “iptal etmeyin,” “onaylayın” and “bulunun” use a consistent professional register. Runtime booking identifiers and names remain untranslated.

## Meaning review

| Key | Source meaning retained |
| --- | --- |
| `refund` | The prohibition remains in force until the supplier confirms the refund. “İptal etmeyin” is a prohibition, not advice. The second sentence explicitly says that making a request does not mean the refund has been confirmed. |
| `price` | The total remains estimated, at EUR 1,250.50, with local taxes excluded. Turkish number presentation is `1.250,50 €`; this is formatting, not currency conversion. |
| `deadline` | Confirmation is required by 17:00 on 10 October 2026. The date is localized to “10 Ekim 2026”; the 24-hour time and literal `Europe/Istanbul` identifier are preserved. “Kadar” retains the deadline. |
| `seats` | Zero availability remains zero; the numbered branches state the available seat count. The source's `=0`, `one` and `other` branches are retained as required by the application contract. |
| `button` | The user requests cancellation; the label does not state that the booking will immediately be cancelled. |
| `preview` | The booking remains pending and explicitly unconfirmed. “Henüz onaylanmadı” does not promise a later confirmation. The name stays inside the original strong markup. |

A model-based comparison of each target message with its source and a separate natural-language read-through were performed. No material source ambiguity was identified within this supplied fixture.

## Checks actually performed

A local Python standard-library check read back the saved `tr.json` and passed:

- Valid UTF-8 JSON, no duplicate keys, exactly the source's six keys and order, with string values.
- Per-message comparison of simple placeholders: `{booking_id}` and `{name}` retain their identities and message assignments.
- The `<strong>` and `</strong>` tag sequence is unchanged, and `<strong>{name}</strong>` is intact.
- The literal `Europe/Istanbul` identifier occurs in the same message and with the same count as in the source.
- A fixture-specific ICU shape check confirmed balanced structure, `count`, `plural`, and the ordered `=0` / `one` / `other` branches. The zero branch retains zero `#` markers and each numbered branch retains one.
- Explicit substitution checks exercised all three retained branches: `=0` with 0 produced “Müsait koltuk yok”; `one` with 1 produced “1 koltuk müsait”; `other` with 2 produced “2 koltuk müsait”. These checks did not select branches through the application's ICU runtime.
- Decimal parsing confirmed that the source and localized amount both equal EUR `1250.50`. The date wording and `17:00` deadline were checked against the source.
- A before/after byte comparison confirmed that `en.json` was unchanged. Its SHA-256 is `5ba07c8da6fe537ed935ead1773026047287b9b6a559480bfa0061c78da2ae82`.

## Remaining limits

The supplied exercise contains no consuming application, full ICU parser, or UI. The fixture-specific parser and explicit branch substitution are structural checks, not evidence of production ICU dispatch or rendering. Fractional counts, localized runtime number substitution and fallback behavior were not exercised in an application.

No live UI, wrapping, truncation, font, accessibility, screen-reader or interaction review occurred. In particular, the longer cancellation label must be checked in its actual control. No native human, translator or travel-domain expert review occurred. No supplier was contacted and no booking was changed.
