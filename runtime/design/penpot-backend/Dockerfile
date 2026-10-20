# Preserve the upstream application, JRE and ImageMagick payloads by digest.
# Its OS package database was deleted after apt changes; do not reuse that layer.
FROM docker.io/penpotapp/backend@sha256:d5b8d69c00a9c4735c10b3612f7e3c18806f879367fa7be7f621058769cb551e AS payload
FROM docker.io/library/debian@sha256:a2ca65377c251208de42e1a2722cb949211236c161b634f6f3663754e2d06850

COPY --from=payload /etc/ssl/certs/ca-certificates.crt /tmp/bootstrap-ca.crt
RUN printf 'Types: deb\nURIs: https://snapshot.debian.org/archive/debian/20261020T000000Z/\nSuites: trixie trixie-updates\nComponents: main\nCheck-Valid-Until: no\n\nTypes: deb\nURIs: https://snapshot.debian.org/archive/debian-security/20261020T000000Z/\nSuites: trixie-security\nComponents: main\nCheck-Valid-Until: no\n' > /etc/apt/sources.list.d/debian.sources \
    && rm -f /etc/apt/sources.list \
    && apt-get -o Acquire::https::CaInfo=/tmp/bootstrap-ca.crt update \
    && apt-get -o Acquire::https::CaInfo=/tmp/bootstrap-ca.crt -y upgrade \
    && apt-get -o Acquire::https::CaInfo=/tmp/bootstrap-ca.crt install -y --no-install-recommends \
       passwd ca-certificates curl fontconfig fontforge libfontconfig1 libfreetype6 \
       libglib2.0-0 libgomp1 libheif1 libjpeg62-turbo liblcms2-2 libopenexr-3-1-30 \
       libopenjp2-7 libpng16-16 librsvg2-2 libtiff6 libwebp7 libwebpdemux2 libwebpmux3 \
       libxml2 libzip5 libzstd1 python3 python3-tabulate tzdata woff-tools woff2 \
    && useradd -U -M -u 1001 -s /bin/false -d /opt/penpot penpot \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* /var/cache/apt/* /tmp/bootstrap-ca.crt \
    && test -s /var/lib/dpkg/status \
    && mkdir -p /opt/data/assets /opt/penpot \
    && chown -R penpot:penpot /opt/penpot /opt/data

COPY --from=payload /opt/jre /opt/jre
COPY --from=payload /opt/imagick /opt/imagick
COPY --from=payload --chown=1001:1001 /opt/penpot/backend /opt/penpot/backend
COPY NOTICE.md /opt/penpot/MIRKET-NOTICE.md
ENV LANG=C.UTF-8 LC_ALL=C.UTF-8 JAVA_HOME=/opt/jre \
    PATH=/opt/jre/bin:/opt/imagick/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
    DEBIAN_FRONTEND=noninteractive TZ=Etc/UTC LD_LIBRARY_PATH=/opt/imagick/lib/deps
USER penpot:penpot
WORKDIR /opt/penpot/backend
CMD ["/bin/bash", "run.sh"]
