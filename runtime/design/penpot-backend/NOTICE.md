# Penpot backend image

The application, JRE and ImageMagick trees are copied unchanged from the pinned
official Penpot backend image. Their upstream licenses and notices remain in
those trees. Penpot is MPL-2.0; its source is https://github.com/penpot/penpot.

Mirket replaces only the operating-system layer with pinned Debian packages,
using the upstream runtime package list. Debian package copyright files and
`/var/lib/dpkg/status` are retained. The image recipe and OS inventory are not
evidence that copied application dependencies are free of advisories.
