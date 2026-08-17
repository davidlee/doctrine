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
fresh-as-of: 2026-08-17 · **Audited — `RV-363` `done`, ten findings, no blocker; slice moves to `reconcile`.** The disclosure was re-derived against the live corpus (32 authored cross-kind edges → exactly the 10 rendered `boundary:` rows; all 13 suppressed targets individually confirmed terminal), `doctrine check gate` exit 0, command tangle **measured 76**. `RV-358`'s five `answered` findings — including its `F-1` blocker — verified terminal against the built tree, so that ledger is `done` rather than waived. Two findings fixed in the audit (`annotated` guard collapsed; `CHR-071` minted), two accepted as owned drift (`ISS-441` extended, `QUE-222`), five routed to the reconciliation brief · slice/`reconcile` · design/**locked** (run `dr-01a00475`, rev 63; all nine sections attested human-lane; `design-accepted` current; gate cleared) · `RV-363`

### Produced

- `RV-363` — the reconciliation audit ledger: brief, ten findings, `## Synthesis`
  and `## Reconciliation Brief`. The brief is the authority on the
  named-output-change list (**eight**, not the seven `F-8`'s append-only
  response recorded).
- `CHR-071` — the `catalog::scan::kref_for` collapse, minted from `notes.md`'s
  reasoning (`F-5`); `references SL-238 --role originates_from`.
- `ISS-441` extended — a **second route** to its false `PASS`, one its two
  leading candidate fixes both miss: a superseded test's name survives inside
  its successor's mandated supersession comment, so the retired row passes
  forever. Measured over `tests/e2e_dep_seq_verbs.rs` with a positive control.
- `src/backlog.rs` — `render_overrides`' boundary line now composes the shared
  `annotated` guard instead of open-coding it (`F-3`); doc comment corrected
  from "one guard, both axes" to name both surfaces. −5 lines, footer
  byte-identical.
- `RV-358` closed out — `F-1` (blocker), `F-3`, `F-5`, `F-9`, `F-10` verified
  against the built tree rather than waived. The close-gate is clear on
  evidence, not on a disposition.

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

#### PHASE-03 (2026-08-17) — `4ad589192`, `9d75444b8`

- **§5 landed greenfield, one commit.** `dep_seq_ref_findings` beside
  `lifecycle_findings`; `read_all_tolerant` + `ReadFailure`; one `extend` in
  `run_doctor` under the existing `RelationIntegrity` category, no new variant.
  `VT-1`…`VT-7` green, `VA-1` zero on the live corpus. `just gate` exit 0 on a
  verified exit code, clippy zero warnings.
- **The wiring shipped WITH the check, not after it** — the sheet had planned them
  as separate commits, which was wrong on `R4`'s own terms: with no consumer, the
  check needs a `dead_code` attribute added and removed inside one phase. The
  three-task split survived; the commit boundary did not.
- **`read_all_tolerant` discloses an unreadable kind *tree* as well as an
  unreadable item.** `entity::scan_ids` returns `Result`, and the obvious
  `let Ok(ids) = … else { continue }` would have been a silent skip inside the
  phase whose purpose is `STD-003` compliance. Rough edge, recorded rather than
  fixed: `read_failure_reason` names `backlog-NNN.toml` even when the failure was
  the sibling `.md`, since `read_item` reads both and the root cause does not say
  which. The root cause disambiguates in practice; §5's example shows the toml.
- **`VT-6` pins its own discriminating power.** It asserts `read_all(root).is_err()`
  as a precondition, so the fixture ordering that makes the tolerant reader
  necessary is proven rather than assumed — without it the test could pass
  vacuously against either reader.
- **`VA-1` returned a false zero twice before it returned a true one.** Both from a
  broken query, not a clean corpus; the positive control is what caught it. See
  `mem.fact.doctor.json-category-is-the-display-name`.
- **The auto-recorded conformance boundary swept in a non-phase commit** — the
  span ran from the rtk-memory correction (`17ec1e09e`) because that landed after
  the `in_progress` flip. Tightened with
  `slice record-delta 238 PHASE-03 --start 4ad589192^ --end 4ad589192`. The tool
  warned; it does not refuse.
- `ISS-441` — minted; see Open.
- The **rtk hazard was retired** (`17ec1e09e`): `mem.fact.rtk.output-filter-rewrites-identifiers`
  superseded by `mem.pattern.verification.suspect-transcription-before-tool`, and
  `execution-protocol.md` §5 corrected. The proxy it blamed had been removed months
  before the sightings it explained. `.doctrine/` changes committed separately from
  the code throughout.

#### PHASE-04 (2026-08-17) — `522765a76`, `ff542d519`/`17b6efc74`, `76fa0ee84`, `cf1ce7bab`

- **§4 and §2's listing half landed** — the largest single surface in the slice.
  `AbsentDrop` gained an axis and a second recording case; `project` was hoisted
  out of `compose` into `list_rows`; `probe_ref`/`probe_boundary` classify what an
  authored ref turned out to be; `render_overrides` lost two parameters, the
  `AbsentDrop` leg, the `Dangling` arm and the terminal suppression, and gained
  the `boundary:` block; `classify_dangling` is deleted; `backlog list` emits the
  count-only stderr advisory. `EX-1`…`EX-8`, `VT-1`…`VT-10`, `VA-1`/`VA-2` all
  discharged. `doctrine check gate` exit 0 **on a verified exit code**, twice —
  after the code commit and again at T5.
- **The conformance span is five commits and every one of them is this phase's**
  — T0, the memory T0 produced, that memory's slug symlink, T1–T4, and T5. The
  flip warned as it always does; no `record-delta` tightening was owed, unlike
  PHASE-03. `.doctrine/` prose was committed separately from code throughout.
- **`D-1` was revised mid-phase after an owner challenge** (`eec5d46d2`, before
  any code). The first answer sited the cross-kind entity seeder in
  `backlog::test_support`; it failed the least-generic-home test, because that
  module is chartered as *the single source of the `backlog-NNN.toml` fixture
  literal* and non-backlog entities are not that literal. It moved to a new
  `#[cfg(test)] pub(crate) mod test_support` in `src/authored_status.rs` — beside
  the reader whose input it authors, and already generic over `KindRef`. This
  **removed** duplication rather than adding it: three hand-rolled copies existed
  and PHASE-04 would have minted a fourth.
- **`plan.toml`'s `EX-7` amended in place** (id kept, reasoning inline, per `R5`)
  — it named two superseded tests and there are three. See `F-2`, and the
  supersession detail in `VA-2` item 4 below.
- **The oracle pass poked no hole in the plan** and corroborated its two riskiest
  calls (`A-2`, the renderer takes rows not the whole probe; `A-3`, the advisory
  fires alongside the cycle warning, not instead of it). Of its three carried
  defects, two were fixed by what landed — the `Absent` trailing space (the
  annotation is appended only when non-empty) and the missing memoisation — and
  the third, `O-1`, was the prototype sharing `F-2`'s blind spot, now moot. The
  fork stays closed (`DEC-242`).
- One friction observation (`99e1ec0f9`): a memory superseded at PHASE-03 was
  still being surfaced by the hook as live guidance.

**`VA-2` — every intentional output change, with its reason.** The reconciliation
brief reads this list. Nothing below is a regression: each item is mandated by
the criterion named beside it, and the fourth is a *scoped* disappearance whose
scope is easy to overstate. Recorded here rather than on the phase sheet — a `VA`
over gitignored runtime state leaves no evidence an audit can re-derive
(`mem_019fd1d862887d42b7a1f88c28fd28a7`).

1. **NEW — the `boundary:` footer block** (`EX-3`/`EX-4`, `DEC-232`).
   `backlog list --by sequence` now emits a second footer block, after
   `overrides:` when both are non-empty, one line per `(dependent, target)` pair:
   `  ISS-327 needs QUE-219 (open)` — dependent-first, relation word, target,
   the target's **own** authored status, no arrow. Axes join on one line; a
   `StatusClass::Terminal` target is suppressed (a satisfied prerequisite
   explains nothing); an unresolvable target is counted, never shown. **Why:**
   `DEC-231` made cross-kind edges disclosed-but-not-ordering, and `DEC-232` gave
   the footer the contract *state only what is needed to understand the rendered
   content* — a live cross-kind prerequisite is exactly that, and the surface
   could not previously represent it. On the live corpus this is **ten** lines.
