# Review RV-372 — reconciliation of SL-246

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed.** Branch `dispatch/246` in `.worktrees/SL-246-audit` — the
capsule generation-17 hand-back (`capsule/SL-246/c17`, tip `22c4e32c7`) merged
with `edge` at `248249f02`. This was **not** a `/dispatch` run and there is no
candidate interaction branch: `SL-246` was driven by `/capsule-driver` in the
main worktree, handed back as a branch plus a whole-state-tree exhibit at
`.worktrees/SL-246-c17/.doctrine/state/capsule-exhibit`. The exhibit's phase
sheets were copied into this tree's runtime state so the harvest could read
them; nothing else was taken from it.

The merge is part of the reviewed surface and is deliberate: the branch had to
be brought current before the ledger could mint, because `RV-371` exists only on
`edge` and an unmerged mint would have collided with it — which is itself one of
this audit's findings.

**Lines of attack.**

1. **Does the tree match the plan's own criteria?** Six phases, each with
   immutable `EN-`/`EX-`/`VT-`/`VA-`/`VH-` rows. Three phases carry `VA` rows
   that are *grep claims about the whole tree*, not about a function —
   `PHASE-01` `VA-1` (exactly one per-kind facet field enumeration survives),
   `PHASE-03` `VA-1` (the block producer is a map, never a `filter_map`),
   `PHASE-06` `VA-1` (absence of the retired leaf's symbols). Each mandates a
   **positive control** at the branch point. Claims of this shape are the ones
   a worker can most easily assert without running, so each is re-derived here
   from the tree with its control.

2. **Is the conformance algebra clean, and is the registry honest?** Three
   selectors were added *during execution* with `DISCOVERED in PHASE-NN` notes
   — `src/design_run/gate.rs`, `src/main.rs`, `tests/e2e_boot_map_golden.rs`.
   Self-reported honesty is the easiest thing in the world to fake, so the
   question is not whether the notes read well but whether `slice conformance`
   is actually clean once they are believed, and whether anything else moved
   that no selector covers.

3. **Where did the implementation diverge from the locked design, and is every
   divergence carried by a record rather than by silence?** `design.md` is
   locked, so a divergence has nowhere to go but an amended criterion plus a
   `DEC`. Three are claimed: `DEC-274` (`SelectedRecord` re-sited to a leaf),
   `PHASE-03` `EX-2`'s blankness predicate, `PHASE-03` `EX-4`'s arity. Each is
   checked for whether it moved a *siting/mechanism* clause (legitimate) or a
   *ruling* (not).

4. **The design's own acceptance measure.** `R3` says a ~30% saving "has to be
   right about what it keeps". `VA-2` demands the claim be re-derived against
   the real corpus with enough detail to re-check, and `VH-1` puts the verdict
   to the human. Both are discharged here against `SL-244`, and the measured
   number is compared to the design's.

5. **What the slice broke elsewhere.** A verb reclaim (`DEC-261`) is a corpus
   event, not just a code event: shipped skill prose, memories, and other
   slices' plans can all cite the old meaning. `EX-7` priced a migration; the
   question is whether the price covered the whole population.

6. **Lifecycle and id hygiene.** Whether the slice's own lifecycle state,
   `notes.md` harvest, and the ids minted during execution are in a state a
   `/reconcile` and `/close` can actually consume.

**Invariants the subject is pinned to.** `C1` (default output unchanged),
`C2` (`knowledge show` byte-identical, its golden unedited), `C4` (one facet
enumeration), `C7` (facet-only read path at `Facets`), `I1`–`I7`, `X1`–`X7`,
`D3`/`D6` (markers composed once, at the per-record producer), `DEC-145`
through `DEC-151` and `DEC-261`; `ADR-001` (layering, and the frozen command
tangle at 76), `ADR-004` (outbound-only storage), `STD-001` (no magic strings),
`STD-002` (naming), `STD-003` (no silent skip); `POL-002` (platform
independence).

**Where the bodies are likely buried.** In descending order of prior
probability: the `VA` grep claims (cheap to assert, expensive to check); the
migration population `EX-7` did *not* enumerate; anything the capsule tier
filed to the backlog instead of doing, since filing is the cheapest way to make
a gap look handled; and the gap between a passing suite and a feature a human
would actually reach for.

## Synthesis

**SL-246 is done, green, and conformant, and its own human verification says the
feature is disappointing.** Both halves of that sentence are load-bearing and
this audit refuses to collapse them into each other.

### The closure story

Six phases landed. The gate is green on the merged surface — `just gate` exits 0
with **7801 tests passing and zero failures**, including
`tests/architecture_layering.rs`, which is what actually proves `DEC-274`'s claim
that the frozen command tangle stays at 76 rather than leaving it as an
assertion. Conformance is clean: **22 conformant, 0 undelivered**, and the six
undeclared paths are all authored governance files a selector was never going to
cover (the `DEC-274` record, the `ADR-001` leaf row, the amended `plan.toml`, the
selector registry itself).

The three `VA` rows were the ones most worth distrusting, because a grep claim
about a whole tree is the cheapest thing in the world to assert without running.
All three were re-derived here with the positive control each mandates:

- **`PHASE-01` `VA-1`** — the `why_matters` probe went 5 hits → 4, and the two
  that went are precisely `format_facet`'s and `facet_json`'s per-kind matches.
  `C4` holds on the render path. The surviving fourth is `render_facet`, which is
  `#[cfg(test)]` and was already so at the branch point (`F-9`).
- **`PHASE-03` `VA-1`** — `render_block` is a `map`, with exactly two cases
  outside it (`Skip`, and the empty selection's `X1` line). No `filter_map`, no
  `?` that could drop a record. `I5`'s never-dropped clause is a property of the
  shape, as designed.
- **`PHASE-06` `VA-1`** — all seven retired symbols return zero hits in `src/`
  and the five deprecated-leaf invocations return zero in `tests/`, against
  4 / 3 / 2 / 5 at the branch point.

The three in-flight divergences from the locked design all moved a *siting* or
*mechanism* clause and left the *ruling* intact, which is the distinction that
makes an amendment legitimate rather than a quiet rewrite. `DEC-274` is the
strongest of the three: it found that `EX-2`'s stated justification was
**factually wrong** about the tier map, declined the precedented escape of
bumping the tangle baseline, and took the leaf siting that delivers `EX-2`'s goal
more completely than `EX-2`'s own letter did. That is the behaviour the design
process is for.

Three selectors were registered mid-execution with `DISCOVERED in PHASE-NN`
notes naming what `design.md` § 5.6's touch-set missed and why. Self-reported
honesty proves nothing on its own, so the check was whether the algebra is
actually clean once they are believed — it is, and `0 undelivered` is the
non-trivial half of that.

### Standing risks

**The measurement will regress.** `R3` predicted the `facets` level at ~30% of
`full`; measured on the real corpus it is **11.5%** (12,873 B against 111,518 B).
That is *better* than designed, for a reason the design did not model: five of
the sixteen records surfaced carry no facet at all, so a third of the block is an
unfilled marker rather than content. As `IMP-403` is worked and the corpus fills
in, the saving shrinks toward the designed 30%. Anyone who takes 11.5% as the
feature's steady-state cost will be wrong (`F-4`).

**`C7` is live and now load-bearing on a backlog item.** The facet-only read at
`Facets` was a design mandate that no plan criterion bound, so the composed read
reads every record's `.md` from disk at every level and discards it at `Facets`.
`CHR-074` owns it. The type's own doc comment asserted the opposite; that half
was repaired in this audit (see below) rather than deferred, because a type that
lies is worse than a gap that is written down (`F-3`).

**Two structural seams in the capsule tier**, both of which will recur on every
capsule-driven slice rather than being lapses by any one worker: nothing in the
tier owns the *slice's* lifecycle state (`F-11`), and nothing in it owns
`notes.md`, so six phases of durable material sat only in `rm -rf`-able phase
sheets (`F-12`). Both were fixed here; neither is fixed at its cause.

**Id allocation is unguarded on a long-lived branch.** `ISS-462` was minted twice
for two different items. `ISS-279` already owns this, and this instance widens it
twice — the kind is a backlog item rather than a review, and the topology is a
capsule branch driven from the *main* worktree with no coordination tree at all
(`F-1`).

### Tradeoffs consciously accepted

`F-7` and `F-8` are recorded as **correct code whose product choice is disputed**,
and are routed forward rather than fixed. `Default = Skip` is `C1` expressed in
the type and is why `I1` holds; the spelling of the knowledge block is
`knowledge show`'s shared producer, which is exactly what makes `C4` and
`STD-001` hold. In both cases the property that makes the code right is the
property the human disliked, and a fix has to break one without losing the other.
That constraint is the valuable part and belongs on the record, not a patch.

`F-6` was **not** taken as `fix-now` despite being a genuine `STD-003`
conformance gap. The repair adds a rendering the design never specified, on both
arms, under `D1`'s one-meaning binding, without re-opening `C2`. That is design
work. It is filed as `ISS-467` with the cheap route identified — the tier is
already a column on the authored table, so the withheld key set is derivable
without a second per-kind match.

### Corrections made in this audit

Two, both stated plainly because a ledger that hides its own repairs is worth
less than one that does not:

1. **`F-3`'s doc-comment half was fixed here, not briefed.** The disposition text
   routed it to the reconciliation brief as a per-slice direct edit. That was
   wrong on the surface: `/reconcile` writes `design.md` and `slice-NNN.md`, not
   `src/`. A doc comment is a code fix in audit scope, so it was made —
   `src/knowledge.rs` `KnowledgeLevel::Facets` now says it never *renders* the
   body but does still read it, names `CHR-074`, and `read_selected`'s comment
   gains the same pointer plus the `fs::metadata` route. `cargo build` and
   `cargo clippy` clean. Only `F-3`'s `C7` half reaches the brief.
2. **This audit broke the repository and fixed it** (`F-14`, the one `blocker`).
   Working around a hand-back worktree whose git linkage points at a host path
   absent in the jail (`F-13`), it exported `GIT_DIR`/`GIT_WORK_TREE` into a
   `cargo test` run; a non-hermetic fixture then wrote `core.worktree` into the
   *shared* `.git/config`, silently repointing every git invocation in the
   repository — from every worktree, including the primary tree on `edge` — at
   one directory. A subsequent `git merge` wrote its results into the wrong tree.
   All of it is reverted and both trees verify clean. The class is filed as
   `ISS-468`, with `ISS-256` as the resolved sibling that makes it a class.
   One residue is deliberately **not** touched: `[user] email = t@t / name = t`,
   the fixture's literal values, remains in `.git/config`. It is inert in the jail
   but would misattribute a host-side commit, and a git identity is the owner's to
   set.

### The verdict, and what it does and does not mean

`VH-1` was discharged by the named human against the built binary on the real
corpus — which is what a `VH` row requires — and came back: *"disappointing as a
feature, but not obviously incorrect."*

That is not a failing audit and must not be recorded as one. Every `EX` row
holds, the gate is green, the invariants are re-derived above. What it means is
that the slice met **objective 1 at the mechanism and not at the outcome**: an
entity read *can* carry its records, and a reader does not yet want to use the
one it carries. The three specific complaints have three specific homes —
`ISS-467` (the withheld tier is a real conformance defect, not taste), and
`IMP-465` (the default, the flags, the styling).

The most useful thing the attestation produced is evidence against a decision the
design had already made: `design.md` § 7.2 `D4` and `DEC-145` both pushed the
discoverability pointer out of scope as "the separate, cheap answer". First real
use says the composed read without that pointer does not get *found*. `IMP-398`
owns the pointer line; this is the argument for promoting it from a nicety to
this feature's other half.

## Reconciliation Brief

Every non-`aligned`, non-`tolerated` finding that touches design or governance,
grouped by the surface `/reconcile` will actually write.

### Per-slice (direct edit)

- **`design.md` § 5.2 / § 5.6 — `C7` is not delivered.** The facet-only read path
  at `Facets` is a design mandate that no plan criterion bound and the
  implementation does not meet: `read_record` (`src/knowledge.rs:1970`) reads the
  `.md` unconditionally at `:1980`, and `render_record` discards it at `Facets`.
  Record the deferral and name its carrier, `CHR-074`. Note for the writer:
  `DEC-149`'s unfilled marker needs the body's *size*, not the body, so
  `fs::metadata` satisfies both the marker and `C7` — the current read is not
  forced by the design. (`RV-372` `F-3`. The doc-comment half of this finding was
  already repaired in-audit; do not re-file it.)

- **`design.md` § R3 / the acceptance measure — the measured number, and why it
  is not the steady state.** Replace or annotate the ~30% claim with the
  re-derived measurement: `design show SL-244 --knowledge {skip,facets,full}` =
  202,243 / 215,116 / 313,761 bytes, so the block is 12,873 B at `facets` against
  111,518 B at `full` = **11.5%**. `full` agrees with `research.md`; only `facets`
  diverges. Record the cause — 5 of 16 records surfaced carry no facet
  (`IMP-403`) — and the consequence: the saving shrinks toward 30% as the corpus
  fills. (`RV-372` `F-4`.)

- **`design.md` § 5.2 — the marker vocabulary is incomplete.** `DEC-149` / `X5` /
  `I6` model three *empty* states (by-design, unfilled, unreadable) and no
  **withheld** state, so at `Facets` a reader cannot distinguish an Argument-tier
  field that was withheld from one that is empty from a kind that has none.
  Record the omission where the next implementer reads it; the work is `ISS-467`.
  (`RV-372` `F-6`.)

- **`slice-246.md` Context — the record census is stale.** It says *"Fifteen
  knowledge records point at the slice — twelve via `shapes` … and three `DEC` via
  `references(concerns)`"*. Measured: **sixteen** — eleven via `shapes`
  (`DEC-120`–`127`, `DEC-138`, `DEC-139`, `QUE-206`) and five via
  `references(concerns)` (`DEC-140`, `-141`, `-142`, `-144`, `EVD-012`).
  `EVD-012` is attributed to `shapes` and arrives via `concerns`. Prefer replacing
  the tally with the query that produces it (`doctrine relation list --target
  SL-244`) — a census counted at scoping time is a measurement with a date on it,
  and this one drifted in six weeks. (`RV-372` `F-10`.)

- **`slice-246.md` — record the `VH-1` outcome.** The slice's human verification
  is discharged with a negative product verdict and no correctness claim; the
  verdict should live in the slice, not only in this ledger. Name its carrier
  `IMP-465` and the split-out `ISS-467`. (`RV-372` `F-5`, `F-7`, `F-8`.)

### Governance/spec (REV)

**None.** Stated explicitly, with the three candidates considered and declined so
`/reconcile` does not re-derive them:

- **`SPEC-019` does not gain the tiering.** `DEC-150`'s Deciding/Argument split is
  a *display* concern under the slice's own carried assumption `A2` — `SPEC-019`
  declares no tiering and `SL-246` is the authority of first impression. `A3`
  keeps the `SPEC-019` four-of-seven gap (`ISS-316`) outside this slice. Nothing
  here changes either.
- **`DEC-145` is not overturned.** `F-7` disputes its product choice and does not
  falsify it; the evidence is recorded in `IMP-465`, and `DEC-145`'s own text
  already names discoverability as out of its scope.
- **`ADR-001` needs no revision.** The slice added one `leaf` row to
  `.doctrine/adr/001/layering.toml` for `src/selection.rs` and left the
  `[tangle_baseline]` ratchet untouched at 76 — additive, precedented, and
  verified by `tests/architecture_layering.rs` in the green gate. `DEC-274`
  explicitly declined the baseline bump that would have needed an argument.

### Not a reconcile surface — recorded so it is not mistaken for one

`plan.toml`'s amended `EX-2` / `EX-3` / `EX-4` rows and the appended `VT-3` are
the *durable record* of this slice's divergences and are correct as they stand.
`EN-`/`EX-`/`VT-` ids are immutable-append, so none of this is a direct-edit
surface. `F-9` (`PHASE-01` `VA-1`'s literal over-claim) is `aligned` for the same
reason — the qualification lives in this ledger, where the next reader running
that grep will find it.

---

## Addendum — `F-15`, raised during synthesis

Line of attack 5 (*what the slice broke elsewhere*) was only half-discharged when
the first fourteen findings were written: the shipped-prose sweep had run, the
corpus sweep had not. Completing it found a third stale population and it is
raised as `F-15` rather than folded into the synthesis, because a finding found
late is still a finding.

`EX-7` priced the `design show` reclaim as four emitted strings, one prose line
and the test call sites. `CHR-073` added five memories. Neither covers **other
slices' authored `notes.md`** — committed, agent-read on resume, and now wrong at
five sites across `SL-247`, `SL-253` and `SL-256`. `SL-233`'s plan and notes cite
the verb as a symbol in a historical record and are correctly unaffected; shipped
prose is clean.

The detail worth keeping: `mem_01a0b2a8431371539e7911821e9c8da4` was recorded
from this slice's own design review (`RV-370` `F-16`) and its thesis is that *a
design which prices a change must name the population, or a reviewer cannot check
the figure*. That memory's own example is this migration's count being wrong by
one population. It is now wrong by two. The lesson was recorded and the count was
never re-derived — which is the more useful version of the lesson.

Disposed `fix-now` by widening `CHR-073` with the five sites enumerated, rather
than by editing three other slices' notes from inside this audit. The
reconciliation brief's landing constraint stands and is now larger: **`CHR-073`
must be discharged with or before `SL-246` lands.**

### Final gate

`just gate` on `dispatch/246` **with this audit's own changes included**: exit 0,
**7801 passing, zero failures**, no suite red.

---

## Reconciliation Outcome

Written 2026-09-20 by `/reconcile`. All 15 findings were terminal (`verified`)
at entry; none were re-dispositioned here — remediation is recorded below, not by
mutating a finding.

### Direct edits applied

`design.md` (locked; `/reconcile` is its sanctioned writer) and `slice-246.md`.
Eleven edits against the brief's five items — the two surplus are single-sentence
pointers, taken with the user's agreement so a superseded figure is not left
reading as settled anywhere in the two documents.

- **`design.md` § 5.2 *The level*** — `Facets`' *"Never reads the `.md`"* named as
  `C7` and as the one clause of the section the slice did not deliver, pointing at
  the note under *Rendering*. The Rust block itself is untouched: the correction
  belongs where the claim is read, not inside the specification of the type.
  (`F-3`)
- **`design.md` § 5.2 *Rendering*** — the *"`C7` discharged"* paragraph is left
  standing as the design's mandate and followed by the reconciliation: no plan
  criterion bound `C7`, `read_record` (`src/knowledge.rs:1970`) reads the `.md` at
  `:1980` for every level, `render_record` discards it at `Facets`. Carrier
  **`CHR-074`**. The brief's route is recorded with it — `DEC-149`'s unfilled
  marker needs the body's *size*, so `fs::metadata` satisfies the marker and `C7`
  together, and the read the code performs is not forced by this design. `IMP-459`
  defers with it. (`F-3`)
- **`design.md` § 5.6 Code Impact** — the `src/knowledge.rs` row's *facet-only read
  path* marked **not delivered**, carried by `CHR-074`. (`F-3`, its § 5.6 half.)
- **`design.md` § 5.2 *The three empty states*** — the marker vocabulary recorded
  as incomplete: three **empty** states, no **withheld** state, so at `Facets` a
  withheld Argument-tier field, an absent one and a kind that has none render
  identically. The section's standing `STD-003` claim is qualified rather than
  deleted — it holds for the *read* and not for the *level*. Carrier **`ISS-467`**,
  with the cheap route (the tier is already a column on the authored
  `FacetFieldRow` table, so the withheld key set is derivable without a second
  per-kind match, preserving `C4`). (`F-6`)
- **`design.md` § 8 `R3`** — the re-derived measurement: 202,243 / 215,116 /
  313,761 B, block **12,873 B at `facets` against 111,518 B at `full` = 11.5%**,
  not ~30%. `full` agrees with `research.md`; only `facets` diverges. Cause
  recorded (5 of 16 records surfaced carry no facet — `IMP-403`) and so is the
  consequence the brief insisted on: the saving **regresses toward the designed
  30%** as the corpus fills, and 11.5% must not be quoted as the steady state.
  (`F-4`)
- **`design.md` § 1 *Target behaviour*, § 3.3 `F3`** — the two other sites carrying
  the ~30% claim gain a one-sentence pointer to § 8 `R3` rather than a second copy
  of the measurement, per the document's own single-copy convention. *(Beyond the
  brief; agreed with the user.)* (`F-4`)
- **`design.md` § 9.4 *By human*** — `VH-1` recorded as discharged with its
  verbatim answer, explicitly as a negative **product** verdict carrying **no
  correctness claim**, beside the standing evidence (every `EX` row holds, gate
  green, conformance 22/0). Carriers `IMP-465` and `ISS-467`. (`F-5`)
- **`slice-246.md` Context** — the fifteen-record census replaced by the query that
  produces it (`doctrine relation list --target SL-244`), with the audit reading
  parenthesised (sixteen; eleven `shapes` / five `concerns`; `EVD-012` arrives
  under `concerns`) and the reason a dated census drifts. (`F-10`)
- **`slice-246.md` Scope & Objectives** — the scope document's own ~30% figure
  annotated with the re-measurement and its direction. *(Beyond the brief; same
  finding, same agreed rationale.)* (`F-4`)
- **`slice-246.md` Verification / closure intent** — new `### Outcome — VH-1
  discharged 2026-09-20`: the verdict verbatim, the mechanism-not-outcome reading,
  the two carriers, `F-7` / `F-8` recorded as *correct code whose product choice is
  disputed* (the property that makes the code right is the property the reader
  disliked, so a fix must break one without losing the other), and the
  discoverability evidence routed to `IMP-398` against `DEC-145` / § 7.2 `D4`.
  (`F-5`, `F-7`, `F-8`)

### REVs completed

**None**, as the brief directs. Its three declined candidates — `SPEC-019`
tiering (a display concern under `A2`), `DEC-145` (disputed, not falsified), and
`ADR-001` (one additive `leaf` row, ratchet untouched at 76) — were not re-opened.

### Not written, and why

- **`plan.toml`.** Its amended `EX-2` / `EX-3` / `EX-4` and appended `VT-3` are the
  durable record of this slice's divergences. `EN-`/`EX-`/`VT-` ids are
  immutable-append, so it is not a reconcile surface. Untouched.
- **`src/`.** `F-3`'s doc-comment half was repaired in-audit and is not re-filed
  here; `/reconcile` writes `design.md` and `slice-NNN.md`.
- **Other slices' `notes.md`.** `F-15`'s five stale sites across `SL-247`,
  `SL-253` and `SL-256` are enumerated in **`CHR-073`** and are discharged there,
  not from inside this pass.

### Findings not reaching a write surface

`F-1`, `F-2`, `F-11`, `F-12`, `F-13`, `F-14` were dispositioned `fix-now` and
remediated within the audit; `F-9` is `aligned`, with its qualification held in
this ledger where the next reader of that grep will find it. None required a
reconcile write.

### Standing before `/close`

`CHR-073` remains **open and gating**: *must land with or before the code*, now
covering five memories plus the five `notes.md` sites `F-15` added. It is a
`/reviewing-memory` pass — its own routed stage, not reconcile's. `CHR-074`,
`ISS-463`–`ISS-469` and `IMP-465` are open and carried; carried assumptions `A2`
and `A3` were not disturbed by the audit and stand as written.

Reconcile pass complete — handoff to `/close`.
