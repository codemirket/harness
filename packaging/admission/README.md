# Dependency admission inputs

These records bind the exact dependencies, archives, source notices and capability
checks used by the build and packaging tools. They are active verification inputs,
not an implementation diary. A proposal alone grants no permission.

- [External pi host](pi-local-candidate.json): exact unbundled pi 1.4.0 + Undici
  8.12.2, approved only for the local candidate. See [host contract](pi-host-repair.md).
- [Browser testing](pi-browser.md): exact three-package disposable local-only batch;
  no production browser admission.
- `m0-*`: locked Rust/TypeScript development closure and private Node.
- `m2-*`: static SQLite and private semantic runtime/model.
- `m3-*`, `m4-*`, `openpencil-*`: exact native browser, video, design and editable
  capability inputs. Target manifests do not themselves qualify a platform.
- `pi-native-base-package-lock.json` and `pi-native-layout.json`: verify that the
  native dependency closure retained its approved versions/integrities through
  the package layout change. Both are consumed by recipient assembly.

Pinned `.proposed` filenames are retained where assembly or an approval record
binds their exact bytes. Consult the owning admission record for scope; do not
rename or rewrite a frozen identity merely to make its filename look current.
Hooks and upstream installers must never run incidentally. No host redistribution
or publication is authorized by a successful local build.