2. **NEW — a count-only advisory on stderr** (`EX-6`, `DEC-236`).
   `UNRESOLVED_ADVISORY` (`backlog.rs:2457`, a named constant per `STD-001`):
   `backlog list: {n} authored needs/after refs name nothing — run `doctrine doctor``,
   emitted from `list_rows` when `probe.unresolved > 0`, **alongside** — not
   instead of — the `Ordering::Degraded` cycle warning; both can fire. It names
   no ref, and fires only under `--by sequence` (`--by id` never composes, so
   never probes). **Why:** `DEC-232` moved every ref-integrity failure off the
   footer to `doctor`; a silent removal would have made the listing quieter about
   a real defect, so the signpost is the price of the deletion. **On the live
   corpus it does not fire** — zero unresolvable authored refs, stderr empty.
   Its population caveat is `F-1` below (*§2's "same population" claim*).
3. **GONE — the listing footer's `dropped (dangling: …)` line form** (`EX-5`).
   Both producing legs are deleted from `render_overrides`: the `AbsentDrop`
   loop and the `Dangling` arm, and with them `classify_dangling` and the
   terminal-suppression block. **Scope, measured, because the wording is shared:**
   pre-phase `src/backlog.rs` carried three occurrences of that line form —
   `:2083` (`run_after`'s `--prune` report), `:2304` (the `AbsentDrop` loop) and
   `:2334` (the `Dangling` arm). Today exactly one survives, `:2279`, and it is
   the `--prune` report, **untouched** — that copy is PHASE-06/07's business, not
   this phase's. `SoftCycleEvicted` and `Contradicted` render byte-identically;
   `tests/e2e_backlog_list_order_golden.rs` passes **unmodified**, which is the
   external proof of that.
4. **SUPERSEDED — three tests, not two** (`EX-7` as amended by `F-2`; `R5`).
   Each replacement carries a `**SUPERSEDES**` doc-comment stating what it now
   asserts and why the old assertion no longer holds:
   - `list_sequence_records_terminal_and_absent_drops_with_status_and_resolution`
     → `list_sequence_suppresses_a_terminal_dep_and_withholds_an_absent_ref`.
     It asserted an `overrides:` block is present and that `ISS-099` renders with
     the word `absent`; both are now false on its own fixture. The claim worth
     keeping — `IDE-019`'s terminal-prerequisite suppression — is re-pinned,
     beside its new sibling: the absent ref is withheld from stdout too, by a
     different route, and neither is silently lost (`doctor` has both).
   - `list_sequence_stays_silent_on_a_cross_kind_drop_but_names_a_malformed_ref`
     → `list_sequence_shows_a_cross_kind_target_and_withholds_a_malformed_ref`.
     Superseded on **both** counts: a resolvable non-terminal cross-kind target
     is the `boundary:` block's whole subject (silence was the honest interim
     answer only while the view could not represent it), and a malformed ref
     leaves this surface entirely for `doctor`.
   - `list_sequence_emits_no_footer_when_every_drop_is_cross_kind`
     → `list_sequence_emits_no_block_header_over_nothing`. The narrower claim it
     was really protecting — no header over nothing — survives, re-pinned on refs
     that genuinely render nothing (unresolvable ones, which are counted).

   The word **`absent`** — the false verb this slice exists to remove — now
   appears nowhere on the **listing** surface. It survives once more in the tree,
   at `backlog.rs:2253`, as `run_after --prune`'s reason word — the same untouched
   copy that owns the surviving `dropped (dangling: …)` line. Both are PHASE-06/07's
   to retire; a sweep that reads either as a PHASE-04 leftover is reading the wrong
   surface.

**`VA-1` — the deletions are deliberate, evidenced by the diff, not by the
absence of a lint warning.** `render_overrides` lost **two** parameters, not one:
`(corpus: &BTreeMap<ItemId, &BacklogItem>, absent: &[AbsentDrop], overrides)` →
`(boundary: &[BoundaryRow], overrides)`. `compose` went from
`(corpus: &[BacklogItem])` to `(inputs: &[OrderInput], boundary: &[BoundaryRow])`,
and its `cmap` build is gone with the `Dangling` arm that was its only reader —
as `Learned` predicted (*"read only by the `Dangling` arm; both die with it"*).
`project` is hoisted into `list_rows`, so the corpus is walked **once**:
`read_all` → `project` → `probe_boundary` (the one new impure read) → `compose`.

**`VT-10` / `EX-8` — preservation, diff-verified over the whole phase span**
(`ff132fe69^..HEAD`, which covers T0 as well as T1–T4). `src/backlog_order.rs`
is absent from the changed-file list, and the span touches `tests/` **not at
all** — zero files. `cargo test --bin doctrine backlog_order` 16 passed,
`… priority::` 303 passed, `tests/e2e_backlog_list_order_golden.rs` 14 passed,
all with no edit. `list_sequence_and_id_share_membership_differ_on_order` does
not appear in the diff of `src/backlog.rs` and is unmodified.

**T1–T4 landed as ONE commit, deliberately (`R4`).** `AbsentDrop::axis` has no
production consumer until `probe_boundary`; that has none until `list_rows`;
`list_rows` needs `probe.unresolved` for the advisory. Splitting meant adding and
removing a throwaway lint attribute at each seam. T0 stayed separate so this
`VA-2` record has nothing to disentangle from a behaviour-neutral refactor.

**PHASE-01's `expect(dead_code)` on `authored_class` self-cleared**, exactly as
its own reason predicted — rustc flagged the expectation unfulfilled the moment
`probe_boundary` landed. Deleted, with a comment recording that it behaved as
designed. Second confirmation of
`mem.pattern.lint.dead-code-derives-count-as-reads` in the opposite direction:
`AbsentDrop`'s new `axis` field needed no attribute at all, being live on arrival
through the struct's existing `PartialEq`/`Eq` derives.

**`RefState` derives only `Clone`.** It holds a `&'static entity::Kind`, which
derives neither `PartialEq` nor `Debug`; deriving them here would mean changing a
shared type for local convenience. `matches!` covers the one discrimination the
module needs. This settles the prototype's finding 6 (*"the probe and row types
carry no derives"*) in the narrow direction — if a later phase wants
`RefState: PartialEq`, that is a real decision about `entity::Kind`, not a
formality.

**Live-corpus re-scan (2026-08-17).** `./target/debug/doctrine backlog list`
exits 0, **stderr empty**, and renders **ten** boundary lines — not §2's eleven —
with `IMP-437 after SL-256` where §2's dated sample says `SL-251`. The corpus
drifted since that snapshot; re-scanned and reported rather than reconciled to
the design, per the standing rule §1 anticipates.

#### PHASE-05 (2026-08-17) — `cf313cbc7`, `42c835b0e`

- **§4's record half landed.** `probe_item_refs` is the second projection over the
  shared `probe_ref` — every declared ref classified, keyed by `(Axis, String)`,
  nothing dropped — threaded into `run_show_inspect` → `format_metadata` so
  `backlog inspect` and `backlog show` state each declared target's status.
  `EX-1`…`EX-4` and `VT-1`…`VT-5` all discharged; `just gate` exit 0 on a
  **verified, unpiped** exit code. Two commits, **both this phase's own** — the
  flip warned as always, no `record-delta` tightening owed (as PHASE-04, unlike
  PHASE-03).
- **The five rendering rules split across two named functions, one rule each.**
  `ref_annotation` carries §4 rule 1 — a *resolvable* backlog target renders bare
  (its status is already on every listing row the reader has), an unresolvable one
  is annotated regardless of prefix. `annotated` carries the `R-c` guard: append
  only when non-empty. The guard is written **once** for both axes rather than
  duplicated at each render site, which is the structural fix for the bug PHASE-04
  caught by inspection (`O-2`, the `Absent` trailing space).
- **`T0` culled three dead parameters instead of suppressing the lint** — the one
  scope judgement in the sheet, outside `EX-1`…`EX-4`, **owner-ruled**.
  `format_metadata` sat at exactly 7 and `clippy::too_many_arguments` fires above
  it; `_estimation_unit`, `_lower_pct`, `_upper_pct` were threaded through four
  signatures and read by none. 7 → 4 → 5, no suppression, no new type. The eight
  test call-site edits are mechanical: `git diff -U0 | grep -c assert` over that
  commit is **0**.
- **The cull stopped one step earlier than the sheet sketched, and that is the
  phase's one real trap.** `lower_pct`/`upper_pct` came from
  `estimate::resolve_confidence`, which is a **five-arm validator**, not a getter.
  Deleting the call with the parameters would have compiled clean, passed every
  test, and silently removed an error path from `backlog show` / `inspect`. The
  call is kept for its `?` and its value discarded. Minted as
  `mem.pattern.refactor.dead-param-cull-can-drop-a-validator`
  (`mem_01a00cd0ade47c30a84dc90b49a20a95`).
- **The oracle pass poked no hole in the plan** — third run of `R1`, third
  no-hole. It confirmed `D-1` (the fork threaded an **8th** parameter with **no**
  `#[expect]`, i.e. it never linted — so it supplies no evidence for the
  annotate-instead route), confirmed `D-2` (the fork builds the map before the
  format match, so its `--json` pays for a probe it never renders), and confirmed
  `A-1`: the fork's `probe_item_refs` has **no memoisation at all**, the same
  omission `O-3` caught in its `probe_boundary` at PHASE-04. **Systematic, not
  incidental** — treat the fork as a source of *shapes*, not of correctness. It
  did supply one shape worth taking: siting §4 rule 1 in a small `ref_annotation`
  wrapper over `render_ref_state`, which the landed renderer did not carry. Fork
  back closed (`DEC-242`).
- **One oracle finding needed no action and is recorded so it is not re-derived.**
  `ref_annotation`'s backlog-bare arm would swallow a `(status unavailable)` on §4
  rule 5's terms — but cannot: `AuthoredStatus::Unavailable` is decided
  **statically from the kind** (`authored_status.rs:253-256`), `DERIVED_STATUS` is
  `[RV]` and `STATUS_LESS` is `[REC]`, and `BACKLOG` intersects neither. If that
  ever changes, `kinds::tests::the_derived_status_kind_set_is_pinned` fails first.
- **Preservation was demonstrated, not asserted.** The phase span touches
  `src/backlog.rs` and nothing else, so `src/backlog_order.rs` and
  `tests/e2e_backlog_list_order_golden.rs` are untouched by construction; and
  `probe_boundary`, `probe_ref`, `render_ref_state`, `render_overrides` and
  `list_rows` are byte-identical to `b81166edb` — checked with `format_metadata`
  as a **positive control** that the comparison can report CHANGED.

**`VA`-style note — the one intentional output change.** `backlog show` and
`backlog inspect` now append the target's own authored status to each `needs` /
`after` entry: `needs: QUE-219 (open), ISS-084`. Additive and rule-bound — a
resolvable backlog target is unchanged, an `Absent` status appends nothing (not
even a space), and `--json` is byte-identical because the map is never built on
that path. **Why:** `DEC-234` puts the full authored record on this surface, and
it lacked only what state each declared target was in.

- **PHASE-06 done — §6's clearing half, minus `--prune`** (`b48cd2341` plan,
  `0d191d6e6` oracle, `2e7a481c9` T1–T2, `a792b4432` T3). `EX-1`…`EX-4` and
  `VT-1`…`VT-6` all discharged; `just gate` and `doctrine check gate` both exit 0
  on **verified, unpiped** codes. Two code commits, **both this phase's own** — the
  flip warned as always, no `record-delta` tightening owed.
