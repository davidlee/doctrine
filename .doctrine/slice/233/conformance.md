# SL-233 — conformance dispositions

Authored at the audit's campaign session **S4**, 2026-08-02, as the second half
of `review-campaign.md` §6's S4 row.

This file carries the **judgement**, which is not derivable. It deliberately does
not copy the report's tables — those are derived and regenerable (the storage
rule):

```bash
doctrine slice conformance 233                              # the registry view
doctrine slice conformance 233 --against 2f70392e5..HEAD    # the git view
```

**The headline: of 24 reported entries, exactly 2 are conformance signal, and
neither is a code defect.** Everything else is either an artefact of how the
report is computed or a selector that was declared defensively and turned out not
to be needed.

---

## 1. Read the report twice — its base is incomplete

`slice conformance` folds a slice's **recorded** source deltas, not its git
range. Only the eight funnel-dispatched phases recorded one; of the seven that
landed via direct commits (07, 08, 10, 11, 12, 15, 16), most never did. So the
default report's *actual* set is a strict subset of what the slice changed, and
five selectors that were in fact delivered are reported undelivered.

Running both views is what separates the two populations:

| view | undeclared | undelivered |
|---|---|---|
| registry (default) | 13 | 11 |
| git (`--against`) | 183 | **6** |

The **undelivered** number is the one the git view corrects — 11 → 6. The
**undeclared** number moves the other way and that direction is noise: the git
view sweeps every incidental backlog item, knowledge record and observation the
slice minted along the way, none of which a design-target selector should ever
have claimed. **Disposition work belongs on the registry view; the git view is
the control that tells you which of its rows to believe.**

---

## 2. Undelivered — 11 reported, 6 real, **0 defects**

### 2a. Five are recording artefacts

`.doctrine/review/320*`, `.doctrine/review/320/**`,
`.doctrine/slice/233/sketches/**`, `Cargo.toml`, `skills-lock.json` — all five
*were* changed inside the slice range. They are invisible to the registry view
only because the phases that changed them recorded no source delta.

**Disposition: no selector action.** The residual is that a non-funnel phase
records no delta unless someone runs `slice record-delta` by hand, so the
registry silently under-reports. That same root cause produced the PHASE-14 row
error corrected in `review-campaign.md` §1, and it is captured as a friction
observation rather than re-derived here.

### 2b. Six are real, and every one dissolves on measurement

**An undelivered selector is not a defect.** A design-target selector is declared
at planning time to mean *this may need to change*; it going untouched can simply
mean the mechanism did not require it. Each of the six was tested rather than
assumed, and all six are that case:

- **`tests/architecture_layering.rs`** — declared for "ADR-001 tier registration
  for `design_run`". The registration **landed**, in
  `.doctrine/adr/001/layering.toml:30` (`design_run = "leaf"`), which the report
  lists as conformant. The test needed no edit because it is
  **discovery-based** — `discover_units()` walks the source tree and
  `extract_edges()` parses imports; it holds no hand-maintained module list. The
  selector anticipated a hand-edit the design did not need.
- **`install/hymns/README.md`** — declared for "hymn band registry". PHASE-07
  shipped `install/hymns/stage/design.md`, the first stage-band *snippet*. The
  README's Band Registry **already listed** the `stage` band before PHASE-07;
  authoring a snippet in a registered band does not touch the registry, and the
  README says so ("Bands are a closed registry, fixed order").
- **`src/reserve.rs`** — declared for PHASE-05's `DEC-086` reserve-then-journal
  protocol. PHASE-05 reached the existing seam **through** `knowledge.rs:1168`
  (`crate::reserve::backend(...)`), which is in its range. The seam was ridden,
  not modified — which is the convention, not a miss.
