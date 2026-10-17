FROM docker.io/library/python@sha256:16bf2b5c59a08523519d3c8589deca849285dee1d62a3653e124c3dba340f15e

# libtorchaudio needs libgomp.so.1 even for the CPU bitmap workflow. The
# immutable Debian snapshot matches the pinned trixie base image's source date.
RUN printf 'Types: deb\nURIs: https://snapshot.debian.org/archive/debian/20261005T000000Z/\nSuites: trixie trixie-updates\nComponents: main\nCheck-Valid-Until: no\nSigned-By: /usr/share/keyrings/debian-archive-keyring.pgp\n\nTypes: deb\nURIs: https://snapshot.debian.org/archive/debian-security/20261005T000000Z/\nSuites: trixie-security\nComponents: main\nCheck-Valid-Until: no\nSigned-By: /usr/share/keyrings/debian-archive-keyring.pgp\n' > /etc/apt/sources.list.d/debian.sources \
    && apt-get update \
    && apt-get install -y --no-install-recommends libgomp1=14.2.0-19 \
    && rm -rf /var/lib/apt/lists/*

# Source archive is downloaded and SHA-256 checked by Mirket before the build.
COPY comfyui-source.tar.gz /tmp/comfyui-source.tar.gz
COPY requirements.lock /tmp/requirements.lock
COPY exclude-3d.py /tmp/exclude-3d.py
RUN mkdir -p /opt/comfyui/headless && tar -xzf /tmp/comfyui-source.tar.gz --strip-components=1 -C /opt/comfyui \
    && rm /tmp/comfyui-source.tar.gz \
    && python /tmp/exclude-3d.py /opt/comfyui \
    && python -m compileall -q /opt/comfyui/nodes.py /opt/comfyui/comfy /opt/comfyui/comfy_api /opt/comfyui/comfy_extras /opt/comfyui/comfy_api_nodes \
    && python -m pip install --no-cache-dir --only-binary=:all: --require-hashes --no-deps -r /tmp/requirements.lock \
    && python -m pip check \
    && rm /tmp/exclude-3d.py

WORKDIR /opt/comfyui
ENV HF_HUB_OFFLINE=1 TRANSFORMERS_OFFLINE=1 HF_HOME=/tmp/hf
EXPOSE 8188
CMD ["python", "main.py", "--cpu", "--listen", "0.0.0.0", "--port", "8188", "--disable-auto-launch", "--disable-all-custom-nodes", "--front-end-root", "/opt/comfyui/headless", "--base-directory", "/workspace", "--input-directory", "/workspace/input", "--output-directory", "/workspace/output", "--temp-directory", "/tmp/comfyui"]