- **The leaf's remove seam now mirrors its append seam.** `remove_needs` beside
  `remove_after`, `RelRemove` beside `RelEdit`, and `remove(path, &RelRemove)` as
  the single IO wrapper. `rel_array_mut(doc, axis)` is the one navigation body
  behind both cores, and its message is **byte-identical** to `remove_after`'s of
  today for `axis == "after"` — which is what lets `EX-1`'s "`remove_after`'s own
  logic is unchanged" hold through a refactor. `append` keeps its own copy
  deliberately: its message interpolates the path and reads "before adding edges".
- **`EX-1`'s call-site census was wrong, and the compiler said so.** Amended at
  planning from three to four (`### Open` carries the design half); reshaping
  `remove`'s signature then produced exactly four caller errors, no more and no
  fewer. The fourth was `run_after_prune`'s removal loop, which took the **argument
  reshape and nothing else** — its `{source}`-as-typed echo is PHASE-07's decision,
  not this phase's inheritance. Minted as
  `mem.pattern.planning.let-the-compiler-recount-the-call-sites`.
- **`after --remove` gates the source only — the phase's deliberate behaviour
  change, and its commit says so.** `a792b4432`'s body names `EX-3`, §6's own
  "deliberate behaviour change" sentence, the PHASE-03 repair path it unblocks, and
  PHASE-02/`VT-3` as what it supersedes. The pin was **rewritten in place**, not
  deleted and re-added, so the diff reads as the supersession it is. Three
  author-time guarantees are given up on the remove path only — the target's
  on-disk resolution, its kind gate, the self-edge refusal — and the replacement
  test asserts the **author-time** gate is untouched, so the widening cannot be
  misread as general.
- **A second e2e test moved for the same reason, and it is not a supersession
  gap.** `after_remove_nonexistent` (SL-105 era) still refuses and still exits
  non-zero; only the *reason* moved, from the author-time gate to the zero-count
  bail. The sheet's STOP condition was watching for a refusal that *disappears*;
  this is a refusal that *relocates*. Comment says so in place.
- **`EX-4`'s bug is preserved on purpose and now has a red test guarding it.** A
  stored `needs = ["SL-1"]` is not cleared by `--remove SL-1` — both parse tiers
  hand off to `canonical_id`, so the needle is `SL-001`, and the verbatim tier does
  not rescue it because `SL-1` *parses*. Closing it later is a `kinds` change with
  five other callers, and is now necessarily deliberate.
- **The oracle pass poked no hole in the plan — fourth run of `R1`, fourth
  no-hole.** It corroborated the leaf shape and `D-1` (it too made the needle one
  shared function, whose name `canonicalise_target` the plan adopted), and it
  diverged once in a way worth keeping: it met the same "the remove path needs the
  canonical source id" requirement with a wrapper that parses the source a second
  time. The plan's reshape removes that double parse instead — one it has carried
  since SL-158. Both PHASE-02-era prototype defects re-confirmed still present; one
  new one logged forward (`### Open`). The prototype still has **no tests** and has
  still **never been compiled** under this repo's denials.
- **§6's own code snippet does not pass this repo's lints** — it renders the needle
  as `.map(..).unwrap_or_else(..)`, which `clippy::pedantic` denies
  (`map_unwrap_or`). Written as `map_or_else`; identical tiers, identical order. Not
  worth a reconcile action, but it is the first instance of the "never linted"
  class found in the **design text** rather than in the fork.
- **`ISS-441` fired again, exactly where the sheet predicted.** PHASE-06 is the
  first phase to touch `src/dep_seq.rs` / `src/commands/dep_seq.rs`, so after its
  `completed` flip **PHASE-07 `VT-4` and PHASE-08 `VT-6` both read `PASS`** on
  keyword coincidence alone. Neither phase exists. Their siblings still `FAIL`
  honestly, which is what makes the two `PASS`es legible as the defect.
- **`QUE-221` answered — the cross-kind pin landed** (`c84966546`), between
  PHASE-06 and PHASE-07, outside any phase's conformance span. That placement is
  deliberate: it is a PHASE-02 debt paid late, not PHASE-07 work, and the window
  closes when PHASE-08 lands. **`PHASE-08` inherits an obligation, not a free
  green** — it must find
  `backlog_after_pins_the_cross_kind_target_refusal_on_both_legs` RED and rewrite
  it in place into its opposite. Detail and the mutation evidence in `### Open`.
- **PHASE-07 landed `--prune`'s probe in two commits, split on what each has to
  explain.** `17e25df18` (T1 red suite + T2 the collapse, batched per `R4` because
  the superseded pins go red the instant the behaviour changes) and `c3d67f773`
  (T3, the canonical source echo). `EX-1`…`EX-5` discharged; `VT-1`…`VT-5` green;
  31/31 in `tests/e2e_dep_seq_verbs.rs`; `just gate` exit 0 at both.
