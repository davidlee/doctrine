# Notes SL-238: Cross-kind backlog ordering admission and override footer

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design surface triage (exploring, 2026-08-15)

Discharges `explore.triage`. Evidence lives in `research/research.md`; this is the
shape of the design surface, not a restatement of it.

### Constraining governance

| authority | what it requires here |
|---|---|
| REQ-238 (SPEC-001 `FR-006`) | the per-kind status-class partition lives in policy — a per-kind terminality question routes through `priority::partition::status_class` |
| ADR-017 | `partition.rs` is the sole engine delta for gating; `QUE open` is `Gating` |
| ADR-009 | slice terminal is `{done, abandoned}` — what `--prune`'s `resolved`/`closed` probe contradicts |
| STD-001 | that inline probe is a duplicated terminal table |
| ADR-016 §2 | terminality is a projection, never a relation label |
| REQ-218 (SPEC-015 `FR-006`) | names `backlog_order` in the requirement — deleting it needs a REV; widening it does not |
| ADR-001 | `backlog_order → backlog` is an *accepted* violation, existing only because `ItemId` is `ItemKind`-backed |

Checked and not applicable: POL-001, POL-002, STD-002, ADR-004, ADR-010,
ADR-013/014, ADR-015 (except on the retire horn), ADR-019, ADR-020. Reasons in
`research.md` Thread 1.

### Shaping decisions

- **D-A. The fork — SETTLED, `DEC-231`.** Neither horn. `--by sequence` is a
  backlog-induced **work order**, not an actionability surface; cross-kind edges
  get no ordering effect on either axis and are **disclosed**, never used to
  order or withhold. `backlog_order.rs` is untouched. Supersedes `DEC-230`,
  which had ruled the opposite (see *Reversal* below).
- **D-B. Where the status probe lives — SETTLED, `DEC-233`.** Both horns the
  node posed are dead: the full `scan_entities` walk is a ~20x regression on a
  0.19s command, and `catalog::scan` is command-tier and *reaches* `backlog`, so
  calling into it closes an ADR-001 cycle. The probe instead composes two
  downward seams — `kinds` (leaf) for resolution, a stat with no file read; and
  `meta::read_meta` (engine) for status, ~15 distinct targets ≈ 8ms. Shell
  returns a three-way (`Some` / `None` / `Unavailable`), pure layer classifies
  via `partition::status_class`.
  **Siting refined at drafting — read `design.md` §3, not this clause.**
  ~~"No new module; the footer shell and the doctor check each call both
  directly."~~ That would have left `catalog::scan::status_and_title_for` as a
  second per-kind reader disagreeing on `RV`. The read moves into a new engine
  module `src/authored_status.rs` and `catalog::scan` becomes its command-tier
  overlay. `DEC-233` is refined, not superseded: its seams, cost and loudness
  rules all stand.
- **D-C. Footer content, direction, dedup — SETTLED, `DEC-232`.** The footer's
  contract is *only what is needed to understand the rendered content*.
  Cross-kind edges with a live target go to a separate `boundary:` block
  (`ISS-327 needs QUE-219 (open)`, dependent-first, relation word, no arrow);
  terminal-target edges are reveal-only; **every ref-integrity failure leaves
  the footer for `doctrine doctor`**, which takes the project-level `AbsentDrop`
  leg with it — so the direction clash dissolves by deletion, not by flipping.
  The doctor check is **folded into this slice**.
- **D-D. Reveal flag shape — SETTLED, `DEC-234`.** No flag. `-a/--all` is
  already taken (row hide-set) and a footer flag would exist to defeat the rule
  `DEC-232` just gave the footer. `backlog inspect <ID>` already prints the full
  record on both axes; it gains only a target status annotation from `DEC-233`'s
  probe. Footer shape is invocation-independent.
- **D-E. Cross-kind clearing — SETTLED, `DEC-235`.** Neither horn. The
  kind-neutral verb already exists — top-level `doctrine after <SRC> <TGT>
  --remove` clears a cross-kind edge today; `backlog after` is a backlog-only
  duplicate that refuses what its twin accepts. So: route `backlog after`'s
  remove/prune to the kind-neutral shell, add `doctrine needs --remove` (new
  `dep_seq::remove_needs` leaf), and collapse **four** copies of the terminality
  probe onto `DEC-233`'s + `status_class`. `parse_ref` untouched.

### Inquiry map (design run `dr-01a00475`)

Resolved: `inq-1` → `DEC-230` (superseded), `inq-2` → `DEC-231`, `inq-3`
(non-durable, moot), `inq-4` (non-durable, ADR-001 paydown falls away with the
widen horn), `inq-6` → `DEC-232`, `inq-5` → `DEC-233`, `inq-7` → `DEC-234`,
`inq-8` → `DEC-235`. **Open: none — 8 of 8 resolved at revision 21.**

