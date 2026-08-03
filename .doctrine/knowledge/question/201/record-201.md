# QUE-201: Interpretation-surface declaration home

**Question.** In shipped doctrine, where does a client project's
interpretation-surface declaration live?

## Answer

Answered by `DEC-136`: the project-owned declaration is a required
`[interpretation]` block in `.doctrine/doctrine.toml`, resolved from the
contracted base commit. A phase work contract may make the resolved policy more
restrictive, but cannot author it, widen permitted execution, weaken required
verification, replace it, or make absence acceptable.

This uses the existing canonical project-configuration surface without adding
a second projected manifest. `DEC-099`'s phrase “default-deny manifest” names
the declaration's semantics, not a requirement for a separate file.

[[interpretation-surface-ownership]] settles *that* the client declares classes
1–3 and that absence is refused. It originally left *where* the declaration
sits to this question; `DEC-136` now settles that remaining choice.

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

## Settlement basis

The SL-241 rig implemented a rig-local per-fixture file and ran it against both
a Rust and a TypeScript fixture. It established that the policy must be read
from the contracted base, leaving sync burden, discoverability, and whether
per-phase variance is actually wanted as the placement inputs. `DEC-136`
settles those ergonomics in the post-spike governance work rather than
pretending the spike selected a production file layout.

It also gains a **probe-evidence input it previously lacked**: the rig now
carries a fixture variant that places a declaration copy *inside* the fixture
repository — manufacturing the exposure candidates 1 and 2 would have in shipped
form — and a capsule that rewrites it. Trusted-side behaviour must be
byte-identical to the run that did not. Without that row this question would
have settled post-spike on argument alone.

## Evidence input (SL-241 PHASE-05)

That row ran, and its result is recorded as **EVD-011**, linked `supports` — the
only probe-evidence input this question has.

Guard probe (e), three legs, all `pass`. Against **F2 (`light-inrepo`)**, the
fixture variant carrying a declaration copy inside the cloned repository, a
capsule that rewrites its own declaration leaves trusted-side behaviour
**byte-identical** to the F1 baseline that had nothing to rewrite — worktree-side
and committed alike. DEC-099 Amendment 2's ruling is therefore observed rather
than argued, and candidates 1 and 2 are confirmed safe on the axis that would
otherwise have decided this silently.

**One thing not to carry across.** The committed-rewrite leg refuses
`conform/undeclared-path`, and *where* it refuses is fixture-specific (F-P05-43):
F2 keeps its declaration at the repository root, which SL-001 declares no
selector for, so conform leg 2 refuses it before anything later ever looks. A
project that declared its own declaration path would get past that leg. The
generalisable claim is the byte-identity, not the token.

## Related

- DEC-136 — the settled home and implementation handoff.
- EVD-011 — the probe-evidence input above.
- [[interpretation-surface-ownership]] — the decision that opens this.
- [[interpretation-surface]] — what is being declared.
- ADR-019 — asset-policy lens for where artifacts live.
