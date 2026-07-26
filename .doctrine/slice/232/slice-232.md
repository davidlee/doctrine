# Corpus-aware memory verify gate

## Context

Split out of **SL-230** by **DEC-027** at RV-307 round 8. SL-230 retains the
memory body-write seam and attestation invalidation; this slice takes the
`verify` gate and the SPEC-007 amendment its contract change requires.

The split is not a fresh start. This slice inherits a design that survived
**eight adversarial rounds and 29 findings** on the gate alone — the decisions,
the measured censuses, and the reasoning are carried over intact in `design.md`.
What it does not inherit is the constraint that made the last two rounds
unproductive: the gate was being designed inside a slice whose charter could not
accommodate the schema change its own decisions kept pointing at.

### The problem

`memory verify` refuses on any dirty working tree. Doing doctrine work means the
authored corpus is essentially always dirty, so verifying a memory is routinely
blocked by edits that say nothing about whether its claim still holds. In
practice agents hit the refusal and reach for `git stash` rather than
`--allow-dirty` (undiscoverable, and shipped after IMP-221 was filed).

The naive fix — ignore doctrine's own authored trees — is wrong, and RV-307 F-1
and F-6 proved it: memory items live at `.doctrine/memory/items/<key>/`, so a
blanket exclusion removes **the memory being verified**, stamping a commit that
provably lacks the attested body. And **81 items in this corpus declare
`.doctrine/**` scopes**, so an ADR a memory explicitly names is claim evidence
exactly as `src/` is.

So the gate must be **claim-aware**: exclude unclaimed dirt, never claimed
evidence. That requires constructing, per memory, the surface its attestation is
actually about — which is where the difficulty lives.

### Why this needed its own slice

Two round-8 blockers, neither a text defect, established that the claim surface
is a larger problem than a body-write slice could carry:

- **RV-307 F-36** — DEC-020 requires `validate` to raise every non-contributing
  scope entry, but D11 leaves `validate` with no contribution probe: it keeps a
  historical, `scope.paths`-gated seam that cannot implement it. Supplying one is
  an undesigned second per-entry git path plus a corpus-wide continuation policy.
- **RV-307 F-37** — the premise that a non-resolving entry contributes nothing is
  **false**. Reproduced on git 2.54.0 by three routes: `missing/../link`, a
  sparse checkout, and a `scope.paths` literal whose filename contains `*`. Each
  contributes while bypassing canonicalisation and reads clean against a dirty
  target, restoring the false attestation I9 exists to close.

Both are open and inherited by this slice.

## Scope & Objectives

### 1. Claim-aware dirty-tree gate for `verify`

- The clean/dirty decision ignores modifications confined to doctrine's own
  authored trees — `.doctrine/**` and the repo-root `memory/` tree — **except**
  the memory's own item directory and its declared scopes, which are the evidence
  the attestation is about.
- `verify` therefore asks two questions: is the **unclaimed** tree dirty, and is
  the **observable** claim committed?
- The exclusion set derives from doctrine-owned path constants, never a hardcoded
  literal list (POL-002, STD-001).
- A dirty **source** tree still refuses; `--allow-dirty` remains the escape hatch
  with its `checkout_state_id` stamping unchanged, taken from an *unexcluded*
  capture (RV-307 F-13).

### 2. The claim-surface constructor

- One ordered algorithm turning a memory's declared `scope.paths` / `scope.globs`
  into git pathspecs, rooted at the **canonicalised uid directory** (F-15: keys
  in `items/` are symlinks and git will not traverse one in a pathspec).
- Scope entries are **untrusted data, never pathspec syntax** (F-18): every entry
  is magic-prefixed so nothing a memory declares can subtract from what it is
  measured against.
- **Resolution must be total over contributing entries** — the open problem.
  F-37 shows resolution failure does not imply non-contribution, so the current
  algorithm's step 4/5 premise needs replacing, not patching.

### 3. Non-contribution reporting, and the declared-boundary question

- Per **DEC-020**, `verify` attests over every non-contributing entry and reports
  each on stderr; `validate` raises them as corpus-health findings. No
  classification: three derived instruments were refuted because each reads local
  repository state (F-21, F-25, F-31).
- **`validate` needs a contribution mechanism it does not have** (F-36). This is
  new design work, not a wiring change.
- **The declared boundary** — a persisted signal marking evidence git is not
  expected to observe — is the stable answer DEC-020 deferred. Scope it here with
  **IMP-318** (persist attested coverage) and **QUE-173** (body digest): all
  three are the same schema change, each making an attestation say more than a
  sha. This is the objective SL-230 structurally could not hold.

### 4. Historical scope consumers — decide, don't inherit

- `validate`'s staleness check and `retrieve::git_facts` both keep a raw
  `scope.paths`-gated seam (R7, F-19/F-24/F-27/F-28). They ask a *historical*
  question where canonicalising against today's checkout erases a committed
  symlink retarget (measured 1 → 0), so they need a **second, history-stable**
  surface rather than reuse of `verify`'s.
- Adoption is a **dataflow change**, not a call-site swap: `collect_all` unions
  `items/` and `shipped/`, so a row's origin is unrecoverable from its uid (F-28).
- Gated on **QUE-175 / OQ-2**: does claim-surface drift feed retrieve-side
  ranking, or only `validate`? Answer it here. Routed as **IMP-317**, to be closed
  `wont-do` if the answer is no.

### 5. SPEC-007 reconciliation — REV-034

The retired "clean working tree, refusing a dirty one" contract lives at three
sites: `spec-007.toml:22`, `spec-007.md:132-133`, and **REQ-147**, whose title is
that contract verbatim (RV-307 F-5). The implementation already diverged when
`--allow-dirty` shipped; this slice changes it further. Applied at close so spec
and code turn over together. Moved here from SL-230 by DEC-027 — the contract is
changed by the gate, not by body-write.

### 6. Verify refusal names its escape hatch

The dirty-tree refusal names `--allow-dirty` instead of prescribing a commit.

## Non-Goals

- **The memory body-write verbs.** SL-230's, and already designed and reviewed
  there — this slice must not re-open them.
- **Attestation invalidation on claim-field edit** (D4/D8/D5). Also SL-230's.
  Note the coupling: SL-230 ships invalidation *without* this slice's relaxation,
  so R4 (mass re-verification friction) runs unmitigated until this lands. That
  is DEC-027's accepted tradeoff and the reason to sequence this next.
- **Loosening `thread_expiry`** or any retrieval-side gate — reviewed canon
  (SL-008 D6).
- **An MCP `memory_verify` tool.** SL-164's stated rationale dissolves once the
  clean-tree precondition relaxes, but re-litigating that exclusion is its own
  decision.
- **Masters coverage.** `verify` is items-only by construction (E2); masters are
  unanchored and `collect_all` never scans them (R5). The digest work in
  objective 3 is what would reach them.

## Risks, assumptions, open questions

- **R-A — the claim surface may not be totalisable from git alone.** F-37 is the
  second time a totality claim over path resolution has failed (F-26 was the
  first). Before asserting a replacement rule, enumerate the reachable shapes and
  probe each — a tool property is a claim needing a falsifier, not a premise.
- **R-B — `validate`'s contribution probe is undesigned** (F-36), including its
  continuation policy when one memory's surface is malformed (F-29's shape).
- **R-C — R4 runs unmitigated** while SL-230 is shipped and this is not.
- **OQ-A — does the declared boundary land as one schema change** with IMP-318
  and QUE-173, or can they be sequenced independently?
- **OQ-B — QUE-175 / OQ-2**, gating objective 4.

## Summary

## Follow-Ups
