ARG DESIGN_IMAGE
FROM docker.io/library/node@sha256:43b9ba1142b8dcf33fa8f922ca1be85714adc444e0586c7cf7726daaeabc1bea AS build
RUN node -e 'require("fs").writeFileSync("/tmp/ca.pem",require("tls").rootCertificates.join("\n"))' \
    && printf 'deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/20261023T000000Z/ trixie main\n' > /etc/apt/sources.list && rm /etc/apt/sources.list.d/debian.sources \
    && apt-get -o Acquire::https::CaInfo=/tmp/ca.pem update && apt-get -o Acquire::https::CaInfo=/tmp/ca.pem install -y --no-install-recommends ca-certificates curl
RUN curl -fLsS https://codeload.github.com/yuzutech/kroki/tar.gz/refs/tags/v0.33.0 -o /tmp/source.tar.gz \
    && echo '62bc12c863d4f6ff05fa835bd5ed2fd42f4134438c85c65fb923cc79397e88ee  /tmp/source.tar.gz' | sha256sum -c - \
    && mkdir /source && tar -xzf /tmp/source.tar.gz --strip-components=1 -C /source
WORKDIR /source/excalidraw
COPY package.json package-lock.json ./
RUN npm ci --ignore-scripts --no-audit --no-fund \
    && node node_modules/esbuild/bin/esbuild assets/index.js --bundle --minify --outfile=assets/index.bundle.js --metafile=assets/bundle-inputs.json \
    && cp -R node_modules/@excalidraw/excalidraw/dist/prod/fonts/. assets/fonts/ \
    && mkdir -p /licenses/dompurify && cp node_modules/dompurify/LICENSE /licenses/dompurify/LICENSE
WORKDIR /runtime
COPY server-package.json package.json
COPY server-package-lock.json package-lock.json
RUN npm ci --omit=dev --ignore-scripts --no-audit --no-fund \
    && mkdir -p node_modules/@kroki && ln -s ../../../lib/browser-instance node_modules/@kroki/browser-instance
FROM ${DESIGN_IMAGE} AS browser
FROM docker.io/library/node@sha256:43b9ba1142b8dcf33fa8f922ca1be85714adc444e0586c7cf7726daaeabc1bea
RUN node -e 'require("fs").writeFileSync("/tmp/ca.pem",require("tls").rootCertificates.join("\n"))' \
    && printf 'deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/20261023T000000Z/ trixie main\ndeb [check-valid-until=no] https://snapshot.debian.org/archive/debian-security/20261023T000000Z/ trixie-security main\n' > /etc/apt/sources.list && rm /etc/apt/sources.list.d/debian.sources \
    && apt-get -o Acquire::https::CaInfo=/tmp/ca.pem update && apt-get -o Acquire::https::CaInfo=/tmp/ca.pem upgrade -y \
    && apt-get -o Acquire::https::CaInfo=/tmp/ca.pem install -y --no-install-recommends ca-certificates fonts-dejavu-core fonts-liberation libasound2t64 libatk-bridge2.0-0t64 libatk1.0-0t64 libatspi2.0-0t64 libcairo2 libcups2t64 libdbus-1-3 libdrm2 libgbm1 libglib2.0-0t64 libgtk-3-0t64 libnspr4 libnss3 libpango-1.0-0 libwayland-client0 libx11-6 libx11-xcb1 libxcb1 libxcomposite1 libxdamage1 libxext6 libxfixes3 libxkbcommon0 libxrandr2 libxshmfence1 \
    && rm -rf /var/lib/apt/lists/* /tmp/ca.pem /usr/local/lib/node_modules /usr/local/bin/npm /usr/local/bin/npx /usr/local/bin/corepack /usr/local/bin/yarn /usr/local/bin/yarnpkg
COPY --from=browser /opt/design/chrome /opt/chrome
COPY --from=build /source/excalidraw/src /opt/kroki/excalidraw/src
COPY --from=build /source/excalidraw/assets /opt/kroki/excalidraw/assets
COPY --from=build /runtime/node_modules /opt/kroki/excalidraw/node_modules
COPY --from=build /source/lib/browser-instance /opt/kroki/lib/browser-instance
COPY --from=build --chmod=0644 /runtime/package.json /opt/kroki/excalidraw/package.json
COPY --from=build /licenses /opt/kroki/licenses
COPY --from=build /source/LICENSE /opt/kroki/LICENSE
RUN ln -s "/opt/chrome/$(test "$(dpkg --print-architecture)" = arm64 && echo chrome-linux-arm64 || echo chrome-linux64)/chrome" /usr/local/bin/chromium
ENV KROKI_EXCALIDRAW_PAGE_URL=http://127.0.0.1:8004/public/index.html KROKI_EXCALIDRAW_ASSET_PATH=/public PUPPETEER_EXECUTABLE_PATH=/usr/local/bin/chromium LEVEL=info XDG_CONFIG_HOME=/tmp/config XDG_CACHE_HOME=/tmp/cache
USER 1001:1001
WORKDIR /opt/kroki/excalidraw
COPY --chmod=0755 entrypoint /opt/kroki/excalidraw/entrypoint
ENTRYPOINT ["/opt/kroki/excalidraw/entrypoint"]
