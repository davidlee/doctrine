# Notes SL-266: Inquiry map tree view

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Design triage (exploring, 2026-09-25)

Evidence: `research/research.md` (runtime, gitignored) — cited below by its
X-/F- labels.

- **Reference, not spec.** The tree mockup is a screenshot of *hydra* (a
  friend's MIT Rust tool built from the user's original idea). It sets the feel;
  its vocabulary (e.g. "cauterised") is not ours.
- **Constraining governance.** SPEC-029 REQ-433 (one envelope, every rendering
  a projection of it) → the tree is an envelope rendering, so the envelope
  gains an opt-in whole-map field (X1). PRD-019 REQ-417 (blocked is derived),
  REQ-425 (never imply the map is complete), success measure "without asking
  the agent to summarise" (X5). STD-001 named constants; STD-003 disclose the
  auto-chosen run.
- **Shaping decisions (proposed).** `--format tree` + `design tree` alias;
  relay obligation rides the envelope, gated on config + derived map change,
  not the digest-bound `inquiry.md` fragment (X2).
- **Open questions.** Envelope field shape; flag partition for tree; default
  run resolution (X4); `map_view` name/default; what counts as a map change;
  glyphs/columns/width; SL-264 `blocking` marker (X6).
- **Assumptions.** The per-turn envelope is not digest-bound the way fragments
  are (inferred — verify in design).
- **Risks.** SL-264 (ready, uncommitted in the primary tree) edits the same
  structs — implementation waits for it. Prune/defer carry no reason (F4);
  out of scope.

## Further review (2026-09-26, after RV-388 concluded)

No further pass needed. RV-388 ran three raiser rounds (codex): 20 findings,
all verified, round 3 raised nothing new. The round-2/3 edits (wrapping,
relay scope, run-selection wording, lazy `[design]` config) were each
re-verified by the raiser. What remains unproven is implementation-level
(wrap goldens, config refusals) and is pinned by VT-7, VT-10 and VT-12, which
the implementation review and audit will exercise.

## PHASE-02 — tree renderer (2026-09-26)

- **VH-1 amended the wrapping rule** (user, recorded on DEC-307): dropped text
  keeps its rails, one level in; the bare 8-column indent is the fallback only
  below `TREE_MIN_DROP_COLS` (16) beside the rails. Legend marks and letters
  are coloured like the nodes. `design.md` sec-3 rule 2 still states the
  locked (rail-less) form — reconcile at `/reconcile`.
- `MapAnswer::Record.form` stays on the model: the locked anatomy prints
  `REC-NNN <title>` for both forms, and `form` is carried by `json --full`
  serialisation — a live reader, so no model change.
- **Gap closed: the envelope had no canonical slice ref** for the header/footer
  (the leaf cannot format ids). User chose option 1: `TurnEnvelope.slice_ref`,
  filled from `project()`'s shell-formatted argument (DEC-292), serialised — an
  additive json key, version unchanged (DEC-291). `tests/e2e_design_show_golden.rs`
  gains the key (outside the declared affected surface; shows as undeclared).
- VH-1 verdict: "legible", after three amendments (rails on dropped text; legend
  marks/letters coloured; cursor suffix open-cyan bold, `pinned` magenta) —
  all recorded on DEC-307.
- Boundary tightened to `a51c614bb^..c47451f1a`; SL-267/SL-264 commits
  interleave it and ride in conformance's undeclared cell.
- Header counts partition the map: open excludes derived-blocked.

## PHASE-03 — command surface and run resolution (2026-09-26)

