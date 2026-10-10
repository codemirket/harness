# Worked IT diagnosis: browser works, desktop client cannot connect

This is a hypothetical investigation. `api.example.test` and the documentation
IP addresses below are illustrative; substitute only an identified, authorized
target. No live system was tested to create this reference.

## Establish comparable observations

The report says a desktop order client fails after VPN connection while the web
portal works. Record the same user's machine, client version/process, VPN state
and failing API hostname. First confirm that the browser makes a fresh request;
a previously rendered page is not evidence of current connectivity. Identify
whether its successful operation even uses the same API and identity.

Assume the following evidence has actually been collected in this example:

| Boundary | Operating-system probe | Failing client path |
| --- | --- | --- |
| Resolver answer for the API | `192.0.2.25` through VPN DNS | `198.51.100.24` in the client's resolver trace |
| TCP on the chosen address, port 443 | Connects | Connects |
| TLS with `api.example.test` as server name | Valid expected identity | Certificate names the previous public endpoint |
| API using a known-good route and the same hostname | Authorized read succeeds | Targeted diagnostic override also succeeds |

This narrows the fault to the client's selected route/resolution path. It does not
prove that the VPN is broken, the password expired, or every device needs a DNS
change. The two clients have not been using an equivalent target.

For an HTTP client that supports it, a temporary per-command address override
can preserve the original URL hostname and TLS server name while selecting a
known-good address. Use the client's documented mechanism. Replacing the URL
hostname with an IP or changing only the HTTP Host header is not an equivalent
TLS test. Disabling certificate verification would conceal the exact evidence
needed here and is unnecessary for this comparison.

## Distinguish possible owners

Inspect how the affected client obtains its endpoint and resolution: application
configuration, a local hosts entry, an application cache, a proxy, a guest/container
resolver or a long-lived connection. The correct operating-system answer does
not eliminate these alternatives. Compare effective state with the supported
configuration owner and last known working state.

Suppose a per-application proxy setting is found pointing to the old public path,
and the documented VPN policy expects the client to connect through its supported
VPN route. The narrow repair is that setting, with its old value recorded. An
authorized task to repair the workstation may already cover this reversible
change. A production DNS-zone edit would affect other clients and is a different
scope; there is no evidence for it in this example.

If restarting the client fixes the issue but its prior resolver/proxy state was
not observed, call the outcome a mitigation. Restart changes multiple inputs:
cache, connection, credential refresh and configuration loading. It cannot by
itself identify which one caused the failure.

## Finish at the affected consumer

Remove the diagnostic override and repeat the original client operation under the
same account. Confirm the effective target now matches the intended route and
that hostname/chain verification remains enabled. Observe the expected API data
or durable result. Reconnect the VPN or reopen the client when persistence across
that event is part of the reported problem.

Keep the final evidence bounded: non-secret error, target identity, relevant
before/after setting, probe result and actual user operation. Do not copy an entire
authentication trace just because it might be useful. If the original operation
is a write with an unknown outcome, reconcile its existing result before repeating
it; a connectivity repair does not make an earlier write safe to duplicate.
