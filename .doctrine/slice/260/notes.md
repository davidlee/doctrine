# Notes SL-260: Design-review finding routing convention and trial

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: <yyyy-mm-dd> · <PHASE-NN | stage> · <head-commit>

### Produced

### Learned

### Open

---

## Design surface triage — exploring stage, 2026-09-19

Bulk evidence lives in `.doctrine/slice/260/research/research.md` (runtime tier,
gitignored; baseline current). This section is the triage over it, plus what the
memory sweep added. Pointers, not copies.

### Constraining governance

Binding: `DEC-103` (two-surface placement is *required*, corollary 2; the
honour-system rules must be labelled unenforced-by-construction, residue rule) ·
`DEC-138` (disposition binds the responder's turn; its consequences make stating
the two-gate asymmetry an obligation on this design) · `ADR-007` D-C5/D-C9b ·
`DEC-126`+`DEC-125` (the design gate attests, it does not check) · `ADR-005`
(the two targets sit in different knowledge tiers) · `DEC-101` (step ids are
API — the authority behind the `reviewing.toml` non-goal) · `POL-001` ·
`SPEC-031` (criterion identity; silent on promotion) · `ADR-017`/SL-060.

Checked, not applicable, each with a reason: `STD-001`, `STD-002`, `STD-003`,
`POL-002`, `ADR-003`, `ADR-013`. `ADR-013` cannot fire at all — `revision change
add` takes a live entity FK and an embedded asset has no entity id.

`RFC-026` is **provenance, not authority** (`ADR-014`: an RFC asserts no canon).
The design *adopts* `P10`; it is not bound by it.

Revision candidates: none, and none possible.

### Shaping decisions already taken in scope

Five research deltas were scope-level and were folded into `slice-260.md` before
this run opened (`4b100ff6f`): lead §1 on `DEC-103`; name the binding gate as the
slice close, not the design lock; carry `P10`'s settle-first test and its
accumulation rule; label the unenforced-by-construction items; and resolve §6's
script hedge to **no tooling at all**, with the tripwire that a `src/` change
newly binds three standards.

### Open questions carried into design

`OQ-1`..`OQ-5` in `research.md`, plus the two the memory sweep raised:

- `OQ-1` the raiser-side ruling redefines what `verify` means. Name it as a
  convention-level redefinition or the two readings collide inside the trial's
  own counting.
- `OQ-2` the finding→criterion promotion leg has no governance owner. Convention
  text, knowledge record, or agent practice?
- `OQ-3` enforcement is the counting pass and only the counting pass. Accept and
  say so.
- `OQ-4` "first phase" is asserted, not governed. Needs a fallback for a `probe`
  that wants a worktree branch.
- `OQ-5` the probe-evidence home rests on `.gitignore`, not on any authority.
  Pick a name; do not cite an authority that does not exist.
- `OQ-6` **(new, from `mem_019f97fcab2e77a28902371f80743605`)** — no verb
  transitions a finding out of `verified` (`src/review.rs:703-728`). So `P10`'s
  accumulation rule can reopen the *design decision* but never a verified
  routed finding's *disposition*; the remedy there is a prose amendment on the
  `RV` `.md`. Which does the convention mean?
- `OQ-7` **(new)** — does the convention name a default route for an
  unclassifiable finding, or require an explicit `review`? (Scope's own `OQ-1`.)

### Risks

- `R1`/`R2`/`R3` as scoped: read-and-not-followed is a real result; three
  ledgers is thin and carries a combined intervention; the routes may need the
  owner to interpret (a `P10` would-kill).
- `R4` **(new, from `mem_019fbc7514097c42aa042ab7bb2206c6`)** — the probe
  adversary clause goes into `--response`, the one field where a double-quoted
  shell argument silently eats every backtick span. `dispose` succeeds, the
  ledger is turn-based, there is no amend verb, so the damage is permanent until
  the raiser contests. The counting pass cannot tell an eaten clause from an
  absent one, which corrupts exactly the `P10` *Open* item §4 answers.
- `R5` **(from `mem_01a0b45d375873e3ae691d75bea8760d`)** — a repair inherits the
  finding's scope and leaves the twin arm; observed four times in one review
  (`RV-370`/`SL-246`). A `demonstrate` route can discharge one arm and read as
  complete, so routing does not by itself close this sibling failure mode.

### Assumptions

- `A1` design-run findings already land on the `RV` ledger, so *"on the existing
  ledger"* is true today and `IMP-392` is not a prerequisite.
- `A2` `--disposition` accepts free text and nothing validates it.
- `A3` **(new)** the shipped convention reaches agents only through a rebuild +
  `doctrine install`; cargo does register `install/` as a build dependency
  (`mem_019e98a783ea7471ac4bfcefdc04ae5e`, re-probed), so the rebuild is not
  manual — but a stale embed is silent, so closure verifies through the render.
