# Bounded YAML parser dependency admission

Reviewed 2026-10-25 under the user's standing approval for dependency
installation. This batch adds `saphyr-parser =0.2.2` with default features disabled,
plus `arraydeque 0.5.1` with no features. The existing locked `thiserror` 2.0.21
and procedural-macro closure are reused without version or feature changes.
The [exact inventory](m3-yaml-inspect.json) records all eight reachable packages,
archive SHA-256 hashes/sizes, selected features, licenses and dependency edges.
No npm package or private runtime is added.

The [parser crate](https://crates.io/crates/saphyr-parser/0.2.2) exposes streamed
source events and spans, without requiring a resolved YAML value tree or expanding
aliases. Its marker documentation says bytes, but the scanner and a discriminating
Unicode test demonstrate character indices. The inspector deliberately uses the
character-iterator API and one bounded original character-to-UTF-8 byte table; it
does not trust the documented index unit. The [arraydeque crate](https://crates.io/crates/arraydeque/0.5.1) is its
fixed-capacity scanner buffer. Both new packages offer MIT or Apache-2.0 and
include the corresponding license files. Existing `unicode-ident` additionally
requires Unicode-3.0 notices. Preserve these notices in final release assembly.
Neither new package contains a build script. Existing procedural macro/build
behavior remains covered by its original admission; the repository build-input
guard must run before compiling.

The new archives were downloaded directly from the crates.io API and matched the
registry checksums recorded in the inventory. Source inspection found no network,
foreign-function or native-library dependency in this new closure. The production
inspector uses `Parser::next_event`, not the parser's recursive tree loading API;
it refuses aliases, merge keys and explicit tags before returning a structure.
There is no conversion to YAML values, tag execution, schema resolution or write.

OSV exact-version queries for all eight packages returned no advisory records on
2026-10-25. The raw finite query/response is retained in the inventory. This is
a dated registry observation, not proof of safety or absence of unknown defects.
Only the locked closure and tested features are selected; parser installation
alone establishes neither semantic navigation nor native client qualification.
