FROM docker.io/library/golang@sha256:fbb7ae40b62ff8afe31662df570d39b4f5052ac32b90dae8e60d7657a20a74f4 AS d2-build
ENV GOTOOLCHAIN=local CGO_ENABLED=0
RUN curl -fLsS https://codeload.github.com/d2lang/d2/tar.gz/refs/tags/v0.9.0 -o /tmp/d2.tar.gz \
    && echo '1256ad3907bceb4fcee7ed40d17c5726f8b602eea900e94500ba3352e96febbc  /tmp/d2.tar.gz' | sha256sum -c - \
    && mkdir /source && tar -xzf /tmp/d2.tar.gz --strip-components=1 -C /source
WORKDIR /source
COPY tools/d2.go.mod ./go.mod
COPY tools/d2.go.sum ./go.sum
RUN go mod download && go mod verify \
    && go build -mod=readonly -trimpath -buildvcs=false -ldflags '-X github.com/d2lang/d2/lib/version.Version=v0.9.0' -o /d2 . \
    && go version -m /d2 > /d2-build.txt

FROM docker.io/library/node@sha256:43b9ba1142b8dcf33fa8f922ca1be85714adc444e0586c7cf7726daaeabc1bea AS guard-build
RUN node -e 'require("node:fs").writeFileSync("/tmp/node-root-ca.pem", require("node:tls").rootCertificates.join("\n") + "\n", {mode: 0o644})' \
    && printf 'Types: deb\nURIs: https://snapshot.debian.org/archive/debian/20261023T000000Z/\nSuites: trixie trixie-updates\nComponents: main\nCheck-Valid-Until: no\n\nTypes: deb\nURIs: https://snapshot.debian.org/archive/debian-security/20261023T000000Z/\nSuites: trixie-security\nComponents: main\nCheck-Valid-Until: no\n' > /etc/apt/sources.list.d/debian.sources \
    && rm -f /etc/apt/sources.list \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem update \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem upgrade -y \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem install -y --no-install-recommends ca-certificates gcc libc6-dev curl unzip xz-utils \
    && rm -f /tmp/node-root-ca.pem \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY tools/runtime-guard.c ./
COPY tools/browser.json ./
RUN cc -O2 -Wall -Wextra -Werror runtime-guard.c -o /mirket-runtime-guard

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

RUN set -eux; \
    build_arch="$(dpkg --print-architecture)"; \
    case "$build_arch" in \
      arm64) browser_key=arm64;; \
      amd64) browser_key=amd64;; \
      *) echo "Unsupported architecture: $build_arch" >&2; exit 1;; \
    esac; \
    browser_url="$(node -e 'const fs=require("node:fs"); const d=JSON.parse(fs.readFileSync(process.argv[1], "utf8")); process.stdout.write(d.archives[process.argv[2]].url)' ./browser.json "$browser_key")"; \
    browser_sha="$(node -e 'const fs=require("node:fs"); const d=JSON.parse(fs.readFileSync(process.argv[1], "utf8")); process.stdout.write(d.archives[process.argv[2]].sha256)' ./browser.json "$browser_key")"; \
    browser_bytes="$(node -e 'const fs=require("node:fs"); const d=JSON.parse(fs.readFileSync(process.argv[1], "utf8")); process.stdout.write(String(d.archives[process.argv[2]].bytes))' ./browser.json "$browser_key")"; \
    browser_archive="$(node -e 'const fs=require("node:fs"); const d=JSON.parse(fs.readFileSync(process.argv[1], "utf8")); process.stdout.write(d.archives[process.argv[2]].archive)' ./browser.json "$browser_key")"; \
    curl -fL --retry 3 --retry-all-errors --connect-timeout 20 --max-time 600 "$browser_url" -o "/tmp/$browser_archive"; \
    test "$(stat -c %s "/tmp/$browser_archive")" = "$browser_bytes"; \
    echo "$browser_sha  /tmp/$browser_archive" | sha256sum -c -; \
    mkdir -p /opt/design/chrome; \
    unzip -q "/tmp/$browser_archive" -d /opt/design/chrome; \
    browser_executable="$(node -e 'const fs=require("node:fs"); const d=JSON.parse(fs.readFileSync(process.argv[1], "utf8")); process.stdout.write(d.archives[process.argv[2]].executable)' ./browser.json "$browser_key")"; \
    test -x "/opt/design/chrome/$browser_executable"; \
    rm -f "/tmp/$browser_archive"; \
    rm -f ./browser.json