- **The SL-105 goldens' fixtures encoded the very bug the phase fixes.**
  `after_prune_drops_resolved` and `after_prune_mixed` set a **slice**'s status to
  `resolved` — a BACKLOG word, outside ADR-009's slice vocabulary. They only ever
  passed because the old probe applied one hardcoded table to every kind alike, so
  they were asserting the cross-kind leak rather than the intended behaviour.
  Fixtures moved to `done`; the behaviour they pin is unchanged. `STOP-2` was
  checked and did not fire — the red is §6's second consequence (`Unrecognised`
  keeps the edge). Worth keeping: §6 names the *widening* (`done`/`answered` become
  prunable) and never says "and a cross-kind status word stops working", but both
  are the same routing change.
- **The stderr disclosure wording is ours, not the design's — a SIXTH named output
  change for the reconciliation brief.** `EX-3` mandates that the unreadable-target
  keep says why and states no string, so T2 authored
  `{source_id} after {to} (rank {r}) kept (unreadable: {err:#})`, mirroring the drop
  line's shape. `{err:#}` renders the cause chain, which carries an absolute path, so
  `VT-3` pins the stable prefix with `starts_with` rather than a byte-exact stderr.
- **Two PHASE-02 pins were REMOVED rather than rewritten** — a disposition `EX-5`
  permits but does not spell out. Each was fully subsumed by a replacement whose doc
  comment names it, with a marker comment left at the removal site so the deletion
  is visible in the file and not only in git. PHASE-08 `EX-3` establishes removal as
  an accepted disposition for pins whose *code* goes away; here the code stayed and
  the *assertion* went away, which is the weaker case. Flagged deliberately so a
  reviewer can disagree cheaply.
- **T3 canonicalised the source echo, and the refactor it forced removed a
  third copy of a line.** All three of `run_after_prune`'s output lines now echo the
  canonical source id (`### Open` carries why the decision was forced, not tasteful,
  and the reconcile action). `resolve_dep_seq_src_path` now returns
  `(PathBuf, String)` — path plus canonical id — because all three of its callers
  canonicalised the returned parts on the very next line and this phase would have
  written the third copy, in the slice whose purpose is removing duplicated tables.
  `resolve_dep_seq_src`'s self-edge refusal compares canonical ids instead of the
  `(prefix, id)` pair; exactly equivalent, since a canonical id carries its prefix.
- **`VA-1` / `VA-2` evidence, recorded here because a phase sheet is `rm -rf`-able**
  (`mem_019fd1d862887d42b7a1f88c28fd28a7`: a `VA` over runtime state leaves an audit
  nothing to re-derive). Measured at `c3d67f773`, each grep with a positive control
  so an empty result is a demonstrated absence:
  - `VA-2` — `"resolved"`/`"closed"` literals in `src/commands/dep_seq.rs`: **0**.
    Positive control, same needle in `src/backlog.rs`: **24** (PHASE-08's leg).
  - `VA-1` — `unwrap_or_default()` in `src/commands/dep_seq.rs`: **0**. Positive
    control, `src/backlog.rs`: **5**.
  - `VA-1`'s second needle, `Err(_) =>` arms, returns **one** hit in
    `src/commands/dep_seq.rs` — `Err(_) => Some("unresolved".to_string())` at `:314`.
    It is **not** a laundered default: the error IS the verdict (a ref that names
    nothing is prunable), which is exactly `EX-4`'s collapse of three reason strings
    onto one token, and the outcome is disclosed on the drop line rather than
    swallowed. Recorded because a literal reading of the criterion returns a hit and
    the next reader should not have to re-derive that it is a false positive.
    Positive control: **41** `Err(_) =>` arms across `src/`.
- **The class sweep found two siblings outside the slice's surfaces, both filed.**
  The classes are *hardcoded terminal vocabulary* and *laundered read*
  (`mem.pattern.review.sweep-defect-class-not-instance`). `ISS-445` — `lazyspec.rs:193`
  spells `"resolved" | "closed"` as the backlog terminal set, the same STD-001 /
  REQ-238 pair SL-238 removes from `--prune`, though its repair differs because it is
  a projection to a foreign wire vocabulary rather than a terminality test. `IMP-443`
  — the 11-site census of `unwrap_or_default()` reads outside `src/backlog.rs`, with
  the discriminating question written down (an absent *optional* body defaulting is a
  total function, not a laundered read; a failed *parse* never is). Neither is
  SL-238's to fix; the census is the deliverable, so nobody re-runs the grep.

- **PHASE-08 landed in three commits: `7be8b55d2` (T0, pre-flip), `1b3b90775`
  (T1, the extracted gate) and `df6185164` (T2, the injection).** T2 is red and
  green together per `R4` — the struct's three edge fields have no production
  reader until `run_after` takes them, so splitting meant adding and removing a
  `dead_code` attribute inside one phase. `EX-1`…`EX-6` discharged; `VT-1`…`VT-6`
  green; 30/30 in `tests/e2e_dep_seq_verbs.rs`, 4410 in the bin suite.

- **`VA-3`, the phase's live result: the command tangle measures 76, unchanged,
  no new accepted violation.** `STOP-1` did not fire — the injection did not leak
  into an import.

  **A green layering suite is NOT evidence for this criterion, and that is worth
  keeping.** The ratchet at `tests/architecture_layering.rs:782-792` raises
  `TangleGrew` only when `actual > baseline`, so green proves ≤ 76 and `VA-3` says
  *unchanged at 76*. The number was read by temporarily setting
  `layering.toml`'s `command` baseline to `0`, running the gate once, reading
  `TangleGrew { baseline: 0, actual: 76 }` out of the violation, and reverting
  (`git diff --stat` clean before the commit). Any future phase asserting an
  *unchanged* tangle needs this, or something like it; asserting a green suite
  answers a weaker question than the one asked.

- **`VA-1` — `backlog` reaches `commands` nowhere in production code.** Measured at
  `df6185164`, with `src/status.rs` (**1** hit) as the positive control so the
  empty result is a demonstrated absence. `crate::commands` occurs six times in
  `src/backlog.rs` and every one is exempt by construction, reported rather than
  filtered away: `:231` and `:5084`/`:5087` are doc prose, and `:5099`-`:5102` are
  the `dep_seq_ops()` fill helper inside `#[cfg(test)] mod tests`. The layering
  visitor collects with `skip_cfg_test` and returns early on a `#[cfg(test)] mod`
  body, so none of them records an edge — which the measurement above then
  confirms empirically rather than by reading the visitor.

- **The slice-wide `all four` laundered-read claim holds, and one of `O-2`'s three
  residuals was misclassified.** After the leg's deletion `unwrap_or_default()`
  returns **3** in `src/backlog.rs` (`:2411`, `:2728`, `:3565`), down from 5; the
  two that died (`:2295`, `:2319`) were the prune-leg target reads, so §7's own
  wording — *`unwrap_or_default()` **on a read or parse*** — reads **0** and the
  four-copy claim is exactly discharged (`commands/dep_seq.rs` reached 0 at
  PHASE-07). Terminal literals in `src/backlog.rs`: **23**, down from 24 with
  `:2301`; the survivors are backlog's own status *vocabulary* (`:507`, `:508`,
  `:526` — it is the authority for those words, not a copy of them) and test
  fixtures. The discriminator, both times, is *a probe of a cross-kind target's
  terminality*, never the literal.

  `O-2` characterised the three residual `unwrap_or_default()` calls as
  "`Option`-chain defaults over no read at all". True of `:2411` and `:3565`;
  **false of `:2728`**, which is `meta::read_metas(…).unwrap_or_default()` in
  `lifecycle_findings` — a real STD-003 silent skip, in a `doctor` path, where a
  degraded read renders as a clean bill of health. Filed as **`ISS-446`** rather
  than folded in: PHASE-08 owns the dep/seq routing seam, and the four reads this
  slice enumerated are all gone. Third time this slice has met a grep-shaped
  criterion whose literal reading misleads, and the first where the *oracle note's*
  discriminator was the thing that was wrong.

