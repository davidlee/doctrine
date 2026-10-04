# ISS-518: review_show unknown finding id returns internal error

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

MCP `review_show` with an unknown finding id (`finding: "F-99"`) answers
JSON-RPC -32603 (internal error). It is the caller's mistake, so it should be
an invalid-params refusal. Needs a typed error out of the `review_ledger`
finding selector so the MCP layer can map it. Surfaced during SL-278 PHASE-03.
