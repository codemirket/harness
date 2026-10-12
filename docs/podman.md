# Execution environment

Mirket keeps its Rust CLI and desktop MCP launch entries native. Motion Canvas
and HyperFrames run offline Linux jobs in a dedicated Podman environment. Penpot
uses five pinned Linux services and a native headless browser that accesses its
loopback frontend. Inkscape, accessibility audits and general browser tools keep
their verified native paths until equivalent container behavior is established.

## Explicit setup

Install the approved official Podman client outside Mirket's immutable package
store, then select its absolute executable. Host software installation and VM
creation require the user's authorization.

```sh
mirket tool register podman --executable /absolute/path/to/podman
mirket environment init
mirket environment start
mirket package install motion-canvas hyperframes
mirket package probe motion-canvas --schemas --project /absolute/project
mirket environment status
mirket environment recover
mirket package status
```

`environment init` downloads the reviewed boot artifact through Mirket's verified
cache. `--image /absolute/path/to/image.raw.zst` accepts only the exact same pin.
The macOS ARM64 profile creates a rootless Apple Hypervisor VM with four CPUs,
6 GiB memory, a 40 GiB disk and one private staging share. It does not start it.
`environment start` checks sharing and private connection identity before boot.
Neither package calls nor MCP initialization starts the VM. An absent or stopped
environment is an actionable readiness error.

The registered client is digest checked. Private XDG configuration, storage,
connection and helper paths avoid selecting an unrelated engine. Short owned
runtime directories under `/private/tmp/mirket-<identity>` accommodate macOS Unix
socket limits. No workload receives that socket. `environment stop` preserves
the VM's images and durable service volumes.
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

## Offline render jobs

Image recipes combine pinned Linux Node, Chromium, FFmpeg/FFprobe, the selected
engine, fonts and Linux libraries. They run no package-manager command or lifecycle
hook. The same package receipt records source integrity, recipe identity and
immutable image ID. The reviewed library donor's tools and browsers are excluded
from the final image. Fixed shared-library families cover observed Chromium/Node
loads and required font/NSS providers; the donor's desktop GUI, Mesa, LLVM,
GStreamer and Perl trees are excluded. Chromium keeps its bundled software GPU
renderer. Dependency provenance is in
[container-runtime.json](../registry/package-locks/container-runtime.json).

Each MCP session has at most one worker, bounded to fifteen minutes. Two workers
may hold execution leases per selected home. Each has two CPU cores, 1536 MiB
memory with no additional swap, 256 processes, a read-only root, UID/GID 1000,
dropped capabilities, no new privileges, private PID/IPC namespaces and no network.
Private temporary filesystems bound work, browser state, shared memory and output.
Only validated job input is staged and mounted read-only; a project or home is
never mounted. Input is limited to 64 MiB, rejects links and special files, and
bounds entry count and depth.

The native private Unix API starts a fixed packaged job launcher. It validates
attached stream framing and final exit status. Only one reviewed output basename
is retrieved from the guest, through a bounded single-file archive. Existing PNG,
media or report validators then run before project publication. Output is limited
to 32 MiB. Queueing, startup, staging, execution and retrieval share the tool's
deadline. Cancellation evicts the captured immutable container ID; cleanup has a
separate bounded grace period. A cancelled create can materialize later on the
remote engine; `mirket environment recover` removes only exactly labeled workers
whose private input directory no longer exists. Active workers and service volumes
are preserved. Missing-stage recovery deliberately does not delete crash leftovers
that still have a staging directory. Failed workers are recreated on the next call.
Container isolation does not make untrusted rendering or a useful-looking result
correct; inspect actual artifacts.

## Measured execution costs

On an Apple M4 Mac with 16 GiB RAM, a matched 960×540, 30 fps, 150-frame Motion
Canvas scene gave these results. Twenty warm frame calls used one persistent MCP
session; p95 uses nearest rank. Installation and filesystem caches were warm, and
the dedicated VM was already running.

| Useful workload | Native macOS | Podman ARM64 Linux |
| --- | ---: | ---: |
| Frame-call median / p95 | 497 / 507 ms | 516 / 537 ms |
| Serialized frame throughput | 2.01 calls/s | 1.93 calls/s |
| 150-frame MP4 export | 1.78 s | 2.07 s |
| Fresh MCP startup median, five sessions | 1.76 s | 2.28 s |

The observed frame median increased 3.7%; video export increased 16.7%. Native
macOS and Linux differ in encoder builds, fonts and isolation. These measurements
do not isolate virtualization overhead. Two independent sessions produced eight
frame outputs at 3.17 calls/s; a third was refused before creating another worker.
Simultaneous renderer and Penpot peak workloads have not been capacity tested.

The primary worker's cgroup recorded a 387 MB peak and 19.45 CPU seconds across
readiness, the frame/video workload and a fixed final counter probe, with no OOM.
This excludes host controller and VM memory. Native process RSS and Linux cgroup
memory have different accounting and do not support a memory-efficiency ratio.
A controlled stopped-VM start reached its usable API in 9.79 s; package calls do
not incur that boot on each request. Full uncached download-to-first-output time
was not measured as one matched experiment.

The two renderer images contain about 769 MB and 750 MB of uncompressed content,
with fourteen of fifteen layers shared. These figures are neither download size
nor incremental disk use. The library-builder download is about 938 MB and the
VM boot archive about 938 MB. Penpot retains its official service closures; its
exporter is substantially larger than the trimmed renderer images. See
[Penpot](penpot.md) for its separately measured useful calls and resource limits.

The lifetime and file-sharing probes informed bounded reuse and narrow staging.
Earlier contended no-op tests favored reuse at the median but had worse p95;
they do not establish a general tail-latency improvement. A warm 32 MiB guest
copy plus filesystem sync took 36.8 ms median through the private share versus
20.0 ms on VM-local storage. Native file-sync results use a different method.
Inputs stay narrowly staged; generated intermediates use private guest temporary
storage, and only validated final output crosses back to the project.

## Services and native exceptions

Penpot's database, Valkey, backend, exporter and frontend have purpose-specific
users, private temporary storage and resource limits. They use one internal
network and VM-owned volumes. Only the frontend is published on host loopback.
No host directory, credential store or Podman socket is mounted. Real backend JSON
is required for readiness. See [Penpot](penpot.md) for data ownership and lifecycle.

Inkscape's supported runtime is a signed macOS application closure. A Linux
container cannot execute its application bundle. Native browser tools need the
selected project's local development server; Penpot's browser needs its published
frontend. Their native paths are explicit exceptions, not container isolation
claims. Browser profiles and staged inputs remain private and bounded.

## Platforms and provenance

The reviewed private host client is Podman 6.1.3; its official package and client hashes, and the ARM64 AppleHV boot artifact identity, are recorded in [container-runtime.json](../registry/package-locks/container-runtime.json). Podman and the machine-image build repository use Apache-2.0; operating-system components keep their own terms and bundled legal material. These source links do not establish complete VM source or license coverage.

The controller is enabled on macOS ARM64. ARM64/x64 Linux payload digests are
pinned independently; they do not establish Linux host readiness. Windows sharing,
controller execution and simulator access are unverified and held. Mirket selects native-architecture images and disables Rosetta; it does not use architecture
emulation. VM shares on another platform must be observed before it is
marked ready.

Linux FFmpeg 9.0.2's reviewed binaries include `--enable-nonfree`. These local
images must not be redistributed. Source/build configuration and notices are
retained. Publishing requires a separately reviewed redistributable encoding
closure; local runtime preparation is not publication authorization.
