# Worked case: stale caches in a macro-enabled model

Use this fictional case to distinguish arithmetic checking, calculation-engine
evidence and preservation of workbook features.

The user supplies `Revenue.xlsm` and asks to change quantity C2 from 2 to 4. The
sheet has these source cells and stored cached values before the edit:

| Cell | Formula or input | Stored cached value |
| --- | --- | ---: |
| B2 | 100 | 100 |
| C2 | 2 | 2 |
| D2 | `=B2*C2` | 100 |
| B3 | 50 | 50 |
| C3 | 3 | 3 |
| D3 | `=B3*C3` | 150 |
| D4 | `=SUM(D2:D3)` | 250 |

The workbook also has a name `Revenue_Total` referring to D4, a VBA project and
an external FX reference with a cached value. None should change for this edit.
The request does not authorize executing macros or refreshing the FX source.

Before the edit, the independent expected subtotal is 2 × 100 + 3 × 50 = 350;
the stored 250 is stale. After setting C2 to 4, expected D2 is 400 and D4 is 550.
Keep the formulas, the named range and the `.xlsm` format. A values-only load and
save or conversion to CSV would not meet the request.

Choose a capable route, save a recoverable edited artifact, recalculate without
incidental external refresh, and reopen it. Verify the intended cell change,
formula results, name, macro payload and external-link representation with the
evidence the selected tool can actually provide. A macro-preservation option is
not evidence by itself; structural preservation is also not proof that the macro
will execute correctly. Do not execute it merely to broaden the check.

If no compatible calculation engine is available, the useful report is that C2
and formula source were checked and the independently expected total is 550,
while recalculated workbook values remain unverified. Do not claim that stale
caches are current or that the external exchange rate was validated.