- **`VT-2` was GREEN before the change — a preservation pin, not a red.** The
  deleted backlog leg computed its own `rank_ceiling`, so the ceiling already
  worked; the plan expected three reds and got two (`VT-1`, `VT-3`). This does not
  weaken it: after the injection it is the one assertion that catches a
  `DepSeqOps::remove` filled with a rank-dropping wrapper — expressible as a
  non-capturing closure coerced to the `fn` type — which would silently widen every
  backlog-scoped delete. Recorded because "the red suite went red" is the usual
  evidence and here it is only two-thirds true.

- **`VA-2` — PHASE-08's additions to the named-output-change list.** Five, not the
  one `EX-6` anticipated. Each is mandated by the criterion beside it:
  1. **`backlog after` accepts a cross-kind target on all three legs** (`EX-3`,
     §9 item 10). `backlog after ISS-001 SL-154` was `unknown backlog prefix
     \`SL\``; it now writes the edge. `QUE-221`'s pin is the before-state.
  2. **`backlog after --remove` accepts an unresolvable target** (`EX-3`, §9 item
     4). The routed remove leg gates the source only, so a ref the doctor reports
     at Error severity is now clearable — the repair-vs-authoring split PHASE-06
     made at the top level, inherited here.
  3. **`backlog needs` refuses `RV`, `REC` and governance targets** (`EX-4`,
     `ISS-368`, §9 item 11). A deliberate refusal of input accepted today, and the
     one item in this list that *narrows* rather than widens.
  4. **The `after` legs' TARGET echo canonicalises** — beyond `EX-6`, which names
     only the source. Probed against `7be8b55d2`: the old append leg echoed `{to}`
     as typed, so `backlog after ISS-1 ISS-2` printed `ISS-001 after ISS-2`; it now
     prints `ISS-001 after ISS-002`.
  5. **`backlog after --prune` inherits every PHASE-07 output change at once** —
     reason words collapse to `unresolved`, the `/resolution` suffix is gone,
     status-less targets become prunable, and an unreadable target is disclosed on
     stderr instead of silently kept. Named separately because a `backlog`-verb
     user sees all five arrive in this phase, not in PHASE-07.

- **Three PHASE-02 pins were REMOVED with the leg they pinned** (`EX-3`, `D-2`),
  each leaving a marker comment naming its replacement. This is the *strong* case
  PHASE-07 flagged its own weaker one against: there the code stayed and the
  assertion went; here the code ceased to exist, so leaving them failing was never
  an option and rewriting them would have pinned a second implementation that no
  longer has a first.

- **The in-module `DepSeqOps` fill is deliberately a second copy of `cli.rs`'s.**
  A shared constructor would make a wrong fill invisible to every test using it.
  What actually guards the production fill is `VT-2`, black-box through the CLI —
  which is `D-1`'s whole argument, and the reason all three of `VT-1`/`VT-2`/`VT-3`
  retargeted out of `src/backlog.rs` at planning.

### Learned

- `mem.pattern.testing.a-golden-can-pass-because-of-the-bug`
  (`mem_01a00d78626974b3b585d0bb18aca669`) — **PHASE-07, minted at harvest.** A
  characterisation golden proves the behaviour it asserts; it does not prove its
  own **fixture** is valid, and where the code applies one rule to every kind an
  invalid fixture is *invisible* — the code never consults the vocabulary the
  fixture violates. Two SL-105 goldens set a slice's status to `resolved`, a
  backlog word, and passed only because the old probe hardcoded one table for all
  kinds. So when a cross-cutting hardcoded rule becomes a per-kind routed one,
  treat every red golden as a **fixture suspect first**: read its literals against
  the entity's real vocabulary, not against what makes the test pass. The failure
  mode it prevents is adjusting production code until an invalid fixture goes
  green — re-implementing the defect. Linked to
  `mem.pattern.testing.grep-for-the-pin-before-characterising` and the stale-test-
  binary memory, both of which are the same shape: a red that is not a regression.
- `mem.pattern.planning.let-the-compiler-recount-the-call-sites` **extended at
  PHASE-07** rather than duplicated — the census also ages **within a phase, by
  your own hand**. The sheet named two `{source}` echo sites for `T3`; by the time
  `T3` ran there were three, because `T2` — an earlier task in the same phase —
  had authored a new stderr line carrying the same interpolation. Re-derive the
  population immediately before the task that edits it, not when the sheet is
  written.
- `mem.pattern.testing.pin-the-refusal-reason-not-the-refusal`
  (`mem_01a00d4c61e27de29fba501edf9b29b7`) — **minted answering `QUE-221`.** A
  before-state pin on a *refusal* is the vacuity-prone case, because after the
  change the command usually **still refuses**, just from a later layer. Here,
  deleting both `require_item` target gates moved the `--remove` leg's refusal
  from the kind gate to the zero-count bail: exit code unchanged, target still
  named, `!success` and `contains` both still green. Only `assert_eq!` on the
  message goes red. So: pin the **reason**, and **mutation-test the pin before
  trusting it** — a characterisation test that is green on first write has proven
  nothing until you have seen it red, and for a before-state pin the natural red
  arrives in a *later* phase, so you must manufacture it now. Completes
  `mem.pattern.testing.grep-for-the-pin-before-characterising`: find the pin,
  then make sure the pin can actually fail.
- `mem.pattern.planning.let-the-compiler-recount-the-call-sites`
  (`mem_01a00d11d24d70a1bf531fe6561c426b`) — **PHASE-06, minted at harvest.** A
  criterion that enumerates call sites is a claim about the tree *at authoring
  time*, and it rots in two ways: line numbers visibly, the **count** silently.
  Re-derive it with a positive-control grep before planning against it. Then the
  stronger move: when the change reshapes a **signature**, the compiler enumerates
  every caller for free and cannot miss one — which is a reason to prefer the
  reshape over an additive overload that leaves old callers compiling. The grep
  lets you plan correctly; the reshape *proves* it. Sibling of
  `mem.pattern.testing.grep-for-the-pin-before-characterising` (PHASE-02): a
  design's account of what it *counted* ages exactly as badly as its account of
  what is *untested*, and neither is a claim a prose reviewer thinks to check.
- **The `dead_code` staging window has an asymmetry worth naming, though the
  corpus already carries it from the other side.** `RelRemove::Needs` compiled
  clean under `cargo test` — the tests construct it — and failed `cargo build`
  with *variant is never constructed*. That is
  `mem.pattern.lint.dead-code-staged-whole-module-three-scopes`'s two-compilations
  point read in reverse, so no new memory was minted. The operational form: **a
  test-only constructor hides the lint from the inner loop**; only a non-test build
  surfaces it, which is why `execution-protocol.md` §5's "add it bare, compile, let
  rustc name the set" means *build*, not *test*.
- `mem.fact.layering.ratchet-green-is-one-sided`
  (`mem_01a00e1bb4d178729f2624d3e9a1c794`) — **PHASE-08, minted at harvest.** The
  tangle ratchet raises `TangleGrew` only when `actual > baseline`, so a **green
  layering suite proves `<=` the baseline and nothing more.** `VA-3` asked whether
  the count was *unchanged at 76*, which is the stronger question, and running the
  suite does not answer it. The recipe is in the memory: set the tier's baseline to
  `0`, run the gate once, read `actual` out of the violation, revert, and confirm
  with `git diff --stat` before committing anything. The asymmetry is correct for a
  *ratchet* — a drop must never fail a build — so this is a limit to work around,
  not a defect to file. It matters precisely where a phase's deliverable IS the
  number, which is this phase and `mem.pattern.lint.mcp-server-entangled-with-core`'s
  −2-predicted/−4-measured case. Same family as
  `mem.pattern.search.negative-grep-needs-a-positive-control`: a passing check whose
  pass condition is weaker than the claim being made on it.

  The workaround's cost is real enough to fix rather than only remember —
  **`IMP-444`**, print `count_tangle_edges` per tier from the reporter so the
  measurement is a *read* instead of a mutation of an authored governance file. A
  friction observation is recorded beside it: reverting that mutation is guarded by
  nothing but the author's care.
- `mem.pattern.verification.removal-claim-attributes-every-survivor`
  (`mem_01a00b2334c07380bedc64a3b9d93383`) — **PHASE-04, minted at harvest.**
  Before recording that an output form was deleted, count its producers in the
  **pre-state** and attribute each post-state survivor to a surface. The `VA-2`
  grep returned 1, not 0, and both instinctive readings were wrong: the string
  had three producers across two surfaces, two deleted and one deliberately
  untouched. A post-state grep alone yields a number with nothing to subtract it
  from, so it cannot tell a deliberate survivor from a missed deletion. Mirror of
  `mem.pattern.harness.grep-negative-needs-positive-control` — there a *zero*
  needs a control; here a *non-zero* needs a census.
