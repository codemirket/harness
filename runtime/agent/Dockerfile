FROM docker.io/library/python:3.13-slim-bookworm@sha256:a1165e272e578941b84abc79e4ab38a0305cd12803a5c4247979ac7655f4d641
ENV PYTHONDONTWRITEBYTECODE=1 PYTHONUNBUFFERED=1 MEM0_TELEMETRY=false LANGSMITH_TRACING=false LANGCHAIN_TRACING_V2=false HF_HUB_OFFLINE=1 FASTEMBED_CACHE_PATH=/opt/model-cache MIRKET_MODEL_PATH=/opt/models MEM0_DIR=/tmp/mem0 OMP_NUM_THREADS=2 TOKENIZERS_PARALLELISM=false LANGGRAPH_STRICT_MSGPACK=true
WORKDIR /opt/agent
COPY requirements.lock /opt/agent/requirements.lock
RUN python -m pip install --require-hashes --only-binary=:all: --no-cache-dir -r requirements.lock \
    && python -m pip check \
    && python -m pip uninstall --yes pip \
    && python -c "import importlib.util; assert importlib.util.find_spec('pip') is None"
COPY bridge.py memory.py workflow.py lifecycle.py model.json bm25.json NOTICE.md /opt/agent/
COPY tests /opt/agent/tests
COPY licenses /opt/agent/licenses
COPY model /opt/models
COPY model-cache /opt/model-cache
RUN python -c "import mem0, langgraph; import spacy; spacy.load('en_core_web_sm'); from fastembed import SparseTextEmbedding; SparseTextEmbedding('Qdrant/bm25')" && chmod -R a+rX /opt/agent /opt/models /opt/model-cache
USER 1000:1000
ENTRYPOINT ["python", "-B", "/opt/agent/bridge.py"]
