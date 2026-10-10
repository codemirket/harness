I wrote [tr.json](recheck/tr.json) and [translation-qa.md](recheck/translation-qa.md) in `recheck/`. The translation passes every automated check, but no native Turkish speaker has reviewed it and it hasn't been seen in the real interface, so the wording still needs a human check before release.

**How each string was handled:**
- **Refund warning:** still a prohibition ("do not cancel"), still waiting on the supplier to confirm the refund, and still says a request is not a confirmed refund. I kept "request" general, because the source doesn't say whether it means a refund or a cancellation request.
- **Price:** written as `€1.250,50` in Turkish number format. The amount and currency are unchanged, and "estimated" and "excludes local taxes" are both kept.
- **Deadline:** `En geç 10 Ekim 2026 saat 17:00'ye kadar (Europe/Istanbul)` ("by 17:00 on 10 October 2026 at the latest"). It's the same moment as the source (14:00 UTC), and `Europe/Istanbul` is untouched.
- **Seat count:** the `=0`, `one` and `other` branches, the `count` variable and the `#` positions are exactly as in the source.
- **Button:** "İptal talep et" means "request cancellation", not "cancel now".
- **Preview:** says the booking is pending and not confirmed. I avoided wording that could read as "rejected" or that promises confirmation is coming.

**Verification:**
- **Parser:** I checked the file with formatjs, a standard library for this message format. It parsed and rendered every string, and the structure matches the English file key for key. formatjs was already installed in another of your projects (`maje-websites`); I loaded it read-only and installed nothing. It is not necessarily the library your app uses.
- **Locale values:** the number, date, time and plural rules come from Node's built-in locale data, not from memory.
- **Broken copies:** I ran the checker against 27 deliberately broken copies, such as a removed "not", a changed decimal, a dropped plural branch, a renamed placeholder or a translated timezone. It caught all of them. On the first run it missed an added byte-order mark at the start of the file because of a bug in my checker; I fixed it and reran everything.
- **Source file:** `en.json` is unchanged (same hash before and after), and nothing was written outside `recheck/`.

**Still open (details in the QA file):**
- **Apostrophe:** Java's built-in message formatter would drop the apostrophe in `17:00'ye`. If your app uses it, the apostrophe needs to be doubled.
- **Style choices to confirm:** `€1.250,50` vs `1.250,50 €`, and `17:00` vs the Turkish style guide's `17.00`.
- **Booking ID wording:** I wrote the ID as "{booking_id} numaralı rezervasyon" ("booking number …"). That works as long as the ID reads like a number or code.
- **Untested in the real UI:** button width, line wrapping and font rendering.

There's an earlier `tr.json` in the parent folder. Mine differs from it in two places: the refund's second sentence ("anlamına gelmez" instead of "değildir") and the deadline phrasing ("17:00'ye kadar" instead of "17:00'de").