- Titles are read in `project()` whenever `detail == Full`, not only on the tree
  path — so `prompt --full` / `json --full` stop reporting every cited record
  as "record not found". `Normal` still reads nothing (sec-2: "filled at `Full`
  only", sited once).
- **Phase decision (not stated in sec-4):** a slice record the scan cannot read
  → the snapshot is *skipped* with that cause, not a candidate — an
  unverifiable status cannot be claimed "open when scanned" (STD-003). A state
  dir that is not a number, or holds no `design.toml`, is not a run → ignored.
- `TREE_COMMAND` widened to `pub(crate)` now: the no-open-run refusal names it.
  PHASE-04's relay line reuses the same constant.
- `design tree` is classified `Read` in `commands/guard.rs` (worker mode lets it
  through), like `show`.
- Help: `design tree --help` shows only the first paragraph of its doc (the
  About block keeps one paragraph); the optional-slice difference is carried by
  the `[SLICE]` arg help instead.
- `select_run` was written before its tests (test-after, not red-first); a
  mutation (locked filter → `Reviewing`) turned `select_run_picks_the_newest_open_run`
  red, so the test bites.
- **VA-1:** `design --help` lists `tree`; `design show --help` lists the `tree`
  value — both asserted in `tests/e2e_subcommand_help.rs`. No reference doc or
  skill enumerates `design` verbs or `--format` values (grep of `install/`,
  `.agents/skills/`, `plugins/`): the mentions are narrative (`routing-process.md`
  core process, `handover`'s `--format status`), so nothing else to change.
- EX-3: no new spelling of the state path. Pre-existing ones (`kinds/mod.rs`
  `state_dir`, test code) are IMP-349's, out of scope.
- Real-corpus smoke: `design tree` picked SL-258 of 3 open runs with nothing
  skipped; `design tree SL-266` is byte-identical to `show --format tree`.
- Per-phase review RV-392 was mandatory (tripwire: edits outside the declared
  selectors, `knowledge.rs` / `slice.rs` / `guard.rs`). It found F-1..F-3, all
  fix-now in `6d9bf2db0` and all verified; the ledger is done. A snapshot filed
  under the wrong slice directory is now skipped; alias dirs are ignored;
  titles go through `read_record`.

## PHASE-04 — delivery: config, map_changed, relay line, prompt text (2026-09-26)

- Config home: `src/design_run/config.rs` (the plan's VT-3 file), not a
  crate-level `*_config.rs` — the resolver is pure and design-run-only.
  `DoctrineToml.design` is `Option<toml::Value>`, raw.
- The config is resolved inside the shell's `apply()` / `start()`, first thing,
  not in `run_*` — so no parameter churn across ~30 inner test callers, and a
  malformed entry refuses before the snapshot is even read.
- Relay text is owned by `tree.rs` (`relay_line`), which shares one private
  `command(slice_ref)` with the footer, so `TREE_COMMAND` + slice is spelt once.
  The gating rule is `design_run::relay(delivery, prior, next, slice_ref)`, called
  by both verbs. `start` compares against the pre-import run.
- `slice_ref(slice)` helper in `design.rs` replaced three `canonical_id` spellings.
- VT-1 (`map_changed_table`) drives each case through `run::apply`. Each `false`
  case also asserts its write *landed* elsewhere (runbook, cursor, acts), so a
  no-op cannot pass as "not a map change". Step discharge needs a
  `derived.gate.runbook` — `one_step_runbook` fixture.
- VT-2 e2e: `doctrine.toml` is `.doctrine/doctrine.toml` (`common::DOCTRINE_TOML`),
  not the tree root. The first run of the sidecar/malformed tests wrote it to the
  root and they went red — which showed the assertions bite. A written adopt
  needs a materialised (marker-bearing) document first. The `[conduct]` control
  is `slice status 233 design` (`load_conduct`).
- Prompt text: SL-264's sentences in the condition are left as they were; only
  the two listing sentences changed. VA-1's "no other viewer" check has to
  ignore line breaks, because the phrase spans one in HEAD (a plain grep missed
  it: 0 in HEAD, a vacuous negative).
- Gate: every leg green except `cordage` `scale_cliffs`
  `evaluate_scales_near_linearly_in_node_count`. That is ISS-331's wall-clock
  false-red under load: it failed in both gate runs and passes on its own.
  `cargo test -p doctrine` is green.
- TDD note: `config.rs` was written together with its tests (test-with, not
  red-first). `map_changed` went red first through a `todo!()`.
- **VA-1 (attested):** `inquiry.md` *Craft* carries "The user's view of the map
  is `doctrine design tree`. When a write tells you to show it, paste that
  output; never substitute a listing of your own." `initial-concerns-recorded.md`
  *What is enough* carries the sec-5 replacement verbatim. With line breaks
  ignored, "no other viewer" appears 0× in both files (1× in HEAD before the
  edit). Neither file orders a paste unconditionally: the only paste is
  conditioned on "When a write tells you to show it".
- Per-phase review: discretionary, and skipped. No tripwire fired — no test
  deleted, nothing waived, every edit inside the sec-6 selectors plus the VT
  files. `/audit`'s pre-close review covers this delta next.
- Boundary: `slice record-delta 266 PHASE-04 --start 94fb3870b^ --end 94fb3870b`.

## Audit (2026-09-26)

- RV-393 (pre-close code review, Opus sub-agent): F-1..F-5 verified, concluded
  (`fd3c53453`, `doc` follow-up). F-4, the user's choice: the design writes read
  only the raw `[design]` entry (`dtoml::load_design_entry`), and
  `DoctrineToml.design` is gone. This supersedes PHASE-04's "`DoctrineToml.design`
  is raw". F-3's typed-field half went to IMP-486.
- RV-394 (conformance audit): 6 findings, all verified, concluded. F-1..F-5 are
  canon drift for `/reconcile` (see its Reconciliation Brief). F-6 is
  tolerated: the per-phase `[start,end]` ranges for PHASE-02/03 span SL-267 and
  SL-264 commits, so 25 of 33 undeclared paths are foreign (friction observation
  recorded).
- Evidence: `doctrine check gate` exit 0 (8877/0; ISS-331 did not fire);
  `slice verify-vt 266` 14/14.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-09-26 · audit done (RV-393, RV-394 concluded); gate green; → reconcile

### Produced

- DEC-303..DEC-310 (design decisions); IMP-472, IMP-473; RV-388 (design ledger)
- SL-266 PHASE-01, landed as `3e55c758b` (merge of `3ba03030e` — `unsettled_needs`
  + the `blockers()` sweep — and `3e745780f` — the `Full`-only whole-map envelope
  field with `project`'s `titles`/`selection`)
- SL-266 PHASE-02 on edge: `a51c614bb`, `d4c6f8ba9`, `78d8e0b44`, `c47451f1a`
  (renderer; VH-1 amendments; `TurnEnvelope.slice_ref`); DEC-307 amended
  (`4362c2516`, `cf02f2735`)
- SL-266 PHASE-03 on edge: `3cdceba18` (verb, format, scan, `select_run`,
  titles at `Full`), `6d9bf2db0` (RV-392 fixes); boundary `3cdceba18^..6d9bf2db0`
- RV-392 (per-phase code review, done, F-1..F-3 fix-now)
- SL-266 PHASE-04 on edge: `94fb3870b` (`design_run::config`, `map_changed`,
  `relay`, relay line in `apply` / `start --from-design`, prompt text)
- RV-393 (pre-close code review, F-1..F-5; fixes `fd3c53453`); IMP-486
- RV-394 (audit ledger + Reconciliation Brief)

### Learned

- Observations (`records/`): 03, 94, 97, c6, 71, db (design-run friction);
  56 (`MapAnswer::Record.form` unread by either rendering), 58 (the fork guard
  vs a concurrent-agent tree), af (`land` refuses `tree-unclean`)
- `mem.pattern.worktree.solo-land-refuses-unclean-shared-tree`
- `blockers()` held a *second* blocked derivation that no `is_blocked` grep
  could reach — sweep the shape, not the callee
  (`mem.pattern.review.sweep-defect-class-not-instance`)
- A solo phase's boundary is never auto-recorded: `land`'s merge commit is
  rejected as `code_end` (non-merge required), so `slice record-delta` is the
  *normal* route for solo, not an escape hatch.
- `mem.pattern.render.textwrap-no-hyphenation-for-ids`; observation `c1`
  (envelope lacked a canonical slice ref)
- `mem.pattern.design-run.leaf-rule-binds-tests` bit: no `crate::` in
  `design_run` tests either (strip-ansi spelled locally in `tree.rs`)
- `mem.pattern.state.dir-scan-canonical-name-and-embedded-id` (RV-392 F-1/F-2)

### Open

- ~~`MapAnswer::Record.form` rendered by neither surface~~ — settled at
  PHASE-02: `json --full` carries it; no model change.
- `/reconcile`: execute RV-394's Reconciliation Brief (F-1..F-5: design sec-3,
  sec-4, sec-5, sec-6 prose + selector add/rm).
- ~~PHASE-04's relay line must reuse `tree::TREE_COMMAND`~~ — done: `relay_line`
  and the footer share one `command()`.
- ~~`src/design_run/tests.rs` reads `UNATTRIBUTABLE`~~ — PHASE-04 modified it.
- ISS-331 (`cordage` `scale_cliffs` wall-clock false-red) reds `just gate` under
  load on this host — twice in PHASE-04. Nothing to do with SL-266, but it will
  red the close gate too. It passes when run on its own.