- `mem.pattern.testing.scannable-entity-fixture-needs-md-and-dates`
  (`mem_01a00b0defd279438fff81cdad1f1a64`) — **PHASE-04 T0**, found after two
  hypotheses were tried and disproved. A fixture that satisfies `meta::Meta` is
  **not** necessarily scannable: `relation_graph::scan_entities` also needs the
  `.md` sibling **and** the toml's `created`/`updated` keys, neither of which is
  a `meta::Meta` field. Either omission yields a **silent wrong count, not an
  error**. The lesson applied for the rest of the phase: on a *move*, preserve
  the fixture body byte-for-byte; normalising it is a separate, later decision.
- `mem.pattern.lint.dead-code-derives-count-as-reads` — **third confirmation, and
  the first in the predicted direction.** PHASE-01's `expect(dead_code)` on
  `authored_class` self-cleared exactly as its own reason said it would: rustc
  flagged the expectation unfulfilled the moment `probe_boundary` landed. In the
  same phase, `AbsentDrop`'s new `axis` field needed no attribute at all, being
  live on arrival through the struct's existing `PartialEq`/`Eq` derives. Compile,
  do not reason — twice more, in opposite directions.
- **A shared type's derives are a decision about that type, not a formality.**
  `RefState` holds a `&'static entity::Kind`, which derives neither `PartialEq`
  nor `Debug`; deriving them on `RefState` would have meant changing `entity::Kind`
  for local convenience. `matches!` covers the one discrimination the module
  needs, so `RefState` derives only `Clone`. This settles the type prototype's
  finding 6 (*"the probe and row types carry no derives"*) in the narrow
  direction — the prototype was right that derives would be wanted, wrong that
  adding them is free.
- `mem.fact.doctor.json-category-is-the-display-name` — **PHASE-03.** `doctor
  --json` wraps rows in `{kind, rows}` and renders `category` as the display name
  (`"Relation Integrity"`), so a `jq` select on the Rust variant matches nothing
  and reports a clean corpus. Two false zeros before the control caught it.
- `mem.pattern.verification.suspect-transcription-before-tool` — **PHASE-03**, and
  the correction of a memory this slice had been relying on. Deterministic tools
  are not the likely defendant when a claim and the source disagree; a documented
  mechanism sitting in context turns a misread into a diagnosis that then hardens
  into corpus. Supersedes `mem.fact.rtk.output-filter-rewrites-identifiers`.
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

> **Collected and routed, 2026-08-17.** Every entry below is now carried by
> `RV-363`'s `## Reconciliation Brief` with a write surface named — read the
> brief, not this list, when driving `/reconcile`. This section stays as the
> evidence trail: each entry records *how* the divergence was found, which the
> brief deliberately does not repeat. Two entries are discharged rather than
> routed — the `kref_for` follow-up is now `CHR-071`, and `ISS-441` gained a
> second route the audit measured. The audit added one the list did not have:
> `tests/e2e_dep_seq_verbs.rs` is missing from the selector registry
> (`RV-363` `F-1`).

- **`ISS-441` — `verify-vt` `PASS`es rows for phases that have not been
  implemented.** Raised at PHASE-03: once a phase modifies a file that later
  phases also name as `test_file`, those later rows leave `UNATTRIBUTABLE` and
  land on `PASS` whenever their keywords happen to occur anywhere in the file —
  `PHASE-04/VT-2` (`terminal`, `boundary`) was the clearest case.

  **Defused for this slice as of PHASE-08, but not fixed.** Every phase is now
  implemented, so there is no unimplemented row left for the defect to produce a
  false positive on: the all-`PASS` summary the audit reads is now true of the
  tree. That is a property of the slice having finished, not of `ISS-441` being
  resolved — the issue stands for the next slice, and a mid-flight `PASS` here is
  still not evidence. What the audit should re-derive independently is the
  handful of rows whose keywords are common words; the phase records name the
  tests directly.
- **The PHASE-08 conformance span carries a foreign commit, and it cannot be
  tightened out.** The boundary is `7be8b55d2..d95f45cb8`, four commits, of which
  `e84b6d9d3` (`slice(SL-258): scope the governing-commitment vertical
  experiment`) belongs to another agent working the shared tree. It sits
  **between** this phase's first and last own commits (`1b3b90775`, `d95f45cb8`),
  and `slice record-delta` takes one contiguous range, so no tightening excludes
  it — it lands in `slice conformance`'s undeclared cell, which is where it is
  meant to be visible. PHASE-08's own delta is exactly the three `SL-238`
  commits; nothing in `e84b6d9d3` touches `src/` or this slice.
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

`QUE-221` dropped from this section at the PHASE-08 harvest — answered, and its
PHASE-08 obligation discharged at `df6185164`. The discharge is recorded on the
record itself and in `### Produced`; the durable half is
`mem.pattern.testing.pin-the-refusal-reason-not-the-refusal`.

  The one thing worth carrying: the pin asserts the refusal **message** by
  equality, and that is load-bearing rather than fastidious. Mutation-tested by
  removing both `require_item(&root, to)?` target gates — the remove leg then
  emits `ISS-001 has no after edge to SL-154`, still non-zero and still naming
  the target, so `!success` and `contains("SL-154")` **both survive the exact
  change the pin exists to catch**. A before-state pin on a refusal must
  discriminate the *reason*, because the post-change tree usually still refuses,
  just elsewhere. Generalised into
  `mem.pattern.testing.pin-the-refusal-reason-not-the-refusal`.
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

- **§5's two-reason taxonomy misclassifies a dangling *bare* ref — low severity,
  owner's call at reconcile.** `EX-5` re-classifies a `parse_resolvable_ref`
  failure by calling `parse_canonical_ref`: failing means *not a canonical ref*,
  succeeding means *no such entity*. A **bare** ref (`42`) is storable — `run_needs`
  (`backlog.rs:1955`) gates on `ensure_ref_resolves`, which accepts the bare form
  (`kinds/resolve.rs:80-96`), and `append_relationship` (`:1855`) writes the string
  **as typed** with no canonicalisation. So when such a target is later deleted,
  `parse_canonical_ref("42")` fails at `rsplit_once('-')` and the finding reads
  *not a canonical ref* where the cause was *no such entity*.

  The statement is literally true and the repair is identical either way, which is
  why this is a wording imprecision rather than a defect. **Measured 2026-08-16:
  zero bare refs in the corpus** — every stored `needs`/`after` value is canonical.
  So the taxonomy is exhaustive in practice and imprecise in principle.

  **Reconcile action (optional):** either note in §5 that a non-canonical stored
  ref reports under the first reason regardless of why it failed, or decline as
  immaterial. No `VT` row mandates covering it and PHASE-03 does not widen the
  check to chase it.

  Raised at PHASE-03 blind planning. Third defect at the same seam the earlier two
  sit on — a design's prose rule versus what the criterion enforcing it can carry.