- **`src/fsutil.rs`** — PHASE-15 `EX-13`'s `LockGuard` extraction target.
  **Not delivered, and lawfully so**: `EX-5` named the fallback in advance ("if
  generalisation proves infeasible within the phase…"), and `DEC-100` records the
  infeasibility and supersedes `DEC-105`. `LockGuard` remains in `src/review.rs`.
  This is the one entry where the selector is undelivered *because the work was
  withdrawn* rather than *because it was unnecessary*, and the withdrawal is on
  the record. **The live obligation `DEC-100` creates is not here but on the
  prose**: no surface may claim preservation or detection. Both S4 PHASE-15
  passes checked that independently and found the claim withdrawn from
  `spec-029.md`, `spec-029.toml`, `install/hymns/stage/design.md` and two
  `design.rs` comments.
- **`tests/e2e_help_families_golden.rs`** — pins the eight top-level command
  *groups* and one representative each. `design` is a new command inside the
  existing `change` group, not a new group, so the golden legitimately did not
  shift.
- **`tests/e2e_boot_map_golden.rs`** — pins the boot map's load-bearing *rules*,
  and asserts that `slice` carries `design`/`plan`/`phases`. That assertion still
  holds, because the deprecated `slice design` shim is exactly what PHASE-14
  `EX-4` preserved.

**Noted, not raised**: the last two together mean nothing golden-tests the new
top-level `design` spine line. That is a coverage observation about the goldens,
not a conformance defect of this slice, and it is recorded here so a later
session need not re-derive it.

---

## 3. Undeclared — 13 reported, **2 real**

### 3a. Eleven are the slice's own process output

Two dispatch bookkeeping files (`boundaries.toml`, `funnel.toml`); the governance
the work itself minted (the `DEC-100` record, its body and its slug symlink, plus
the `DEC-099` supersession edit); the `RV-324` ledger, its body and its slug
symlink; and the slice's own two entity files (`notes.md`, `slice-233.toml`).

**Disposition: accept, no selector action.** A selector list declares what a
slice sets out to *change*; it cannot anticipate the governance records that
doing the work *mints*. Requiring it to would mean declaring an id before it is
allocated. This is `IMP-347`'s class — it already `originates_from` SL-233 and
already carries the PHASE-01 instance that blocked a phase mid-flight.

### 3b. Two are real, and they are the whole signal in this report

`src/commands/verify.rs` and `tests/runbook_fixture/mod.rs`, both added by
**PHASE-16** (`80a1eba6d`, `f8a1a0487` — the `doctrine verify` family and the
runbook clearance clause).

The design-target selectors enumerate the command tier **by hand** —
`commands/design.rs`, `commands/mod.rs`, `commands/cli.rs`, `commands/guard.rs` —
and the test tier as the glob `tests/e2e_design_*.rs` plus three exact names.
PHASE-16 added a new command-family file and a new test-fixture module, and
neither matches any of them.

Both *are* matched by `scope-relevant` selectors (`src/commands/**`, `tests/**`),
which conformance does not consult: it cross-checks **design-target** selectors
only. So the paths are inside the slice's declared scope while being outside its
declared *targets*, and the report is right to flag them.

**Disposition: add two design-target selectors at reconciliation** —
`src/commands/verify.rs` and `tests/runbook_fixture/**`. This is a declaration
gap, not a code defect, and it is the cheapest repair available. It is recorded
here rather than applied because `/reconcile` is the sole explicit writer of
reconciled truth.

---

## 4. What this leg deliberately did not settle

- **Eight of `RV-325`'s seventeen findings** (F-6, F-7, F-9, F-10, F-12, F-13,
  F-14, F-15) promised `plan.toml` criterion amendments or sketch text, not
  implementation. A code funnel structurally cannot see them. **They belong to
  the audit's spec-coherence leg** and are not dispositioned here.
- **`ISS-289`** — PHASE-15 `VT-2` mandates
  `an_edit_inside_materialises_guarded_window_is_never_certified`, which was never
  written; what landed is `a_foreign_write_after_materialises_rename_is_not_certified`
  (`tests/e2e_design_state.rs:346` — both S4 passes misreported this line as 353
  and 354 respectively, which is why it is stated here measured). `verify-vt 233`
  exits 1. A standing
  disposition, deliberately unrepaired — not a finding, and every S4 raiser on
  PHASE-15 tripped it exactly as predicted.

---

## 5. Spec coherence — settled at reconcile, 2026-08-02

§4 left the eight `RV-325` findings that promised `plan.toml` criterion
amendments or sketch text to the spec-coherence leg, on the grounds that a code
funnel structurally cannot see them. That leg ran at reconcile. **All eight
landed; no write was needed.** The check is a verification, not a repair — each
finding's response names what it appended, and the question was only whether the
artefact carries it.

| finding | promised | verified |
|---|---|---|
| F-6 | PHASE-08 `VA-7` appended, destination vocabulary gains *stage fragment, naming the asset* | present in `plan.toml` |
| F-7 | a pre-registered falsifier in PHASE-09's criteria and the sketch | present |
| F-9 | the withdrawn discriminator replaced at five sites across `thin-adapter.md` + `target-machine.d2` | zero live hits for *can a verifier ever corroborate* / *leaves a trace*; *completability at the boundary* present 3× in each file |
| F-10 | the `DEC-104` exemption removed | zero live hits; the one surviving *does not reopen DEC-104* is quoted inside a blockquoted amendment narrative (*this paragraph previously said*), which is retained history |
| F-12 | `EX-8(d)` corrected to three causes | *prematurely attested, or by step text that misled a compliant agent* present |
| F-13 | `stand` removed from the resolution set | `EX-8` accretes append-only, and a later clause states *`STAND` IS REMOVED FROM THE RESOLUTION SET (F-13)* with the round-6 warrant explicitly lapsed-and-replaced — correct under the immutable-append rule |
| F-14 | firing condition widened past the enumeration | *incidental context exhaustion or compaction* present in both `thin-adapter.md` and `plan.toml` |
| F-15 | contest rejected, exposition repair only | the `(b)`-fails-therefore-`(a)` route present on the pinned surfaces |

Every negative above ran with a positive control, and one first attempt was
discarded for want of it: a line-level `grep -v standard|standing|understand`
over `plan.toml` returned **zero** `stand` hits, because every line carrying the
bare word also carried one of the filtered words. The filter had eaten the
evidence rather than the noise. Re-run without it, the hits are three, and they
are the append-only removal narrative — the opposite conclusion.

`EX-11`'s `SPEC-029` question needed no work here; both PHASE-15 passes had
already confirmed the withdrawal independently (§2b).
