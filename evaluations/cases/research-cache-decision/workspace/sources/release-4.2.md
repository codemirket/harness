# Atlas 4.2 release behavior

Published 2026-03-20. Applies to Atlas 4.2.0 and later 4.2 patch releases unless amended.

Property GET responses may be served from a cache keyed by organization and request query. A successful property mutation invalidates that organization's matching property-read entries before the mutation response is sent. Different organizations do not share cached property data. A cache miss reads the backing store.

This document covers the default server configuration. It does not describe the optional edge response cache or prove a particular deployment's settings.
