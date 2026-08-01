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

**Safety is no longer a discriminator between them** (DEC-099 Amendment 2,
RV-340 F-5). Candidates 1 and 2 both live inside the repository a capsule
clones, which would have made this an unwitting security choice: a worker could
rewrite its own declaration to narrow `exec:` or blank `verify:`. DEC-099 now
rules that the declaration is read from the contracted base `B`, never from a
harvested result, which makes all three candidates equally sound. What remains
is genuinely an ergonomics question — which is what it was always supposed to
be.

## What settles it

Not argument. The SL-241 rig implements a rig-local per-fixture file and runs
it against both a Rust and a TypeScript fixture; whichever friction that
surfaces (sync burden, discoverability, per-phase variance actually wanted or
not) is the input. Settles during the post-spike REV, not in SL-241.

It also gains a **probe-evidence input it previously lacked**: the rig now
carries a fixture variant that places a declaration copy *inside* the fixture
repository — manufacturing the exposure candidates 1 and 2 would have in shipped
form — and a capsule that rewrites it. Trusted-side behaviour must be
byte-identical to the run that did not. Without that row this question would
have settled post-spike on argument alone.

## Related

- [[interpretation-surface-ownership]] — the decision that opens this.
- [[interpretation-surface]] — what is being declared.
- ADR-019 — asset-policy lens for where artifacts live.
