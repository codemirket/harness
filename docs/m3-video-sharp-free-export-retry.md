# Sharp-free development export retry — 2026-10-26

**The corrected task profile passed its parser and packet-free policy controls,
but the fresh resource guard refused before export launch.** No Mirket, Node,
Chromium or media process started; no video operation or output was created.
The native lane was released without another guard poll or retry. Sharp-free
rendering remains unqualified.

This is separate evidence from the preserved
[first export attempt](m3-video-sharp-free-export-qualification.md). That failed
profile, receipt and report remain byte identical. The
[dependency review](m3-video-sharp-dependency-review.md) still establishes only
its successful Node import/configuration/job-creation phase. Production
installed loading still supplies `video: None`; this retry proposed only
`mirket dev serve`, with no installed/release qualification or manifest change.

## Saved profile and discriminating controls

The corrected copy is:

`/Users/nazmiyilmaz/.codex/task-state/harness-rewrite-implementation/video-sharp-free-export-retry-20261026/offline-localhost.sb`

Its **280 bytes** have SHA-256
`52cdc993dc30ba3994bdc0417676687c6f29bd4ee7d0f15d01c80f7acfff688c`.
The only profile change replaces the three unsupported numeric
`127.0.0.1:*` address filters with macOS-supported `localhost:*`. Line 4 is now:

```scheme
(allow network-outbound (remote ip "localhost:*"))
```

The exact saved file passed
`sandbox-exec -f <saved-profile> /usr/bin/true` with exit zero and empty stderr.
Subsequent controls used that same profile and existing `/usr/bin/python3`:

| Control | Observed result | Network effect |
| --- | --- | --- |
| Bind `127.0.0.1` on an ephemeral port | Allowed | Socket immediately closed; no listen/connect/send. |
| UDP association to `127.0.0.1:9` | Allowed | Local loopback association only; no send. |
| UDP association to `127.0.0.2:9` | Denied with `EPERM` / errno 1 | Local loopback only; no send or external route. |

The paired UDP checks distinguish an allowed destination from a refused one
through the same API, without an external network effect. They establish these
specific saved-profile observations, not every networking path or product
containment.

An earlier expected-denial control for wildcard bind `0.0.0.0:0` **failed**:
the bind was allowed. That hypothesis and its assertion failure are retained.
The socket was closed without listen/connect/send, and the profile was not
changed to hide the result. Binding a wildcard address was therefore not used
as evidence of outbound denial. The later paired association controls passed.

## Fresh guard and exact stopped boundary

After the parent explicitly confirmed that Chromium and the Rust/release-store
work had released the native lane, the Chromium worker was notified to defer
native work. On macOS 26.5.2 ARM64, the single fresh preflight at
**08:01:52 UTC** recorded:

| Guard input | Observation | Required condition |
| --- | --- | --- |
| Normal-pressure predicate | **false** | Successful `sysctl` with stripped pressure-level output `1`. |
| System memory free | **50%** | At least 50%. |
| Disk free | **47,123,972,096 bytes** | At least 26,843,545,600 bytes / 25 GiB. |
| Task data | **2,399,529 bytes** | At most 2,147,483,648 bytes / 2 GiB. |

The failed Normal predicate refused the attempt before runtime identity checks
or subprocess creation. The runner retained the predicate, but **did not retain
the raw `sysctl` return code/stdout**. It cannot distinguish a non-Normal level
from a failed command and does not establish `Warning=2`. No additional pressure
query was made to replace that missing observation. No threshold was relaxed.

There are zero outgoing or incoming MCP frames, no operation receipt, an empty
private state directory and no worker trace or MP4. The prepared six-package
Sharp-denial instrumentation never ran. Owned-path process inspection found no
remaining process. The parent and Chromium worker received explicit native-lane
release messages. This was one refused launch preflight, not a failed render
and not a successful Sharp-free negative control.

## Frozen candidate and prepared-input drift

The existing reduced candidate remains **82 package instances, 5,892 files,
65,459,748 bytes**, with manifest SHA-256
`589fba4abecfd7d80966e22330cb914ddf01c2abc6025c58ed3467f64de6c012`.
Every file was rehashed after the refusal. Its six Sharp subtree packages remain
absent; its source and installed runtime selection were not changed.

The separate retry project contains the same seven input files as the first
attempt: editable motion composition, local font, Anime.js, two WAV tracks,
checkerboard PNG and supplied caption JSON. The proposed operation remains one
two-second 640×360, 12 fps, 24-frame development MP4. Input hashes match the
earlier prepared fixture; no output/content, audio or cancellation check ran.

A preliminary inline identity-refresh script failed Python syntax before any
statement executed. Its diagnostic is retained. The runner then reached only
the resource preflight; it did not validate or execute its prepared tools.
Static post-refusal inspection found that the parent build had replaced the
development Mirket binary while the native lane was held:

| Mirket identity | Bytes | SHA-256 |
| --- | ---: | --- |
| Prepared earlier identity | 86,309,272 | `d338f769e0ce95d45741dfa20dcb57d6512a9af232d11a3a003962ea668cf1de` |
| Observed after refusal | 86,306,504 | `b14034bc5693e3a8039179cc444b13a87090af593412a3a029ecb0790a216324` |

The explicit Node, compiled video/generic adapters, Chromium launcher and media
files still match their prepared hashes. Neither Mirket identity was invoked
by this attempt. Any later authorized run must refresh and bind the intended
binary identity before launch, coordinate the lane and pass a fresh strict
guard. The already retained candidate and prior failure must remain intact.

Actual development exports for the four profiles and cancellation remain open.
The separate installed Video component/loader/inventory work, recipient source
and rights, final component selection and platform gates also remain open. The
passing profile controls do not remove the Sharp/libvips release gate.

Private evidence is under
`~/.codex/task-state/harness-rewrite-implementation/video-sharp-free-export-retry-20261026/`:
the corrected profile and parser result, all passing and failed local policy
controls, prepared project/wrapper/hook/runner, single guard sample, empty RPC
record, run result and post-refusal identities. `final-receipt.json` is
**12,280 bytes**, SHA-256
`518983865ec9fc5bc85e48172a778f52a510f457a82ae47459d6cc1139c4956b`.
It binds the candidate, controls, preserved earlier evidence, failed guard and
no-process outcome. This report is the only repository file added by this
follow-up; the earlier reports, README, runtime source and manifests were not
edited.