### The reversal — read this before re-opening the fork

DEC-230 ruled `--by sequence` an actionability surface that must gate. **That was
wrong and DEC-231 supersedes it.** The reasoning that overturned it, so it is not
re-derived:

1. `--by sequence` does **not** withhold on a backlog-internal live `needs`
   today — `ISS-X needs ISS-Y`, both open, shows both with Y before X. Only
   *terminal* items leave the node set. So gating cross-kind dependents while
   ordering internal ones is asymmetric.
2. Origin intent settles it: SL-051 states *"ordering is a sort, not a view"*,
   keeps membership unchanged, and names blocked nodes as retained participants.
   The code says work order throughout (`src/backlog.rs:81`, `:1113`, `:1190`);
   `next` explicitly builds only the actionable set
   (`src/priority/surface.rs:384`, `:406`). Actionable `--by sequence` is
   retrofit, not recovered intent.
3. The pressure to make it gate came from `doctrine next` having **no kind
   filter** — so "the actionable backlog" was not expressible in one command.
   That is a gap in the actionability surface, misdiagnosed as a defect in the
   ordering one. Raised as **IMP-432**.
4. Phantom admission does not work either: cordage assigns longest-path
   **levels** before `NodeId` (`crates/cordage/src/resolve.rs:635`,
   `lib.rs:366`), so admitting a cross-kind node demotes its dependent behind
   *every* level-0 node rather than stating "IMP-390 comes after SL-251" — and
   there is no principled `exposure`/`created` for a node that has neither.

### Risks

- ~~**R1 — ordering divergence.**~~ Dissolved by `DEC-231`: no node admission, no
  comparator change, no adapter change. Row order is untouched.
- **R2 — golden churn.** Realised, not hypothetical: `74b773690` shipped
  suppression-by-default on one leg and locked it with two tests
  (`src/backlog.rs:5301`, `:5332`). They assert cross-kind drops are *silent*,
  and the settled design *discloses* them — so both must be superseded
  deliberately, not relaxed. `DEC-232` supersedes `:5301` on a **second** count:
  its `not-a-ref` leg asserts a malformed ref is named *in the footer*, which
  now moves to `doctor`.
- **R5 — surface growth.** `DEC-232` folds a `doctor` ref-integrity check into
  this slice. Deliberate (the footer removal is only honest with somewhere for
  the errors to go), but it is real added surface: a check, its wiring, and its
  tests. Guard against it pulling further scope — the check is
  `ensure_ref_resolves` over each item's `needs`/`after` under the existing
  `RelationIntegrity` category, nothing wider.
- **R3 — untested leg.** `run_after --prune` has no test coverage at all, in
  **either** copy. Characterisation tests precede any probe change there. And
  the prune fix is a deliberate *behaviour* change (an edge onto a `done` slice
  becomes prunable), not a refactor — it must be tested as one, never smuggled
  through the duplication cleanup (`DEC-235`).
- **R4 — soft-axis over-reach.** An `after` edge onto an unrecognised-status
  target must not withhold or order. The conservative blocker rule is
  implemented over `dep_overlay` only (`src/priority/channels.rs:58`, `:66`);
  treating status uncertainty as a gate would silently harden a preference into
  a blocker.

### Assumptions

- ~~**A1**~~ — *resolved by `DEC-233`.* `status_class` covers the kinds, but the
  gap is upstream of it: `RV`'s status is **derived** at command tier
  (`review::derived_status_string`), so no engine-side reader can obtain it.
  Handled as an explicit `Unavailable` arm keyed off a pinned kind set, never a
  silent Terminal. `REC` is not a gap — `status_class(kind, None)` defines it as
  Terminal. Follow-up: `IMP-433`.
- **A2** — no non-backlog entity authors a `needs`/`after` edge whose *target* is
  a backlog item in a way this slice would newly order. **Still unverified.**
- **A3** — the corpus's cross-kind edges were authored through `doctrine needs`
  (validated by `kinds::ensure_ref_resolves`), not `backlog after`, which cannot
  express them. Consistent with `--remove`'s inability to clear them.

### Superseded by measurement

~~The scope's claim that harm is latent — *"all ten targets happen to be
`done`"*~~ — **this section is spent.** The scope was rewritten at the
`inquire.scope` gate (`037b98b06`) and now carries the corrected measurement
itself: 30 authored cross-kind edges, 25 reaching the footer, 10 with a
non-terminal target.

