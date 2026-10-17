FROM docker.io/library/node@sha256:b20fe038d9592822dd534600a1ddf467e985178aa3fed60ec441a056c2cc45bd

RUN node -e 'require("node:fs").writeFileSync("/tmp/node-root-ca.pem", require("node:tls").rootCertificates.join("\n") + "\n", {mode: 0o644})' \
    && printf 'Types: deb\nURIs: https://snapshot.debian.org/archive/debian/20261005T000000Z/\nSuites: bookworm bookworm-updates\nComponents: main\nCheck-Valid-Until: no\n\nTypes: deb\nURIs: https://snapshot.debian.org/archive/debian-security/20261005T000000Z/\nSuites: bookworm-security\nComponents: main\nCheck-Valid-Until: no\n' > /etc/apt/sources.list.d/debian.sources \
    && rm -f /etc/apt/sources.list \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem update \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem install -y --no-install-recommends ca-certificates curl xz-utils fonts-dejavu-core chromium ffmpeg procps=2:4.0.2-3 \
    && test -x /bin/ps && /bin/ps -ww -axo pid=,ppid=,pgid=,lstart=,command= >/dev/null \
    && rm -f /tmp/node-root-ca.pem \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/design
COPY tools/package.json tools/package-lock.json ./
RUN npm ci --omit=dev --ignore-scripts --no-audit --no-fund \
    && node -e "import('@resvg/resvg-js').then(()=>import('satori'))" \
    && node node_modules/hyperframes/bin/hyperframes.mjs --help >/dev/null \
    && node_modules/.bin/odiff --help >/dev/null \
    && npm cache clean --force

RUN set -eux; \
    build_arch="$(dpkg --print-architecture)"; \
    case "$build_arch" in \
      arm64) typst_arch=aarch64; typst_sha=5aa8d74a3d906e60ea12a66ac2f37f8eef1b14cbad7182a745e393a10c23dcee;; \
      amd64) typst_arch=x86_64; typst_sha=a6d077d0a95eed5a2eba715b2dae06be954f624ccbf85758a03f389ded33118c;; \
      *) echo "Unsupported architecture: $build_arch" >&2; exit 1;; \
    esac; \
    curl -fL --retry 3 "https://github.com/typst/typst/releases/download/v0.15.1/typst-${typst_arch}-unknown-linux-musl.tar.xz" -o /tmp/typst.tar.xz; \
    echo "${typst_sha}  /tmp/typst.tar.xz" | sha256sum -c -; \
    tar -xJf /tmp/typst.tar.xz -C /tmp; \
    install -m 0755 "/tmp/typst-${typst_arch}-unknown-linux-musl/typst" /usr/local/bin/typst; \
    rm -rf /tmp/typst.tar.xz "/tmp/typst-${typst_arch}-unknown-linux-musl"; \
    typst --version

COPY tools/*.mjs ./
COPY tools/tool-policy.json ./tool-policy.json
COPY tools/tool-budgets.json ./tool-budgets.json
COPY tools/NOTICE.md ./NOTICE.md
COPY penpot-session.mjs ./penpot-session.mjs
RUN mkdir -p /workspace \
    && chmod 0644 /opt/design/*.mjs /opt/design/NOTICE.md /opt/design/package.json /opt/design/package-lock.json /opt/design/tool-policy.json /opt/design/tool-budgets.json \
    && chown node:node /workspace /opt/design
USER node
RUN node --check /opt/design/workspace-server.mjs \
    && node --check /opt/design/penpot-session.mjs \
    && node -e 'const policy = JSON.parse(require("node:fs").readFileSync("/opt/design/tool-policy.json", "utf8")); if (!Object.keys(policy).length) throw Error("Design tool policy is empty")'
WORKDIR /workspace
ENV HYPERFRAMES_NO_UPDATE_CHECK=1 \
    HYPERFRAMES_NO_TELEMETRY=1 \
    HYPERFRAMES_BROWSER_PATH=/usr/bin/chromium \
    HYPERFRAMES_FFMPEG_PATH=/usr/bin/ffmpeg \
    HYPERFRAMES_FFPROBE_PATH=/usr/bin/ffprobe
CMD ["node", "/opt/design/workspace-server.mjs"]
