# EVD-011: A capsule rewriting its own in-repo declaration leaves trusted-side behaviour byte-identical

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

## Datum

Guard probe (e) of the SL-241 C3 rig, three legs, all `pass`. It runs against
**F2 (`light-inrepo`)** — a fixture variant built for this purpose, carrying a
copy of the interpretation-surface declaration **inside** the repository the
capsule clones. That manufactures the exposure QUE-201's candidates 1 and 2
would have in shipped form; in the rig as originally drawn the attack was not
live, because the declaration sits outside the clone.

| leg | fixture | observed | what it establishes |
|---|---|---|---|
| baseline | `light` (F1) | `no refusal` | the run the substitution legs are compared against — F1 keeps its declaration outside the clone, so the capsule has nothing to rewrite |
| `decl-worktree` | `light-inrepo` (F2) | `no refusal` — **byte-identical to the F1 baseline** | the control plane never reads a declaration the capsule can write |
| `decl-committed` | `light-inrepo` (F2) | `conform/undeclared-path` | the S-side substitution never reaches the control plane's read either |

The evidence is **the comparison**, not the claim: byte-identity of trusted-side
behaviour between the run whose declaration was rewritten and the run that had
nothing to rewrite.

## What it settles for QUE-201

DEC-099 Amendment 2 (RV-340 F-5) ruled that the declaration is read from the
contracted base `B`, never from a harvested result — which is what makes all
three candidate homes equally sound and reduces QUE-201 to ergonomics. That
ruling is now **observed rather than argued**. Without this row QUE-201 would
have settled post-spike on argument alone.

## Do not generalise the third leg's token — F-P05-43

*Where* the committed rewrite refuses is **fixture-specific**. F2 keeps its
declaration copy at the **repository root**, and SL-001 declares selectors for
`src/**` and `.doctrine/**` only — so conform leg 2 refuses it as undeclared
before leg 3 or anything later ever looks.

A project that declared its own declaration path would get past leg 2, and the
clause that would still hold is the one this leg actually asserts: **the control
plane resolved B's command regardless.** The leg therefore records the refusal as
an *observation* and asserts the provenance separately — it does not score the
token against an expectation, because the expectation would be about SL-001's
selector list rather than about the model.

A reader who takes `undeclared-path` here as "the model refuses declaration
substitution at conform" has learned something false about every other project.
The generalisable claim is the byte-identity, not the token.

## Related

- [[interpretation-surface-declaration-home]] — QUE-201, the question this
  informs.
- SL-241 PHASE-05 T6, guard (e); `~/capsules/probes/guards/results.tsv`.
- Finding F-P05-43 (the token is fixture-specific); D-P05-22 (guard (e)'s three
  legs and its OID-excluding observable).
- EX-11 / VA-3 — the criteria this row discharges.
