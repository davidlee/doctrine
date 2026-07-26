# REV REV-041 — reconcile SL-230

Revision (ADR-013) — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

Reconciliation of **SL-230** (memory body-write seam) against **RV-313 F-6**, the
audit's one governance-surface finding. One `modify` row, against **SPEC-007**.

### What is wrong

SPEC-007 § "Git-anchored staleness" promises that every undecidable reachability
case — **naming `non-ancestor anchor` explicitly** — resolves to an explicit state,
"never a silent hide or a silent over-trust". `memory_health_findings` Checks 2 and
4 do precisely the forbidden thing in that case: `commits_touching` returns `None`
for a `verified_sha` that is not an ancestor of HEAD, and both call sites
(`let Some(n) = … && n > 0`) let the `None` fall out and emit nothing — *cannot
determine* rendered identically to *no drift*, on **67 of 115** anchored memories.

Audit deliberately did not adjudicate this, because **the sentence does not
determine its own scope**. Reconcile went back to the spec's two tiers and found
the ambiguity is not a soft reading but an outright **self-contradiction**:

| tier | text | implies |
|---|---|---|
| `spec-007.md` Overview (`:15`) | "the scope-aware `find`/`retrieve` reader with its deterministic ranking **and git-anchored staleness**" | staleness is a property *of the reader* |
| `spec-007.toml` `responsibilities[20]` | "**Compute git-anchored staleness** …" — its own list item, *separate from* `[19]` "Provide the scope-aware reader (`src/retrieve.rs`)" | staleness is an engine-wide responsibility |

Two further facts, recorded because they bound what this REV may honestly assert:

- **`validate` is absent from SPEC-007 entirely** — `validate`, `health` and
  `finding` occur **zero** times in the spec. The spec never names the surface whose
  conformance F-6 asks about.
- The guarantee's remedy vocabulary is **render vocabulary**. It promises resolution
  to one of five *states*, two of which (`unanchored`, `reference`) exist only to be
  rendered. `validate` emits findings, not states, and has no mode in the list
  corresponding to "emit a health finding". Declaring `validate` non-conformant
  against the five-state contract *whole* would be a verdict it cannot discharge.

### What this REV does

It splits the sentence along the seam the evidence already marks, rather than
choosing one of the two readings wholesale:

- the **five-state render contract** binds the `find` / `retrieve` axis — that is
  where a staleness state is rendered;
- the **prohibition on silent over-trust is surface-independent** and binds *any*
  git-anchored staleness computation in this engine, `memory validate`'s health
  checks included.

This is not a split-the-difference fudge. It distinguishes the *render contract*
(reader-scoped, and vocabulary-bound) from the *epistemic prohibition*
(engine-wide), a distinction the spec's own two tiers already gesture at and
neither currently states. It also removes the Overview/`[20]` contradiction, which
would otherwise leave the spec asserting both readings after this REV lands.

**Consequences, recorded explicitly.** `memory validate`'s staleness checks are
**non-conformant** with SPEC-007 as amended: a silent "no drift" on a non-ancestor
anchor is a silent over-trust, whatever surface computes it. **ISS-257** is
therefore a **conformance fix**, not an improvement. The code outcome was identical
under every reading audit considered — ISS-257 is the fix either way, and the
conforming shape already exists one module away (`src/coverage.rs:150-166`,
`IsStale::{Fresh,Stale,Unknown}` with a documented `None ⇒ Unknown` contract over
the *same* seam). What this REV moves is the spec's clarity and ISS-257's standing.

The defect is **not** in `commits_touching`'s ancestry guard, which is correct and
must stay. It is at the two call sites.

### Change rows

- **[RV-313 F-6] `modify` SPEC-007** — amend § "Git-anchored staleness" to state
  which surfaces the guarantee binds; amend the Overview sentence and
  `responsibilities[20]` so both tiers agree. Surfaced for manual landing.

**No REQ status changes.** No requirement was verified or falsified by SL-230's
evidence; recorded so it is not re-derived downstream.

## Before / after excerpts

### 1. `spec-007.md` § "Git-anchored staleness" — the guarantee sentence

**Before**

> Every undecidable reachability case (shallow/partial clone, detached HEAD,
> rebase, non-ancestor anchor, non-git project) resolves to an *explicit* state —
> `fresh` / `stale` / `unknown` / `unanchored` / `reference` — never a silent hide
> or a silent over-trust.

**After**

> Every undecidable reachability case (shallow/partial clone, detached HEAD,
> rebase, non-ancestor anchor, non-git project) resolves to an *explicit* state —
> `fresh` / `stale` / `unknown` / `unanchored` / `reference` — never a silent hide.
> That five-state resolution is the **render contract**, and it binds the `find` /
> `retrieve` axis, which is where a staleness state is rendered.
>
> The prohibition on **silent over-trust is surface-independent**: it binds *any*
> git-anchored staleness computation in this engine, including `memory validate`'s
> health checks. A computation that cannot decide reachability must say so on
> whatever surface it speaks, and must never render *cannot determine* as *no
> drift*. A surface that emits findings rather than states discharges this by
> emitting a finding, not by falling silent.

### 2. `spec-007.md` Overview (`:15`) — the contradiction

**Before**

> the scope-aware `find`/`retrieve` reader with its deterministic ranking and
> git-anchored staleness,

**After**

> the scope-aware `find`/`retrieve` reader with its deterministic ranking,
> git-anchored staleness as an engine-wide computation,

### 3. `spec-007.toml` `responsibilities[20]` — mirror the split

**Before**

> Compute git-anchored staleness in four explicit modes (…), resolving every
> undecidable reachability case to an explicit
> fresh/stale/unknown/unanchored/reference state rather than a silent hide.

**After**

> Compute git-anchored staleness in four explicit modes (…). On the `find`/`retrieve`
> axis every undecidable reachability case resolves to an explicit
> fresh/stale/unknown/unanchored/reference state rather than a silent hide; the
> prohibition on silent over-trust is surface-independent and binds every
> git-anchored staleness computation, `memory validate`'s health checks included.
