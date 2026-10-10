# Translation QA: `en.json` → `tr.json` (tr-TR)

**Scope.** This covers the six UI strings of a professional travel operations interface. The register is formal Turkish for Turkey.

**Fixture.** The fixture is fictional. No supplier was contacted and no booking was viewed or changed.

**Source.** `en.json` was not edited. Its SHA-256 was the same before and after the work: `5ba07c8da6fe537ed935ead1773026047287b9b6a559480bfa0061c78da2ae82`.

## Terminology

| English | Turkish | Note |
|---|---|---|
| booking | rezervasyon | The standard travel-operations term, used in both `refund` and `preview`. |
| supplier | tedarikçi | The usual B2B term for the hotel or service provider. |
| refund | iade | |
| request (noun) / request (verb) | talep / talep etmek | The same root is used in `refund` and `button`. This makes it clear that the button creates a request and does not perform the result. |
| confirm / confirmed | onaylamak / onaylanmış | One root is used for every "confirm": the supplier confirming the refund, the user confirming by the deadline, and the booking not being confirmed. *teyit* and *konfirme* are not mixed in. |
| cancel | iptal etmek | |
| pending | beklemede | |
| available (seats) | müsait | This is the travel-inventory availability term (*müsaitlik*). *boş koltuk* would also be natural for a transport-only product. |
| estimated total | tahmini toplam | |

**Register:** formal *siz* (*-iniz/-unuz*, *iptal etmeyin*, *onaylayın*). The button uses the bare imperative (*İptal talep et*), which is the normal convention for Turkish UI buttons.

## Meaning checks per key

The back-translations below are deliberately literal.

### `refund`

**Turkish:** Tedarikçi iadeyi onaylayana kadar {booking_id} numaralı rezervasyonu iptal etmeyin. Talep, onaylanmış bir iade anlamına gelmez.

**Back-translation:** Until the supplier confirms the refund, do not cancel booking number {booking_id}. A request does not mean a confirmed refund.

**Preserved:**
- It is still a prohibition (*etmeyin*), not advice.
- The condition is still the *supplier* confirming the *refund*.
- The second sentence still says that a request is not a confirmed refund.
- *Talep* stays generic, like "A request" in the source. It isn't narrowed to "refund request" or "cancellation request", because the source doesn't say which one is meant.

### `price`

**Turkish:** Tahmini toplam: €1.250,50. Bu tutar yerel vergileri içermez.

**Back-translation:** Estimated total: €1,250.50. This amount does not include local taxes.

**Preserved:**
- The currency is still EUR and the value is still 1250.50. Only the separators changed to the Turkish format; there is no currency conversion.
- Both qualifications are kept: "estimated" and "excludes local taxes".
- *içermez* avoids the *dahil/dâhil* spelling split.

### `deadline`

**Turkish:** En geç 10 Ekim 2026 saat 17:00'ye kadar (Europe/Istanbul) onaylayın.

**Back-translation:** Confirm by 17:00 (Europe/Istanbul) on 10 October 2026 at the latest.

**Preserved:**
- It is the same instant: 2026-10-10 17:00 in Europe/Istanbul, which is GMT+03:00 (14:00 UTC).
- "By" becomes *en geç … -e kadar* ("no later than"). This is the normal Turkish deadline phrasing, so it reads as a deadline and not as an appointment time.
- Date before time is the normal Turkish order.
- `(Europe/Istanbul)` is unchanged.
- The suffix *'ye* follows the spoken form *on yediye*.
- No object or weekday was added. The source doesn't say what is being confirmed.

### `seats`

