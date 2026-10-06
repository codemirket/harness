# Local catalog HTTP client

Work only in this scratch `workspace/`. Implement `client.py` using the Python standard library. The verifier runs an in-process loopback fixture server; no external service, account, or external network access is permitted.

Follow [protocol.md](workspace/protocol.md). Implement `list_all(base_url, token)` to read every item page in order and `create_item(base_url, token, name)` to perform one mutation. Send the required authorization header on every request. A write whose response disappears after the server accepted it is uncertain: raise `MutationUncertain` and do not automatically replay it. Avoid unbounded waits; use request timeouts. Keep public functions importable, and do not start a persistent service.

You may check locally with `python3 -I ../verify.py .` from this workspace. That verifier creates and shuts down its own loopback server. Report the observed read, successful write, and uncertain-write behavior.
