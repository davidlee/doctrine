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
fresh-as-of: 2026-08-16 · **PHASE-02 completed** (2 of 8) · design/**locked** (run `dr-01a00475`, rev 63; `RV-358` **waived** with a reasoned disposition; all nine sections attested human-lane; `design-accepted` current; gate cleared) · `a9ef162d0`, clean of mine

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

#### PHASE-01 (2026-08-16) — `8e0c4fbdd`, `843fba6f3`, `8cda64e97`, `13808f354`

- `src/authored_status.rs` — the sole per-kind status reader, engine tier. Three
  arms static on `kinds::STATUS_LESS` / `DERIVED_STATUS`; absorbed
  `catalog::scan::title_for`. `catalog::scan::status_and_title_for` is now the
  command-tier overlay over it; its two inline `"REC"`/`"RV"` arms are gone.
- `kinds::{STATUS_LESS, DERIVED_STATUS, AuthoredStatus}`;
  `priority::partition::authored_class`; `layering.toml` engine row (18 now).
- `execution-protocol.md` — **new authored artefact**, the `DEC-242` three-seat
  protocol as practised plus its findings. Binds PHASE-02…08. Read it before
  `/phase-plan`.
- `plan.toml` PHASE-01 `VT-2` **amended** (id unchanged, text replaced with the
  reasoning inline) — see Open for the design-text half.
- All eight PHASE-01 VT/VA criteria satisfied; `just gate` green (117 suites,
  clippy zero warnings); `catalog`/`search`/`map` green **unmodified** (EX-9);
  tangle baseline unmoved at 76; `src/meta.rs` unmodified (EX-8, diff-verified).
- **Type prototype, two rounds** (fork `proto/SL-238-types`, uncommitted,
  disposable). Round 1 found five issues, three accepted and landed as revisions
  58/59. Round 2 proved all three and found **no new design defects** — the rank
  ceiling honoured identically on both legs with two edges to one target at ranks
  1 and 5; the admission refusal byte-identical across `backlog needs` and
  `doctrine needs` (same sha256); the unpadded-ref bound exactly the
  short-hyphenated form. Detail in the *Type prototype* section above.
- `DEC-242` — *Skeleton-informed implementation with a blind test author.*
  Accepted. Three seats per phase (planner / test author / code author); the test
  author may **not** read the prototype, which is the clause that makes this not
  prototype promotion. `concerns` `SL-238` and `RFC-026`.
- `ISS-368` — `backlog needs` accepts targets `doctrine needs` refuses. The
  shipped half of `RV-358` `F-5`, filed separately because it is live whether or
  not this slice lands. `ISS-046` was the mirror defect and is closed.
- `CHR-068` — `tests/architecture_layering.rs:8,22` cite a stale `command=120`
  against the real `command = 76` (`layering.toml:190`). Found by the prototype
  while checking §7's preservation claim; the design is right, the comment is not.
- `RV-358` **waived**, not conducted. `F-1` rests at `answered` and concluding the
  pass would have meant asserting the raiser's role over a review this side
  responded to. The waiver reason — durable on the run, not restated here — names
  which four of the five unverified findings the prototype supports and how.
- Two further friction observations: `design apply`'s `adopt_authored` needs
  section fingerprints no command emits (four source reads + a Python
  reimplementation of `document::parse`; `RFC-026` `E8.7` again), and
  `DOCTRINE_WORKER=1` refuses every authored write in a worker fork, blocking the
  live smoke-tests such a fork exists to run.
- **The lock carries its own caveat**, printed by the engine at the transition:
  *"locked on an auditable agent claim of user acceptance — not authenticated
  proof of a human act."* v1 has no authenticated human identity; the attestations
  are an agent's record of a human's claim.

#### PHASE-02 (2026-08-16) — `28cbab394`, `7e07913ac`, `8cef217ca`, `a9ef162d0`

- **Characterisation landed, test-only.** Seven pins in
  `tests/e2e_dep_seq_verbs.rs` plus two annotations on incumbents: the `closed`
  vocabulary and `/resolution` suffix byte-exact, the silent keep on an
  unreadable target (including the empty stderr — the `STD-003` defect pinned as
  silence), the bare `to = "154"` dropped with the target provably unread, and
  both unremovable edges surviving their own `--remove`. Both `--prune` copies.
  Thirteen mutation checks, each confirming the intended assertion fires by
  message; one deliberately *non*-firing, which is what evidences the
  as-typed/canonical echo divergence between the two copies.
- **`plan.toml` PHASE-02 `VT-1`/`VT-2`/`VT-3` amended** (ids unchanged, reasoning
  inline): `test_file` moved to the black-box golden file. The wording they pin
  goes straight to `io::stdout()` with no render seam, so an in-module test
  cannot observe it and extracting a seam would breach `EX-3`. `verify-vt` judges
  the row against the named file, so this was load-bearing, not documentary.
- **`execution-protocol.md` revised** — `R1` promoted to the **default** (plan
  blind, commit the plan, *then* read the fork as an oracle against it); §3 gains
  the fourth prose/criterion defect; §6's loop shows the one-seat default with
  the two-seat arrangement as fallback; §2's fourth prohibition qualified.
- **PHASE-01's source-delta was never recorded** — it was flipped retrospectively,
  so its eight `VT` rows read `UNATTRIBUTABLE` throughout its own harvest. Repaired
  with `slice record-delta 238 PHASE-01 --start 8e0c4fbdd^ --end 13808f354`; all
  eight now `PASS`, as do PHASE-02's three. Eleven criteria of audit evidence
  recovered, silently absent until then.
- `QUE-221` — minted; see Open.
- `just gate` **green, verified on a real exit code** (117 suites, clippy zero
  warnings). An earlier claim of green rested on a pipeline whose status came
  from `tail` and established nothing; re-run properly. `.doctrine/` changes
  committed separately from the test commit throughout.
- The phase ran **one-seat** under `R1` — planner blind, oracle pass after, then a
  single Opus implementer. The oracle poked no hole in the plan: it corroborated
  the ground truth, corrected one annotation, and yielded two prototype defects
  carried to PHASE-07 (a `Terminal` branch minting reasons for states
  `authored_class` may make unreachable; an `eprintln!` against
  `print_stderr = "deny"`, `Cargo.toml:268`).

### Learned

- `mem.pattern.testing.grep-for-the-pin-before-characterising` — **PHASE-02, and
  the reason this phase nearly wrote the wrong tests.** A design's claim that a
  surface is untested is unverified prose that decays faster than its claims
  about behaviour, because tests land continuously and designs lock. Grep the
  test tree for the *verb*, not the production identifier (a black-box golden
  never contains it), then read what the incumbents actually assert — a
  `contains()` on a substring present in both before- and after-states pins
  nothing.
- `mem_019f89125fb275a2895bf58b5e29ed95` **extended** — a phase flipped to
  `in_progress` retrospectively stamps no `code_start_oid`, so *every* one of its
  `VT` rows reads `UNATTRIBUTABLE` permanently, not just the rows whose file is
  new, and `completed` does not repair it. Non-halting, so nothing goes red to
  tell you. Selector membership is not the attribution mechanism and does not
  rescue it. The `record-delta` escape hatch is the repair.
- `mem.fact.layering.gate-blind-to-an-edgeless-module` — **PHASE-01, measured.**
  A new root module declared in `main.rs` but carrying no `use` lines passes the
  whole `tests/architecture_layering.rs` suite with no `Unclassified` finding.
  The gate reports it only once it acquires edges. So "gate green" does not mean
  "classified", and a phase that lands a module shell and defers its
  `layering.toml` row will look correct until the next phase turns red.
- `mem.pattern.lint.dead-code-derives-count-as-reads` — **confirmed twice in one
  phase, in opposite directions.** `AuthoredStatus` needed no staging attribute
  (its `PartialEq`/`Eq` derives are a live use of the enum itself, not merely of
  its fields); `authored_class`, a plain `fn`, needed one and it is fulfilled.
  One task apart, same phase. Compile, do not reason.
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
  **and** launder a failed read into an empty status word; the kind-neutral
  clearing verb already exists and `backlog after` is its duplicate; nothing
  re-checks dep/seq refs after authoring. **Corrected at PHASE-02:** the clause
  that read *"`--prune` has no test coverage in either copy"* is false — five
  SL-105 goldens exist. They pin the decision, never the rendered reason; see
  Open for the design-text half.

### What a further review pass would probe

Written 2026-08-16, after `RV-358`'s verification round — the last pass conducted.
Two adversarial passes ran: the raise round (9 findings) and the verification round
(4 contests + `F-10`). Every finding is dispositioned; none was withdrawn or
deferred, and all ten were verified correct against source before integration.

A third pass is **not** owed for coverage, and is worth running only if the owner
wants the second repair adversarially tested the way the first one was. What it
should probe, in priority order:

1. **The injection repair itself, which no reviewer has seen.** `F-1`'s first
   repair was contested and withdrawn; its replacement (fn-pointer injection, §6)
   was authored after the verification round and has had no adversarial read. It
   is the highest-value target by construction — the same position the withdrawn
   repair occupied when it looked fine.
2. **`RefState`'s rendering table** (§4) — the `Absent` → no-parenthesis rule is
   new and interacts with `inspect`'s target-annotation rule. Is a `REC` target
   distinguishable from an un-annotated one?
3. **Cross-section drift from repeated rewrites.** `F-5` was exactly this failure
   — a claim corrected in two sections and left standing in a third.
4. **§7's evidence claims**, since `F-10` found one that was simply false. Every
   "this existing test already proves X" assertion deserves the same check.

### Self-attack round (2026-08-16, revisions 53 → 57)

A read-only self-attack over all ten dispositions and the whole document, before
any section attestation was spent. Priority 1 above was correct: **the unseen
injection repair carried the most defects.** Nine repairs landed; the corpus
figures were re-derived from scratch. Nothing here has been adversarially read.

- **The `backlog needs` gate reproduced `F-1`.** §6's repair for `F-5` called
  `kinds::is_admissible_dep_target`, which does not exist — the function is
  `commands::dep_seq`'s (`dep_seq.rs:34`), and its message reads
  `knowledge::RecordKind::ALL`. Calling it is the `backlog → commands` edge `F-1`
  ruled out. Repaired by injection: `AfterOps` becomes `DepSeqOps` with a fourth
  member, and §6 now states the **rule** (every command-tier operation arrives by
  injection) rather than enumerating three operations.
- **The injection had no route.** `cli.rs`'s `Command::Backlog` arm calls
  `backlog::dispatch(command, color)` (`cli.rs:1705`) and reaches no `run_*`
  function, so "cli.rs fills `AfterOps`" was unimplementable as written.
  `dispatch` is now the named injection point.
- **The stderr advisory could not count one of its three classes.** A ref to an
  absent backlog id (`ISS-999`) parses and reaches the adapter, so it was never an
  `AbsentDrop` and never counted — a `doctor` error with no signpost. `project`
  now records it (§4), pure and without changing adapter inputs.
- **`inspect`'s two annotation rules contradicted each other** on that same class.
- **`--prune` deleted live bare refs.** The probe kept `parse_canonical_ref`,
  which rejects the bare form and routes `Err` to *prunable*.
- **§3's layering table omitted `catalog → authored_status`**, which §1's diagram
  draws and §8 commits to — and its prose said the `authored_status` consumers
  "cost no edge", which is true of `authored_class`, not of them.
- **`read`'s `&'static KindRef` was uncallable** from the one consumer §8 names.
- **Two counts were wrong** (`main.rs` 95→94 root `mod` lines; `meta` seventeen→
  fifteen consumer modules), and **every corpus figure had drifted** — 31 edges,
  not 30, after `ISS-367 after SL-256` landed the same day. Swept into
  `slice-238.md` and `DEC-235` (whose `context` argued from the 21/9 miscount and
  now carries an appended correction). §1 now says the figures are a dated
  snapshot, so the next drift reads as drift rather than as an error.

**What a third pass should probe first, now:** the four repairs above that
*added* something — the `DepSeqOps` fourth member, the `project` case, the
`inspect` rule's second clause, and `--prune`'s resolver swap. Each is a
self-authored repair to a self-authored repair, which is the position `F-1`'s
withdrawn module occupied. Per `mem.pattern.review.bind-scope-bar-and-never-self-rule`,
authoring the bar disqualifies the author from ruling on compliance with it.

### Type prototype (2026-08-16, fork `proto/SL-238-types`)

The owner ran the design's type model through a compiler instead of through more
prose: a throwaway implementation by a third agent (Deepseek) in
`.worktrees/proto-SL-238-types`, uncommitted, ~626 insertions over 9 files plus a
new `src/authored_status.rs`. Reported `cargo check --bin doctrine` clean,
`architecture_layering` green (tangle baseline 76, `authored_status` classified
engine), with live smoke-tests of the footer, `inspect`, `--prune`, the clearing
verbs and the stderr advisory. The `doctor` check (§5) was not written; test
modules do not compile, by design — call-site churn from the signature changes.

**It implements revision 57, not the fork's checked-out design.** The fork sits at
`0691a8c4c`, whose committed `design.md` predates every self-attack repair; the
prototype nonetheless carries `DepSeqOps`, the `dispatch` injection point,
`AbsentDrop`, `admit_target`, `prune_verdict` and `authored_class`, so it was
built from the main tree's uncommitted working copy. Anyone re-entering that fork
will read a stale design beside current code.

**It found two real defects, both inside the injection repair** — the region the
section above named as priority 1 for a third pass, and both on the `DepSeqOps`
fourth-member / signature surface that list called out first. Verified against
source here, not taken on report:

1. **`DepSeqOps`'s rank types do not match the functions they must hold**
   (design.md:1272-1274). `run_after_edge` takes `rank: i32`, not `Option<i32>` —
   `rank == 0` is already the "no rank" sentinel (`dep_seq.rs:166`,
   `cli.rs:741`), so the `Option` invents a `None`/`Some(0)` distinction nothing
   consumes and the pointer cannot hold the function. Worse on the other leg:
   `run_after_remove` takes `rank: i32` as an **upper bound** — "only edges with
   rank ≤ N are removed" (`cli.rs:738-740`) — and the design's
   `remove: fn(Option<PathBuf>, &str, &str)` drops it, so routing
   `backlog after --remove --rank N` through `ops.remove` would silently discard
   the documented ceiling. That is a behaviour regression §7's preservation
   clause would not have named. `run_needs_remove`'s rankless signature
   (design.md:1186) is correct — the `needs` array carries no rank.
2. **`ensure_admissible_dep_target(kind)` cannot render its own message**
   (design.md:1355). The `ensure!` it extracts (`dep_seq.rs:92-97`) interpolates
   `{target}` — the caller's ref string — *and* `tkref.kind.prefix`. A signature
   carrying only `&'static Kind` has lost the ref, so the refusal renders
   `` `ADR` is a ADR entity `` with the prefix doubled. The stated goal at
   design.md:1354 is that both paths "refuse in one voice"; the signature defeats
   it. Widening to `fn(&'static Kind, &str)` fixes it.

Three lesser findings, reasoned but not source-verified to the same depth:

3. **Non-padded stored refs are unremovable.** §6's three-tier needle
   (design.md:1209-1212) canonicalises through `kinds::canonical_id`, so a
   hand-authored `needs = ["SL-1"]` is sought as `SL-001` and never matches — the
   verbatim tier only fires when *both* parses fail, and `SL-1` parses. Bare
   `154` → `SL-154` works. Narrow: such a ref resolves, so §5's check would not
   report it. Worth one line in §7 rather than a mechanism.
4. **`prune_verdict`'s `Unavailable` reason arm is unreachable** —
   `authored_class(Unavailable) == Unrecognised ≠ Terminal`. Consistent with
   design.md:1412-1414, which already says `Unrecognised` covers that case and
   keeps the edge; a prototype artefact, not a design defect.
5. **The absent-backlog-id fact is carried twice.** The adapter still computes a
   `Dangling` override *and* `project` now records the `AbsentDrop` (§4's second
   case, added in the self-attack round). Correct per §2 — `render_overrides`
   drops `Dangling` — but the adapter's computation is only un-rendered, not
   retired. Worth stating where §4 claims the class is counted once.
6. **The probe and row types carry no derives.** `RefState`, `BoundaryRow`,
   `BoundaryProbe` and `PruneVerdict` will need `Debug`/`Clone`/`PartialEq`/`Eq`
   before §7's tests can assert on them. Deliberately **not** a design change —
   `design.md` declares no derive on any type it shapes, and adding attributes to
   a design document is the wrong altitude. Recorded here so it survives the
   fork, which is disposable; it belongs to whoever executes §7.

**Disposition (2026-08-16).** Findings 1, 2 and 3 were verified against source and
landed in design revisions 58 (adoption) and 59 (materialise, byte-identical),
touching `sec-2`, `sec-6` and `sec-7` only. No attestation was spent — every
section was already `review=outstanding`. Findings 4 and 5 needed no design
change; 6 defers to execute. A second prototype round was then issued against the
same fork to prove 1–3 compile and behave, and to keep hunting.

**What the experiment establishes about method, separate from the findings.** The
two confirmed defects are both type-level and both sat in prose that had already
survived a nine-repair self-attack by its own author. Neither was findable by
re-reading; both fell out of a compiler in one pass, at a token cost the owner
puts on par with one round of design-text editing. The self-rule bar
(`mem.pattern.review.bind-scope-bar-and-never-self-rule`) is what the prototype
routes around: it does not care who authored the signature.

### Open

- ~~**Run at `drafting`, all 9 sections `review=outstanding`**~~ — **closed
  2026-08-16.** Every gate act is now current and the run is `locked` at rev 63.
  Query the run, never this line: `doctrine design resume 238`.
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
- **§3 rule 2 and §7's VT-5 gloss overclaim — prose fix at reconcile, owner
  accepted 2026-08-16.** Both say a future kind that derives its status *and is
  not added* to `DERIVED_STATUS` "must fail a test, not degrade quietly". An
  equality pin (`DERIVED_STATUS == [RV]`, as VT-5 specifies and PHASE-01 T1
  landed) fires when a kind **is** added, never when one is omitted — omission
  leaves the assertion true and the suite green. The pin is a tripwire on
  *intent*, not a completeness check.

  The safety holds by a different mechanism than the one named: an omitted
  derived-status kind takes the common arm, `meta::read_meta` fails on its
  missing top-level `status`, and `STD-003` turns that into a disclosed `Err`.
  Loud, not silent. So the defect is the claim, not the protection — and the
  irony is worth keeping: this slice exists because a footer stated a claim it
  had not checked.

  **Reconcile action:** correct §3 `The three standing rules the degradation
  carries` (rule 2) and §7 `The probe` (the VT-5 bullet) to state what the pin
  does, and to name the strict-read failure as what enforces completeness. No
  code change; the owner declined the alternative (a per-kind fixture asserting
  every kind outside `STATUS_LESS ∪ DERIVED_STATUS` authors a top-level status —
  24 fixtures to make an already-loud failure louder).

  Raised by the `DEC-242` blind test-author seat at PHASE-01 T1, which is the
  arrangement working as designed: a prototype-informed author would have
  transcribed the pin and never questioned the prose around it.

- **§6 and §7 both say `--prune` has no test coverage; it has five tests —
  prose fix at reconcile.** `design.md:1487` ("no test coverage at all, in
  either copy") and `design.md:1665` ("no coverage in either copy") are false as
  of `7958af7ca`. `tests/e2e_dep_seq_verbs.rs` carries five SL-105-era prune
  goldens: `after_prune_drops_resolved`, `after_prune_noop`, `after_prune_mixed`,
  `after_prune_absent_target`, `backlog_after_prune`.

  The substance survives — every behaviour PHASE-02 `EX-1` names is genuinely
  unpinned, because those goldens assert `contains("resolved")` /
  `contains("dropped")` rather than the rendered reason, so the
  `resolved`/`closed` split, the `/resolution` suffix, the silent keep on an
  unreadable target and the bare-ref deletion all pass through them unobserved.
  Two live consequences the design does not account for:

  1. §7 `Preservation` does not list these five, so PHASE-06/07 will break
     `after_prune_absent_target` (its `absent` reason becomes `unresolved`)
     without the design having declared it a deliberate supersession. PHASE-02
     `D2` pre-empts this by annotating the test in place.
  2. Read literally, §6 makes PHASE-02 look greenfield, and the `resolved` and
     `absent` pins get written twice.

  **Reconcile action:** correct both sentences to say the existing goldens pin
  the *decision* (which edges survive) and not the *rendered reason*, and add the
  five to §7 `Preservation` with `after_prune_absent_target` marked superseded by
  PHASE-07. No code change.

  Raised at PHASE-02 planning, from a context blind to the type prototype
  (`execution-protocol.md` `R1`). Same seam as the three PHASE-01 defects — a
  design claim and the criterion resting on it agreeing with each other and
  disagreeing with the tree — but reached from the other side: not *write the
  assertion*, but *look for the assertion that already exists*. Now landed in
  `execution-protocol.md` §3 and in
  `mem.pattern.testing.grep-for-the-pin-before-characterising`: **before pinning
  a before-state, grep for the pin.**

- `QUE-221` — whether `backlog after`'s cross-kind target refusal needs a
  before-state pin. Deliberately changed behaviour by §6, but outside §7's named
  characterisation set and outside PHASE-02's `EX-1`/`EX-2`, so PHASE-02 raised it
  rather than deciding it. **Answerable only before PHASE-07 lands** — after that
  the pin cannot be written at all. One test if accepted; zero cost if declined.
- **§7's VT-2 bullet contradicts §3's own arm table — design-text fix at
  reconcile, owner accepted 2026-08-16. `plan.toml`'s VT-2 row is already
  amended.** §7 says a derived-status kind *"with no toml at all still returns
  `Unavailable`"*. §3's arm table gives both special arms a **lenient title
  read**, and `read` returns `Authored { status, title }` — so an absent file is
  an `Err`, and `Ok(Unavailable)` is unreachable for that fixture. The two
  clauses cannot both hold.

  Resolved in favour of the arm table, on three independent supports:

  1. **§3 rule 3 outranks it.** An arm that short-circuits before reading would
     return `Ok(Unavailable)` for a *corrupt* `RV` toml — a broken file and a
     tooling limit sharing one signal, precisely what rule 3 and `STD-003`
     forbid.
  2. **The overlay needs the title.** `catalog::scan` reads every `RV` on every
     walk and `ScannedEntity.title` feeds the priority display surfaces; an arm
     that returns before reading has no title to give.
  3. **`EX-9` decides it.** Today `status_and_title_for("RV")` calls `title_for`,
     which `read_to_string`s the file, so a missing `RV` toml already `Err`s.
     Short-circuiting would be a behaviour *change* in a phase whose exit
     criterion requires the `search`/`map`/`catalog` suites green **unmodified**.

  So `without reading` names the **status** read, which is never attempted — not
  the title read, which always is. A missing or corrupt file is `Err` on every
  arm.

  **Reconcile action:** correct §7 `The probe`'s VT-2 bullet to fixture *a
  derived-status kind whose toml does carry a status* rather than *no toml at
  all*, and say which read "without reading" refers to. `design.md` is locked at
  rev 63, which is why this is a reconcile action and not an edit here.

  Raised by the blind seat at PHASE-01 T2 as its OQ-A/OQ-B — one ruling settles
  both. Second design defect the arrangement has surfaced.
