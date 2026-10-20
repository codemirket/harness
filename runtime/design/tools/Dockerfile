FROM docker.io/library/node@sha256:b20fe038d9592822dd534600a1ddf467e985178aa3fed60ec441a056c2cc45bd AS guard-build
RUN node -e 'require("node:fs").writeFileSync("/tmp/node-root-ca.pem", require("node:tls").rootCertificates.join("\n") + "\n", {mode: 0o644})' \
    && printf 'Types: deb\nURIs: https://snapshot.debian.org/archive/debian/20261020T000000Z/\nSuites: bookworm bookworm-updates\nComponents: main\nCheck-Valid-Until: no\n\nTypes: deb\nURIs: https://snapshot.debian.org/archive/debian-security/20261020T000000Z/\nSuites: bookworm-security\nComponents: main\nCheck-Valid-Until: no\n' > /etc/apt/sources.list.d/debian.sources \
    && rm -f /etc/apt/sources.list \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem update \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem -y upgrade \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem install -y --no-install-recommends ca-certificates gcc libc6-dev curl xz-utils \
    && rm -f /tmp/node-root-ca.pem \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY tools/runtime-guard.c ./
RUN cc -O2 -Wall -Wextra -Werror runtime-guard.c -o /mirket-runtime-guard

# Rebuild the existing esbuild dependency with a patched Go runtime and retain
# Go module/build metadata. The npm prebuilt binary strips that scanner input.
RUN set -eux; \
    build_arch="$(dpkg --print-architecture)"; \
    case "$build_arch" in \
      arm64) go_sha=a06e99309dbcbdca1a8ccd2282c3ee146e55930886e26b4f1193269587165f49;; \
      amd64) go_sha=65efbc80fdc1fa2cd2631cad5b481777a68e8725f34e89f50fc6ac714e379b40;; \
      *) exit 1;; \
    esac; \
    curl -fL --retry 3 "https://go.dev/dl/go1.27.3.linux-${build_arch}.tar.gz" -o /tmp/go.tar.gz; \
    echo "${go_sha}  /tmp/go.tar.gz" | sha256sum -c -; \
    tar -xzf /tmp/go.tar.gz -C /usr/local; \
    curl -fL --retry 3 https://codeload.github.com/evanw/esbuild/tar.gz/refs/tags/v0.28.2 -o /tmp/esbuild.tar.gz; \
    echo '300162f899361d9f0263065f8728ef9086dcfc1a3d36bcbc4e0abe5de9c58bf8  /tmp/esbuild.tar.gz' | sha256sum -c -; \
    tar -xzf /tmp/esbuild.tar.gz -C /build; \
    cd /build/esbuild-0.28.2; \
    GOTOOLCHAIN=local CGO_ENABLED=0 /usr/local/go/bin/go build -mod=readonly -trimpath -buildvcs=false -o /esbuild ./cmd/esbuild; \
    /usr/local/go/bin/go version -m /esbuild > /esbuild-build.txt; \
    grep -q 'go1.27.3' /esbuild-build.txt; \
    test "$(/esbuild --version)" = '0.28.2'; \
    mkdir /esbuild-licenses; \
    cp LICENSE.md /esbuild-licenses/esbuild-LICENSE; \
    cp internal/xxhash/LICENSE.txt /esbuild-licenses/xxhash-LICENSE; \
    cp /usr/local/go/LICENSE /esbuild-licenses/go-LICENSE; \
    cp /root/go/pkg/mod/golang.org/x/sys@v0.0.0-20220715151400-c0bba94af5f8/LICENSE /esbuild-licenses/x-sys-LICENSE

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


FROM docker.io/library/node@sha256:b20fe038d9592822dd534600a1ddf467e985178aa3fed60ec441a056c2cc45bd

RUN node -e 'require("node:fs").writeFileSync("/tmp/node-root-ca.pem", require("node:tls").rootCertificates.join("\n") + "\n", {mode: 0o644})' \
    && printf 'Types: deb\nURIs: https://snapshot.debian.org/archive/debian/20261020T000000Z/\nSuites: bookworm bookworm-updates\nComponents: main\nCheck-Valid-Until: no\n\nTypes: deb\nURIs: https://snapshot.debian.org/archive/debian-security/20261020T000000Z/\nSuites: bookworm-security\nComponents: main\nCheck-Valid-Until: no\n' > /etc/apt/sources.list.d/debian.sources \
    && rm -f /etc/apt/sources.list \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem update \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem -y upgrade \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem install -y --no-install-recommends ca-certificates fonts-dejavu-core chromium ffmpeg \
    && rm -f /tmp/node-root-ca.pem \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/design
COPY tools/package.json tools/package-lock.json ./
RUN npm ci --omit=dev --ignore-scripts --no-audit --no-fund \
    && node -e "import('@resvg/resvg-js').then(()=>import('satori'))" \
    && node node_modules/@open-pencil/cli/bin/openpencil.js --help >/dev/null \
    && node node_modules/hyperframes/bin/hyperframes.mjs --help >/dev/null \
    && node_modules/.bin/odiff --help >/dev/null \
    && npm cache clean --force \
    && rm -rf /usr/local/lib/node_modules /usr/local/bin/npm /usr/local/bin/npx /usr/local/bin/corepack /usr/local/bin/pnpm /usr/local/bin/yarn /usr/local/bin/yarnpkg

COPY --from=guard-build /esbuild /opt/design/esbuild-patched
COPY --from=guard-build /esbuild-build.txt /opt/design/esbuild-build.txt
COPY --from=guard-build /esbuild-licenses /opt/design/esbuild-licenses
RUN node -e 'const fs=require("node:fs"); const p=`/opt/design/node_modules/@esbuild/linux-${process.arch}/bin/esbuild`; fs.copyFileSync("/opt/design/esbuild-patched",p); fs.chmodSync(p,0o755); fs.unlinkSync("/opt/design/esbuild-patched")'

COPY --from=guard-build /usr/local/bin/typst /usr/local/bin/typst
COPY --from=guard-build /mirket-runtime-guard /opt/design/runtime-guard
COPY tools/*.mjs ./
COPY tools/chromium-launcher ./
RUN chmod 0755 /opt/design/chromium-launcher
COPY tools/openpencil-tools.json ./openpencil-tools.json
COPY tools/tool-policy.json ./tool-policy.json
COPY tools/tool-budgets.json ./tool-budgets.json
COPY tools/NOTICE.md ./NOTICE.md
RUN mkdir -p /workspace \
    && chmod 0644 /opt/design/*.mjs /opt/design/NOTICE.md /opt/design/package.json /opt/design/package-lock.json /opt/design/tool-policy.json /opt/design/tool-budgets.json /opt/design/openpencil-tools.json \
    && chown node:node /workspace
USER node
RUN node --check /opt/design/workspace-server.mjs \
    && node -e 'const policy = JSON.parse(require("node:fs").readFileSync("/opt/design/tool-policy.json", "utf8")); if (!Object.keys(policy).length) throw Error("Design tool policy is empty")'
WORKDIR /workspace
ENV HYPERFRAMES_NO_UPDATE_CHECK=1 \
    HYPERFRAMES_NO_TELEMETRY=1 \
    HYPERFRAMES_BROWSER_PATH=/usr/bin/chromium \
    HYPERFRAMES_FFMPEG_PATH=/usr/bin/ffmpeg \
    HYPERFRAMES_FFPROBE_PATH=/usr/bin/ffprobe
CMD ["node", "/opt/design/workspace-server.mjs"]
