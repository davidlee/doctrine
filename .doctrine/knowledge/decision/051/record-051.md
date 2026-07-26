# DEC-051: Give observations a UUID-native keyset read surface

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

Observations are UUID-native ledger records, not authored entity kinds.
SPEC-028 will therefore declare bare UUID as their canonical identity form and
an explicit exception to the prefixed entity-ID convention.

The read surface has observation-specific `show`, `list`, and `search`
arguments. It does not flatten SPEC-013's mandatory entity-kind
`CommonListArgs` spine or join the entity list-conformance matrix. It does reuse
the common command grammar and table/JSON rendering conventions and helpers
where applicable, with observation-specific black-box parse and rendering
goldens.

`show <uuid>` resolves the authoritative record independently of kind,
timestamp, current-view state, or filesystem enumeration order.

`list` orders results by `recorded_at` descending and then UUID. Pagination is
an opaque keyset cursor over that ordered pair, not an offset. A continuation
therefore resumes strictly after its last returned key: observations appended
at the head between requests do not duplicate or shift rows already traversed.
The contract does not claim a frozen corpus snapshot.

`search` reuses the shared lexical tokenizer and case-folding rules. Every
query token must occur somewhere in the combined searchable text comprising
summary, detail, and string facet values. Matching is Boolean, deterministic,
and unranked; `(recorded_at, uid)` remains the result order. The default surface
searches the resolved current view, while an explicit history mode may include
inactive primary observations and correction records.

This resolves the review's SPEC-013 concern narrowly: observations reuse
shared leaves and presentation conventions, but extending the entity-list
contract and its pagination semantics is outside SL-231.
