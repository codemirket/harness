# Atlas 3.9 operator note

Published 2025-11-12. Applies to Atlas 3.9.x.

The legacy property-read cache uses a global URL key and a 60-second TTL. A write does not actively invalidate that cache, so operators used a bypass header for read-after-write checks. This behavior was documented for the 3.9 service line and should not be projected onto later versions without evidence.
