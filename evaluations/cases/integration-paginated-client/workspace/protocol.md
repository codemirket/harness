# Fixture HTTP protocol

The evaluation server binds only to `127.0.0.1` at an ephemeral port. The caller receives `base_url` and `token`; never hard-code either. Every request needs `Authorization: Bearer <token>`.

`GET /items` returns HTTP 200 JSON `{ "items": [{"id": "...", "name": "..."}], "next_cursor": "..." | null }`. When `next_cursor` is non-null, request `GET /items?cursor=<URL-encoded cursor>` and append the next page. Preserve server order. A failed read should raise an exception; do not silently return a partial list.

`POST /items` accepts JSON `{ "name": "..." }` with `Content-Type: application/json`. A normal success returns HTTP 201 JSON `{ "id": "...", "name": "..." }`. If the connection closes or a response cannot be confirmed after the request, the server may already have applied the write. Raise `MutationUncertain` and do not retry the POST. An explicit HTTP error is a confirmed refusal and should raise a different exception.
