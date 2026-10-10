# Translation QA: en.json → tr.json (tr-TR)

Scope: the six strings in `en.json` for a professional travel operations interface.
The fixture is fictional. No supplier was contacted and no booking was changed.
`en.json` was not modified (SHA-256 before and after:
`5ba07c8da6fe537ed935ead1773026047287b9b6a559480bfa0061c78da2ae82`).

## Terminology

| English | Turkish | Rationale |
|---|---|---|
| booking | rezervasyon | Standard term in Turkish travel operations; used the same way in `refund` and `preview`. |
| supplier | tedarikçi | Standard B2B travel term for the hotel or service provider. |
| refund | iade | Standard term for a returned payment. |
| request (noun / verb) | talep / talep etmek | Used in both `refund` and `button`, so the UI makes clear that a request is a separate step from a result. |
| confirm / confirmed | onaylamak / onaylanmış | One verb for every "confirm" (supplier confirms the refund, user confirms by the deadline, booking not confirmed). I did not mix it with *teyit* or *konfirme*. |
| cancel / cancellation | iptal etmek / iptal | Standard. |
| pending | beklemede | Standard status label. |
| available (seats) | müsait | The usual availability/inventory word in Turkish tourism. *Boş koltuk* would also work. |
| estimated total | tahmini toplam | Keeps the "estimate" qualification. |

The register is formal throughout: *-iniz/-unuz* possessives and the *-in/-ın* imperative (*iptal etmeyin*, *onaylayın*).

## Per-key meaning checks

The back-translations are literal, to show what the Turkish actually says.