- **§2's "same population" claim overstates what the stderr advisory can count —
  prose fix at reconcile; `plan.toml`'s `EN-2` restates the same claim.** §2 says
  *"the advisory's count and §5's check report the same population"*
  (`design.md:373-374`). They do not, on two independent counts, and neither is
  fixable without a change §4 explicitly forbids:

  1. **Terminal dependents.** `project` iterates only non-terminal items
     (`backlog.rs:742`), so a broken ref authored on a terminal item never becomes
     an `AbsentDrop`, never reaches `probe_boundary`, and cannot be counted. §2
     names this class itself two paragraphs earlier — *"Five of the 31 edges are in
     that position. Their refs are still checked by `doctor`"* (`:328-331`).
     `dep_seq_ref_findings` walks every item including terminal ones (PHASE-03
     `EX-4`), so `doctor` sees them and the advisory cannot.
  2. **Unreadable items.** `dep_seq_ref_findings` emits `read_all_tolerant`'s
     `ReadFailure`s as findings (`backlog.rs:2493-2497`); the listing path uses
     fail-fast `read_all` (`:1208`) and has no corresponding disclosure.

  What *does* hold is parity over the three broken-ref **classes** — malformed,
  absent backlog id, unresolvable cross-kind — on the live projection, which is
  what §2's fourth advisory bullet was actually arguing for and what `VT-7` and
  `VT-8` assert. **No `VT` row asserts population parity**, so no criterion is
  wrong and no code changes: §4 `:742-744` forbids the second corpus traversal
  that closing the gap would require (*"there is no second traversal of the corpus
  asking the same question a different way"*), so the divergence is a priced
  tradeoff, not an oversight.

  The claim is the defect, and it is this slice's own defect class: a surface
  stating something it had not checked. §2's argument against a partial signpost
  — *"worse than no signpost, because the reader who follows it once and finds it
  complete will trust it when it is not"* — applies to the advisory as designed.

  **Reconcile action:** narrow §2's parity sentence to the class claim, and name
  the terminal-dependent and unreadable-item gaps as accepted costs of the
  single-walk constraint. `design.md` is locked at rev 63, hence a reconcile
  action rather than an edit. `EN-2`'s parenthetical in `plan.toml` carries the
  same overstatement; left in place as an entrance criterion that is met on the
  reading that matters (PHASE-03 landed), with the correction recorded here.

  The narrowing above is accepted and unconditional. The **separable** owner
  call — whether the advisory's own wording should soften — is now `QUE-222`,
  minted at PHASE-04 harvest so it is tracked rather than buried in this entry.
  It does not gate the narrowing.

  Raised at PHASE-04 blind planning. Fourth defect at the same seam.

- **`QUE-222` — should the unresolvable-refs advisory soften its wording?**
  Whether *"N authored needs/after refs name nothing"* implies a completeness
  against `doctor` that it structurally cannot have (see `F-1` above for the two
  gaps). Answerable at reconcile; one string if accepted, zero cost if declined.
  The count is load-bearing in three tests, so accepting is not purely cosmetic.

- **PHASE-05 `EX-2` has no verification row that can observe it — owner's call at
  reconcile, no code change.** `EX-2` requires that *"`run_show_inspect` threads
  the map into `format_metadata`"*. `run_show_inspect` writes straight to
  `io::stdout()` (`backlog.rs:1879`) with no render seam, so no in-module test can
  observe the threading — it can only compose `probe_item_refs` and
  `format_inspect` itself, which is what `VT-3`/`VT-4` do. The wiring between them
  is verified by reading. §7 lists no assertion for it either.

  **No plan amendment and no code change.** `VT-1`…`VT-5` are all writable exactly
  as specified, so nothing is wrong with the criteria that exist; the gap is that
  `EX-2` sits above them with nothing pointing at it.

  This is the **same seam PHASE-02 hit**, where three `test_file` rows moved to a
  black-box golden. The difference is that PHASE-02's wording had a black-box home
  and this one does not: `tests/e2e_inspect_golden.rs` is about `doctrine inspect`,
  not `backlog inspect`. Pinning `EX-2` means a new e2e file for three lines of
  wiring.

  **Reconcile action:** either accept `EX-2` as agent-verified and record it as
  such, or add the e2e file. Recommendation is to accept, keep the Table arm's
  composition to a single expression so misthreading is hard, and have `VT-3`'s
  test state what it does not prove rather than letting the suite look stronger
  than it is.

  Raised at PHASE-05 blind planning. **Fifth defect at the same seam** — a
  criterion stated in prose that the verification set cannot carry.

  **Status after PHASE-05 execution (`42c835b0e`): both mitigations implemented,
  the owner call still open.** The Table arm passes `&probe_item_refs(&root,
  &item)` as a single inline expression, and `VT-3`'s doc comment states that it
  composes probe → renderer itself and therefore does **not** observe the wiring.
  So what remains for reconcile is only the disposition — accept `EX-2` as
  agent-verified and record it, or commission the e2e file. Nothing further is
  owed in code.

- **`kref_for` is duplicated in `catalog/scan.rs:1372` — follow-up, deliberately
  outside PHASE-04.** PHASE-04 `D-1` promotes the generic entity-seeding helpers
  (`seed_toml`, `seed_status_bearing`, `seed_status_less`, `kref_for`) out of
  `authored_status.rs`'s private `mod tests` into a `pub(crate) mod test_support`
  beside the reader whose input they author, and collapses `backlog.rs:5716`'s
  slice-hardcoded `seed_slice_entity` into it. That leaves one copy uncollapsed:
  `catalog/scan.rs`'s private `kref_for`.

  Left deliberately. PHASE-08 `EX-9` requires the `search` / `map` / `catalog`
  suites green **unmodified**, and re-pointing that helper's import is precisely
  the churn that makes "unmodified" ambiguous at audit. Cheapest resolution is to
  fold it in during PHASE-08, when that file is open for other reasons and the
  criterion can be stated to permit an import-only move — or to leave it and
  accept one duplicated four-line function. Owner's call; no correctness impact
  either way.

- **§6's `Three call sites move to the new form` is short by one — prose fix at
  reconcile.** `design.md:1187` says three, and `plan.toml` `EX-1` transcribed the
  same census (`backlog.rs:2077`, `backlog.rs:2099`, `commands/dep_seq.rs:167`).
  There are **four** production callers of `dep_seq::remove`: the fourth is
  `commands/dep_seq.rs:254`, the removal loop inside `run_after_prune`. It is not
  optional — reshaping `remove` to take `&RelRemove` breaks it at compile time, so
  it moves in PHASE-06 whether or not the design counted it. Two of the three
  quoted line numbers were also stale (PHASE-05 grew `backlog.rs`; now `:2353` and
  `:2375`), and the leaf's own two test call sites (`src/dep_seq.rs:849`, `:877`)
  move with them.

  Nothing about the design's substance changes — `run_after_prune` is rewritten
  wholesale by PHASE-07, so its site is "being rewritten anyway" exactly as the two
  backlog-side sites are. Only the count and the citations are wrong.

  **Reconcile action:** correct `design.md:1187` to four call sites, naming
  `run_after_prune`'s loop and PHASE-07 as its second rewrite. No code change.
  `plan.toml` `EX-1` is already amended in place (id kept, reasoning inline, per
  `execution-protocol.md` `R5`).

  Raised at PHASE-06 planning, blind to the type prototype (`R1`). Third instance
  of the `execution-protocol.md` §3 seam, and the second reached from PHASE-02's
  direction — not *write the assertion*, but *count the population the criterion
  quantifies over*. A prose reviewer reads "three call sites" and has no reason to
  run the grep; the compiler does it for free the moment the signature moves.

- **§6 names four `--prune` consequences; there is a FIFTH — design-text fix at
  reconcile, owner accepted 2026-08-17. `plan.toml`'s `EX-4` is already
  amended.** `authored_class(kind, AuthoredStatus::Absent)` returns `Terminal`
  (`src/priority/partition.rs:244-248`), and `Absent` is what
  `authored_status::read` returns for `STATUS_LESS`, which is `[REC]`
  (`src/authored_status.rs:55-56`). So under the collapsed probe an `after` edge
  onto a `REC` becomes **prunable**. Today it is **kept**: a `REC` toml carries no
  `status` key, so `unwrap_or("")` matches neither terminal literal
  (`src/commands/dep_seq.rs:292-293`).

  Two problems, and the second is the one that blocked the implementer: it is an
  unnamed behaviour change, and `EX-4` had no string for it — the terminal reason
  is the status word, and `Absent` has no status word.

  **Resolved by rendering the class, not the status: `dropped (dangling:
  status-less)`.** It stays honest and reuses no token that already means
  something else (`unresolved` means *the ref names nothing*, which is a
  different claim). The alternative — treat `Absent` as *keep* — was refused: it
  contradicts `status_class`'s own documented meaning (a status-less kind is
  context-only and default-excluded, which is why the table returns `Terminal`)
  and would need §3's rule restated to accommodate one caller.

  Reachability is low but not nil, and the shape matters: `REC` is not an
  admissible `after` target (`src/commands/dep_seq.rs:114-133` — work-like or
  *knowledge* record; `REC` and `RV` are both excluded), so the authoring gate
  will never create such an edge. Only a hand-authored one reaches the probe —
  which is exactly the population §6's bare-ref consequence exists to serve.

  **Reconcile actions:** §6 gains the fifth consequence; the brief's
  named-output-change list gains `dropped (dangling: status-less)`.

