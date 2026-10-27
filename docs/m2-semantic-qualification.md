# Semantic memory qualification

The selected macOS ARM64 model is `sentence-transformers/paraphrase-multilingual-MiniLM-L12-v2` at revision `e8f8c211226b894fcb81acc59f3b34ba3efd5f42`, using the pinned dynamic-int8 ARM64 graph and the private ONNX Runtime 1.31.0 CPU provider. [The runtime manifest](../packaging/admission/m2-semantic-minilm.runtime.json) fixes every model/tokenizer/runtime file, source URL, size and SHA-256. The pinned model card declares Apache-2.0; the private ORT license and third-party notices are retained. No runtime download or host model fallback is used.

The frozen 122-query evaluation ran against the production Rust source and freshly assembled private package on macOS ARM64. It passed every observable gate with zero eligibility or coverage errors:

| Measure | Observed |
| --- | ---: |
| Hybrid Recall@5 | 0.9875 |
| Hybrid nDCG@5 | 0.9448 |
| English → English Recall@5 / nDCG@5 | 1.0000 / 0.9247 |
| English → Turkish | 1.0000 / 0.9226 |
| Turkish → English | 0.9375 / 0.9375 |
| Turkish → Turkish | 1.0000 / 0.9846 |
| Identifier/path top-1 | 18/18 |
| Ambiguous evidence complete@5 | 8/8 |
| Warm end-to-end p95 | 227.5 ms |
| Cold model load / indexing | 5.27 s / 28.83 s |
| Peak process RSS (`/usr/bin/time -l`) | 1,297,825,792 bytes |

The complete report SHA-256 is `9a903ddc6039b2c5a56e21335865236d2ec41d63725783f39bf397b035fa7d50`; the frozen runner SHA-256 is `3cb69ebebf99c8bcb30bf7c20d19a94913e484c8b6d03c22e47a5fdf5790a271`. The selected model identity binds its revision, graph hash, tokenizer artifacts, runtime and numeric contract. The evaluation corpus and thresholds are in [`memory_retrieval.rs`](../tests/memory_retrieval.rs) and its [fixture](../tests/fixtures/memory-retrieval.json). An installed disposable release then passed exact package verification, an indexed canonical save, cross-language semantic recall and `doctor` using [`semantic-installed-smoke.mjs`](../tests/semantic-installed-smoke.mjs). The broader [memory smoke](../tests/memory-smoke.mjs) passed correction, forgetting, pending-index recovery and managed backup paths with the same package.

The native warm-supervisor fixture [`warm-embedding-smoke.py`](../tests/warm-embedding-smoke.py) also passed with the selected package and a network-denied model subtree. It observed one stable model child across two requests (317.5 and 224.9 ms), then confirmed typed `parent_gone` cleanup, child exit, heavy-slot marker removal and empty scratch removal 138 ms after closing parent stdin. The cold ready observation was 5.80 s. Its task-state report SHA-256 is `5db3d296d47eda45d9bab55421db9ca6c679060d27808a49a8db8956ca712e9f`; sampled RSS is not a peak or an idle-memory result.

The separate real 300-second idle run also passed: the same model child remained owned while idle, then returned typed `idle_expired` with confirmed cleanup after 300.129 s. Its report SHA-256 is `f354475bdbb11c95bcdcc5d424bac08236fa0379b34516011f87227b93ee8a0a`.

The [installed semantic smoke](../tests/semantic-installed-smoke.mjs) was then extended to two simultaneous MCP server connections to the same disposable release and project. One client indexed a sourced Turkish fact; both recalled it through English/Turkish semantic queries. On macOS 26.5.2 ARM64 with 16 GiB physical RAM, `ps` sampled the exact descendants of the two server PIDs. A second run kept both connections open beyond the 300-second model idle deadline:

| Sample | Summed process-tree RSS | Observed descendants |
| --- | ---: | --- |
| Before semantic use | 28,528 KiB | Two server processes only |
| After both clients used semantic recall | 1,906,928 KiB | Each server, supervisor and native model leaf |
| 30 seconds idle | 1,548,976 KiB | Both model leaves still resident |
| 310 seconds idle | 40,288 KiB | Both model child trees exited; servers remained |

The report is retained outside shipped source at `~/.codex/task-state/harness-rewrite-implementation/dependencies/semantic/two-client-installed-20261026.json`, SHA-256 `01a433809b5e2839b0096694c62f2aaf1b7d530182fed0a0bc11eb93e6e92616`. Its release ID is `sha256:83cd280748a6ab307a69b93e218d8b4577cd707905cb78a60c73dfa90c1c3f6a`; the tested core and Node adapter SHA-256 values were `75e557a48ee52559b9811bd6796e1133d5821ed6f0a119e000bc1b6a7f3d4239` and `3ea3e1a548b5be6b70ece6487c7ce34f05ead1b8ecf1e824eabf8a4637f8be9e`. Summing per-process RSS double-counts shared physical pages and these points are not peaks or latency distributions. The test did not run a concurrent design/navigation/video job, simulate system memory pressure, or use actual Codex chats.

The 2026-10-26 runs above establish the frozen local fixture, installed two-client route and observed idle retirement, not general answer support. [The later simplification run](simplification-qualification-2026-10-27.md) measured concurrent semantic/Design/navigation development load and warm end-to-end query inference plus hybrid search at 10,000 synthetic one-chunk heads. Its p95 was 404.8 ms across 30 sequential queries on a 16 GiB macOS ARM64 host. The same run rechecked the frozen bilingual quality fixture after FTS query optimization. The scale fixture does not qualify 100,000 heads, natural multi-chunk distribution, installed-client latency, unseen answer support or abstention. Windows native loading and actual Codex Desktop GUI activation remain unmeasured. Windows x64/ARM64 package assembly is deliberately refused until the licensed private CRT closure and native tests are complete. The user's active installation remains unchanged.
