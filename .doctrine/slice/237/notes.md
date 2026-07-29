# Notes SL-237: Primary-resolve runtime phase state

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-29 · stage: design authored, RV-322 round 5 integrated + REV-043 split, NOT locked · 73e94ecd

### Produced

- SL-237 — scope doc (`slice-237.md`), selectors seeded, status `design`.
- CHR-050 — runtime-state scope audit for `.doctrine/state/dispatch/` + `review/`
  (the deliberately-excluded siblings). Committed.
- `research/research.md` — pre-design evidence artefact, self-serve (both spawned
  threads failed). **Runtime tier, gitignored** — harvest its durable findings at
  close before it is discarded.
- obs `019fa916-1a31` — `.doctrine/state/` flat-drawer survey cost.
- obs `019fa932-8c8f` — `pi --print` buffering makes a working research thread
  indistinguishable from a hung one.
- obs `019fa953-d669` + **ISS-276** — research agent scripts hang on tool-using
  prompts (3/3 failed); trivial no-tool prompt returns instantly. Committed.
- obs `019faac3-3fe6` — a multi-file grep silently omitted matches from its
  FIRST file argument; would have falsified the fan-out finding.
- obs `019fac1d-4d73` + **ISS-277** — `doctrine reseat` cannot renumber ANY
  review (strict `Meta` read requires the derived `status`). Committed.
- obs `019fac68-aad5` — no `spec req show`: inspecting one requirement means
  reading the whole spec.
- obs `019fac6d-a1fb` — the review jail mounts `.git` read-only, so a reviewer
  cannot commit its own ledger delta; the responder imports it.
- obs `019fac70-2fb7` — a reviewer with workspace-write rewrote `notes.md`'s
  single-copy Harvest section (112 lines → 14). Restored from HEAD.
- `design.md` — authored; internal pass (§10, 4 findings) **and** external
  RV-322 pass (9 findings) integrated.
- obs `019fac8e-812d`, `019fac8e-813a`, `019fac94-746c` — round-5 reviewer obs.
- **RV-322** — external design review, raiser codex/GPT-5.5. **18 findings over
  5 rounds, all accepted, all dispositioned.** `active`, await=**raiser**.
  No finding ever attacked the core mechanism. Rounds 3–5 hit only
  governance-document accuracy; round 5 verified F-10's sweep clean and raised
  F-18 (sweep frontier too narrow). (Renumbered by hand from RV-320 after an id
  collision — see ISS-277.)