- **§6's `--prune` census is wrong in two ways — prose fix at reconcile.** Found
  at PHASE-07 planning by re-deriving the count before planning against it
  (`mem.pattern.planning.let-the-compiler-recount-the-call-sites`), the second
  time that habit has paid inside this slice.

  1. **The count conflates two populations.** §6 opens *"Four hardcoded copies of
     `status == "resolved" || status == "closed"` — two functions … each of which
     reads and parses the target twice"*. There are **two** such literal
     comparisons, not four (`commands/dep_seq.rs:293`, `backlog.rs:2301`): the
     *describe* pass re-reads the target but renders `status`/`resolution` rather
     than re-testing terminality. What there are four of is **read-parse blocks**,
     which is exactly what §6's own next paragraph says, correctly. So the
     substance is right and only the opening sentence's label is wrong — but it is
     the sentence `plan.toml`'s PHASE-07 objective copied verbatim, and §7's
     agent-verified bullet repeats it as *"with the four known sites named"*. A
     planner hunting four terminal literals finds two and has to decide whether it
     has a search bug. Both `plan.toml` halves amended in place at PHASE-07
     planning; the two design-text instances are the reconcile action.
  2. **All four line numbers are stale**, the same drift PHASE-06 found in `EX-1`.
     `backlog.rs:2019-2022`/`:2043-2046` are now `:2295`/`:2319`;
     `commands/dep_seq.rs:202-206`/`:222-226` are now `:287`/`:307`.

  Also amended: `EX-3` demanded all four laundered reads be gone, which PHASE-07
  cannot do — `backlog.rs`'s two die with the duplicate prune leg that PHASE-08
  `EX-3` owns, and that leg cannot go before PHASE-08's injection gives `backlog`
  a route to the shared operation. Scoped to this phase's two; the slice-wide
  claim survives as `VA-1`'s grep at PHASE-08, where it is actually checkable.

- **PHASE-07 must decide `run_after_prune`'s echo prefix deliberately.**

  > **RETRACTED IN PART, 2026-08-17 (PHASE-07 oracle pass) — the prototype does
  > NOT change the echo.** This entry was headed *"the prototype changes it
  > silently"* and claimed *"the fork's `run_after_prune` echoes `{source_id}`
  > (canonical) on all three of its output lines"*. That is false. The fork's
  > `run_after_prune` echoes `{source}` **as typed**, at
  > `proto/SL-238-types:src/commands/dep_seq.rs:247` and `:261` — byte-identical
  > to today. The `{source_id}` echoes I saw are on the *append* and *remove*
  > legs (`:125`, `:152`, `:169`, `:173`), which canonicalise today as well.
  >
  > I attributed a grep hit in the file to the function I was thinking about,
  > without confirming the quote with `Read` before it entered a finding. That is
  > precisely what `execution-protocol.md` §5 says not to do, and §5's own
  > correction has the diagnosis: **grep is deterministic; the agent quoting it is
  > not.** The fork was never the evidence for anything here.
  >
  > **The decision below is unaffected** — it rests on PHASE-08 `EX-3`/`EX-6`,
  > which are plan authority. Only the *reason to look* was spurious.

  PHASE-02 `VT-2` pinned the divergence as one to preserve: the backlog copy
  echoes the canonical id, the top-level copy echoes as typed.

  It bears on PHASE-06 only as a hazard: `T1` must touch `run_after_prune`'s
  `dep_seq::remove` call at `:254` because the signature reshape forces it, and
  the fork shows how easily an echo change rides along. PHASE-06's `R2` holds the
  line — reshape the argument, change nothing else.

  **DECIDED at PHASE-07 planning, 2026-08-17 — unify on the canonical source id,
  and the decision is FORCED rather than tasteful.** The reasoning above contains
  an error worth naming, because it is the one that made this look open: it says
  "PHASE-08 `EX-6` covers only the routed `backlog after` legs", treating that as
  disjoint from `run_after_prune`. It is not. PHASE-08 `EX-3` makes **all three**
  `run_after` legs — append, `--remove`, `--prune` — run the injected operation,
  and the injected prune operation *is* `run_after_prune`. So after PHASE-08 its
  echo **is** the routed leg's echo, and `EX-6` ("echo strings on the routed
  `backlog after` legs unify on the canonical source id") cannot hold unless
  `run_after_prune` echoes canonical.

  Preserving as-typed is therefore not the conservative option — it would silently
  **regress** `backlog after --prune`'s existing canonical echo to as-typed at
  PHASE-08, an output change in the wrong direction that no criterion names.

  PHASE-07 makes the change, not PHASE-08: it rewrites both `writeln!` sites
  anyway under `EX-4`, and `EX-5` already obliges it to supersede PHASE-02's prune
  pins — so both output changes land in one phase, with one supersession to read.
  The prototype reached the same shape; it simply never showed its working, which
  is `§4`'s point about reading the fork for shapes and never for authority.

  **Reconcile action:** the brief's named-output-change list must carry
  *`after --prune`'s source echo becomes canonical* explicitly. §7 line 1732 names
  "the canonical-id echo on the routed `backlog after` legs", which covers the leg
  after routing but not the top-level verb's own change at PHASE-07.

  Third prototype defect logged forward from a `DEC-242` oracle pass; the two from
  PHASE-02 (`phase-02.md ## Findings F-3` — the `eprintln!` against
  `print_stderr = "deny"`, and the `Terminal` branch minting reasons for states
  `authored_class` may make unreachable) were both re-confirmed still present.

- **A shipped memory recommends the module this design withdrew — corpus
  correction, not a reconcile action.** `mem.fact.layering.gate-measures-top-level-modules`
  (trust `high`) closes its *"the repair that works"* section with: *"`SL-238` did
  this twice — `src/authored_status.rs` for the per-kind status read,
  `src/dep_seq_ops.rs` for the kind-neutral dep/seq operations."*

  The first half landed. The second names a module that does not exist
  (`ls src/dep_seq_ops.rs` → no such file) and that `design.md:1345-1364` records
  as the alternative which **lost**: an engine-tier module calling command-tier
  `partition::authored_class` is an *upward* edge, worse than the cycle it was
  introduced to remove, and no sub-classification row launders it. The design's
  answer for this seam is fn-pointer injection.

  So the memory offers a rejected repair, for this exact seam, at high trust, to
  any future agent planning a layering fix — and it does so inside the section a
  reader consults precisely when they have stopped reading the design. Corrected
  at PHASE-08 harvest: drop the `dep_seq_ops` clause, keep `authored_status`, and
  name the injection idiom (`mem.pattern.lint.back-edge-tangle-inject-fnptr`) as
  what SL-238 did on the second seam.

  Raised at PHASE-08 blind planning. The seam is a familiar one — a record written
  mid-slice against a decision that later reversed — but this is the first instance
  in the *memory corpus* rather than in `design.md` or `plan.toml`, and the corpus
  has no reconcile pass to catch it.

- **`notes.md`'s `kref_for` entry cites a criterion PHASE-08 does not have.** The
  `catalog/scan.rs:1372` follow-up above says *"PHASE-08 `EX-9` requires the
  `search` / `map` / `catalog` suites green **unmodified**"*. PHASE-08 carries
  `EX-1`…`EX-6`. `EX-9` is **PHASE-01**'s (`plan.toml:56`), and it is discharged.

  The disposition does not change — leave the duplication — but the reason does,
  and the difference matters to whoever picks it up. It is not "a criterion forbids
  the churn until PHASE-08"; it is that PHASE-08 opens `src/backlog.rs`,
  `src/commands/dep_seq.rs` and `src/commands/cli.rs` and nothing else, so the
  entry's *"when that file is open for other reasons"* never comes true in this
  slice. The `kref_for` collapse wants its own backlog item, not a phase.

  Raised at PHASE-08 blind planning. Fourth stale cross-reference in this slice's
  records (after `EX-1`'s call-site census, `EX-3`'s four line numbers, and the
  `--prune` echo attribution) — the class `execution-protocol.md` §5 is about, and
  the cheapest falsifier remains the same: resolve the id before repeating it.
