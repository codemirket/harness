# Agent runtime

Mem0 OSS 2.2.1 (mem0ai/mem0), Apache-2.0, owns memory storage, retrieval and history.
LangGraph 1.2.16 and langgraph-checkpoint-sqlite 3.1.2 (langchain-ai/langgraph), MIT,
own durable workflow transitions and checkpoints. FastEmbed 0.9.0 (qdrant/fastembed),
Apache-2.0, runs local ONNX embeddings. Qdrant client stores vectors locally; no
Qdrant server or hosted Mem0 service is required.

The multilingual MiniLM model and Qdrant BM25 assets are pinned in model.json and
bm25.json with exact revisions, file sizes and SHA-256 digests. Both are Apache-2.0.
spaCy and its pinned English en_core_web_sm model provide optional keyword/entity
processing locally. The English NLP model does not establish universal language
coverage. The dense model has a 128-token window; overlong facts must be split.

Python distributions retain their own license metadata in the image. Exact
dependency versions and accepted distribution hashes are in requirements.lock.
Upstream source notices and license copies are retained by Mirket. No upstream
bootstrap, signup, hosted migration, plugin installer or runtime model downloader
is executed. Model retrieval occurs only during explicit package preparation.

Owned adapters disable Mem0 inference: the authorized host supplies concise facts.
The custom Mem0 provider configs are fixed in code to accommodate the pinned
factory/config-validator mismatch; callers cannot supply providers or class paths.
LangGraph is used as the library, without Agent Server or LangSmith hosting.
