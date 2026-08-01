# QUE-201: Interpretation-surface declaration home

**Question.** In shipped doctrine, where does a client project's
interpretation-surface declaration live?

[[interpretation-surface-ownership]] settles *that* the client declares classes
1–3 and that absence is refused. It deliberately does not settle *where* the
declaration sits.

## Candidates

1. **A `doctrine.toml` block** — config-tier, one file per project, discoverable
   with the rest of the project's doctrine settings. Risk: config drifts toward
   a junk drawer, and this is a security-significant declaration.
2. **A dedicated default-deny manifest** — the direct dual of
   `.worktreeinclude`, same idiom, same default-deny semantics, visibly
   security-significant by being its own artifact. Risk: a second manifest to
   keep in sync.
3. **A field on the work contract** — per-contract rather than per-project,
   which allows a phase to narrow its own surface. Risk: per-contract means an
   agent-authored value on a security boundary.

## What settles it

Not argument. The SL-241 rig implements a rig-local per-fixture file and runs
it against both a Rust and a TypeScript fixture; whichever friction that
surfaces (sync burden, discoverability, per-phase variance actually wanted or
not) is the input. Settles during the post-spike REV, not in SL-241.

## Related

- [[interpretation-surface-ownership]] — the decision that opens this.
- [[interpretation-surface]] — what is being declared.
- ADR-019 — asset-policy lens for where artifacts live.
