# Notes SL-231: Friction observation ledger and capture interface

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-28 · all five phases completed; verify-vt green on
PHASE-01/02/03/05, PHASE-04 pending its derived registry row · b5428c47

### Produced

- PHASE-05 — distribution and dogfood activation, run **inline in the
  coordination tree by the orchestrator**, not dispatched (D1). Boundary row
  `[ce148fb08, b5428c47]`. Four commits: `a82089b5` (the ignore rule + VT-1),
  `b6ce22c7` (VT-2 e2e), `e89216e0` (the three prose surfaces),
  `b5428c47` (the withhold-tier fix). All five phases now `completed`;
  `slice verify-vt 231` exits 0 with PHASE-05 VT-1/2/3 PASS.
- One distributed ignore rule — `.doctrine/observations/**/.tmp.*` — appended to
  `install/manifest.toml` and this repo's `.gitignore`, riding the existing
  `ensure_gitignored` leg. No new install machinery (D2). Records stay authored:
  committed, diffable, visible in review.
- `Tier::ObservationTemp` — a new withhold tier the `.gitignore`/WITHHELD parity
  gate forced; see Learned.
- Capability-aware capture activated on three surfaces telling one story:
  `install/using-doctrine.md` (client-facing: verbs, authored durability,
  review-noise cost, repo-wide vs local exclusion, the correlation/audit
  forfeit), `.doctrine/governance.md`, and `.doctrine/rfc/011/rfc-011.md`.
  `case-notes.md` retained in full and still cited as the historical evidence
  base — closed to new entries, not migrated (EN-2).
- IMP-331 — the `/dispatch-agent` skill's `arm-spawn` template omits the
  mandatory `--phase` half.

- PHASE-04 — confined-worker MCP capture delivered on the **claude/Opus** arm
  (first phase of this slice not on pi/deepseek), driven through the funnel:
  worker self-commit `b3f28288a` via the gated `worker_commit`, imported
  (`51472f5e1`), verified green on the `gate` cadence (`da159611e`), concluded
  (`413d762ab`), fork reaped (`ce148fb08`). Boundary row
  `[d598dd4c1, da159611e]`. `check regression diff --base d598dd4c1` clean.