- **DEC-095..098** — the four locked design decisions.
- **REV-043** — one claim at three authorities: ADR-006 D2/D4/**D9**, four
  SPEC-012 surfaces, five PRD-015 surfaces, + REQ-296 revision. **Split at
  round 5** — confinement-model argument removed, not resolved.
- **QUE-199** — is REQ-304's "by construction" accurate given cooperative
  confinement? Parked out of REV-043. Not caused by SL-237.
- **IMP-354** — platform-agnostic seatbelt/bwrap worker confinement wrapper.
  Parked. Not a precondition for this slice.
- design-target selectors recorded (6 paths).

### Learned

Durable findings — candidates for `/record-memory` at close, not yet recorded:

- `primary_worktree` (`src/git.rs:564`) forks `git worktree list --porcelain`
  **per call, uncached**. Any path constructor that resolves it internally
  inherits that cost per invocation.
- `slice list` fans `phase_rollup` over **every** slice meta
  (`src/slice.rs:2158-2164`); with 221 slice dirs here, a per-call git resolution
  in `phases_dir` becomes ~221 subprocesses. The fan-out is at command level, not
  in the inner loops.
- `run_reconcile_phases`' refuse-when-live bail (`src/slice.rs:1236-1244`)
  **precedes** its gather (`:1252`), so its `coord_rows` is always `None` — one of
  `gather_truth_inputs`' two callers has never exercised the coord branch.
- ~~ADR-006's "per-worktree by construction" sits in **Context**, not a
  Decision.~~ **Superseded in `/design`.** True of Context force 2, but the
  research round stopped there. ADR-006 **D4** ("runtime state stays
  gitignored/per-worktree") and **D2** ("the coordination/runtime tier is
  withheld") are **Decisions**, and both are falsified. Hence REV-043.
- **There are TWO 221× fan-outs, not one.** `list_rows` (`slice.rs:2161`) *and*
  `lazyspec::load_slices` (`lazyspec.rs:495` → `load_plan:518` →
  `phase_rollup:535`). The research round found only the first.
- `conformance_outcome` (`slice.rs:2852`) states ISS-269's defect in its own doc
  comment: *"`root` resolves both the shared registry (via the primary worktree)
  and the local phase-sheet state tree."*
- A newtype makes a fan-out **visible at review, not impossible**. The missed
  second fan-out is the proof; don't let a type stand in for a call-site check.

From RV-322 (external), all verified at source:

- **`project_root` is ONE name doing TWO jobs** — where state lives, *and* which
  tree's git is asked. `capture_phase_boundary` uses it for
  `resolve_ref(project_root, "HEAD")` (`state.rs:629`) and
  `live_worktree_for_ref` (`:617`). Any migration that swaps it for a single
  primary handle records the **primary's** HEAD as a solo worktree's boundary.
- **A warning is not a mitigation.** Observability does not stop the write that
  follows it. Fallback-on-fault must refuse, not warn.
- **The census axis was wrong.** `phases_dir` callers cannot find *hand-built*
  state paths. `integrity.rs:322-323` builds `root.join(kind.state_dir)` and
  would have silently stopped guarding. Census `.doctrine/state` *construction*.
- **`pi-spawn-confined.sh:110` sets `DOCTRINE_WORKER=1` unconditionally** — so
  worker_mode holds via the env leg regardless of the marker, and the CLI guard
  refuses before any EROFS is reachable.
- **ADR-008 D-B3's "claude cannot be wrapped" is STALE.** The claude arm ships
  PreToolUse confinement (`dispatch-agent/SKILL.md:177-179`: nested bwrap +
  Edit/Write denial). Needs its own correction; NOT amended by REV-043.
- **PRD-015 was never swept pre-design.** Invariant 2 is falsified; ~~REQ-296 and
  REQ-304 are **preserved**~~ **REQ-296 is REVISED** (rounds 3+4) and REQ-304 is
  preserved (allowlist `is_withheld` at
  `src/worktree/allowlist.rs:170` untouched; OS floor both arms).
- **Single-homing trades divergence for a race.** `edit_phase_sheet` is unlocked
  RMW. Two copies could diverge but not race; one file can race. No lock is
  designed — bounded by D2/D9 sole-writer + the single-operator precondition.
- **"No copy" proves non-DIVERGENCE, not non-CORRUPTION.** A lost update on a
  single shared file needs no second copy. This inference was made three times
  across rounds 1/3/4 in three different documents before it was named; it is the
  most reusable defect this slice produced. Candidate memory at close.
- **A correction is not complete until every sibling assertion is dispositioned.**
  Read-and-patch failed three consecutive rounds. What worked: turn the accepted
  claim into a grep **with a positive control**, then patch the list — not the
  reviewer's citations. Round 4: 12 sites, 6 named. The unnamed half included both
  `.toml` tiers, a diagram contradicting its own contract table, and an instance
  no finding mentioned. Candidate memory at close.
- **An enumeration is only as complete as the frontier you point it at.** Round 5
  (F-18) caught the sequel: round 4's sweep ran over a *hand-picked file list*
  (the docs already under revision), so ADR-006 D9 and three SPEC-012 surfaces
  survived. Fixing instance-blindness left scope-blindness. Pick the frontier
  from the *claim's* authorities, not from the files you happen to be editing.
  Candidate memory at close.
- **A line-oriented grep misses line-wrapped prose claims.** The round-5 control
  for "worker-sole-writer free" did NOT hit `adr-006.md:222` — the phrase wraps.
  Multiline (`re.S`) sweep found it. A control proves the pattern works on the
  instance you already know; it does not prove the pattern's *shape* fits the
  corpus. Candidate memory at close.
- **De-duplicate, don't align.** Round 4 "fixed" drift between two surfaces by
  making them say the same sentence; round 5 showed that is the drift mechanism,
  not the cure. DEC-098's title was fixed by *removing* a restatement. Prefer
  each surface saying the minimum it needs.

### Open

All pre-design opens are settled. Remaining:

- **RV-322 is `active`, await=`raiser`.** All 18 findings dispositioned across 5
  rounds. Round 5's three dispositions (F-1, F-12, F-18) are unverified by the
  raiser. **Recommendation on record:** don't spend round 6 on the whole design —
  if another pass is wanted, target REV-043's proposed text alone, where the last
  three rounds' defects all lived.
- **Design lock NOT granted.** All passes integrated and the design assessed
  ready; the user has not approved. `/plan` is gated on that.
- **Residual ripple risk (the live one for implementation).** F-5/F-13 proved the
  census axis was wrong — `integrity.rs` hand-builds `root.join(kind.state_dir)`
  and never calls `phases_dir`, so a call-site census cannot find such sites.
  No guarantee another exists. §5.2's contract table + V-11/V-17 pin the known
  ones. Expect this to surface as a red test, not a design rewrite.
- **REV-043 is `proposed`** — not approved, not applied. `SL-237 needs REV-043`
  is now an authored gate (RV-322 F-7): **PHASE-01 does not start until REV-043
  is `approved`**. Its `[[change]]` rows are ADR-006, SPEC-012, PRD-015.
- **ASM (carried)** — no repo ⇒ the given root is its own primary. Load-bearing
  for ~20 existing tests on non-git tempdirs; a design invariant (§5.5 I-1), not
  error handling.

Settled this session:

- **DEC-095** — resolution mechanism: `PrimaryRoot` newtype minted in the shell.
- **DEC-096** — `resolve_phase_truth` narrows (keeps the `Some` matrix);
  `--across-trees` → `--truth`.
- **DEC-097** — mint the `phases` symlink in the primary only.
- **DEC-098** — migrate `boundaries_path` in-slice; it becomes infallible.
- ~~**QUE (settled)** — spec sweep: no blocking REQ in SPEC-012 / SPEC-021 /
  PRD-015.~~ **FALSIFIED by RV-322 F-1.** The pre-design sweep checked REQ-297 and
  stopped. PRD-015 REQ-296 **is** revised (rows 3b); REQ-304 is parked as QUE-199.
  The original claim is left struck rather than deleted — it is why F-1 existed.
- **QUE (settled)** — worker read exposure: **accepted explicitly**, not fenced
  (`design.md` §7 D5); now a stated non-goal and named in REV-043.
- **OQ-1 (closed in review)** — DEC-098's contract change hides nothing; both
  propagating sites fail closed for independent reasons (`design.md` §10 R-1).
- **Correction landed** — `slice-237.md` objective 5 now reads "narrow", and
  objectives 6/7 were added.
- **DEC-095 amended ×6** — F-3 (two-root split), F-6 (`resolve` fallible for a
  fault, total only for "no repo"), F-9/F-C (`assume` is a **test** seam;
  amendment 5 supersedes 3), F-B (fallible constructor for writers only),
  **F-10 (amendment 6: the split is by TYPE — `PrimaryRoot` / `ReadRoot`, one-way
  `From`, no reverse conversion; amendment 4's API superseded in place)**. The
  newtype survived every round; what moved was the parameter, then the type.

Follow-ups raised, not owned by this slice:

- **ISS-277** — `reseat` cannot renumber a review.
- **QUE-199** — REQ-304 "by construction" vs cooperative confinement. Filed.
- **IMP-354** — platform-agnostic seatbelt/bwrap confinement wrapper; the
  normative `dispatch-subprocess` skill spawns unconfined and its
  "Confined (bwrap)" annotation is not implemented. Filed.
- **ADR-008 D-B3 staleness** — the claude-arm "cannot be wrapped" concession is
  stale. Named in REV-043 as a follow-up; **still no backlog item** — distinct
  from IMP-354, which is the codex/pi arm.
