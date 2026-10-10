# Worked case: preserve a prohibition and ICU tokens

Use this fictional product message when reviewing semantic regressions in
structured translation. The target is formal Turkish for Turkey (`tr-TR`);
customer names are supplied at runtime and must not be translated.

Source:

```json
{
  "billing_hold": "Do not charge {customerName} until approval is recorded.",
  "approval_count": "{count, plural, =0 {No approvals} one {# approval} other {# approvals}}"
}
```

Candidate:

```json
{
  "billing_hold": "Onay kaydedilene kadar {customerName} adlı müşteriden tahsilat yapmayın.",
  "approval_count": "{count, plural, =0 {Onay yok} one {# onay} other {# onay}}"
}
```

The candidate preserves the prohibition until recorded approval, the customer
placeholder, the numeric `count` argument and ICU structure. The `one` branch is
retained here for compatibility with the source; which branch executes depends
on the target locale and the project's ICU implementation. Do not infer grammar
quality from branch coverage alone.

Check the actual parser and runtime at counts 0, 1 and 2. A candidate that
renames `customerName`, translates `plural`, drops the `other` branch, changes
the count to a text substitution, or says to charge before approval fails even
if it is valid JSON and sounds fluent. Preserve the same condition when editing
for length; deleting negation is not acceptable compression.

This is a worked candidate, not evidence of native-speaker approval. If it has
not been rendered in the product, wrapping and screen-reader behavior remain
unverified. For an Arabic version, also test the layout with a mixed-script name
and an identifier such as `AB-120`; do not reverse that identifier or mirror a
brand mark to make the screen appear RTL.