- Plan amendment (`5e152bdf2`, `d598dd4c1`) — the two `verify-vt` rows the
  RV-317 F-4 remediation broke, settled per the `/plan` narrow-amendment route:
  PHASE-02 VT-3 re-pointed onto live keywords; PHASE-03 VT-3 **split** into
  VT-3 + a new VT-4 (forced by the one-`test_file`-per-VT mandate once the
  criterion's two halves came to live in different files). Ids appended, never
  renumbered. All 14 landed-phase VT criteria now PASS.
- Two e2e cases written to close PHASE-03 VT-4's real gap rather than narrow the
  criterion to what already passed (user's call):
  `dispatched_marked_fork_refusal_points_to_observation_record_broker` and the
  negative `leaked_env_on_nonlinked_tree_refuses_without_broker_advice`; the
  pre-existing case renamed to `solo_marked_fork_…` for the shape it actually
  asserts. Both proven load-bearing by two independent, mutually discriminating
  mutations. `tests/e2e_observation.rs` refactored onto one `spawn` helper.

- RV-317 — ledgered code review of the landed PHASE-01..03 delta (facet
  `code-review`, three lens-diverse read-only deepseek passes + orchestrator
  probing). 11 findings: 1 blocker / 2 major / 7 minor / 1 nit. IMP-329 minted
  from F-5. **All eleven now verified; RV-317 is done**, so F-1 no longer gates
  closure.
- RV-317 remediation — two orchestrator-directed deepseek turns on forks off
  `dispatch/231`, each adjudicated against the built binary before commit:
  `23c4309ed` (turn 1: F-1/F-2/F-9/F-10/F-11 — the hostile-input and
  byte-safety batch) and `d4a042e39` (turn 2: F-3/F-4/F-6/F-7/F-8 plus F-5's
  interim guard). Both are standalone commits OUTSIDE any phase boundary row —
  `record-delta --commit S` pins one commit's patch, so PHASE-04's base must be
  set to `d4a042e39`.
- ISS-267 — `e2e_backlog_filter_alias` fails in any `--worker` fork (strips the
  env leg, not the marker leg). Found running the full suite in turn 2,
  confirmed pre-existing at the base commit. ISS-260's class, unswept.

- PHASE-03 — CLI, reads, and corrections delivered on the pi/deepseek arm, then
  driven through the funnel: imported (`6a07967c2`), verified green
  (`ef7c9b455`), concluded (`0fe9572b`), fork reaped. Boundary row
  `[4163c554b, 6a07967c2]`. Conformance 11/11 declared source paths.
- PHASE-03 needed THREE orchestrator cleanup turns — more than P01/P02
  combined. Each finding was concrete and gate-invisible:
  1. `escape_hostile` iterated bytes and passed them through `char::from(u8)`,
     a Latin-1 mapping, so every multi-byte UTF-8 char was corrupted
     ("é" → "Ã©"). The suite was green only because every test string was pure
     ASCII. Now char-wise with C1 handling, pinned by
     `non_ascii_content_survives_rendering_intact`.
  2. The adapter hand-rolled `filter_and_order`, duplicating `query::query`,
     which already supports `Projection::History`. The tell: the worker
     annotated `Projection::History` as `expect(dead_code, "PHASE-04")` while
     hand-implementing history mode. Comparators were byte-identical, so the
     defect was latent drift, not a live bug. `filter_and_order` deleted; both
     paths route through the service.
  3. EX-5 was not fully discharged: an unescaped newline in a comfy-table cell
     let crafted content render as an apparent extra row. Fixed with ONE
     escaper taking an `EscapeContext` (Inline for cells, Block for detail) —
     not a second escaper, which would have repeated defect 2.
- Finding 3 came from asking the worker for its READ on an ambiguous point
  rather than mandating a fix. It identified the attack correctly but proposed
  an unworkable remedy (escape newlines always, re-apply formatting after —
  impossible once content newlines are indistinguishable from layout ones).
  Worth repeating the technique; not worth accepting the answer unexamined.

- PHASE-02 — no-clobber publication and store delivered on the pi/deepseek arm,
  then driven through the funnel: imported (`70f131d43`), verified green
  (`c699fd99b`), concluded (`7e6f7c0b`), fork reaped. Boundary row
  `[da66aa111, 70f131d43]`. Conformance 4/4 declared source paths; the single
  `--strict` failure is the ISS-264 machinery false positive.
- PHASE-02's fork was minted FUNNEL-BOUND up front
  (`worktree fork --slice 231 --phase PHASE-02 --worker` into
  `<coord>/.worktrees/SL-231-p02`) and the worker attached via `PI_REUSE_FORK=1`,
  so IMP-328 cost nothing this phase — no re-fork, no cherry-pick, and the
  import resolved first try.
- Orchestrator-directed cleanup turn on the PHASE-02 fork BEFORE the delta
  commit, fixing two defects a green gate cannot catch: (a) STD-001 — the
  reserved publication-temp prefix was a literal in `fsutil` and an independent
  private const in `store`, a silent corpus-corruption path if either drifted;
  now one `fsutil::PUBLICATION_TEMP_PREFIX` with a test crossing both code
  paths. (b) parallel implementation — `ensure_dir_components` was a near-verbatim
  copy of the extracted `ensure_parent_dirs`; both now route through one
  `create_dir_component` helper whose bool return preserves entity.rs's
  rollback contract. Fixed pre-commit deliberately: `record-delta --commit S`
  pins ONE commit's patch, so a post-import fix would fall outside the phase's
  boundary row.

- PHASE-01 — typed observation core delivered on the pi/deepseek arm and driven
  through the full funnel: imported (`1d8cc08ae`), verified green on the `gate`
  cadence (`02da8ebf4`), concluded (`addc6d178`), fork reaped. Boundary row
  `[0d2cb5671, 1d8cc08ae]` recorded.
- PHASE-01's green is now INDEPENDENTLY established, discharging the delivery
  caveat: the unmarked coord-tree gate passed, and `check regression diff`
  against the B baseline showed no new or changed failures. The single baseline
  failure at B — `architecture_layering_gate` raising `StaleEntry("observation")`
  because the ADR-001 leaf entry was pre-seeded ahead of the module — flipped to
  `fixed` by the import, which is the mechanical proof that EX-5's leaf
  classification is live.
- ISS-263, ISS-264 — two dispatch-funnel defects found while landing PHASE-01
  (opaque import fault on a bad fork name; conformance `--strict` false positive
  on the import-landed funnel row).
- ISS-260 — ADR golden worker-marker skip read only the env leg; fixed on edge
  (9cd0e7706) and merged into `dispatch/231`
- IMP-328 — pi spawn fork is unbound to slice/phase, so a pi-arm fork is
  unresolvable by the funnel import resolver
- `scripts/pi-spawn-confined.sh` — PI_REUSE_FORK / PI_THINKING / PI_TOOLS env
  overrides (e2dbf0198); parity scraper re-verified
- dispatch coordination for SL-231 established; ADR-001 `observation = "leaf"`
  pre-seeded on the coord branch (095fca404) because `.doctrine/` is a worker
  forbidden zone
- SL-231 — five-phase executable plan authored, critically strengthened,
  materialised, reviewed, and advanced to ready (commits aee493b2..2baca05d)
- post-ready plan review — removed brittle line anchors and broad selectors,
  aligned scope exclusions, and carried REQ-412 purity verification through
  store embedding and final-state verification
- PHASE-01 through PHASE-05 — runtime sheets materialised under
  `.doctrine/state/slice/231/phases/`
- IMP-322 — make Pi research runners tolerate read-only session homes
- pre-design research re-baselined after orchestrator fallback; both mandated
  Pi producers failed before repository inspection on read-only `/home/david/.pi`
- quick check passed with repository-pre-existing warnings; full gate not run
  for this governance-only planning unit

### Learned

- **Adding a `.doctrine/` line to `.gitignore` is never a one-file change.**
  `every_runtime_gitignore_glob_is_classified` reds on any new runtime-tier glob
  until it declares whether a worker fork is denied it or regenerates it
  (`WITHHELD` / `DERIVED_RUNTIME`). Invisible from the design and from the §7
  touch-set; only the gate surfaces it. PHASE-05's rule needed a new
  `Tier::ObservationTemp`, since a leftover publication temporary is neither
  fork-regenerated nor coordination scratch.
- **A regression baseline captured after the delta is worse than none.**
  `check regression capture --base <B>` runs the suite on the CURRENT tree and
  labels it `B` — it does not check `B` out. Capturing at the phase tip recorded
  a self-inflicted failure as pre-existing, and the diff then reported it
  `persistent (pre-existing) — fix the trunk`, which reads as an inherited
  defect and invites waving it through. The funnel captures pre-spawn for
  exactly this reason. When a "persistent" failure's name is anywhere near the
  change, run the test and read the panic rather than trusting the partition.
- **UNATTRIBUTABLE VT rows on the claude arm mean a missing registry row, not a
  missing test.** `slice conformance` names it directly ("completed phase
  PHASE-NN has no recorded source-delta row"). The claude arm derives the
  registry at `dispatch sync --prepare-review` from the committed boundaries
  ledger and gates completeness there, so the rows resolve at the slice-level
  conclude cadence. Check `conformance` before chasing the test.
- **A phase has two memory surfaces — the CONTENT it changes and the MECHANICS
  that drive it — and `/retrieve-memory` scoped to only the first is a silent
  half-probe.** `mem_019f9effcf4a7922b31c1a1b37841d06` documents the half-arm
  trap below verbatim, including a "why an orchestrator walks into it" section;
  it was missed because the phase-plan retrieval was scoped to PHASE-04's files,
  and that memory is tagged on the *command* surface. Cost a full worker turn.
- `dispatch arm-spawn` needs BOTH `--slice` and `--phase`: a half-arm binds
  nothing, and `worker_commit` refuses `unprovable-fork` **at hand-back** — the
  failure is end-loaded, so the whole worker turn is spent before it surfaces.
  The `/dispatch-agent` skill's own template still reads
  `arm-spawn --base <B> [--slice <N>]`, which produces exactly this (IMP-331).
- The corpus sanctions two recoveries from an unbound fork (re-arm + re-spawn,
  or fallback-A live-worktree import) and says there is no re-bind verb. A third
  was taken here — hand-repairing `slice`/`phase` into the coord-tree
  `DispatchRecord`. It worked and preserved the gated-commit path, but it
  defeats the fork-time-snapshot property the binding exists to provide.
  **Recorded as a disclosed deviation, not a practice.**
- **A handover's recommendation is a hypothesis, not a finding — re-derive it
  against the code before acting.** This packet's PHASE-03 VT-3 recommendation
  ("add a solo non-linked marked-fork case") describes a test that cannot pass:
  `marker.rs` computes `marker_leg = is_linked && marker_present`, so a marker
  on a non-linked tree is inert, and design §3.4 has both solo and dispatched
  forks LINKED — the *env* leg is what separates them. The packet had also
  mislabelled its own landed test's shape. Following it would have produced a
  vacuous test, which is the very defect RV-317 F-4 was raised about.
- A worker prompt can prescribe something the architecture forbids. This one
  told PHASE-04's worker to reuse `escape_hostile` from `src/commands/…`;
  `mcp_server` and `commands` are both command-tier and SL-203 deliberately
  severed that back edge, so the import would have re-formed the SCC and red-ed
  `architecture_layering` — which the same prompt forbade retuning. The worker
  found the only consistent path (move the shared items down into the
  `observation` leaf) and reported it as a deviation. **Check a prescribed reuse
  against ADR-001 direction before writing it into a prompt.**
- The pre-spawn base-clean beat is `doctrine check prove`, and clippy + a green
  suite is NOT a substitute — `prove`'s fmt-check leg is the one that caught an
  unformatted call in `5e152bdf2`. A RED base is a BASE defect: fixed
  operator-side in its own commit (`d598dd4c1`), never folded into a worker
  delta.
- PHASE-01's EX-5 is not worker-satisfiable: `architecture_layering_gate` raises
  `Unclassified` for any `src/` unit absent from `.doctrine/adr/001/layering.toml`,
  which workers may not write. Orchestrator must pre-seed the classification onto
  the coord branch before forking. Same shape applies to PHASE-05's
  `.doctrine/governance.md` + `rfc-011.md` (orchestrator-authored).
- `warnings = "deny"` + `unused = "deny"` + `allow_attributes = "deny"` make a
  consumer-less new module uncompilable; the one sanctioned escape is a single
  module-level `#![expect(dead_code, reason = …)]`.
- DEC-044, DEC-045, DEC-046, DEC-047, DEC-048, DEC-049, DEC-050, DEC-051,
  DEC-052 — UUID identity, correction, publication, capture, query, enrichment,
  safety, and authored-storage contracts
- EVD-002 — `claude -p` is the first candidate for trustworthy token telemetry
- RV-311/F-1 — marked solo worktrees defer friction for coordination-tree
  capture
- `mem.pattern.review.sweep-defect-class-not-instance` — all three PHASE-03
  cleanup fixes were correct and all three classes had surviving siblings
  (RV-317 F-1/F-2/F-6/F-7). Sweep the delta for the class before closing a
  cleanup turn.
- `mem.fact.dispatch.deepseek-review-capability` (extended) — the passes found
  real defects incl. the blocker; two calibration failures worth planning around
  are diff-relative line numbers and a "CLEAN" section covering a live panic.
  Reviewer passes and empirical probing of the built binary are complementary.
- **Remediation turns confirm the same asymmetry: strong at the work, unreliable
  at judging it.** Both turns self-reported `DEVIATIONS: NONE / UNCERTAIN: NONE`
  with clippy clean and a green suite; both had exactly one finding that had not
  actually landed. Turn 1: F-1 half-closed (single-line header fields escaped
  with the multi-line context, so the injection still reproduced through a
  different view than the test asserted). Turn 2: the F-5 guard was decorative
  (`..Default::default()` in the fixture meant BOTH walks skipped a new field and
  the counts still matched) and the F-4 replacement mutated process-global CWD.
  Neither was catchable by any gate. The catch in both cases cost one command:
  **re-run the finding's own reproduction against the built binary.** That should
  be a named step in the remediation loop, not orchestrator discretion.
- Prompt precision transfers directly. Turn 1's one real miss traces to the
  instruction "Block for the multi-line detail view" — a *locus* rule where the
  contract is a *destination-shape* rule. The worker followed what was written,
  exactly. Naming the concept in code (`escape_metadata`) closed it better than
  restating the enum would have.
- A guard that cannot fail is worse than no guard. Prove a new regression test
  actually fails on the mistake it targets — for the facet cardinality guard that
  meant injecting a one-sided field addition and watching it stay green, then
  re-proving both failure modes after the fix.

### Open

- **The conclude cadence is the next stage, and trunk has moved 63 commits past
  the fork-point.** `dispatch status` prescribes
  `dispatch refresh-base --slice 231` before `prepare-review`. Worth doing while
  the two known authored divergences are still in context (see below) — each
  conflict is then one phase's delta rather than a pile at candidate-create.
- **Two authored files have forked coord↔edge, both with edge ahead.**
  `notes.md` (edge canonical — the coord copy is the 40-line fork-time version)
  and `.doctrine/governance.md` (edge carries a research-section addition the
  coord copy lacks; different hunk from PHASE-05's, so it should merge clean).
  The general rule is edit-the-canonical-copy
  (`mem_019f738d55707ed1a1204bd6288bf7db`), but note this slice inverts that
  memory's stated default: here the PRIMARY is canonical, not the coord tree.
- PHASE-04's three VT rows read UNATTRIBUTABLE because PHASE-04 has no
  conformance registry row yet — confirmed via `slice conformance`, not
  inferred. The claude arm derives that row at
  `dispatch sync --prepare-review`, which also gates completeness. `verify-vt`
  exits 0.
- PHASE-05's own selectors were declared mid-phase (`src/install.rs`); the
  phase also touched `src/worktree/allowlist.rs`, which has no design-target
  selector. Expect it as an undeclared conformance cell at audit — adjudicate
  it there rather than pre-emptively widening the selector set.
- PHASE-04 worker UNCERTAIN items, unadjudicated and due at audit: facet
  `schema_version` defaulting, and caller-supplied `*_origin` values riding
  through as sent.
- PHASE-04 T6 was minimal — the shipped worker definition gained the capability
  token but no body prose describing it. Likely PHASE-05's scope; confirm there
  rather than leaving it to audit.
- QUE-176 — trustworthy per-harness token instrumentation boundaries
- IMP-319 — subprocess-worker observation capture parity
- IMP-320 — configurable observation guidance in boot context
- IDE-005 — harness identification through bounded environment enrichment
- IMP-328 — pi spawn fork unbound to slice/phase (blocks funnel import on the pi
  arm until the script derives branch/dir from `(slice, phase)`)
- `slice verify-vt 231` is a SLICE-level conclude-cadence gate and remains
  pending until all five phases land — not a PHASE-01 gap.
- PHASE-01 conformance is substantively clean (6/6 declared source paths hit);
  its one `--strict` failure is the ISS-264 machinery false positive, not a
  worker scope violation.
- No ledgered code review exists for the PHASE-01 delta. The informal deepseek
  pass produced one confidently-wrong finding and silently skipped the check
  flagged as most important, so it is not sufficient evidence on its own. VT-4
  was adjudicated against the landed code instead: all six purity categories are
  pinned and the leaf classification is enforced by the real gate. The residual
  is that no observation-SPECIFIC upward-dependency negative test exists — the
  gate's generic rejection tests cover the mechanism.
