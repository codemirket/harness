# Private general Python: macOS ARM64 candidate

The selected CPython 3.13.17 normal-GIL runtime passed a fixed local qualification on macOS 26.5.2 ARM64. This admits the exact candidate for local integration under the user's full rewrite authorization and explicit selected-tree qualification authorization. The local installed SQL profile is qualified separately below; complete shipping integration, native Windows and older macOS execution remain unqualified. This is the general worker runtime for Mirket-owned operations. Decision 79 assigns Office workflows to the host; no separate Office Python is packaged.

The source is [python-build-standalone release 20261021](https://github.com/astral-sh/python-build-standalone/releases/tag/20261021), source commit `0c5e581b54a4736cdbbe68bdd6d7e571a1dca11e`, asset `cpython-3.13.17+20261021-aarch64-apple-darwin-install_only.tar.gz` (25,404,044 bytes), SHA256 `9c985cbf327425873469896ce748c7134b6deca8edb81e93c589ad2fc5aff445`. CPython source resolves to `64e080d38154391ff2d2caa1161c298bb7cd8f30`. No binary bytes were changed, source rebuilt, upstream helper executed, or host installation modified.

## Deterministic selection and rights

[The selection manifest](m3-python-macos.selection.json) enumerates every retained and omitted archive member, including hashes, sizes, modes and the two relative interpreter symlinks. It retains 545 regular files and two symlinks: 46,602,502 regular-file bytes. Thirteen supplemental notices add 95,846 bytes, for 46,698,348 bytes (44.54 MiB) before filesystem overhead. The selection omits 1,110 members totaling 19,098,245 bytes. The original fully qualified runtime remains separate and unchanged.

The retained headless standard library includes `ctypes`, subprocess/multiprocessing, codecs, compression, SQLite, TLS, introspection and public unittest/doctest APIs. The shared libpython, sysconfig data, sysconfig Makefile and `pyconfig.h` remain for runtime introspection and future separately admitted binary wheels. Remaining development headers, build scripts, package-management closure (`pip`, `ensurepip`, `venv`), GUI closure (Tk/Tcl/Itcl/Thread, idle, turtle), and upstream fixture/debug material are omitted. Built-in C test modules cannot be removed without rebuilding the executable and are not claimed removed.

[The notice inventory](m3-python-macos.notices.json) maps every retained native file to component versions, rights and exact notice sources. It embeds all supplemental notice text as UTF-8, with independent byte counts and SHA256 values; assembly must preserve those bytes. The archive's Python license and ctypes/macholib notice remain selected. Bundled native components include bzip2 1.0.8, libffi 3.8.0, liblzma 5.8.4, Expat 2.9.0, mpdecimal 4.0.0, OpenSSL 3.5.9, SQLite 3.53.1 and CPython's incorporated HACL, BLAKE2 and mimalloc code. Their retained notices cover permissive licenses or public-domain dedication. This is a component-based notice review, not universal legal or advisory certification.

The three retained Mach-O files are the interpreter, shared libpython and `_dbm` extension. Their recorded native imports use only explicit macOS system libraries/frameworks; none imports Tcl/Tk. Archived initialization source contains no built-in `_tkinter`, and the removed `_tkinter` extension was the GUI bridge. System libSystem, zlib, libedit, ncurses, panel, CoreFoundation and SystemConfiguration are required OS components, not redistributed payload members. `_dbm` uses OS NDBM. Mach-O deployment metadata says macOS 11.0 minimum; actual qualification covers only the platform above.

## Runtime contract and evidence

Invoke the absolute private interpreter with `-I -S -B`. These flags isolate Python startup and avoid site initialization/bytecode writes; they do not establish a security sandbox or prevent all native-library environment influence. Future adapters must supply controlled environments, explicit verified dependencies and owned process lifecycles. No package manager, environment creation, GUI, implicit dependency fetch, or arbitrary wheel compatibility is promised. Any future wheel path must be admitted explicitly; the required site-packages directory is not automatically added under `-S`.

The selected specimen ran once through two identical fixed probes, before and after relocation to a path containing spaces and Unicode, followed by one cancellation child. Both probes returned exit 0 with empty stderr. Checks covered Unicode JSON, decimal, compression, hashing, XML, ctypes callback, SQLite transaction/integrity, exact private subprocess interpreter/flags and loaded-module origins. They confirmed the omitted modules unavailable and the required sysconfig/libpython files present. Cancellation received SIGTERM, was reaped, and left no process group. Startup sentinels and the entire 611-entry runtime tree were unchanged at all five checkpoints.

Each interpreter subtree actually ran through `/usr/bin/sandbox-exec -p '(version 1) (allow default) (deny network*)'`; the fixture driver and native process observations remained outside that sandbox. This records an applied network-deny policy, not packet capture. Fixed limits were 60 seconds total, 12 seconds per child and 32 KiB per output stream. The small owned subprocess inside the stdlib probe uses post-capture checking; this is not an arbitrary-input adapter output guarantee. Timing/RSS observations under concurrent integration work support no general performance claim.

Frozen evidence identities (SHA256):

| Evidence | Digest |
| --- | --- |
| Original exact selection | `863912d2a51c072d8cd669ef0c175b2d6c5ae4e314261198ecbe8126e82222e0` |
| Driver | `9d184f4900e8796a6a403f10ff5db67b52661c9519deb1f751acfdf148642a49` |
| Probe | `0286d60361a63eec458e4b7148bacdaf802c7001e5dc0e740b01026939349b40` |
| Pre-execution freeze | `567d6eb30b490f8346ac4afe16b83bad293fc8334a6a9a4cba80902e5ce05d96` |
| Result | `6688b000e92ca5424e90587b5a3146c8f182fbfe39df3ce2059dc06a2f5881d1` |
| Canonical runtime tree | `86aa0290fab286061fb3ca51a8e8d66ec614975efbd6e3ae9b4187f66777a79c` |

Raw evidence and the separate prepared runtime remain in task-state `dependencies/python-general/headless-probe/qualification-01/`. The two JSON admission files retain unchanged member pins and add qualification metadata; they are not the original frozen selection bytes. The admission files are consumed by the repository assembler described below; they do not configure or install a runtime.

The interpreter's SQLite 3.53.1 is older than Mirket's separately admitted canonical-memory SQLite 3.54.1; it must not substitute for that database owner or be described as containing later fixes. The Windows source distribution's Microsoft runtime redistribution terms remain a separate unresolved admission boundary. No Windows payload or licensing exception is admitted here.


## Local assembly

`scripts/assemble-python-macos.py` assembles the exact selected tree from an already downloaded archive. Run it on macOS with `python3 -B scripts/assemble-python-macos.py --archive /absolute/archive.tar.gz --output /absolute/new-directory`. The destination must be absent beneath an existing private, stable parent. The output contains `python/` and the selected `notices/python-general/` files; it is not an installation into the user's home.

The assembler checks the archive size and hash before reading members and again before publication. It rejects duplicate, unlisted, noncanonical and special members, checks selected content hashes and modes, preserves the two exact relative interpreter links, and writes the embedded notices byte for byte. Darwin's exclusive rename publishes the complete staged directory without replacing an existing destination, including an empty directory created during publication. It never runs the interpreter, fetches dependencies or invokes upstream helpers. The private stable parent is a precondition; this is not protection against a hostile same-user filesystem mutator or a claim of crash-durable installation.

Nine focused synthetic tests passed, including corrupt archive/member/notice bytes, traversal and escaping links, special/duplicate/unlisted members, a FIFO archive path, wrong modes, existing output and a destination-creation race. Assembly of the exact candidate additionally verified all 547 selected members and 13 notices. These checks establish assembly fidelity. A separate installed SQL fixture subsequently passed resolution, staging, activation and real SQL execution from a relocated private home on macOS 26.5.2 ARM64. It verified normalized installed permissions and exact aliases, unchanged installed bytes and source files, and rejection of altered Python bytes or an escaping alias without changing the active release. See [the SQL qualification](../../docs/m0-sql-qualification.md) for its limits. The original admission JSON records its initial qualification state; later installed-profile evidence does not change the selected dependency bytes.