| Key | Turkish | Literal back-translation | Checks |
|---|---|---|---|
| `refund` | Tedarikçi iadeyi onaylayana kadar {booking_id} numaralı rezervasyonu iptal etmeyin. Talep, onaylanmış bir iade değildir. | Until the supplier confirms the refund, do not cancel booking number {booking_id}. A request is not a confirmed refund. | It is still a prohibition (*etmeyin*). The condition is still the supplier's confirmation of the refund. The second sentence keeps the request/confirmed distinction. *Talep* stays generic, just like "A request" in the source, so it doesn't narrow the meaning to "cancellation request" or "refund request". |
| `price` | Tahmini toplam: €1.250,50. Bu tutar yerel vergileri içermez. | Estimated total: €1,250.50. This amount does not include local taxes. | Same currency (EUR) and same value (1250.50), written in Turkish number format. Both the "estimated" and "excludes local taxes" qualifications are kept. I used *içermez* rather than *dahil değildir*, which avoids the *dahil/dâhil* spelling split. |
| `deadline` | En geç 10 Ekim 2026 saat 17:00'de (Europe/Istanbul) onaylayın. | Confirm at 17:00 (Europe/Istanbul) on 10 October 2026 at the latest. | Same instant: 2026-10-10 17:00 in Europe/Istanbul, which is 14:00 UTC. "By" becomes *en geç* ("at the latest"), so it is still a deadline and not an appointment. The order is date then time, as is normal in Turkish. The timezone literal is unchanged. The suffix *'de* follows the spoken form *on yedide*. |
| `seats` | {count, plural, =0 {Müsait koltuk yok} one {# koltuk müsait} other {# koltuk müsait}} | =0: No available seats / one: # seat(s) available / other: # seat(s) available | The variable `count`, the selectors `=0`, `one` and `other`, the ordering and the spacing are all unchanged. `#` appears once in `one` and once in `other`, and the `=0` branch doesn't use it, matching the source. The `one` and `other` texts are identical because Turkish nouns stay singular after a numeral (*2 koltuk*, not *2 koltuklar*). Both branches are kept because the app requires this schema, and `tr` really does have both `one` and `other` categories in CLDR. |
| `button` | İptal talep et | Request cancellation | It is still a request, not an immediate action. The tempting *İptal et* ("Cancel") would contradict the refund warning. Sentence case matches the source. |
| `preview` | <strong>{name}</strong>, rezervasyonunuz beklemede, onaylanmış değil. | {name}, your booking is pending, not confirmed. | `<strong>` still wraps exactly `{name}`. I used *onaylanmış değil* ("is not confirmed"), a statement of state. I avoided *onaylanmadı*, which can read as "was not approved", i.e. rejected. I also avoided *henüz* ("yet"), which would add an expectation the source doesn't state. |

## Locale presentation

I took the values from Node.js's own ICU data (Node v24.19.0, ICU 78.3, CLDR 48.0), not from memory:

- `Intl.NumberFormat('tr-TR', {style:'currency', currency:'EUR'}).format(1250.50)` → `€1.250,50`
- `Intl.DateTimeFormat('tr-TR', {dateStyle:'long', timeZone:'Europe/Istanbul'})` → `10 Ekim 2026`
- `Intl.DateTimeFormat('tr-TR', {timeStyle:'short', timeZone:'Europe/Istanbul'})` → `17:00`
- `Intl.PluralRules('tr-TR').resolvedOptions().pluralCategories` → `one, other`
- `Intl.NumberFormat('tr-TR').format(1000)` → `1.000` (this is what `#` produces for large counts)

## Preservation and verification performed

I ran a one-off Node.js checker against the saved `tr.json` and the unchanged `en.json`. It wasn't saved, so nothing extra was added to the directory. It reported **no failures**. It checked:

1. The keys are the same as `en.json`, in the same order. The file is valid JSON, UTF-8 without BOM, with a trailing newline like the source.
2. The `{…}` placeholders in each key match the source exactly (`{booking_id}`, `{name}`).
3. The HTML tags in each key match the source, and `<strong>` still wraps `{name}`.
4. `seats` parses as an ICU plural on `count` with the exact skeleton `{count, plural, =0 {…} one {…} other {…}}`, and `#` is used in the right branches.
5. The date, time and EUR amount strings equal the ICU-generated `tr-TR` values above. The English `1,250.50` is gone, and `(Europe/Istanbul)` is present verbatim.
6. These semantic markers are present: the estimate qualifier, the tax exclusion, the deadline wording, the negative imperative, the supplier-confirmation condition, the request≠refund sentence, request wording on the button, and pending/not-confirmed in the preview (with no "confirmed" or "not approved" wording).
7. No common English words are left, apart from the timezone literal.

The checker also rendered `seats` with a small local ICU-plural renderer that uses `Intl.PluralRules('tr-TR')`:

| count | tr-TR | en |
|---|---|---|
| 0 | Müsait koltuk yok | No seats available |
| 1 | 1 koltuk müsait | 1 seat available |
| 2 | 2 koltuk müsait | 2 seats available |
| 1000 | 1.000 koltuk müsait | 1,000 seats available |

**Failure probes:** I ran the same checker against ten deliberately broken copies of `tr.json`, and it caught every one:

- translated placeholder
- dropped `one` branch
- renamed `count`
- English number format
- localized timezone name
- button changed to *İptal et*
- preview saying "confirmed"
- dropped `<strong>`
- dropped "estimated"
- missing key

## Remaining limits

- **No native-speaker human review and no live UI review took place.** The fluency and naturalness judgements above are mine. The checks only show that the structure, formats and key meaning markers are right. A Turkish-speaking reviewer from travel operations should approve the wording before release.
- **The plural message wasn't rendered by the application's own i18n library.** I used a minimal local ICU-plural renderer, because no message-format library is installed and the task disallows installing one. It proves the schema matches the source and that Turkish plural selection behaves correctly. It doesn't prove the app's runtime output.
- **The English-word check is a heuristic word list,** not a full language detector.
- **House-style choices to confirm:**
  - EUR follows CLDR (`€1.250,50`). Some Turkish style guides prefer `1.250,50 €` or `1.250,50 EUR`.
  - The time uses the CLDR colon (`17:00`). The TDK spelling guide prefers `17.00`.
  - The amount, date and time are hard-coded in the source string. If the app later formats them at runtime, these literals should become placeholders in both locales.
- **`{booking_id} numaralı rezervasyonu`** adds the noun *rezervasyon* and the word *numaralı*. That avoids attaching a Turkish case suffix directly to an unknown ID, whose vowel harmony can't be predicted. It assumes the ID reads as a booking number or code, which is common for alphanumeric references.
- **`{name}` is used without an honorific** such as *Sayın* or *Bey/Hanım*, to keep the source structure. No grammatical gender is involved in Turkish.
- **Button width:** *İptal talep et* (14 characters) is slightly shorter than "Request cancellation" (20 characters). Its rendered width in the real UI wasn't checked.
