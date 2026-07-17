# ISS-228: boot --check validates against the binary's own embed — stale binary silently rolls back governance

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Observed (SL-214 audit, RV-280)

Session-start regeneration of `.doctrine/state/boot.md` by the PATH release
binary (`~/.cargo/bin/doctrine`, stale embedded `install/` assets) silently
dropped the `/knowledge` routing row landed in SL-214 PHASE-02 — and
`doctrine boot --check` reported `clean`, because check compares the snapshot
against the *same binary's own embed*, not against the repo's current
`install/` source. Regenerating with `./target/debug/doctrine` (fresh embed)
restored the row.

## Impact

Any governance change that rides embedded assets is rolled back on every boot
regen by an older binary, with a false-clean check. In the doctrine repo this
recurs every session until a release ships; in client repos it recurs until
the client updates.

## Candidate directions

- Stamp embed provenance (build hash/version) into boot.md; `--check` warns on
  generation by an older embed than the one that last wrote the snapshot.
- In the doctrine repo itself, prefer disk `install/` over embed when present.
