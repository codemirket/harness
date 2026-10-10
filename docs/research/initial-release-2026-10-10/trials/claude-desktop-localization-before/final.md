I've finished the Turkish translation (`tr.json`) and the QA notes (`translation-qa.md`). An automated check of the structure and formats passed. Nobody fluent in Turkish has reviewed the wording, and it hasn't been tried in the live interface.

| Key | Turkish |
|---|---|
| refund | Tedarikçi iadeyi onaylayana kadar {booking_id} numaralı rezervasyonu iptal etmeyin. Talep, onaylanmış bir iade değildir. |
| price | Tahmini toplam: €1.250,50. Bu tutar yerel vergileri içermez. |
| deadline | En geç 10 Ekim 2026 saat 17:00'de (Europe/Istanbul) onaylayın. |
| seats | {count, plural, =0 {Müsait koltuk yok} one {# koltuk müsait} other {# koltuk müsait}} |
| button | İptal talep et |
| preview | `<strong>{name}</strong>`, rezervasyonunuz beklemede, onaylanmış değil. |

**Cancellation and refund state:**
- **Refund message:** it still says not to cancel until the supplier confirms the refund, and that a request is not a confirmed refund.
- **Button:** it reads "request cancellation", not "cancel".
- **Preview:** it says the booking is not confirmed, worded so it can't be read as "rejected".

**What the check covered:**
- **Structure:** same keys in the same order, same placeholders and `<strong>` markup, and the exact `{count, plural, =0 … one … other …}` format with `#` in place. Both the `one` and `other` versions are kept even though they're identical, because Turkish doesn't change the noun after a number.
- **Number and date formats:** the EUR amount, date and time match what Node's built-in Turkish locale data produces. `Europe/Istanbul` is unchanged.
- **Seat counts:** displayed correctly for 0, 1, 2 and 1000 using a small renderer I wrote; no message-format library is installed, so it wasn't the app's own.
- **The check itself:** I ran it against ten deliberately broken copies, such as a translated placeholder, a dropped plural case or a "Cancel" button, and it caught all of them.
- **Source file:** `en.json` is byte-for-byte unchanged.

**Choices for you to confirm (details in the QA file):**
- **Currency:** `€1.250,50` is the standard Turkish locale format. Some Turkish style guides prefer `1.250,50 €`.
- **Time:** `17:00` uses a colon. The Turkish Language Association (TDK) style is `17.00`.
- **Booking ID:** I added "numaralı rezervasyon" ("booking number") so no Turkish grammatical ending has to attach directly to an unknown ID.

A Turkish-speaking reviewer from travel operations should approve the wording before release.

Files are in [the task directory](.):
- tr.json
- translation-qa.md
