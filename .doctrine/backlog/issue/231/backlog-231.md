# ISS-231: Lifecycle status verbs regress authored updated date across local midnight

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Observed on 2026-07-19 in `Australia/Melbourne`, shortly after local midnight.
`RFC-021` already had `updated = "2026-07-19"`; running:

```text
doctrine rfc status --status resolved RFC-021
```

correctly changed the status but regressed `updated` to `2026-07-18`. The
process environment's UTC date was still July 18, suggesting lifecycle writers
derive authored calendar dates from UTC rather than the project/user timezone.

Expected: a lifecycle mutation must not move `updated` backwards. Its date
source and timezone policy should be explicit and tested across a local-midnight
boundary. At minimum, preserve an existing later date when the derived date is
earlier.
