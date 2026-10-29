# External pi host admission

The exact pi 1.4.0 unbundled host with Undici 8.12.2 is approved for the local
candidate. [The admission record](pi-local-candidate.json) owns its pinned input
hashes and scope. It does not authorize live installation, redistribution,
production browser dependencies or publication.

Use `@earendil-works/pi-coding-agent/dist/cli.js`. The default bundled CLI embeds
older Undici bytes and is refused. The admitted host remains external to Mirket.
The exact closure has 122 package instances and 15,249 unique archive files.
The repaired lock, inventory and advisory snapshot retain their frozen `.proposed`
filenames; approval is carried by the separate admission record, not the filename.

Undici 8.12.2 is MIT and requires Node >=22.19.0. All lifecycle hooks, including
its prepare script, remain disabled. The retained registry advisory response is
an observation at acquisition time, not a continuing security guarantee.

`scripts/freeze-pi-host.py` derives `pi-host-files.json.proposed` from the exact
locked archive integrities, not an arbitrary installed tree. The extension checks
the complete closure and entry point, rejects unexpected or changed code and
requires the qualified private Node. [Qualification](../../docs/qualification.md)
states the tested lifecycle, provider, tamper and performance boundaries.
