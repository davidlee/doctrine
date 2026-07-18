# REQ-171: Distribution is one self-contained binary with a compile-time embed as the storage mechanism

## Statement

Distribution is one self-contained binary whose compile-time embed is the *storage*
mechanism — no network fetch, no sidecar asset bundle. The embed supplies the bytes;
it does not define the installed fileset (that is the projection policy, REQ-165).

## Rationale

The single-artefact, no-network distribution property is retained, but reframed:
embedding is storage, not the installed set. This keeps the distribution guarantee
while removing the embed-equals-projection conflation (ADR-019). Forward-intent —
`pending` until an implementation slice lands it.
