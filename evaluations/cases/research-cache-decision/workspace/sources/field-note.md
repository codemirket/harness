# Field note: unexpected stale property read

Recorded 2026-04-02 by the local acceptance team. The team saw one old property description immediately after a successful edit. The note says, "all GETs are globally cached and mutations never invalidate," but it did not capture a server version, tenant ID, response headers, request query, or edge-cache setting. A later fresh browser session showed the new description. The observation is genuine; its explanation was not isolated.
