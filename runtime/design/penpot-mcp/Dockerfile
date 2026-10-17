FROM docker.io/library/node@sha256:b20fe038d9592822dd534600a1ddf467e985178aa3fed60ec441a056c2cc45bd AS build
ENV COREPACK_HOME=/opt/corepack COREPACK_ENABLE_AUTO_PIN=0

# Mirket checks the source archive SHA-256 before passing this closed context.
COPY penpot-source.tar.gz /tmp/penpot-source.tar.gz
RUN mkdir -p /opt/penpot && tar -xzf /tmp/penpot-source.tar.gz --strip-components=1 -C /opt/penpot \
    && rm /tmp/penpot-source.tar.gz \
    && corepack enable \
    && cd /opt/penpot/mcp \
    && corepack pnpm install --frozen-lockfile \
    && WS_URI=http://penpot-mcp:4402 PENPOT_MCP_PLUGIN_SERVER_HOST=0.0.0.0 corepack pnpm run build

FROM docker.io/library/node@sha256:b20fe038d9592822dd534600a1ddf467e985178aa3fed60ec441a056c2cc45bd
ENV COREPACK_HOME=/opt/corepack COREPACK_ENABLE_NETWORK=0 COREPACK_ENABLE_AUTO_PIN=0

# `concurrently --kill-others` uses ps to terminate the other child when one
# exits. The pinned slim base omits it, so install only procps from the signed,
# immutable Debian snapshot. _apt must be able to read this temporary CA PEM.
RUN node -e 'require("node:fs").writeFileSync("/tmp/node-root-ca.pem", require("node:tls").rootCertificates.join("\n") + "\n", {mode: 0o644})' \
    && printf 'Types: deb\nURIs: https://snapshot.debian.org/archive/debian/20261005T000000Z/\nSuites: bookworm bookworm-updates\nComponents: main\nCheck-Valid-Until: no\nSigned-By: /usr/share/keyrings/debian-archive-keyring.gpg\n\nTypes: deb\nURIs: https://snapshot.debian.org/archive/debian-security/20261005T000000Z/\nSuites: bookworm-security\nComponents: main\nCheck-Valid-Until: no\nSigned-By: /usr/share/keyrings/debian-archive-keyring.gpg\n' > /etc/apt/sources.list.d/debian.sources \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem update \
    && apt-get -o Acquire::https::CaInfo=/tmp/node-root-ca.pem install -y --no-install-recommends procps=2:4.0.2-3 \
    && rm /tmp/node-root-ca.pem \
    && rm -rf /var/lib/apt/lists/*

# pnpm's runtime link graph is inside mcp/node_modules, except the official
# plugin-types workspace link. The install store and unrelated Penpot source
# trees are build-only. Keep the official license, lock, package manifests,
# compiled server/plugin, API resources and the pnpm/Vite runtime closure.
COPY --from=build /opt/penpot/LICENSE /opt/penpot/LICENSE
COPY --from=build /opt/corepack /opt/corepack
COPY --from=build /opt/penpot/plugins/libs/plugin-types /opt/penpot/plugins/libs/plugin-types
COPY --from=build /opt/penpot/mcp/package.json /opt/penpot/mcp/package.json
COPY --from=build /opt/penpot/mcp/pnpm-lock.yaml /opt/penpot/mcp/pnpm-lock.yaml
COPY --from=build /opt/penpot/mcp/pnpm-workspace.yaml /opt/penpot/mcp/pnpm-workspace.yaml
COPY --from=build /opt/penpot/mcp/README.md /opt/penpot/mcp/README.md
COPY --from=build /opt/penpot/mcp/node_modules /opt/penpot/mcp/node_modules
COPY --from=build /opt/penpot/mcp/packages/common/package.json /opt/penpot/mcp/packages/common/package.json
COPY --from=build /opt/penpot/mcp/packages/common/dist /opt/penpot/mcp/packages/common/dist
COPY --from=build /opt/penpot/mcp/packages/common/node_modules /opt/penpot/mcp/packages/common/node_modules
COPY --from=build /opt/penpot/mcp/packages/server/package.json /opt/penpot/mcp/packages/server/package.json
COPY --from=build /opt/penpot/mcp/packages/server/dist /opt/penpot/mcp/packages/server/dist
COPY --from=build /opt/penpot/mcp/packages/server/data /opt/penpot/mcp/packages/server/data
COPY --from=build /opt/penpot/mcp/packages/server/node_modules /opt/penpot/mcp/packages/server/node_modules
COPY --from=build /opt/penpot/mcp/packages/plugin/package.json /opt/penpot/mcp/packages/plugin/package.json
COPY --from=build /opt/penpot/mcp/packages/plugin/dist /opt/penpot/mcp/packages/plugin/dist
COPY --from=build /opt/penpot/mcp/packages/plugin/vite.config.ts /opt/penpot/mcp/packages/plugin/vite.config.ts
COPY --from=build /opt/penpot/mcp/packages/plugin/node_modules /opt/penpot/mcp/packages/plugin/node_modules

# The frozen build already installed all dependencies. pnpm 12 otherwise
# retries installation during `run`/`exec` when its build-only store is absent.
# Vite preview has its own allowlist; admit only the private service hostname.
RUN node -e 'const fs = require("node:fs"); const file = "/opt/penpot/mcp/packages/plugin/vite.config.ts"; const source = fs.readFileSync(file, "utf8"); const needle = "allowedHosts: [],"; if (source.split(needle).length !== 2) throw Error("Pinned plugin preview host configuration drift"); fs.writeFileSync(file, source.replace(needle, "allowedHosts: [\"penpot-mcp\"],"))' \
    && ! grep -q '^verifyDepsBeforeRun:' /opt/penpot/mcp/pnpm-workspace.yaml \
    && printf '\nverifyDepsBeforeRun: false\n' >> /opt/penpot/mcp/pnpm-workspace.yaml \
    && chmod -R a+rX /opt/corepack \
    && cd /opt/penpot/mcp \
    && corepack pnpm --version \
    && test "$(corepack pnpm config get verifyDepsBeforeRun)" = false \
    && node -e "require('fs').accessSync('packages/server/data/api_types.yml'); require('fs').accessSync('packages/plugin/dist/manifest.json')"

ENV PENPOT_MCP_SERVER_HOST=0.0.0.0 \
    PENPOT_MCP_SERVER_PORT=4401 \
    PENPOT_MCP_WEBSOCKET_PORT=4402 \
    PENPOT_MCP_PLUGIN_SERVER_HOST=0.0.0.0 \
    PENPOT_MCP_REPL_ENABLE=false \
    PENPOT_MCP_DEVENV=false
WORKDIR /opt/penpot/mcp
EXPOSE 4400 4401 4402
CMD ["./node_modules/.bin/concurrently", "--kill-others", "--names", "mcp,plugin", "corepack pnpm --dir packages/server run start", "corepack pnpm --dir packages/plugin exec vite preview --configLoader runner --host 0.0.0.0 --port 4400 --strictPort"]
