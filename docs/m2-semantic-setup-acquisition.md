# Setup-acquired semantic payload — macOS ARM64

The approved MiniLM/ONNX Runtime selection now has a first-party setup route.
The Mirket release download may omit its 172,668,906 payload bytes while
retaining the 2,149-byte `runtime/semantic/package-receipt.json` and the full
release manifest. `scripts/install-macos.sh` accepts that receipt-only source,
obtains the exact ONNX Runtime archive and eight selected model files, verifies
archive and individual file sizes/SHA-256 values, and restores the complete
release in private staging. The ordinary Mirket setup transaction verifies the
whole manifest before changing managed destinations. Runtime inference remains
offline and uses only the verified private model and library paths.

The model's four tokenizer/config files are stored beneath `model/onnx/` for
the local FastEmbed contract, but their pinned upstream files reside at the
repository root. The previous `url` metadata incorrectly inserted `/onnx/`
for those files; this was discovered by the real network setup attempt and
corrected in [the authoritative selection](../packaging/admission/m2-semantic-minilm.runtime.json).
Their bytes and model revision did not change. The corrected selection changes
the receipt's manifest digest, so complete release sources must be assembled
with the corrected receipt and a core compiled from the corrected source.

For an offline installation, the Mac installer accepts
`--semantic-runtime-archive /absolute/onnxruntime-osx-arm64-1.31.0.tgz` and
`--semantic-model-root /absolute/selected-model-root`. The model root must
contain the eight selected files at their **local** paths, including the four
`onnx/` copies. These inputs receive the same size and hash checks as network
downloads. Either option may be supplied independently; omitted inputs come
from their fixed HTTPS URLs. No Python, pip, model hub client, install script,
runtime download or host model cache is used by setup or inference.

## Disposable verification on 2026-10-26

- macOS 26.5.2 ARM64: a receipt-only source with `semantic,skills` installed
  successfully using the exact local ONNX Runtime archive and model files.
  `mirket doctor --json` reported healthy registration, guidance, skills and
  release. The installed semantic bytes then passed the two-client MCP smoke:
  a sourced Turkish fact was retrieved through English/Turkish semantic queries.
- A second disposable installation fetched the ONNX Runtime archive and all
  eight model files over HTTPS. Its active release ID was
  `sha256:878a4ecc6bc091297e83e966a95922a071649e32b6589e749a3d6c50cd099e62`;
  doctor reported `semantic,skills` healthy. The acquired payload passed the
  two-client MCP smoke after the URL correction. This is the same release
  identity generated from the complete corrected source.
- A one-byte change to the supplied `onnx/config.json` was rejected before
  update. A linked `onnx/` directory in a supplied model root was also
  refused. Doctor still reported the prior healthy release with the same ID.
- A combined `browser,semantic,skills` source omitted both bulky payloads.
  The installer restored the pinned browser archive and supplied semantic
  inputs in one private staging copy. Setup, doctor and an installed MCP
  `runtime_status` call passed for release
  `sha256:a9e6160bc81354c1f1cd078ffb2c29efbd6c94e4624b86df7efb8825da5715c0`.
  A separate browser-only installation remained healthy after the shared
  staging-path change.
- `sh -n scripts/install-macos.sh` and
  `python3 -m unittest tests.test_semantic_installer_pins -v` passed. The latter
  binds the installer's fixed archive/member/model paths, sizes, hashes and
  receipt hash to the authoritative admission.

Private input, source and result evidence is retained under
`~/.codex/task-state/harness-rewrite-implementation/semantic-setup-acquisition-20261026/`.
The disposable homes are separate from the user's `.mirket` installation and
Codex settings. The installed test is a backend MCP journey, not Codex Desktop
GUI activation. Windows semantic payloads still lack an admitted app-local CRT
closure and native execution; this Mac route does not qualify them.