**Two residual errors in the scope, found at drafting (2026-08-16).** A full
re-scan reproduces every figure above exactly, but `slice-238.md` still carries
(1) *"Twenty-one of the thirty are on the `needs` axis"* — it is **16 `needs` /
14 `after`** — and (2) `src/cli.rs` under Affected surface, which does not exist;
the file is `src/commands/cli.rs`. `design.md` §1 and §8 are correct on both.
Fix the scope directly; it is outside the design run.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-16 · design/**reviewing** (run `dr-01a00475`, rev 53; `RV-358` ten findings all dispositioned; runbook 3/3 discharged; gate to `locked` holds on three USER acts) · 582300f14 + uncommitted

### Produced

- `design.md` — all nine sections rewritten against a nine-finding critical read,
  then **all nine revised again** against `RV-358`'s nine findings. Materialised at
  rev 48. All of it **uncommitted** by owner request (git provisioning under test).
- `RV-358` — external adversarial design review (codex/GPT-5.5), **two rounds**.
  Round 1 raised nine findings (1 blocker, 5 major, 3 minor); all nine verified
  correct against source and integrated. Round 2 verified five and **contested
  four**, plus raised `F-10`; all five of those verified correct too, and three of
  them were defects introduced by the round-1 integration. Ten findings, all
  dispositioned, none withdrawn or deferred.
- **`F-1` twice over — the design's central layering result.** Routing
  `backlog::run_after` into `commands::dep_seq` closes a command-tier cycle, and
  because the two modules are in different SCCs it merges two clusters. The first
  repair (a new engine module `src/dep_seq_ops.rs`) was **withdrawn**: `--prune`
  classifies through `partition::authored_class` and `priority` is command tier
  (`layering.toml:109`), so an engine-tier module calling it is an *upward* edge —
  worse than the cycle. Final repair is dependency inversion: `cli.rs` injects the
  three operations as `AfterOps` fn-pointers, precedented by `BacklogTableFn`
  (`backlog.rs:1636`) and by `mem.pattern.lint.back-edge-tangle-inject-fnptr`.
  Nothing moves. §6 keeps the withdrawn option as a named rejected alternative.
- `partition::class_of` renamed **`authored_class`** — `channels.rs:37` already
  has a private `class_of`, found while adjudicating `F-9`.
- `DEC-236` body amended (`F-7`): the signpost's count key was authored as
  `(dependent, ref)` pairs while also requiring the both-axes case to count twice —
  incompatible. Now `(dependent, axis, ref)` occurrences. Decision unchanged;
  only the key was mis-spelled.
- `ISS-366` — an empty review ledger derives `done · await=none`
  (`src/review.rs:1103`), contradicting the seed template's own `ADR-007` D-C8
  comment. Found because `RV-358` read `done` before anyone had raised anything.
  Adjacent to `IMP-433`; out of this slice's blast radius.
- `STD-003` — *No silent skip: a degraded read is disclosed.* `required`. Minted
  from this slice; `SL-238` and `RSK-013` are `governed_by` it. Commit `35e19a5af`.
- `DEC-236` — the count-only stderr signpost for unresolvable refs; `shapes`
  `SL-238`. The one drafting overrun no accepted decision covered.
- `DEC-230`…`DEC-235` — unchanged from the prior pass; `DEC-233`'s siting is
  refined by `sec-3`, not superseded.
- `IMP-432`, `IMP-433` — unchanged follow-ups.
- Selector set reconciled to §8 twice: `+src/authored_status.rs`, `src/meta.rs`
  demoted to `scope-relevant`, then `+src/dep_seq_ops.rs` and `+src/main.rs` after
  `RV-358`. Eleven design-targets now.
- Two friction observations under `.doctrine/observations/records/` (rtk output
  mangling; `design apply` payload-shape lookup — the latter is live evidence for
  `RFC-026` `E8.7`).
- **No code written. `doctrine check gate` not run and not owed** — this pass
  touched only `.doctrine/` prose.

### Learned

- `mem.pattern.layering.direction-is-not-cohesion` — a gate-clean downward edge
  can still be the wrong siting; read the target module's charter.
- `mem.fact.rtk.output-filter-rewrites-identifiers` — proxied grep silently
  substituted identifiers; never take a name's spelling from that output.
- `mem.fact.layering.gate-measures-top-level-modules` — the ADR-001 gate extracts
  edges at top-level-module granularity and `discover_units` admits only top-level
  names, so a `"a::b" = "engine"` sub-classification row is recorded but never
  participates in the tangle count. Reaching a new *function* in a module you
  already import is free; reaching into a module you do not import is a new edge no
  matter how deep the target sits, and no sub-row rescues it.
- The same siting error was made twice in this design, at two different seams
  (`meta` for the status read, `commands` for the dep/seq ops), and `mem.pattern.
  layering.direction-is-not-cohesion` caught only the first. The generalisation now
  in `sec-1`: a seam two modules share goes **below both**, never beside whichever
  one wrote it first.
- `catalog::scan::status_and_title_for` (`scan.rs:410-430`) is already the
  per-kind status reader `DEC-233` describes — the ADR-001 refusal is about the
  *call*, not the code, so the answer is to move it down, not to write a second.
- `src/meta.rs`'s charter is **zero per-kind knowledge** (`meta.rs:3-13`), with
  ~17 consumers resting on it. `src/integrity.rs:19-20` already carries
  `engine → {kinds, meta, entity}`, so a new engine module adds no module edge.
- `ensure_ref_resolves` returns `Result<()>` (`kinds/resolve.rs:33`);
  `parse_resolvable_ref` (`:63`) is the pair-returning delegate, and it accepts
  the **bare** id form as well as canonical.
- `parse_resolvable_ref`'s dangling message interpolates `dir.display()`
  (`:74-77`) — an absolute path, unusable verbatim in a golden-tested finding.
- `doctor` leg **#7 TomlParse** exists (`doctor.rs:50-51`) at *Warning*, while
  ref integrity is *Error* — a corrupt toml is reported more quietly than a
  malformed ref naming it.
- `render_overrides`'s `corpus` param and `compose`'s `cmap` build are read
  **only** by the `Dangling` arm (`backlog.rs:2316`, `:2338`); both die with it.
- **Corpus re-measured 2026-08-16** — 30 cross-kind edges, 5 off terminal
  dependents, 25 reaching the footer, 10 boundary lines, 15 silent, 15 distinct
  targets, and **zero unresolvable authored refs**. Every prior figure reproduces
  except the axis split, which is **16 `needs` / 14 `after`**, not 21/9.
- `doctrine design apply` ignores unknown payload keys silently, so a wrong key
  burns a revision and looks like success; the schema is only in
  `src/design_run/submission.rs:687` and `:124`.
- (Carried, still live) four `--prune` probe copies hardcode `resolved`/`closed`
  **and** launder a failed read into an empty status word; `--prune` has no test
  coverage in either copy; the kind-neutral clearing verb already exists and
  `backlog after` is its duplicate; nothing re-checks dep/seq refs after
  authoring.

### What a further review pass would probe

Written 2026-08-16, after `RV-358`'s verification round — the last pass conducted.
Two adversarial passes ran: the raise round (9 findings) and the verification round
(4 contests + `F-10`). Every finding is dispositioned; none was withdrawn or
deferred, and all ten were verified correct against source before integration.

A third pass is **not** owed for coverage, and is worth running only if the owner
wants the second repair adversarially tested the way the first one was. What it
should probe, in priority order:

1. **The injection repair itself, which no reviewer has seen.** `F-1`'s first
   repair was contested and withdrawn; its replacement (`AfterOps` fn-pointer
   injection, §6) was authored after the verification round and has had no
   adversarial read. It is the highest-value target by construction — the same
   position the withdrawn repair occupied when it looked fine.
2. **`RefState`'s rendering table** (§4) — the `Absent` → no-parenthesis rule is
   new and interacts with `inspect`'s "annotate every cross-kind target" rule.
   Is a `REC` target distinguishable from an un-annotated one?
3. **Cross-section drift from three rewrites.** Nine sections have now been
   declared three times. `F-5` was exactly this failure — a claim corrected in two
   sections and left standing in a third.
4. **§7's evidence claims**, since `F-10` found one that was simply false. Every
   "this existing test already proves X" assertion deserves the same check.

### Open

- **Run at `drafting`, all 9 sections `review=outstanding`** — the next stage is
  `reviewing`. The `draft.selectors` runbook step is discharged and
  `drafting-ready` is declared; the one gate still shut is
  `governing-context-recorded`, because linking `STD-003` moved the slice's
  `governance-edges` fingerprint and expired the earlier `governance-confirmed`.
  It needs a **user** act. Query the run, not this line:
  `doctrine design resume 238`.
- ~~**`slice-238.md` carries two known errors**~~ — **closed 2026-08-16.** The
  21/9 axis split now reads 16/14 with the miscount named, and `src/cli.rs`
  is now `src/commands/cli.rs`. Direct edits, outside the design run.
- `DEC-236` — accepted, but folded into `design.md` §9 as an overrun; confirm the
  reviewer reads it as covered rather than as an uncovered divergence.
- **`IDE-019` divergences (2), for reconcile** — footer-vs-`doctor` siting, and
  the declined `--verbose`/`--explain` flag. Both deliver its intent, neither its
  mechanism.
- **`RSK-013` and `catalog::scan:243/246`, `:289/292`** — live `STD-003`
  violations outside this slice's surfaces.
- **A2 unverified** — whether any non-backlog entity authors a `needs`/`after`
  edge whose *target* is a backlog item. Inert either way.
- **Scope grew five times now** — the three owner-accepted at inquiry, plus the
  `catalog::scan` collapse and the two stderr notices. Phase plan should be built
  against §8's file list, not "make the footer honest".
