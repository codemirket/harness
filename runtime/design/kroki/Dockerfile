ARG DESIGN_IMAGE
FROM docker.io/library/node@sha256:43b9ba1142b8dcf33fa8f922ca1be85714adc444e0586c7cf7726daaeabc1bea AS build
RUN node -e 'require("fs").writeFileSync("/tmp/ca.pem",require("tls").rootCertificates.join("\n"))' \
    && printf 'deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/20261023T000000Z/ trixie main\n' > /etc/apt/sources.list && rm /etc/apt/sources.list.d/debian.sources \
    && apt-get -o Acquire::https::CaInfo=/tmp/ca.pem update && apt-get -o Acquire::https::CaInfo=/tmp/ca.pem install -y --no-install-recommends ca-certificates curl maven openjdk-21-jdk-headless
RUN curl -fLsS https://codeload.github.com/yuzutech/kroki/tar.gz/refs/tags/v0.33.0 -o /tmp/source.tar.gz \
    && echo '62bc12c863d4f6ff05fa835bd5ed2fd42f4134438c85c65fb923cc79397e88ee  /tmp/source.tar.gz' | sha256sum -c - \
    && mkdir /source && tar -xzf /tmp/source.tar.gz --strip-components=1 -C /source
COPY jackson-boms.mjs /tmp/jackson-boms.mjs
RUN node /tmp/jackson-boms.mjs
WORKDIR /source
RUN mvn --batch-mode --no-transfer-progress -pl server -am package -DskipTests -Dmaven.gitcommitid.skip=true

# Reviewed upstream native engines. Only the selected executables are copied.
FROM docker.io/yuzutech/kroki@sha256:a4341ac44cd2590124da7142d13fdf5d83eadc2b94c023d018a2227695a30da1 AS engines
FROM ${DESIGN_IMAGE} AS native
FROM docker.io/library/node@sha256:43b9ba1142b8dcf33fa8f922ca1be85714adc444e0586c7cf7726daaeabc1bea
RUN node -e 'require("fs").writeFileSync("/tmp/ca.pem",require("tls").rootCertificates.join("\n"))' \
    && printf 'deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/20261023T000000Z/ trixie main\ndeb [check-valid-until=no] https://snapshot.debian.org/archive/debian-security/20261023T000000Z/ trixie-security main\n' > /etc/apt/sources.list && rm /etc/apt/sources.list.d/debian.sources \
    && apt-get -o Acquire::https::CaInfo=/tmp/ca.pem update && apt-get -o Acquire::https::CaInfo=/tmp/ca.pem upgrade -y && apt-get -o Acquire::https::CaInfo=/tmp/ca.pem install -y --no-install-recommends openjdk-21-jre-headless graphviz fonts-dejavu-core fonts-liberation libpango-1.0-0 libpangocairo-1.0-0 libjpeg62-turbo \
    && rm -rf /var/lib/apt/lists/* /tmp/ca.pem /usr/local/lib/node_modules /usr/local/bin/npm /usr/local/bin/npx /usr/local/bin/corepack /usr/local/bin/yarn /usr/local/bin/yarnpkg
COPY --from=build /source/server/target/kroki-server.jar /opt/kroki/kroki-server.jar
COPY --from=build /source/LICENSE /opt/kroki/LICENSE
COPY --from=engines /usr/bin/plantuml /usr/bin/plantuml
# Graal native PlantUML loads its matching AWT/JVM sidecars at runtime.
COPY --from=engines /usr/bin/libawt_headless.so /usr/bin/libawt.so /usr/bin/libawt_xawt.so /usr/bin/libfontmanager.so /usr/bin/libfreetype.so /usr/bin/libjavajpeg.so /usr/bin/libjava.so /usr/bin/libjvm.so /usr/bin/liblcms.so /usr/bin/libmlib_image.so /usr/bin/
COPY --from=engines /usr/bin/vega /usr/bin/vega
COPY --from=native /usr/local/bin/d2 /usr/bin/d2
COPY --chmod=0644 gateway.mjs /opt/kroki/gateway.mjs
ENV KROKI_SAFE_MODE=SECURE KROKI_ENABLE_COMPANION_DISCOVERY=false KROKI_DOT_BIN_PATH=/usr/bin/dot KROKI_PLANTUML_BIN_PATH=/usr/bin/plantuml KROKI_VEGA_BIN_PATH=/usr/bin/vega KROKI_D2_BIN_PATH=/usr/bin/d2
USER 1001:1001
ENTRYPOINT ["node","/opt/kroki/gateway.mjs"]
