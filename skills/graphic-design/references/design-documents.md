# Editable design documents

Use the official Penpot MCP/plugin through the `design` package when the
deliverable is a structured, editable visual document. Mirket forwards the
upstream MCP tool names and schemas unchanged. Confirm the requested portable
format and actual upstream schema before creating a file. Inside the private
design network the MCP endpoint is on port 4401 at `/mcp`, the plugin manifest
is on port 4400 and its persistent WebSocket is on port 4402. Use the assigned
loopback host ports reported by `package start` or `package status` for host
access; the WebSocket port is not separately published. A saved local handle
alone does not satisfy a portable-file
request.

Compose with actual copy, reading order, typography roles, space and palette.
Compile `design/tokens.json` through Style Dictionary first and use the emitted
Penpot tokens in the editable file.
Use only features that the saved format preserves. Inspect the saved document's
IDs, geometry, text and relationships, refine using stable identities, then
reopen and export the saved result. Keep editable source separate from static
previews. Inspect those previews at intended sizes for hierarchy, text fit,
clipping and consistency.

Design-file writes may persist before a request is cancelled. Keep credentials
outside source and report the inspected file, exports and unverified behavior.
