# Turkish localization QA

Target: `tr-TR`, professional travel operations. Source: `en.json`; deliverable: `tr.json`. The localization skill and its message-integrity reference guided the checks. Review was performed by the translating agent, without delegation.

## Terminology and meaning checks

| Key | Terminology and semantic review |
| --- | --- |
| `refund` | **Tedarikçi** means supplier, **rezervasyon** means booking, and **geri ödeme** means refund. “Onaylayana kadar … iptal etmeyin” preserves the prohibition: do not cancel until the supplier confirms the refund. “Talepte bulunmak … onaylandığı anlamına gelmez” explicitly distinguishes making a request from refund confirmation. It does not imply that money has already been returned or guarantee a refund. `{booking_id}` identifies the booking without requiring a suffix on the dynamic value. |
| `price` | **Tahmini toplam** preserves the estimate qualification. “Yerel vergiler bu tutara dahil değildir” explicitly excludes local taxes. The amount remains EUR 1,250.50; there is no conversion to TRY. |
| `deadline` | **Onaylayın** retains the instruction to confirm; **kadar** retains the deadline. `10 Ekim 2026` is 10 October 2026, and `17:00` and `Europe/Istanbul` are unchanged. The Turkish time suffix in `17:00'ye` fits the sentence. |
| `seats` | **Koltuk** means seat; **müsait** means available. The zero message says no seats are available. Both numbered branches use `# koltuk müsait`; Turkish uses the singular noun after a number, so the two branch texts intentionally match. |
| `button` | **İptal talebinde bulun** means “Request cancellation.” It initiates a request in the wording, without asserting immediate cancellation or completion. |
| `preview` | **Beklemede** preserves pending status, and **henüz onaylanmadı** explicitly states that the booking is not yet confirmed. It makes no promise of future confirmation. `<strong>{name}</strong>` is preserved exactly, with no translation of the dynamic name. |

All six messages were compared with the source for actors, negation, conditions, state, quantities and qualifications, then reread as Turkish interface text. This is an agent language review, not human approval.

## Structure and locale verification

Checks ran against the saved files using Python's standard library and the existing Node.js v24.19.0 runtime (ICU 78.3). No dependencies were installed.

- Both files parse as UTF-8 JSON without duplicate keys. `tr.json` is NFC-normalized, contains six nonempty string values, and has exactly the source keys in the same order: `refund`, `price`, `deadline`, `seats`, `button`, `preview`.
- Per-message placeholder inventories match. `{booking_id}`, `{name}`, literal `Europe/Istanbul`, and the `<strong>` tags remain intact. A separate check confirms that the tags still enclose `{name}`.
- A fixture-specific full-message schema check confirms the unchanged ICU variable `count`, argument type `plural`, branches `=0` / `one` / `other`, and `#` counts of 0 / 1 / 1 respectively. No branch was removed to simplify Turkish grammar.
- Decimal parsing reconciles source `€1,250.50` and target `€1.250,50` to the same value, `1250.50`. The target matches `Intl.NumberFormat('tr-TR', { style: 'currency', currency: 'EUR' })`, including the euro symbol and Turkish separators.
- `Intl.DateTimeFormat` with `tr-TR` and `Europe/Istanbul` confirms `10 Ekim 2026` and `17:00` for the deadline. The timezone identifier remains literal.

The required schema was exercised with explicit `=0` precedence, `Intl.PluralRules('tr-TR')` for category selection, and `Intl.NumberFormat('tr-TR')` for `#` substitution:

| Count | Branch | Simulated output |
| --- | --- | --- |
| 0 | `=0` | Müsait koltuk yok |
| 1 | `one` | 1 koltuk müsait |
| 2 | `other` | 2 koltuk müsait |
| 5 | `other` | 5 koltuk müsait |
| 1250 | `other` | 1.250 koltuk müsait |

Six deliberately broken, in-memory copies remained valid JSON but were rejected by the structural checks: a renamed booking placeholder, renamed `count`, removed `one`, removed `other`, missing `#`, and a name moved outside `<strong>`. These probes check the resource contract; they do not establish linguistic quality.

The source remained byte-for-byte unchanged. Its SHA-256 before and after translation was:

```text
5ba07c8da6fe537ed935ead1773026047287b9b6a559480bfa0061c78da2ae82
```

## Remaining limits

- No native Turkish human, independent reviewer or travel-domain human reviewed the translation.
- No application or MessageFormat runtime was supplied. The plural checks are a fixture-specific schema check and Intl-backed simulation, not execution through the application's ICU message parser.
- No live UI was available or reviewed. Wrapping, truncation, font rendering, accessible output and actual placeholder interpolation remain unverified in the product.
- The fixture is fictional. No supplier was contacted and no booking was changed.
