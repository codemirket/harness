# Penpot runtime notices

Penpot 2.18.3 source: https://github.com/penpot/penpot/tree/ff7a66f51938b2293260ea27a64e2c4e0833b5e3

Official Linux ARM64 service images are pinned by their architecture manifest digest. Their retained image contents include upstream notices for Penpot, PostgreSQL, Valkey and native/browser dependencies. Mirket preserves image contents and does not reconstruct them or run an upstream package installation script. The attached manifest inventory pins the service closure. PostgreSQL and Valkey are internal services. A registered Podman client and Mirket-owned VM are explicit host prerequisites, not silently installed. The fixed first-party plugin runs only in a real permission-checked Penpot workspace.

The service closure contains frontend, backend, exporter, PostgreSQL and Valkey. Mirket uses one private internal Podman bridge for these services and publishes only the frontend on host loopback. The pinned frontend's supported [air-gapped configuration](https://github.com/penpot/penpot/blob/ff7a66f51938b2293260ea27a64e2c4e0833b5e3/docker/images/files/nginx-entrypoint.sh), `enable-air-gapped-conf`, disables external template and Google Fonts proxy routes. The normal registered Plugin API supplies design operations within the frontend; it does not depend on the upstream MCP or admin-console service.

Source Sans Pro 1.050 WOFF files are preserved byte-for-byte from the pinned frontend image. The SVG export embeds these files including their complete license name-table metadata. FONT-OFL.txt retains the exact embedded SIL Open Font License 1.1 notice for convenient inspection. No external font is downloaded.
