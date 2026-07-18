# REQ-164: Embed the semantic source roots at compile time as the storage mechanism

## Statement

Embed the semantic source roots into the binary at compile time (rust-embed) as
the *storage* mechanism. Embedding is storage only — it does not define or imply
the projected fileset; projection is drawn from the manifest's projection policy
(REQ-165), not from the embed contents.

## Rationale

Separating embedding (storage) from projection (what lands) per ADR-019 is the
load-bearing move of RFC-021 C1: the installed surface must stop tracking embed
convenience. Forward-intent — `pending` until an implementation slice lands it.
