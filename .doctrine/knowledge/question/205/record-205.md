# QUE-205: Capsule forensic archive storage and retention

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Question

Where should production execution-capsule forensic exhibits live, how long
should they be retained, and what durable evidence should remain after an
exhibit expires?

The exhibits include the capsule's Git bundle or equivalent object archive,
worker commit history, verification logs, metadata, and any permitted
transcript material. They may be large or binary, so treating them as ordinary
committed authored files can grow the repository without bound. Treating them
as ordinary runtime state instead makes them disposable and unsuitable as the
only evidence supporting an admission decision.

## Evidence already established

- SL-241's capsule spike showed that both measured ingestion mechanisms preserve
  worker commit identity. A bundle is a viable archive exhibit, but reading it
  adds a trusted-side file-ingestion boundary.
- The spike committed the summaries and proof objects needed to support its
  claims while leaving uncited, reproducible raw logs in disposable runtime
  state. That is a project-local spike disposition, not production retention
  policy.
- ADR-019 requires storage, publication, and projection to be decided
  independently according to semantic ownership and lifecycle.
- POL-002 forbids shipped correctness from depending on a client repository's
  transient local state or undeclared conventions.

## Candidate postures

1. **Commit every exhibit.** Strong co-location and ordinary Git provenance,
   but unbounded binary growth makes this a poor production default.
2. **Keep exhibits only in gitignored local storage.** Operationally simple and
   cheap, but host cleanup can erase the only copy; this cannot carry durable
   admission evidence by itself.
3. **Separate durable journal from retained exhibit.** Commit or otherwise
   durably store a small trusted-side admission record containing identities,
   hashes, verdicts, and archive references. Store large exhibits behind an
   explicit archive backend with a retention policy; a local gitignored backend
   may be a project-local default, while shipped Doctrine owns the backend and
   expiry contract. Expiry removes the exhibit, not the admission record.

## Answer

DEC-133 separates the durable trusted-side admission journal from forensic
exhibits that may expire. The journal preserves identities, hashes, verdicts,
archive references, and the exhibit's lifecycle state. Large exhibits live
behind a separate archive boundary and may follow a short retention horizon.

The decision deliberately does not prescribe a duration, quota, or project-,
slice-, or machine-level configuration hierarchy. Those policy controls remain
available to introduce when operational evidence makes their ownership clear.

CHR-053 may now bring RFC-025 into line with this answer. CHR-054's later
revision scoping must consume the decision rather than inventing a different
archive policy implicitly.
