# Execution environment
Mirket keeps its Rust CLI and desktop MCP launch entries native. Selected local
services use the dedicated Podman environment. The design package owns its
service lifecycle and shared workspace.

## Explicit setup

Select an approved native Podman client and import its exact client and AppleHV
helpers into Mirket's dependency tree. Import retains the external source as an
installation input; subsequent Mirket VM operations use the pinned local copy.
Host software installation and VM creation require the user's authorization.

```sh
mirket tool register podman --executable /absolute/path/to/podman
mirket environment localize-client
mirket environment init
mirket environment start
mirket environment status
mirket package install design
mirket package start design
mirket package status
```

`environment init` downloads the reviewed boot artifact through Mirket's verified
cache. `--image /absolute/path/to/image.raw.zst` accepts only the exact same pin.
The macOS ARM64 profile creates a rootless Apple Hypervisor VM with four CPUs,
6 GiB memory, a 40 GiB disk and one private staging share. It does not start it.
The VM requires at least 6 GiB and reserves at least half of physical host RAM
for macOS and desktop applications. On a 16 GiB host running multiple design MCP
connections, allocate 8 GiB while the owned VM is stopped:

```sh
mirket package stop design
mirket environment stop
mirket environment resize --memory-mib 8192
mirket environment start
mirket package start design
```

Resizing preserves the disk and workspace and never stops a running VM implicitly.
The design worker has a 2 GiB memory limit, a 512 process/thread limit, and loads rendering dependencies only
when invoked; idle discovery and cached token reads do not load the renderers.
Image builds run one stage at a time with a 2 GiB memory/swap ceiling and 512
process resource limit. These are separate from the running service budgets.
Startup sums the enabled service caps and requires another 512 MiB of usable VM
RAM for the OS. Core fits the default 6 GiB VM; all groups need the 8 GiB VM.
Native render admission rejects competing jobs before loading renderers. No local
image-generation model is installed. Input and output bounds still apply.
`environment start` checks sharing and private connection identity before boot.
Neither package calls nor MCP initialization starts the VM. An absent or stopped
environment is an actionable readiness error.

For an existing installation with a package selection to replace, use the new
release binary's `mirket setup --target all --package design --yes` after
`environment start`, or run `package install design`. Both explicitly replace the
saved selection and reconcile the owned client bindings before later updates.

The installed executable, approved Podman client, `gvproxy`, `vfkit`, verified
downloads, machine configuration, disk, image storage, durable volumes, package
state and disposable host work all live under the selected `~/.mirket` root.
Private XDG configuration, storage and runtime paths avoid selecting an unrelated
engine. A short `~/.mirket/p` prefix accommodates macOS Unix socket limits;
homes whose path exceeds the socket budget fail before VM creation or start.
No workload receives that socket. `environment stop` preserves the VM's images
and durable service volumes.
The local client and both helpers have separate digest receipts. Initialization
and startup require executable `gvproxy` and `vfkit` before downloading a boot
image or issuing VM commands. Import supports Homebrew's split installation
layout without consulting global Podman configuration. Existing exact Mirket
helper configuration is migrated; other configuration changes are refused.

The private host runtime lives at `~/.mirket/p`. `environment status` reports
that path and whether the Podman client was imported. It probes readiness and
is not a purely passive file read. The managed VM and client use only the
selected Mirket home; an installation with a different runtime path is not
adopted or migrated by 0.0.1.

Each native API client retains a private socket hard link for its lifetime so an
unlinked endpoint cannot reuse the captured inode and pass the identity check.
The final client clone's drop removes that pin.
Before stopping, Mirket reads the bounded loopback VM inspection response and
checks its actual private disk, EFI, sharing and socket identities. A changed
endpoint, unavailable identity or foreign runtime stops the operation before any
stop request is sent.
The checked device layouts include the one-time ignition socket on first boot
and its absence on later boots. Both retain the same required disk, EFI store,
single share and runtime socket. Extra, duplicated or differently owned devices
are refused. This follows the pinned [Podman Apple VM startup](https://github.com/containers/podman/blob/v6.1.3/pkg/machine/apple/apple.go)
and was exercised against the real restarted macOS ARM64 machine.

## Design service boundary

The selected design package owns its local services and shared workspace. The
host directory is the selected home's
`.mirket/execution/podman/staging/design/workspace`, mounted read/write as
`/workspace` in every design container. Direct design tool file paths are
relative to that mount, including OpenPencil document sources and outputs. Project directories
are not mounted or expanded implicitly. `package start` and `package status`
report the host path and assigned loopback host ports. These host ports are
allocated automatically and saved with the package state; callers use the
reported values rather than the services' internal port numbers. If a saved
host port is occupied while its service is absent, start assigns another free
loopback port. An existing owned container keeps its recorded mapping.

Core has no public browser or raster-processing service: Playwright and Sharp run
as bounded owned processes in design-tools. The optional diagrams group uses Kroki
on internal port 8000. OpenPencil runs headlessly in core without additional
ports, services or persistent volumes. Only enabled groups
start. `environment status`, package status and a useful tool call establish
separate readiness levels.

## Platforms and provenance

The selected private host client and boot artifact are pinned by the current
package source inventory. Podman and the machine-image build repository use
Apache-2.0; operating-system components keep their own terms and bundled legal
material. These source links do not establish complete VM source or license
coverage.

The verified controller target is macOS ARM64. Other hosts require direct
runtime and workspace-sharing evidence before being marked ready.
