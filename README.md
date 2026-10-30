<img src="website/assets/mirket-logo.png" width="80" alt="Mirket" />

# Mirket

**A coding agent harness for Pi.** Mirket adds project memory, selected code
navigation, design and video tools, and workflow guidance to Pi's coding agent.
Pi owns models, authentication, sessions, native tools and MCP lifecycle. Mirket
owns explicit scoped authorization for its effect-bearing operations.

[Website source](website/) · [Documentation](docs/README.md) ·
[Release assets](https://github.com/codemirket/harness/releases/tag/v0.0.1)

The **0.0.1 macOS ARM64 prerelease** has a **macOS 26.0-or-newer installation
floor**, without a major-version ceiling, and requires the exact external
**Pi 1.4.0** host acquired by the release helper. The release was qualified on
**macOS 26.5.2 ARM64**; installation eligibility does not establish macOS 27
behavior. See [qualification](docs/qualification.md) for tested identities,
outcomes and remaining limits.

## What it adds

- **Project memory:** canonical SQLite facts, source and revision tracking,
  correction, expiry, forgetting and lazy local semantic retrieval.
- **Code navigation:** inspection, definitions and references over explicitly
  selected JavaScript and TypeScript sources, bound to their revisions.
- **Design:** tokens, editable vectors and D2 diagrams with SVG or bounded PNG
  output. Raster/Sharp transformations remain excluded.
- **Video:** editable HyperFrames projects and verified FFmpeg/ffprobe exports.
  Rendering needs an explicitly selected Chrome binding; finished media still
  needs visual and decoded-output review.
- **Workflow guidance:** selected Pi skills and profiles with concrete routing,
  safeguards and evidence requirements. See the [skill contract](docs/skills.md).

The optional `/mirket-research QUESTION` route requires Pi's configured
`openai/gpt-6-astra` access through OpenAI Responses. Optional local browser QA
uses a separately selected browser and external Playwright MCP peer. These routes
require their configured access and readiness; a tool listing alone does not
establish either. See [qualification](docs/qualification.md) for tested scope.

## Install with a coding agent

Give an agent with local terminal access this instruction:

```text
Install Mirket from https://github.com/codemirket/harness/releases/tag/v0.0.1.
Read README.md, docs/setup.md and docs/release-0.0.1.md from that repository.
Check the macOS ARM64 installation floor and report whether this OS has qualification evidence.
Download all seven release assets and verify SHA256SUMS before running scripts.
Use the documented permanent private install root. For a first installation,
acquire the exact Pi 1.4.0 host with acquire-pi-host.py and run install-pi.py.
For an existing Mirket installation, use the explicit migrate-pi.py procedure
in docs/setup.md after verifying both old and new packages. Preserve existing Pi
authentication, sessions and unrelated settings; stop on an unsupported layout.
Do not run npm, upstream installers or hooks, or substitute PATH executables.
Report the installed paths and checks actually completed. Leave optional browser
choices and model authentication to the interactive Pi onboarding below.
```

Installation uses verified release scripts; optional browser choices happen in
Pi TUI. An agent's completion message is not runtime readiness evidence.

## Install manually

Download these seven files from the [0.0.1 release](https://github.com/codemirket/harness/releases/tag/v0.0.1)
into one temporary directory: `mirket-pi-v0.0.1-macos-arm64.tar.gz`,
`install-pi.py`, `migrate-pi.py`, `acquire-pi-host.py`, `host-lock.json`, `LICENSE`
and `SHA256SUMS`. For an existing Mirket installation, follow the
[upgrade procedure](docs/setup.md#upgrade-an-existing-mirket-installation).
For a first installation, run this from the download directory:

```sh
set -e
shasum -a 256 -c SHA256SUMS
umask 077
install_root="$HOME/.local/share/mirket"
mkdir -p "$install_root/releases"
chmod 700 "$install_root" "$install_root/releases"
release_root="$(mktemp -d "$install_root/releases/0.0.1.XXXXXXXX")"
tar -xzf mirket-pi-v0.0.1-macos-arm64.tar.gz -C "$release_root"
python3 acquire-pi-host.py \
  --package "$release_root/package" --lock host-lock.json \
  --output "$release_root/pi-host"
python3 install-pi.py \
  --package "$release_root/package" --host "$release_root/pi-host" \
  --install-root "$install_root"
"$HOME/.local/bin/pi"
```

Keep the extracted package and acquired host at those permanent paths. Each
installation uses a fresh release directory, including a rerelease of the same
version. The helper registers Mirket in standard `$HOME/.pi/agent` when present,
or creates a private agent directory under the install root. It preserves
authentication, sessions and unrelated settings and refuses unsupported launcher
layouts. It checks native health and Pi registration before replacing the
launcher. See [setup](docs/setup.md) for supported layouts, backups and recovery.

If Mirket is already installed, extract the new archive into a separate permanent
directory and follow the [explicit upgrade procedure](docs/setup.md#upgrade-an-existing-mirket-installation).
The standalone fresh installer refuses to replace a different package path.

## First run

In Pi TUI, configure Pi's model authentication if needed, then:

1. Run `/mirket setup`. For video Chrome, the separate browser QA executable and
   the optional Playwright MCP peer, choose **Install pinned version**, **Use
   installed tool** or **Skip**. The default package contains none of these three.
   Setup discloses and verifies downloads; it runs no upstream installers/hooks.
2. Reload Pi after completed choices. Ask Pi to run `mirket_doctor` once, then
   run `/mirket doctor` to inspect actual runtime readiness. Until that tool runs,
   the command can show `Awaiting actual runtime_status identity`.
3. For configured browser QA, run `/mirket-browser open http://127.0.0.1:PORT`
   and confirm the displayed project, origin and runtime identities. The grant
   lasts at most five minutes or 64 calls; cancellation revokes it.

Skipping optional tools leaves memory, navigation, D2 and bundled design routes
available. Installed jobs recheck their exact executable bindings and do not
download on first use or search PATH. Mirket-owned processes and the browser
proxy are not an OS sandbox.

## Use Mirket in a project

After installing the current 0.0.1 package, start Pi from the project root and run
`/mirket-project init`. This registers the installed package in
`.pi/settings.json` and adds a managed section to `AGENTS.md`. With no arguments,
it preserves an existing Mirket skill selection; to choose a profile or exact
skills, use `/mirket-project init web-product +js-ts-navigation`.

Review the generated changes, reload Pi, ask Pi to run `mirket_doctor`, then run
`/mirket doctor`. `/mirket-project status` reports project synchronization.
Initialization enables updates to the managed section from Mirket skills and
tools actually used in this project. These observations do not establish runtime
readiness or install dependencies. Existing project guidance and unrelated Pi
settings are preserved; conflicting managed content is refused. See
[project setup](docs/setup.md#project-setup) for the file and conflict contract.

## Develop and verify

The repository has one Rust crate and one compiled TypeScript package under
`pi/`. Read [AGENTS.md](AGENTS.md), the [architecture](docs/architecture.md) and
[native build guide](docs/native-build.md) before changing it. Run
`python3 scripts/check-build-inputs.py` before any Rust compilation, use the locked
dependency closure and verified private Node, and test with disposable private
homes and projects. The build guide lists the Rust, adapter and MCP gates.

[Source provenance](skills/PROVENANCE.md), [runtime choices](packaging/README.md#runtime-choices)
and [third-party scope](packaging/THIRD-PARTY.md) distinguish methods, code,
fonts and runtime rights. Failed and unrun evidence remains in
[qualification](docs/qualification.md) and [history](docs/history.md).
