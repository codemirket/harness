# Historical evidence

The pre-onboarding dependency review records remain recoverable from Git commit
`cd4ba7198f16e3e9ba625123016a9f2582dfea14`, under `packaging/admission/`.
Every archived file was compared byte for byte with that commit before removal.
Use `git show cd4ba7198f16e3e9ba625123016a9f2582dfea14:packaging/admission/FILENAME`
to inspect a record. The package assembler does not ship Git history or these records.
The runtime now reads only the byte/source pins under `packaging/runtime/`.

The [qualification record](qualification.md) separates the earlier package's
actual identity and outcomes from the later onboarding/package trial. Earlier
results do not qualify later bytes. Failed, skipped and unrun outcomes remain
evidence.
