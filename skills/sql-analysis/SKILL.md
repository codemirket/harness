---
name: sql-analysis
description: "Write or review a SQL query that answers a data question using explicit dialect/schema, row grain and result verification. Skip offline definition/reference navigation."
---

# SQL analysis

Establish the question, dialect, schema, row grain, keys, units and expected population. Inspect only the data/access context authorized for the task. Separate query authoring from database execution; neither implies credentials or production access.

Reason through join cardinality, null behavior, filters, grouping, window partitions and date/time boundaries. Make denominators and deduplication rules explicit. Prefer readable stages when they expose intermediate invariants rather than hiding row multiplication in a final DISTINCT.

Run in an authorized environment when execution is requested/available. Reconcile representative records, counts and totals against an independent query or fixture. Include duplicate keys, nulls, empty groups or boundary dates when they can expose a convincing incorrect result.

Return the query, dialect/schema assumptions and result checks. Stop when the question is answered to the supplied contract or report the exact missing schema/data access. A parse or navigation result does not establish a correct aggregate, and no live query is implied by a SQL file.