FROM docker.io/library/node@sha256:43b9ba1142b8dcf33fa8f922ca1be85714adc444e0586c7cf7726daaeabc1bea

RUN node -e 'require("node:fs").writeFileSync("/tmp/node-root-ca.pem", require("node:tls").rootCertificates.join("\n") + "\n", {mode: 0o644})' \
    && printf 'Types: deb\nURIs: https://snapshot.debian.org/archive/debian/20261023T000000Z/\nSuites: trixie trixie-updates\nComponents: main\nCheck-Valid-Until: no\n\nTypes: deb\nURIs: https://snapshot.debian.org/archive/debian-security/20261023T000000Z/\nSuites: trixie-security\nComponents: main\nCheck-Valid-Until: no\n' > /etc/apt/sources.list.d/debian.sources \
    && rm -f /etc/apt/sources.list \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem update \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem upgrade -y \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem install -y --no-install-recommends \
       bubblewrap tini ca-certificates fonts-dejavu-core fonts-liberation \
       libasound2t64 libatk-bridge2.0-0t64 libatk1.0-0t64 libatspi2.0-0t64 \
       libcairo2 libcups2t64 libdbus-1-3 libdrm2 libgbm1 libglib2.0-0t64 \
       libgtk-3-0t64 libnspr4 libnss3 libpango-1.0-0 libwayland-client0 \
       libx11-6 libx11-xcb1 libxcb1 libxcomposite1 libxdamage1 libxext6 \
       libxfixes3 libxkbcommon0 libxrandr2 libxshmfence1 \
    && rm -f /tmp/node-root-ca.pem \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /opt/design
COPY tools/package.json tools/package-lock.json ./
RUN npm ci --omit=dev --ignore-scripts --no-audit --no-fund \
    && node -e "import('@resvg/resvg-js').then(()=>import('satori'))" \
    && node node_modules/@open-pencil/cli/bin/openpencil.js --help >/dev/null \
    && node_modules/.bin/odiff --help >/dev/null \
    && npm cache clean --force \
    && rm -rf /usr/local/lib/node_modules /usr/local/bin/npm /usr/local/bin/npx /usr/local/bin/corepack /usr/local/bin/pnpm /usr/local/bin/yarn /usr/local/bin/yarnpkg


COPY --from=guard-build /usr/local/bin/typst /usr/local/bin/typst
COPY --from=d2-build /d2 /usr/local/bin/d2
COPY --from=d2-build /d2-build.txt /opt/design/d2-build.txt
COPY --from=d2-build /source/LICENSE.txt /opt/design/licenses/d2-LICENSE
COPY --from=d2-build /source/THIRD_PARTY_NOTICES.txt /opt/design/licenses/d2-NOTICES
COPY --from=guard-build /mirket-runtime-guard /opt/design/runtime-guard
COPY --from=guard-build /opt/design/chrome /opt/design/chrome
COPY tools/*.mjs ./
COPY tools/chromium-launcher ./
COPY tools/browser.json ./
RUN chmod 0755 /opt/design/chromium-launcher
ENV MIRKET_PLAYWRIGHT_VERSION=1.64.1 MIRKET_PLAYWRIGHT_CHROMIUM_REVISION=1248 MIRKET_PLAYWRIGHT_CHROMIUM_VERSION=156.0.8078.47
COPY tools/openpencil-tools.json ./openpencil-tools.json
COPY tools/tool-policy.json ./tool-policy.json
COPY tools/tool-budgets.json ./tool-budgets.json
COPY tools/NOTICE.md ./NOTICE.md
RUN mkdir -p /workspace \
    && chmod -R a+rX /opt/design/chrome \
    && chmod 0644 /opt/design/*.mjs /opt/design/NOTICE.md /opt/design/package.json /opt/design/package-lock.json /opt/design/tool-policy.json /opt/design/tool-budgets.json /opt/design/openpencil-tools.json /opt/design/browser.json \
    && chown node:node /workspace
USER node
RUN node --check /opt/design/workspace-server.mjs \
    && node -e 'const policy = JSON.parse(require("node:fs").readFileSync("/opt/design/tool-policy.json", "utf8")); if (!Object.keys(policy).length) throw Error("Design tool policy is empty")'
WORKDIR /workspace
ENTRYPOINT ["/usr/bin/tini", "-g", "--"]
CMD ["node", "/opt/design/workspace-server.mjs"]
