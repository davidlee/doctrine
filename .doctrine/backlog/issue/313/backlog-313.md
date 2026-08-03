# ISS-313: Published glossary has stale knowledge status vocabularies

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed

`doctrine library show reference/glossary.md` publishes the decision-record
status vocabulary as:

```text
pending | active | superseded | withdrawn
```

The live CLI rejects `doctrine knowledge status DEC-133 active` and reports its
actual vocabulary as:

```text
proposed | accepted | rejected | superseded
```

The generated decision template also seeds `proposed`, confirming that the
published reference is stale rather than the newly-created record being
malformed.

## Expected

The published glossary and any other shipped reference surfaces derive from or
exactly match each knowledge kind's CLI lifecycle vocabulary. Add a consistency
check so a future schema change cannot leave the published table behind.

## Impact

Agents following the mandated CLI/reference-doc workflow are instructed to run
invalid transitions. The CLI fails safely, but the mismatch costs a failed write
attempt and makes the authoritative lifecycle unclear.