**Turkish:** {count, plural, =0 {Müsait koltuk yok} one {# koltuk müsait} other {# koltuk müsait}}

**Back-translation:**
- =0: No available seats
- one: # seat available
- other: # seats available

**Preserved:**
- `count`, `plural`, the `=0`, `one` and `other` selectors and their order are all unchanged.
- `#` is used in `one` and `other` only, the same as the source.
- The `one` and `other` texts are identical because Turkish nouns stay singular after a numeral (*2 koltuk*).
- Both branches are kept because the application requires this schema. CLDR `tr` also really has the `one` and `other` categories.

### `button`

**Turkish:** İptal talep et

**Back-translation:** Request cancellation

**Preserved:**
- It is still a request. *İptal et* ("Cancel") would perform the action and contradict the `refund` warning.
- The capital is the dotted *İ* (U+0130).

### `preview`

**Turkish:** <strong>{name}</strong>, rezervasyonunuz beklemede, onaylanmış değil.

**Back-translation:** {name}, your booking is pending, not confirmed.

**Preserved:**
- `<strong>` still wraps exactly `{name}`.
- *onaylanmış değil* states the current state.
- *onaylanmadı* was avoided because it can read as "was not approved", i.e. rejected.
- *henüz* ("yet") was not added, because it would imply that confirmation is expected.

## Locale presentation

All values below were generated with Node.js v24.19.0 (ICU 78.3, CLDR 48.0). None was typed from memory.

| Call | Result |
|---|---|
| `Intl.NumberFormat('tr-TR', {style:'currency', currency:'EUR'}).format(1250.50)` | `€1.250,50` |
| `Intl.DateTimeFormat('tr-TR', {dateStyle:'long', timeZone:'Europe/Istanbul'})` | `10 Ekim 2026` |
| `Intl.DateTimeFormat('tr-TR', {timeStyle:'short', timeZone:'Europe/Istanbul'})` | `17:00` |
| Same instant, `timeZoneName:'longOffset'` | `GMT+03:00`, i.e. 14:00 UTC |
| `Intl.PluralRules('tr-TR')` categories | `one`, `other` |
| `Intl.PluralRules('tr-TR')` selection for 0, 1, 2, 1000 | `other`, `one`, `other`, `other` |

Because 0 falls into the `other` category, the `=0` exact-match branch is what produces the zero-seat text.

## Verification performed

### Tooling

The checker was a one-off Node.js script run from stdin, so no file was added to the task directory.

It parsed and formatted messages with formatjs: `intl-messageformat` 10.7.18 and `@formatjs/icu-messageformat-parser` 2.11.4. That library was already present on this machine in an unrelated project. It was loaded read-only and nothing was installed.

It is a reference ICU implementation, **not necessarily the application's own i18n runtime**.

### Results on the saved `tr.json`: no failures

The checker confirmed:

1. **File format:**
   - The file is valid UTF-8 with no BOM.
   - It is valid JSON.
   - The keys are identical to `en.json`, in the same order.
   - It uses the same 2-space layout and trailing newline as the source.
2. **ICU and markup skeleton:** for every key, the parsed ICU/markup skeleton matches the source exactly, ignoring literal text. The skeletons are:
   - `refund`: `arg:booking_id`
   - `seats`: `plural:count:cardinal:0{=0[] one[#] other[#]}`
   - `preview`: `tag:strong(arg:name)`
   - `price`, `deadline`, `button`: literal only
3. **Apostrophe in *17:00'ye*:** no apostrophe comes directly before `{ } # |`, which would start ICU quoting. formatjs outputs the `deadline`, `price` and `button` strings byte-identical to their input, so the apostrophe stays literal.
4. **Locale values:** the EUR amount, date and time equal the Intl values in the table above. The English `1,250.50` is gone, and `(Europe/Istanbul)` is present verbatim.
5. **Meaning markers:** each of these is present:
   - the prohibition and its supplier-refund condition
   - request ≠ confirmed refund
   - the estimate qualifier
   - the local-tax exclusion
   - the deadline wording
   - request wording on the button
   - pending/not-confirmed in the preview, with no "confirmed" or "rejected" wording
6. **Leftovers:** no listed English words remain (the timezone literal is excluded from this check), and the dotted *İ* is correct.

### Rendered output (formatjs, locale `tr-TR`)

Sample values were `booking_id = BK-20481` and `name = Ayşe Yılmaz`.

**`seats`:**

| `count` | tr-TR | en |
|---|---|---|
| 0 | Müsait koltuk yok | No seats available |
| 1 | 1 koltuk müsait | 1 seat available |
| 2 | 2 koltuk müsait | 2 seats available |
| 21 | 21 koltuk müsait | 21 seats available |
| 1000 | 1.000 koltuk müsait | 1,000 seats available |

**`refund`:** Tedarikçi iadeyi onaylayana kadar BK-20481 numaralı rezervasyonu iptal etmeyin. Talep, onaylanmış bir iade anlamına gelmez.

**`preview`:** `<strong>Ayşe Yılmaz</strong>, rezervasyonunuz beklemede, onaylanmış değil.`

### Failure probes

The checker was run against 27 deliberately broken copies, and it caught all 27:

- **refund:**
  - negation removed (*iptal edin*)
  - condition dropped
  - request = refund (*anlamına gelir*)
  - placeholder translated
  - apostrophe placed before `{`
- **price:**
  - decimal changed (`1.250,05`)
  - English number format
  - "estimated" dropped
  - tax exclusion inverted (*içerir*)
- **deadline:**
  - date changed
  - time changed
  - timezone localized (*Avrupa/İstanbul*)
- **seats:**
  - `one` branch dropped
  - `count` renamed
  - `plural` keyword translated
  - `#` replaced by text
- **button:**
  - changed to *İptal et*
  - undotted *Iptal*
- **preview:**
  - says *onaylandı*
  - `<strong>` dropped
  - English word left in
- **file:**
  - missing key
  - extra key
  - key order changed
  - BOM added
  - invalid UTF-8
  - trailing newline removed

The first run caught only 26 of 27. The miss was the BOM probe, and the defect was in the checker: `TextDecoder` strips a BOM by default. The check was changed to inspect the raw bytes, and all checks were then rerun.

## Remaining limits

**No native-speaker review and no live UI review took place.**
- Fluency and register judgements are the translator's own.
- The automated checks prove structure, values and the presence of meaning markers. They do not prove that the language reads naturally.
- A Turkish-speaking travel-operations reviewer should approve the wording before release.

**Runtime is unverified.**
- Rendering used formatjs, not the application's own library.
- `java.text.MessageFormat` treats every single apostrophe as a quote. With that library, *17:00'ye* would render as *17:00ye*, so the apostrophe would need to be written as `''`. ICU4J/ICU4C (default apostrophe mode) and formatjs keep it literal, as observed above.
- If the app renders `<strong>` some other way, for example as raw HTML, it behaves the same as the source because the markup is unchanged.

**Layout is unverified.**
- *İptal talep et* has 14 characters, compared with 20 for "Request cancellation".
- Wrapping and truncation of all strings, the font's rendering of *İ ı ş ğ ç ö ü*, and screen-reader output were not checked in the real UI.

**House-style choices to confirm:**
- **Euro amount:** written in the CLDR order (`€1.250,50`). Some Turkish guides prefer `1.250,50 €` or `1.250,50 EUR`.
- **Time:** written with the CLDR colon (`17:00`). The TDK spelling guide prefers `17.00`.
- **Terminology:** *müsait* vs *boş koltuk*.

**Assumptions about placeholder text:**
- **`{booking_id} numaralı rezervasyonu`** adds the word *rezervasyon* and *numaralı*. This avoids attaching a Turkish case suffix directly to an unknown ID, whose vowel harmony can't be predicted. It assumes the ID reads naturally as a booking number or code; that holds for typical alphanumeric references such as `BK-20481`.
- **`{name}` has no honorific** such as *Sayın* or *Bey/Hanım*, to keep the source structure. It is unknown whether `{name}` is a first name or a full name. Addressing someone by bare name with *siz* is common in Turkish product notices. A stricter house style may want *Sayın* for full names.

**Hard-coded values.** The amount, date and time are literals in the source strings. If the product later formats them at runtime, they should become placeholders in both locales.
