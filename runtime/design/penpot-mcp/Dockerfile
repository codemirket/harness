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

# Ship the compiled server and its locked production closure, not the workspace tooling.
COPY production.mjs /opt/production.mjs
RUN node /opt/production.mjs \
    && cd /opt/server \
    && corepack pnpm@12.8.1 --version \
    && corepack pnpm@12.8.1 install --prod --frozen-lockfile --ignore-scripts --store-dir /opt/penpot/.pnpm-store

FROM docker.io/library/node@sha256:b20fe038d9592822dd534600a1ddf467e985178aa3fed60ec441a056c2cc45bd
COPY --from=build /opt/penpot/LICENSE /opt/penpot/LICENSE
COPY --from=build /opt/server /opt/server
COPY --from=build /opt/penpot/mcp/packages/plugin/dist /opt/plugin
COPY serve.mjs /opt/serve.mjs
RUN chmod 0644 /opt/serve.mjs \
    && chmod -R a+rX /opt/server /opt/plugin /opt/penpot/LICENSE \
    && rm -rf /usr/local/lib/node_modules /usr/local/bin/npm /usr/local/bin/npx /usr/local/bin/corepack /usr/local/bin/pnpm /usr/local/bin/yarn /usr/local/bin/yarnpkg
ENV NODE_ENV=production \
    PENPOT_MCP_SERVER_HOST=0.0.0.0 \
    PENPOT_MCP_SERVER_PORT=4401 \
    PENPOT_MCP_WEBSOCKET_PORT=4402 \
    PENPOT_MCP_PLUGIN_SERVER_HOST=0.0.0.0 \
    PENPOT_MCP_REPL_ENABLE=false \
    PENPOT_MCP_DEVENV=false
USER node
WORKDIR /opt/server
RUN node --check /opt/serve.mjs \
    && node -e "const fs = require('node:fs'); for (const file of ['dist/index.js', 'data/api_types.yml', '/opt/plugin/manifest.json']) fs.accessSync(file, fs.constants.R_OK)"
EXPOSE 4400 4401 4402
CMD ["node", "/opt/serve.mjs"]
