# Execution environment
Mirket keeps its Rust CLI and desktop MCP launch entries native. Selected local
services use the dedicated Podman environment. The design package owns its
service lifecycle and shared workspace.

## Explicit setup

Install the approved official Podman client outside Mirket's immutable package
store, then select its absolute executable. Host software installation and VM
creation require the user's authorization.

```sh
mirket tool register podman --executable /absolute/path/to/podman
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
The design worker has a 3 GiB memory limit, a 512 process/thread limit, and loads rendering dependencies only
when invoked; idle discovery and cached token reads do not load the renderers.
These bounds leave headroom for concurrent sessions, but do not guarantee that
arbitrary image models or unbounded simultaneous rendering fit the device.
`environment start` checks sharing and private connection identity before boot.
Neither package calls nor MCP initialization starts the VM. An absent or stopped
environment is an actionable readiness error.

For an existing installation with a package selection to replace, use the new
release binary's `mirket setup --target all --package design --yes` after
`environment start`, or run `package install design`. Both explicitly replace the
saved selection and reconcile the owned client bindings before later updates.

The registered client is digest checked. Private XDG configuration, storage,
connection and helper paths avoid selecting an unrelated engine. Short owned
runtime directories under `/private/tmp/mirket-<identity>` accommodate macOS Unix
socket limits. No workload receives that socket. `environment stop` preserves
the VM's images and durable service volumes.
Machine helpers use the selected client's directory and its exact adjacent
`libexec/podman` directory. Initialization and startup require executable
`gvproxy` and `vfkit` there before downloading a boot image or issuing VM
commands; a client-only installation fails with the missing helper's name.
This supports Homebrew's split installation layout without consulting global
Podman configuration. The client digest does not independently pin helper bytes;
those remain part of the approved host installation. Existing exact Mirket
helper configuration is migrated; other configuration changes are refused.
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
relative to that mount; the official Penpot `export_shape` and `import_image`
`filePath` arguments require absolute `/workspace/...` paths. Project directories
are not mounted or expanded implicitly. `package start` and `package status`
report the host path and assigned loopback host ports. These host ports are
allocated automatically and saved with the package state; callers use the
reported values rather than the services' internal port numbers. If a saved
host port is occupied while its service is absent, start assigns another free
loopback port. An existing owned container keeps its recorded mapping.

Inside the private design network, Browserless serves browser control on port
3000, Kroki diagrams on 8000, imgproxy raster transforms on 8080 and ComfyUI
graphs on 8188. Penpot's frontend uses internal port 8080. The official Penpot
MCP listens on internal port 4401 at `/mcp`; its plugin manifest is on internal
port 4400 and the persistent plugin
WebSocket on internal port 4402. The WebSocket port is not separately published
to the host. ComfyUI serves graph API calls without a standalone browser UI;
model checkpoints are user supplied under `/workspace/models`. The remaining
design tools run through
the package's direct schemas. `environment status` confirms the private VM;
package status and a useful tool call establish separate readiness levels.

## Platforms and provenance

The selected private host client and boot artifact are pinned by the current
package source inventory. Podman and the machine-image build repository use
Apache-2.0; operating-system components keep their own terms and bundled legal
material. These source links do not establish complete VM source or license
coverage.

The verified controller target is macOS ARM64. Other hosts require direct
runtime and workspace-sharing evidence before being marked ready.
