# Notes SL-233: CLI-managed design runs and inquiry maps

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-08-02 · **ALL 16 PHASES `completed`. THE SLICE IS AT `audit` (`5521c579`). PHASE-09 CLOSED AT `25350b07` (code) — `VH-1` ACCEPTED BY THE OWNER, HEDGED, and the hedge is written into the artefact at `97856e36`: `pre-registration.md` §9 gains the S1/S3 single-arm disclosure and a new §10 fixing the first CHR-049 run as a PILOT and PRE-REGISTERING the re-run trigger (a discriminator — high delivery with low adoption — not a threshold). **THE BLOCKER IS NOT THE AUDIT, IT IS THE LANDING: `dispatch sync` HAS NEVER RUN, trunk moved 195 commits past the fork-point, and SIX DEC IDS COLLIDE.** See *Open*.** T0 was a SETTLED-STATE DISTILLATION, and that is the reusable point: T0 is a SETTLED-STATE DISTILLATION, and that is the reusable point: `EX-8` and `VA-5` carry ELEVEN rounds of append-only amendment, so read linearly they present WITHDRAWN REQUIREMENTS BEFORE THEIR WITHDRAWALS — `stand` (round 6), checks (6)-(8) (round 10), the `reword answers VA-4` gloss (round 7), `checkable in run state` as a classification limb (round 11), and five more. `evaluation/pre-registration.md` §8 registers all nine dead clauses with the round that killed each. Authoring the kit from the criterion transcript would have shipped requirements that do not exist, and because `VA-5` is the criterion that VERIFIES the kit that error would have been self-concealing. Two findings recorded in the sheet: **F-P09.2** — RFC-021 has TWO stage axes (staged descent 1-5, and the eight-stage adherence pipeline) and `VA-2` means the pipeline, of which the scope names the last five; **F-P09.3** — `VA-4`'s literal text expects the ADHERENCE signal to revise the placement, which `EX-8` round 4 (F-10) ruled a category error, and `VA-4` was never amended; T0 satisfies `VA-4`'s substance without re-inserting the F-10 incoherence, and the tension is flagged for the owner rather than amended. Venue confirmed: `.dispatch/SL-233` IS ADR-012's COORDINATION worktree (populated `.doctrine/state/`), so `VA-1` is satisfiable here; what it excludes is a worker fork. · **PHASE-08 IS `completed` — `VH-1` ACCEPTED BY THE OWNER 2026-08-02, on the stricter reading (D10 is reversed and bounds nothing, so only D7 and D8 were live; the one residue row, the Locked exit at `SKILL.md:44-48`, rests on D8 alone). EVERY EX/VT/VA/VH CRITERION IS DISCHARGED at the final code commit `ce53a2ba`. `EX-8` stays DEFERRED to CHR-049 and `EX-5` is discharged BY DELETION — both owner-ruled, neither an unmet criterion. ALL 16 PHASES ARE NOW COMPLETE (this clause is superseded; it recorded the position at PHASE-08 close).** Execution record follows.** T5 (both `EX-9` tests, `VA-NC` red evidence recorded incl. the deliberate bogus-command leg) and T3 (`install/routing-process.md` — canonical family, `EX-10` acronym rehome) are GREEN; T1 (the `design-prompts` assets) is GREEN — `F-P08.2` SETTLED, owner ruled option (d) at `3516a8b3`: a step id is an identity term, not a label, and `RUNBOOK_STEP_ID_BYTES` is now derived from `DESIGN_ID_BYTES` rather than independently chosen. THE WHOLE SUITE IS GREEN (`check commit` EXIT=0, census 5990 ≥ 5988) — which also cleared `e2e_design_review`, RED 7/74 at `78d00a07` despite that packet claiming otherwise (`F-P08.3`); ISS-290 carries the unguarded class defect and its one deliberately-unrepaired sibling (`F-P08.4`). **T2 SHIPPED** (`59c5bdab`) — the design `SKILL.md` is 214 lines/10,178 B → **57/2,600**, and `EX-10`'s atomic rehome is DISCHARGED with both halves in that one commit. **T6 SHIPPED** (`0b00d789`) — `EX-3`'s tail ran clean, and on two owner rulings `EX-5` is discharged by **DELETING** `.doctrine/routing-process.md` (nothing projects it; `doctrine library show reference/routing-process.md` is the reader's route) while `EX-8` is **DEFERRED to CHR-049** (both channels install from the github remote at `main`, so the lockfile cannot be regenerated truthfully pre-close). **T4, T7 AND T8 ARE NOW DONE — EVERY TASK IN PHASE-08 IS COMPLETE AND EVERY `EX` IS MET; ONLY `VH-1` (the owner's) REMAINS.** T4 (`EX-7`, the handover adapter) SHIPPED at `ce53a2ba` — the owner released its STOP 2026-08-02 as an interactive editing pass and ruled its three open shape calls (keep the full weight; leave `--known-revision`/`--known-fragment` to `/design`; keep *declared baseline*). `handover/SKILL.md` 114 lines/5,037 B → **155/6,878**, one new section between `## Harvest first` and `## The dial`, which is DEC-058's own ordering. The sharp edge is handled: `design show --format status` signals absence as an ERROR whose text discriminates *no run* from *runtime loss*, and the shipped prose FORBIDS reaching for `--from-design` to manufacture a handover — a reconstructed run infers no attestation, receipt or gate clearance, so its reference would promise more than it holds. Two constraints found in passing: `tests/e2e_claude_install.rs:601` bans the literal `` `slice design` `` across `plugins/**` (complied with), and `skills-lock.json`'s `computedHash` is NOT a working-tree sha256 — the entries are `sourceType: github`, so the lock hashes PUBLISHED content and is stale for every skill on this branch by construction, which is the same root cause CHR-049 already defers. T7's `VA-4`/`VA-7` disposition table sums to the incumbent in lines AND bytes, uses all seven of `VA-7`'s vocabulary slots, and FOUND AN UNDELIVERED DISPOSITION — `:194` to the sealed stage hymn, which T1's checklist never carried; delivered at `def8b809`. T8 discharged every agent-mode criterion, **re-run at the post-T4 tip `ce53a2ba`** (`VA-1` boot --check EXIT=0, `VA-2` porcelain empty, `VA-G` gate EXIT=0 with the same 114 suites / zero FAILED as before T4, census 5990 ≥ 5988); `record-delta` re-issued `--end ce53a2ba`, PHASE-08 `VT-1` and `VT-2` both PASS. Owner's `VH-1` round 1 landed three prose corrections (`F-P08.8`). ISS-291 raised. **`VH-1` WAS THE OWNER'S AND WAS NOT SELF-CERTIFIED; the owner accepted it at round 2 and PHASE-08 IS NOW `completed`.** EN-2 was met before any of it: RV-325 IS TERMINAL — `done · await=none`, 17/17 findings `verified`, 59 ledger turns, closed at `73139a61e` after ELEVEN rounds; every RV ledger this slice owns is terminal** — dispositions **D1–D14** in `sketches/target-machine.d2`, prose in `sketches/thin-adapter.md`; **the round-11 addition is `The gate, applied` — the nine 2a completion conditions PHASE-08 must author from** · plan amended pre-start and again mid-review: **`VA-4` judgement clause withdrawn to `VH-1`** (`43986ffe`), then **`EX-10` / `VA-7` appended to PHASE-08 and `EX-8` / `VA-4` / `VA-5` to PHASE-09** (`6958c635`, `0cfd87b1`, `e2a6b8ee`) · **DEC-103** (delivery at the point of effect) and **DEC-104** (heuristic sets are stage framing, not runbook checklists; discriminator amended to *completability at a boundary* at round 3 F-3) both `accepted` · **IMP-373 (`set` mode) stays deferred on a RE-WARRANTED basis — the `on merit` claim is withdrawn as false; five order-independent steps ship under `sequence` and the tension is conceded, with an evidence-triggered repayment condition** · **SUPERSEDED — all 16 phases completed; this recorded the position at PHASE-08 close** · PHASE-16's four hardens-on-ship contracts are SETTLED at phase-plan and shipped in code — see *Produced* · **VT-1 SEVEN of seven green**, each observed red on its own seam (**VA-NC fully answered**); every PHASE-16 criterion discharged, each negative with a real positive control · **source delta recorded** (`d77f1b4d7`…`7c0a6c9e`), so PHASE-16's VT rows are PASS not UNATTRIBUTABLE · **`verify-vt` now exits 1 on TWO FAILs, neither PHASE-08's nor PHASE-16's** — PHASE-09 `VT-1` (that phase has not run) and PHASE-15 `VT-2` (ISS-289, an audit judgement, deliberately unrepaired). Was three; PHASE-08's cleared at `ce53a2ba` · sketch at **revision 3 plus the round 4-7 amendment blocks**; PHASE-09 `EX-8`/`VA-5` amended four times over (round 4 F-10, round 5 F-12, round 6 F-13, round 7 F-14+F-15) and **DEC-104's structured tier amended with them at round 7**, PHASE-16 `EX-7`/`EX-14` ANNOTATED with discharged obligations preserved (round 5 F-11) · **execution mode: solo-in-worktree, NOT dispatch** · **every RV ledger this slice owns is terminal: RV-315, RV-320, RV-321, RV-323, RV-324 all `done · await=none`** · PHASE-08's `EN-2` design gate is CLOSED — it was met before execution began (RV-325 terminal), and with `VH-1` accepted no governance thread on this phase remains open

### Produced

- **PHASE-09's evaluation kit — the instrument CHR-049 runs** (`25350b07`;
  T0 at `7cb6896d`). `.doctrine/slice/233/evaluation/`: `pre-registration.md`
  (the operative rule set, written before any run), `rubric.md` + `rubric.toml`
  (five evidence classes scored **separately**, four bands, no composite and no
  per-class weights), `protocol.md` (the standing `context_state` obligation, the
  induced break, the sibling contrast, and the deny / inconclusive / 5-of-9
  coverage rates the kit reports about itself), `collectors.toml` (four signals,
  the reopening's two arms, the nine-obligation gate record, the cost and
  acceptance-basis instruments), `fixtures/` (known-good, known-bad, and the
  concrete T-base artefact — blob `65b6c45b`, 214 lines / 10,178 B), and
  `tests/e2e_design_evaluation.rs` (exactly two tests, the names being the
  contract). Green from the coordination tree; `check gate` EXIT=0 at the final
  commit; `VT-1` PASS. **`VH-1` is with the owner and the phase stays
  `in_progress` — an agent cannot certify that a rubric measures something the
  user cares about.**

- Scope and research handoff established; entered design (commits
  0b2744ca, 9012408f, 19721727).
- design.md authored and internally reviewed through DEC-056…DEC-088
  (6835ba7f…d8a4cf66, 181776552).
- RV-315 driven to `done` at round 47 (F-1…F-15 all terminal), then
  reopened with F-16…F-19 from plan-grounding research (d6537dae5).
- RV-315 **terminal at round 64, all 20 findings** (rounds 4–5 raised by
  external codex bench). F-16/17/18 verified first pass; F-19 contested
  and repaired; F-20 raised against the repair and paid.
  20a838a7, 69c48a95, 15835f6c, a0f6ba1c, d3c74c5f, d5e1a9c0, a2e235b3,
  7de1f628.
- DEC-092 (authored watermark) — **accepted** (6f3240cf7).
- Plan authored (cf19aedeb), revised once (475858835), then attacked
  and repaired: 12 phases, execution order 01→02→03→04→05→06→11→12→
  10→07→08→09. Four selectors added; five broken VT mandates fixed.
- Scope B2 amended: descent is exactly 2 new specs + SPEC-019 + SPEC-023.
- Round-2 research packet: 6 `pi-scout` threads, `research.md` refreshed
  and baseline re-stamped. **Gitignored tier** (`.gitignore:48`) — exists
  in the primary working tree only.
- ISS-265, ISS-266 (0364bd5b, f36949f5); RFC-011 case notes x3.
- Dispatch set up: coord worktree `.dispatch/SL-233` on `dispatch/233`,
  base 2f70392e5, all 12 phase sheets materialised. Claude arm.
- **PHASE-01 complete** (coordinator-only, solo). PRD-019 *Managed design
  workflow* (14 reqs, REQ-414…427) and SPEC-029 *Design run engine*
  (11 reqs, REQ-428…438) authored `active`; SPEC-019 gained one provenance
  acknowledgement, SPEC-023 one seal slot `stage/design`. All 7 EX and
  5 VA green; conformance `undeclared (0) / conformant (86)`.
  0fb590e53 → 954e10d6f → 9bc1e729a.
- PHASE-01 concluded by hand (the coordinator-only ritual) — boundary row
  `2f70392e5 → 9bc1e729a` recorded at `8fc865297`.
- **ISS-275 / DEC-094** — `review`'s root guard narrowed from "any linked
  worktree" to "worker fork", unblocking the design gates in the coord tree.
  Landed on `edge` (`1946d0b0`), flowed in via `refresh-base` (tip `abad2f1`).
  IMP-346, IMP-347, ISS-273, ISS-274 also captured.
- **PHASE-02 EN-2 gate, half done.** `sketches/projection-bounds.md`
  authored (`06c9946ef`), RV-320 raised against it by codex — **seven
  blockers, all substantively valid**; sketch rev 2 (`990ade71`) and
  dispositions F-1…F-7 (`9ed8ab55`).
- **RV-320 two verify rounds run.** Round 1 (`89fe74db`): F-1/4/5/6 verified,
  F-2/3/7 contested — all three contests valid. Sketch rev 3 (`5fe46ec7`),
  PHASE-03 EX-13/VT-5/VA-6 appended (`adf7ecea`), re-disposed (`3bad9af0`).
  Round 2 (`7c574ef3`): F-2/F-3 **verified**; F-7 contested again, also valid.
  Sketch rev 4 + EX-14/VT-6/VA-7 (`615d293f`), re-disposed (`617cdcfb`).
- **Ceiling decided (user).** `ENVELOPE_NORMAL_BUDGET_BYTES` 16384 → **24576**,
  sketch rev 5 (`305e4d4e`), settled *before* round 3 so the reviewer was not
  asked to adjudicate the author's own open question. Headroom 808 B → 9000 B.
- **RV-320 round 3 run** (`b0611dbb`). F-7 contested a **third** time on three
  counts, all valid: EX-14 forbade `ENVELOPE_*` in the persisted row then bound
  the stored reason *to* `ENVELOPE_REASON_BYTES` (F-1's class a **fifth** time,
  inside the criterion written to stop it); ids were truncatable so two sharing
  a 32-byte prefix rendered identically (F-7's own subject failing); EX-13 and
  EX-14 were simultaneously live and contradictory. Sketch **rev 6** states
  **the layer rule** as the mechanism instead of a fourth patch, plus plan
  EX-15/VT-7/VA-8 and PHASE-05 EX-10 (`d31bed5d`); F-7 answered (`2b1fe716`).
  **No byte arithmetic moved** — 145 / 264 / 15,576 / 9000 all recomputed.
- **RV-320 round 4 run** (`a5028b79`). F-7 contested a **fourth** time, again on
  three valid counts, and **rev 6's own falsifier fired on rev 6's own fix**:
  (i) nothing tested that *every* run-local-id admission path refuses an
  over-bound value — rev 6 made ids render whole, which moved the load-bearing
  guarantee from the renderer to admission without making admission total;
  (ii) VA-8's "reachability" grep was the containment check's error one level
  up — it proves sensitivity to one spelling, and a copied literal or renamed
  constant evades it; (iii) `CHANGE_REASON_INPUT_BYTES` = 2048 was correctly
  layered, correctly enforced, and **entirely underivable** — the sixth
  instance, inside the repair. Sketch **rev 7** widens the mechanism to a
  **pair** — layer rule + **provenance rule** — deletes 2048 (the reason is now
  stored unbounded, consistent with section bodies), and makes enforcement
  structural: a module dependency rule on ADR-001's existing
  `tests/architecture_layering.rs`, and one validating constructor for every
  run-local id. Plan EX-16, VT-8, VA-9; VT-7 legs 2/3 and VA-8 leg 3 withdrawn
  (`37b112ee`). F-7 answered (`a5028b79`). Arithmetic unmoved again.
  Ledger `await=raiser`, 6 of 7 terminal, F-7 answered awaiting a **fifth**
  verify.
- **Sketch rev 8** (`0ae2f398`), prompted by no round. The owner pointed at
  `.doctrine/adr/001/layering.toml` and one read falsified rev 7's enforcement
  claim: the map models **one** axis (altitude, `leaf|engine|command`) and pure
  constants classify `leaf`, the tier everything may depend on — the checker
  would have **licensed** the violation. Replaced by **Rust privacy**
  (`ENVELOPE_*` private to the rendering module ⇒ a storage reference is a
  compile error). Limit named, not papered over: privacy misses a *copied
  literal*; that is STD-001 review territory. Seventh instance of the class,
  and the first caught by a rule in the sketch rather than by a reviewer.
- **RV-320 TERMINAL** (`8b36b352`) — 7/7 verified, `await=none`, 33 rounds.
  Round 5 was deliberately **scoped**: the question put to the raiser was not
  "is the sketch perfect" but "is the material-delta contract buildable and
  reviewable-against", with remaining scrutiny moved to code review of the
  implementation. Verified on that basis; the raiser explicitly treated the
  durable response's ADR-001 clause as stale and adjudicated rev 8 instead.
  **PHASE-02 EN-2's ledger condition is satisfied.**
- **Process signal, retained for the record.** Four adversarial rounds, four
  contests, all substantively valid — and every one found the defect in the
  **repair**, not the original. What was under test had become the author's
  method rather than the design. **The resolution taken** (owner's call): stop
  absorbing rounds at the design gate, scope round 5 to "buildable and
  reviewable-against?", and move the remaining scrutiny to a **code review of
  the implementation** once PHASE-02/03 build it. That review is now an owed
  step, not an optional one — it is where the copied-literal gap and the
  constructor-bypass question actually get settled.
- Memory `mem.pattern.projection.bound-must-not-bind-the-record` **generalised
  to both halves** of the layer rule on `edge` (`f314eebe`); the original entry
  stated only half, which is what let rev 4 violate it while quoting it.
- Two friction observations, both on `edge`: `review contest` has no durable
  rationale field (`8d2fa56d`), and `review dispose` has **no responder amend
  path** (`17339298`) — a response written against rev 7 could not be corrected
  after rev 8 landed, so the correction had to ride the next round's brief.
  They compound: neither side can put a correction into the durable record
  without waiting for the other's turn.

- **PHASE-02 landed** (`e26755dc1` lifecycle, `2cd7915a5` source, `5edf47338`
  refactor; boundary `e26755dc → 5edf4733` at `88fa13ffb`). `src/design_run/` —
  ten files, ADR-001 tier `leaf`, crate out-degree zero. All eight §9.1 tests
  green, VT-1/2/3 PASS, `doctrine check gate` exit 0, census 5205 → 5213.
- Memory `mem.pattern.lint.dead-code-module-level-vs-cfg-attr` recorded on
  `edge` (`226f5eed`), and the verify-vt UNATTRIBUTABLE memory amended there with
  the dispatch-arm variant. One friction observation on `/phase-plan`.
- **PHASE-03 implemented and landed** (`1f90ae7a`, 18 files). Schema-versioned
  snapshot riding `wire.rs`; revision CAS; submission idempotency with expiry;
  bounded receipt eviction preserving latest + delegation-pinned; the change log
  at full stored fidelity with explicit floor and candidate-order indexing;
  `src/design_run/render/` holding private `ENVELOPE_*`; `DESIGN_ID_BYTES`
  admission inside the one validating constructor; DEC-092's three watermark
  rules with two comparison bases. New: `src/commands/design.rs`,
  `src/design_run/{bounds,change_log,snapshot,run}.rs`,
  `src/design_run/render/{mod,change_row}.rs`, `tests/e2e_design_state.rs`.
  `check gate` exit 0 (106 targets), regression diff clean, three narrowed sweeps
  zero hits. **Landed via fallback live-worktree import** — see Learned.
- **Plan amended for PHASE-03** (`7c4eb95b`), owner ruling 2026-07-29: appended
  EX-17 (test-location split; EX-11's substance unchanged), VT-9 (the two
  relocated tests anchored in `src/commands/design.rs`), VA-10 (the count check
  split across two suites), VA-11 / VA-12 (narrowed sweeps replacing VA-2 / VA-3,
  which were unsatisfiable as written). VT-2's pattern list narrowed to the five
  §9.2 names that stay, under the VT-7 precedent. Ids immutable — appended, never
  renumbered; verified no other phase and no prior criterion changed a byte.
- IMP-349 on `edge` (`39c06ca7`) — `.doctrine/state/slice` is single-sourced in
  name only: one named constant plus **nine** bare literals, three inside
  `state.rs`'s own tests. EX-8/VA-3 were authored believing otherwise. Five
  friction observations on `edge` (`0e36f858`, `7d7ba123`).
- **PHASE-03 closed out** — STD-001 fix (`4e2ad8e6`, three bare `96` plus a
  fourth bare `12`), boundary `7c4eb95b → 4e2ad8e6` (`93f82e08`), VT-1…VT-9 PASS.
  EX-18 + VA-13 (`9a307fe0`) legitimise the partial `design show` as a *plan*
  defect, not a scope widening. RV-321 (`257c5b7c`) — external adversarial review
  of the landed delta, four findings, all disposed; F-2's framing half answered
  AGAINST. EX-11 + VA-7 (`b0e5e908`) carry the accepted ones into PHASE-04.
- **PHASE-04 complete** — boundary `b0e5e908 → 228a7ed3`, import `38c3c2f0`,
  reaped. `TurnEnvelope` in `src/design_run/render/envelope.rs`, a *descendant*
  of `render/`, which is what lets EX-4's "one module" coexist with EX-16(c)'s
  compiler-enforced privacy: no `ENVELOPE_*` widened beyond the `#[cfg(test)]`
  precedent. Eleven further caps private. Sparse contract, traversal declarations,
  `resume`, the eviction ladder and global totals; `design` unhidden into FAMILIES
  `change`. Gate exit 0, S1 regression clean, VT-1/VT-2 PASS. Selector commits
  `2e6fec26`→`27e6bfcd` declare `src/kinds/{dirs,mod}.rs` `design-target`.
  Full VA-by-VA evidence in the PHASE-04 runtime sheet `## Findings`.
- **Base refreshed** — 31 trunk commits merged into `dispatch/233`, `trunk:
  stable`, gate re-run green. Halted first on an RV-320 id collision; resolved by
  renumbering SL-237's review to RV-322 (ISS-277, ISS-279).
- Harvest artifacts on `edge` (`a51f6697`): ISS-279 + two extended memories.
  Four friction records (`384f03f8`).
- **PHASE-05 complete.** Semantic checkpoints; evidence for QUE-180 and ASM-006
  (both still to settle — see *Knowledge carried*). IMP-353 filed for
  `WITHDRAWN_STATUSES` drift.
- **PHASE-06 EN-2 design gate — CLEARED.** `sketches/marker-grammar.md`, five
  revisions, **RV-323 terminal at round 30** (`54f1ce59`): `done · await=none`,
  6/6 verified. Raiser codex (non-author per EN-2), responder orchestrator.
  Rev 2 `dcb1…`→ rev 4 `acead397` (executed incumbent table, EX-14, VA-5/VA-7
  behavioural) → round 4 disposed `a85ae92d` → **rev 5 `de6e53c4`** → IMP-355
  cross-ref `1048adb6` → probe completed `81e2f4e1` → **round 7 terminal**
  `54f1ce59`. Harvest on `edge`: `c6ec5a52`, `add4ae4d`, `e108f4b6`.
  Round 7 verified F-3 and F-6 and **declined to re-open F-1** on the disclosed
  non-idempotence: F-1's byte-exact framing repair and the
  `materialise(adopt(prettier(D)))` equality never depended on one-pass
  idempotence.
- **Rev 5's method change, and it is the reusable part.** Rev 4 executed claims
  about the *incumbent*; rev 5 extended that to claims about the *proposed*
  rules via a **generated differential** — committed at
  `sketches/probe/title-differential.py`, with rev 4's defective rule retained
  inside it as the positive control. It found two defects, one of which the
  contest had not reached, and showed that the fix that looked obviously right
  closed **zero** divergences of 4,224.
- **Plan revised — PHASE-06 split three ways** (`7e8f78aa`). It still carried
  ten separable deliverables and eighteen named tests behind one design gate,
  for one worker holding one commit. Now: PHASE-06 the wire contract, **PHASE-13**
  the write side (markers / framing / document order), **PHASE-14** the read side
  (re-adoption / refusals / shim). Rationale in `plan.md` § *The phases that were
  doing too many jobs*. Ids immutable — moved criteria are pointers restated at
  their new home, the EX-7 → PHASE-11 precedent.
- **PHASE-06 landed** (`564e8775` → import `6a7624e8` → verified `813ca372` →
  concluded `d748bb1c`). `EX-12` collapse, `EX-13(b)` derived title at the
  declare arm, `EX-14` `deny_unknown_fields`. New leaf `src/design_run/section.rs`
  carries the derivation. `VT-4/5/6` PASS; `VT-1/2/3` waived against live
  mandates in PHASE-13/14. **`VA-NC` complete, not partial** — first time in
  three phases.

- **PHASE-13 landed** (worker `f4edc9e6` → import `dfdf1b5e` → verified
  `5acb8d30` → concluded `1577163e` → reaped). New pure leaf
  `src/design_run/document.rs` carries the whole grammar: the strict marker
  recogniser, the shape test, escape/unescape, render and parse. `blocks.join` is
  deleted and the framing affix is uniform; `trim_end` is gone from both former
  sites and `authored_sections` with it. `Section` gained `seq` (claimed from the
  existing `MapGroup::next_seq`, preserved on re-declare); `SectionGroup`'s id
  ordering is untouched and `document_order` is a separate accessor. `DesignId`
  gained the `[A-Za-z0-9_-]` charset arm; `CarriageReturnInDocument` is minted
  fieldless and enforced at declare. materialise now rides `entity::write_body`.
  **`VT-1`/`VT-2` PASS; `VA-NC` complete — all four reds observed at the mandated
  kind** (byte inequality, wrong acceptance, wrong order, recovery failure at
  159/200 generated bodies). S1 clean vs `03339c62`.

**What PHASE-14 inherits from PHASE-13 — read before planning it**

- **`tests/e2e_design_materialise.rs` holds EXACTLY two tests and PHASE-14 adds
  EXACTLY six.** PHASE-13 `EX-8` is a floor, PHASE-14 `EX-8` is an *exactly* over
  the finished file, and the arithmetic (2 + 6 = 8) means the floor is not an
  invitation. PHASE-13 deliberately routed all its extra coverage to
  `tests/e2e_design_state.rs`.
- **Marker recognition is STRICT — sketch §(a) was implemented over §(c).** §(c)
  calls a right-trimmed *k*=1 shaped line a marker; §(a) says a marker is "one
  line, at column 0, and nothing else on that line". Strict won. This cannot
  affect the round trip (write escapes every shaped line, so a *k*=1 shaped line
  never appears inside a materialised body) — it bites only on a **hand-edited**
  document, which is PHASE-14's whole subject. **PHASE-14 must confirm the strict
  reader's refusal is the intended §(e) row-5 behaviour**, which is its criterion,
  not PHASE-13's.
- **PHASE-13's two parse refusals are produced but not surfaced.**
  `authored_section_digests` calls `document::parse(...).unwrap_or_default()`, so
  a `StructuralDeletion` / `UnterminatedDocument` document yields an empty digest
  map and re-adoption refuses with the generic `AdoptionMarkersInvalid` — exactly
  the incumbent's behaviour for an unreadable document. **Surfacing the specific
  refusal is PHASE-14's protocol and is owed.** The seam carries a comment saying
  so.
- **`CarriageReturnInDocument` is fieldless**, because §(e) row 1 is a
  whole-document byte test and PHASE-14 must reuse the variant at parse where no
  id exists. Cost: the declare-side refusal cannot name which section of a batch
  carried the `\r`.
- **Recognition requires a `sec-` id**, not merely a well-formed `DesignId` —
  answer (a) writes `section-id ::= "sec-" body`, so `<!-- doctrine:section
  inq-1 -->` is neither shaped nor a marker. Pinned in a unit test.
- **The incumbent's slack-marker tolerance is gone.** `<!-- doctrine:section␠␠
  sec-1 -->` is no longer accepted as `sec-1`; it is body text, and is not
  escaped on write either. Deliberate per §(a); do not read it as a regression.

**Reconcile candidates opened by PHASE-13** (detail in the phase sheet, F-1…F-14)

- The two `EX-8`s are in latent tension: a "floor" that permits exactly what a
  later "EXACTLY" forbids.
- `VA-2`'s zero-hit grep is unsatisfiable as literally worded — answer (c)
  *mandates* a right-trim on the very path the grep sweeps. Discharged by
  enumerating each hit as recognition-only; zero hits in `src/commands/design.rs`
  still held.
- `EX-6`'s `seq` criterion does not say what a **re-declare** does; preserving the
  existing `seq` was derived from the `run.rs:461` inquiry-node precedent, because
  claiming fresh would move existing prose to the end of the document on every
  edit — the defect `EX-6` exists to close, through the other door.
- Filed rather than carried: **IMP-357** (test binaries duplicate the design-run
  fixture; PHASE-14 doubles it). **ISS-241** already covered the `record-delta`
  gap and gained a second independent confirmation.

**PHASE-14 complete — re-adoption, its refusals, and the deprecated shim (8/14)**

- **Two workers, one phase.** `agent-a7d4ed42` landed 7 of 8 (`f4edc9e6`-class
  fork → import `c744e58b`); `agent-a6112343` landed `EX-4`'s live-run arm
  (`0c0fe413`). Concluded at `befe4934`, both forks reaped, coord tip `e93289bd`.
  `VT-1`/`VT-2`/`VT-3` **all PASS**; S1 regression vs base `e35bbe5b`: **no new
  or changed failures**; full `check gate` green in the coord tree.
- **The read side exists.** `document::parse` is one ordered reader taking an
  optional held-set — §(e) rows 1,2,3 unconditionally, 4,5 only with a run, 6,7
  per region — with the order pinned by a six-adjacent-pair two-fault test. Four
  refusals minted (`MarkerFreeAddition`, `DuplicateMarker`, `UnknownMarker`,
  `MissingMarker`); `unwrap_or_default()` is gone from the shell seam so the
  specific refusal now reaches the caller. `DerivedInput.authored_sections`
  carries body + fingerprint + position; `adopt_authored` stores the prose.
- **`VA-7` found a live bypass, not just a risk.** `adopt_authored` was patching
  a **cloned** `Section` and never deriving a title. There is now exactly one
  construction path (`seat_section`), and both declare and adopt go through it.
- **`EX-4` cost a plan-shaped decision.** The shim's live arm needs
  `slice → commands`, which 2-cycles against `commands → slice` at
  `cli.rs:1488`: measured `TangleGrew { Command, 76 → 135 }`. Owner chose
  remedy (a) — move `run_design` into `src/commands/design.rs`, route the
  variant at `cli.rs`, keep `DESIGN_DEPRECATION_NOTICE` in `src/slice.rs`
  (`EX-4` and `VT-2` both require it there). Re-measured at **76**, the
  untouched baseline. `layering.toml` never edited.
- Filed: **IMP-358** (layering gate reports no tangle count when green),
  **IMP-359** (`arm-spawn` accepts arming a phase past the worker-commit
  position), **IMP-360** (section reorder produces no change row). **ISS-274**
  gained the answer to its own open question — the stale index is **four**
  funnel verbs, `conclude` included. Eight friction observations.

**Reconcile candidates opened by PHASE-14** (detail in the phase sheet)

- **D4's counter growth**: re-adopt claims *fresh* `seq` values in marker order
  rather than renumbering to `0…n-1`, to preserve `next_seq`'s never-reused
  invariant. Measured: `next_seq` 2 → 4 on a two-section reorder, i.e. the
  counter grows by the section count on **every** re-adopt, including a no-op.
- **§(e) now evaluates before `AdoptionStale`**, which `EX-1` lists first —
  structurally forced (the digest map is an input to `adopt_authored`). A
  payload both stale *and* marker-broken reports the marker refusal.
- **A landed PHASE-03 fixture changed.** `adopt_authored_crosses_divergence_and_rebaselines_alone`
  adopted a headless body, which `EX-6` now refuses at the adoption door. The
  behaviour-preservation gate could not hold literally; one fixture, subject
  unchanged.
- **`VA-NC`'s mandated red kind was unreachable for three of five refusals**,
  not the two the sheet predicted — `MissingMarker` is the same shape. Reported,
  not manufactured.
- `slice::dispatch`'s `SliceCommand::Design` arm is now dead by construction and
  **no test pins that**; `write_slice_meta` duplicates an inline TOML literal in
  a landed test (deliberately not refactored); `EX-4`'s "warns on every
  invocation" sits inside its *no-run* sentence while the constant's own doc
  says every invocation of the shim — the worker emits on **both** arms.
- **PHASE-07 partially landed** (`in_progress`, T0–T4a of T11). `1c09a319` — the
  five install assets: sealed `install/hymns/stage/design.md` (the first
  stage-band hymn ever authored), four fragments under `install/design-prompts/`,
  `stage/design` added to `[hymns].seal`, five `publication/manifest.toml`
  addresses as that manifest's first `fixed` (non-customizable) entries.
  EX-1/EX-3/EX-4 + VT-1 satisfied; test 1 of EX-8 green. `ecaf490c` —
  `src/design_run/prompt.rs`, the closed four-variant `Fragment` catalogue with
  `for_stage` exhaustive over `Stage::ALL`, three inline tests. EN-2 discharged by
  probe (sheet F-3). Remaining: T4b–T11 (shell wiring, tests 2 and 3, delivery
  tail, gate, record-delta).
- **PHASE-15 `completed`.** RV-324 driven terminal — F-5 verified at round 22
  (`903642b5`). All six findings terminal.
- **RV-321 driven terminal, after three sessions of slipping** — F-1/F-2/F-4
  verified and F-3 contested (`9330efc1`), F-3 re-disposed (`cddd828f`), F-3
  verified (`2a6379e4`). Round 14. Raiser codex, thread
  `019fb29d-bf8b-7740-8745-85c4f6b4cbd4`; RV-324's remains
  `019fb27d-e700-7bb0-b4b7-cc9ce8f39508`. Kept as separate sessions on purpose.
- **IMP-371** — mandated evidence in the disposable tier can vanish
  (`1b60afab`, primary tree, with a `slice phase` friction observation).
- **Two D2 models of the PHASE-08 surface** (`a6af8f30`):
  `sketches/workflow-as-prose.d2` (the incumbent skill's eight states plus the
  process its prose implies) and `sketches/surface-as-built.d2` (the
  instruction-emitting surface as built, shapes encoding the wiring). Both
  verified against the binary and source, not the plan. SVG only — d2's PNG
  path needs a playwright driver it cannot fetch in the jail.
- **The design conversation converged, and PHASE-16 is its product.**
  `sketches/runbook-runner.md` (the obligation runbook runner, now **revision 2**
  after an external prose review), **DEC-101** and **DEC-102** both `accepted` and
  `shapes SL-233`, `plan.toml` gains PHASE-16 *placed* between PHASE-07 and
  PHASE-08, `plan.md` gains its rationale and a corrected execution-order line.
  **IMP-372** (customization resolves to nothing) and **ISS-284** (no revision
  recheck before persist) filed on `edge` in the primary tree. No production code
  changed.

- **PHASE-16 implementation, tasks T1–T6 + T9 landed** (`8cabff60`, `58e65bd4`,
  `a14aa046`, `f8a1a048`, `80a1eba6`, `39735b12`; test-helper prep at `62299e1c`). The four contracts sketch §0 named as
  hardening-on-ship are settled **in code**, not merely in prose: (1) the seventh
  act's wire key is **`discharge`**; (2) the step-definition digest is
  **`runbook-step.v1`**, netstring-framed (`len:value`) over
  version/`id`/`text`/`required`/argv-arity/argv, with the version hashed *into* the
  material and stored *on* the record; (3) the discharge record is a `RunbookGroup`
  beside `fragments`, unit-enum outcome plus `Option` payload fields, constructors
  the only route in; (4) the five shipped step ids ship verbatim in
  `install/design-prompts/exploring.toml`. New: `src/design_run/runbook.rs` (leaf,
  crate out-degree zero), `gate::boundary_runbook`, `run::RunbookFacts` on
  `DerivedInput`, `ChangeEvent::StepDischarged`, and four `Refusal` variants
  (`DischargeReasonMissing`, `DischargeNotAtCursor`, `RunbookAbsent`,
  `RunbookNotDischarged`).
- **Five of VT-1's seven tests are green** in `tests/e2e_design_runbook.rs`:
  `editing_a_step_definition_makes_its_discharge_stale` (the fixture test over the
  `#[path]` model — also discharges VA-7 both ways), `a_skip_without_a_reason_is_refused`,
  `a_sequence_refuses_a_discharge_that_is_not_at_the_cursor`,
  `gate_refuses_to_advance_while_a_required_step_is_undischarged`, and
  `a_failing_verifier_refuses_the_discharge_and_surfaces_its_output`. Each was observed
  RED independently, which settles VA-NC's seam question empirically: they do **not**
  collapse into one commit.
- **T5 · the `doctrine verify` family** (`80a1eba6`) — `src/commands/verify.rs`, distinct
  from the pre-existing engine-tier `src/verify.rs` and saying so. Joins `FAMILIES` under
  `infra`; both `cli.rs` integration points moved (classification + the hard census, now
  56). Its only contract is exit code plus explanatory output: `slice research` is advisory
  (`Ok(())` on every outcome) **and** mints an absent baseline, so a runbook keying on it
  would have a check that satisfies itself on first run.
- **T6 · verifier execution** (`39735b12`) — `runbook::interpolate` is pure and in the leaf,
  substituting WITHIN each argv element and never re-splitting one (VA-6, pinned by its own
  test with a space and a `;` in one value); results enter through `DerivedInput`, never
  payload; execution sits outside the admit→persist span (VA-2). A skip runs no check —
  a skip is a disclosed deviation from doing the step, so there is nothing to corroborate.
  New `Refusal::VerifierFailed { step, exit, output }`, one variant for both "ran and
  objected" and the fail-closed "no result where a check applies".
- **The fixture repair, and where a shared test helper goes.** T4's gate clause reds 15
  pre-existing fixtures across `e2e_design_{review,state,delegation}` — one cause, their
  seeds walk the `exploring` edge, and that fallout is `EX-8`'s own evidence. The seeds now
  discharge the runbook like any other caller. Owner's placement ruling: **a sensible module
  imported by just its callers, not every e2e test** → `tests/runbook_fixture/`
  (`EXPLORING_STEPS` + `discharge_body`, reaching nothing outside itself).
- **The three-copy `sha256` e2e helper is consolidated** into `tests/common::sha256`
  (`62299e1c`) — one of the handover's three unfiled-and-owed backlog items,
  discharged rather than filed.
- **T7 · gate-time re-verification** (`02bceb29`) — `EX-11`. A required step's check
  re-runs shell-side before a stage act crosses the edge it guards, and the results
  ride the same derived-input path into `advance`, so a record cannot carry a stale
  pass through the gate. Three shape changes, decided rather than discovered:
  `DerivedInput.verification` → **`verifications: Vec<_>`** (two seams read it —
  `EX-10`'s discharged step and `EX-11`'s already-verified ones — and one payload can
  carry both acts; a step is checked at most once per submission);
  **`StepVerification` moved into the `runbook` leaf** (a fact about a *step*, and
  `standing()` needs it without the leaf reaching back into `run`); and
  **`RunbookStanding.regressed`** surfaced as a *field* on
  `Refusal::RunbookNotDischarged`, not a second variant — `EX-8` budgets one gate
  refusal and the fact refused is one fact, but `stale` (definition edited) and
  `regressed` (world moved under an unchanged definition) want different repairs.
  A withdrawn discharge returns the **cursor** to its step, which is what makes the
  repair reachable: the sequence admits only the step at the cursor. A skipped step
  is not re-checked, for the reason a skipped discharge is not checked.
  VT-1 #6 `a_verifier_that_regressed_since_discharge_blocks_the_gate` — sixth
  independent red. Census 5946 → **5957**.
- **T8 · envelope rendering** (`7c0a6c9e`) — `EX-14`/`EX-15`/`EX-16`/`EX-9`, `VA-3`.
  The rendering is **pure**: `Runbook::section(discharges, current, verified)` takes
  exactly what `standing` takes and derives the standing itself, so a call site
  cannot pair a rendering with a standing that disagrees with it. Shell-side
  `runbook_section` sits beside `fragment_section` (A4), reads the asset, and owns
  no formatting. **A4's second clause was unnecessary and `TurnEnvelope` is
  untouched** — the cursor is derived from the runbook's declared ORDER plus current
  digests, neither of which is in the snapshot, so a pure `assemble` could not have
  built it; but `Runbook::parse` already holds the step text, so the leaf renders the
  whole section and the shell supplies only bytes. Less shell logic than the
  assumption anticipated. Only the cursor step carries its `text` — every step's
  prose is both the deferred `set` half and the token regression `EX-14` forbids,
  and the e2e asserts that absence behaviourally rather than as a byte count.
  `EX-15` is inherited, not restated: the outcome word is read off the record and
  `Verified` is reachable only through `Discharge::verified`, which only a zero exit
  calls. **T8's decision — `regressed` renders**, beside `stale`, own word, own
  repair clause (a regression reported as stale sends the reader to re-read a step
  that has not changed). **Residual:** it is structurally empty on the read path,
  because `standing`'s third argument is what checks returned *this invocation* and
  a read runs none; populating it would put an unbounded subprocess behind
  `design resume`, which no criterion asks for and which cuts against `S2`/`VA-2`.
  A regression is named where it is detected — at the gate. Ship-and-learn, beside
  T7's F9. VT-1 #7 `an_attested_step_is_never_rendered_as_verified` — **seventh and
  last independent red; VA-NC fully answered**. Census 5957 → **5988**.
- **T10–T12 · close-out — all three were confirmations, and none needed authoring.**
  *T10 (`EX-17`)*: DEC-102's delivery claim reads identify-and-defer in three
  places and IMP-372 carries all four named resolution items. One residual, filed
  as sheet finding **F12**: DEC-102 cites the item by *kind* ("a backlog item"),
  not by id, and the two records currently sit in **different trees** — DEC-102
  here, IMP-372 in the primary — so neither a prose id nor a `link` resolves from
  either side until the slice lands. **Reconcile item**, not an `EX-17` miss.
  *T11 (layering)*: regenerated from `dump_real_graph`, never hand-guessed —
  `design_run (out=0, in=1)` with zero outbound edges (A1 holds) and
  `commands (out=64, in=2)`. `runbook.rs` and `commands::verify` each join an
  umbrella already at their tier, so **no mixed umbrella and no edit to
  `layering.toml`** — which is also what `VA-5`'s `.doctrine/adr/` fence requires.
  `architecture_layering_gate` green.
  *T12 (criteria)*: `VA-1`/`VA-2`/`VA-5`/`VA-ES`/`VA-G` discharged, each negative
  paired with a **real** positive control. `VA-1`'s was staged fresh rather than
  recalled from F4 — an eighth act added to one table only reds the suite (exit
  101) naming the asymmetry; reverted. `VA-5`'s fences are 0 bytes with controls
  on *both* axes (same range/different pathspec, and the **same pathspec form**
  matching real files in other ranges). `VA-ES`'s escape pattern was proved live
  against a genuine `#[ignore]` elsewhere in `tests/`.
- **`slice record-delta 233 PHASE-16 --start d77f1b4d7 --end 7c0a6c9e`** — the
  multi-commit escape hatch, since `--commit S` records only `[S^,S]`. `--end` is
  the cumulative **code** tip (T8), not the harvest commit, per the flag's own
  "pre-knowledge-record". PHASE-16's VT-1/VT-2 moved **UNATTRIBUTABLE → PASS**;
  slice-wide **37 PASS · 3 WAIVED · 1 UNATTRIBUTABLE · 3 FAIL**. `verify-vt` still
  exits 1 and **none of the three FAILs is PHASE-16's** (PHASE-08/09 not started;
  PHASE-15 VT-2 is ISS-289, an audit judgement).
- **PHASE-08 `EN-2` — the design conversation, then the paperwork** (2026-08-01;
  `a152c631`, `efeed98e`, `f7d9f45e`, `e03778c0`; plan `43986ffe`; backlog
  `ac523a22` in the PRIMARY tree). Order per the owner's 2026-07-30 ruling:
  conversation first, sketch back-solved.
  *The artefacts*: `sketches/target-machine.d2` — **diagram C**, third of the
  triptych after `workflow-as-prose.d2` (claimed) and `surface-as-built.d2`
  (built, and **pre-PHASE-16**: it says "six writer acts", now seven). Carries
  **D1–D12**, and is the primary decision record — `sketches/thin-adapter.md`
  (450 lines) is its prose. Where they differ the diagram governs.
  *The frame*: a three-tier mediation axis — engine invariant / runbook step /
  skill prose — whose first boundary is **not invented here**: it is the
  authoring rule already shipped at `install/design-prompts/exploring.toml:8-13`,
  and DEC-102 states the same line on the asset-policy axis.
  *The structural finding (D5)*: `can_advance` is total on non-terminal stages,
  so there are exactly **five stages and four forward edges**
  (`src/design_run/gate.rs:186-204`). An obligation lands on one of four edges or
  it stays prose. There is no fifth option — which is what made the disposition
  finite rather than a matter of taste.
  *The result*: eight prose states → five engine stages; **three of the eight are
  not states** (write-design.md is an effect; integrate-feedback is work inside
  Reviewing; propose-approaches is craft inside Inquiring). Guardrails triaged
  4 delete / 1 to the sealed hymn / 2 to steps / 3 kept. **Three runbooks**
  (edges 2/3/4), all `mode = "sequence"`.
- **IMP-373 — runbook `set` mode deferred**, authored in the primary tree.
  Deferral is clean: `Mode` is a **one-variant enum** and `mode = "set"` is a
  hard parse refusal with a test locking it (`src/design_run/runbook.rs:201-206`,
  `:986-993`) — no stub, no dead arm, no half-shipped surface. Blocked on a real
  question, not on effort: `EX-14` bounds rendering so only the **cursor** step
  carries its full text, and a coverage set has no cursor.
- **DEC-103, DEC-104 — both `accepted`, both owner-ruled, both from RV-325.**
  DEC-103 (delivery at the point of effect) answered the DRY-deletion defect;
  DEC-104 answered RV-325 F-2's nineteen-step edge 3 by splitting tier 2 into
  **2a runbook step** (`*.toml`, edge-keyed, dischargeable) and **2b stage
  fragment** (`*.md`, stage-keyed, every turn, not dischargeable). DEC-104's
  discriminator was **amended at round 3 F-3** from *could a verifier ever
  corroborate this* to **completability at a boundary**, with verification
  strength demoted to an orthogonal evidence axis plus a codicil (an unverified
  2a step must warrant its gate). Amended again at round 4 F-10 to carry a
  **named reopening condition**. `b756efb1`, `67506340`, `6958c635`, `0cfd87b1`.
- **RV-325 driven to 10 findings across four rounds, all dispositioned, none
  contested.** F-1 double-disposition; F-2 the nineteen-step gate (→ DEC-104);
  F-3 the unreproducible discriminator; F-4 deletion-before-replacement (→
  `EX-10`, closes IMP-376 in-phase); F-5 the unsettled edge-2 fork (→ two
  bounded steps); F-6 `VA-4`'s missing fragment slot (→ `VA-7`); F-7 the
  unfalsifiable trade (→ `PHASE-09 EX-8`); F-8 `Fragment::for_stage` cross-stage
  delivery; F-9 the surviving classifier + an unreconciled count; F-10 the
  falsifier exempting DEC-104. External reviewer: codex.
- **A self-audit pass after round 4 found four more defects the reviewer had
  not raised** — all in the *repairs*, not the artefact — and fixed them
  (`e2a6b8ee`). See *Learned*.
- **IMP-377** — `review dispose` should take `--response-file` / `-F -`;
  authored in the primary tree with the memory and observation (`001aa528`).

### Learned

- **`VA-5`'s subject routing is a *set-disjointness* property, and writing it that
  way is what made it checkable.** The criterion asks that the three delivery
  signals be incapable of arguing for reclassification and that the classification
  signal be incapable of satisfaction by delivery proxies. Stated as prose it is a
  thing a reader agrees with; stated as two namespaced evidence-key sets in
  `collectors.toml` it is one `is_disjoint` call, and adding a single shared key
  fires it by name. The general move: **when a criterion forbids a trade between
  two things, give the two things disjoint vocabularies and assert the
  disjointness** — the prohibition then survives an author who has not read the
  criterion.

- **A data-driven test's first RED is usually worthless, and it looks fine.**
  `tests/e2e_design_evaluation.rs` scores from authored TOML, so its `VA-NC` red
  was *"rubric.toml is not readable"* — which proves the test opens a file and
  nothing more. Data-driven assertions fail **soft**: an empty `BTreeSet` makes
  `is_disjoint`, `is_subset` and `all()` all return true, so every loop and
  comparison downstream of the read was untested. Both load-bearing assertions
  were therefore mutation-checked against a *complete* kit and the mutation
  reverted. Recorded durably as
  `mem.pattern.tests.mutate-the-data-not-just-delete-it`, linked to
  `mem.pattern.tdd.wire-before-guard`, which is the same distinction for code.

- **The fixture must not state its own result.** The obvious shape for a scored
  transcript fixture is to write the score in it; that makes the ordering
  assertion a restatement of the fixture and the negative control theatre. The
  fixtures record only *observed evidence keys* and the test derives the band from
  the rubric — and it asserts that neither fixture carries a `score` or `band`
  key, so the shortcut cannot be reintroduced later. The known-bad evidence set is
  additionally a **strict subset** of the known-good one (asserted), so the
  ordering isolates the rubric rather than comparing two unrelated runs.

- **The repo's closed-vocabulary guards all fire, and each should be closed at
  SOURCE rather than widened.** Three fired on PHASE-16 in one session: the
  writer-act set-equality table named `discharge` as the key production checked and
  the table did not; `every_material_event_kind_persists_a_change_row` demanded an
  exercise for the new event; and `design_prompts_have_no_consumer_outside_the_design_run`
  caught **doc comments** spelling the store name literally. The third is the
  instructive one — the fix was to derive from `STORE`, so the allowlist did not
  widen. A guard widened every time it fires has been converted into a changelog.
  Two of the three were unanticipated by the design, which is ship-and-learn paying
  immediately.
- **Making a guard real breaks every fixture that used to walk past it, and that is
  the deliverable rather than a cost.** The `gate::advance` runbook clause reds 15
  existing tests across `e2e_design_{delegation,state,review}`, all one cause: a
  fixture advancing out of `exploring`. `EX-8` says that without the clause the
  runner is decoration; the fixture fallout is the evidence that it is not.
- **`src/verify.rs` already exists** (engine tier, coverage-verification config,
  `.doctrine/adr/001/layering.toml:73`). The sketch's "`verify` is free of collision
  at head" was about the **command** name and holds; the **module** name does not.
  The `doctrine verify` family's handler must be `src/commands/verify.rs` — a
  distinct path — with a doc comment saying so.
- **A content-binding digest over user prose needs length framing, not a separator.**
  `commands::design::acceptance_digest` joins on `\u{1f}` and is correct for ITS
  inputs (digests and closed labels, none of which can contain the separator). A
  runbook step's `text` is arbitrary project prose, so the same shape would let two
  different definitions encode identically. The two encodings differ deliberately.
- **A derived cursor beats a stored one.** The runbook cursor is the first step with
  no live discharge — no second source of truth, and an edited step rewinds the
  cursor for free while later discharges stay intact. The same rule `ReviewStanding`
  follows: derived on every evaluation, never stored (DEC-066/DEC-067).

- **A raiser's contest rationale is not stored on the finding.** `review contest`
  flips status only; its `--note` lands in
  `.doctrine/state/review/NNN/baton.toml`'s `handoff` array and is documented as
  ephemeral ("durable justification belongs in a finding"). `review show` does not
  surface it. Read the baton before answering a contest.
- **A `VA` that mandates evidence "recorded in the phase sheet" mandates it into
  the disposable tier.** RV-321 F-3 was contested for exactly that gap while its
  fix was landed and working. Carry such output in the finding's *response* too —
  that tier is authored. IMP-371.
- **The eight-state → runtime disposition, measured not guessed** (source for
  PHASE-08's `EN-2`(a)): one state fully moved (write design.md → `materialise` /
  `adopt_authored` / watermark), one correctly merged (propose approaches into the
  inquiry loop), roughly four partial, roughly four dark. **The pattern: Doctrine
  took the state and the invariants; it did not take the checklists or the craft.**
  Detail in the two D2 sketches.
- **`Stage::Exploring` has no fragment of its own** — `Fragment::for_stage` maps
  `Exploring | Inquiring → Fragment::Inquiry` (`src/design_run/prompt.rs:106`),
  and `inquiry.md` covers nodes and dispositions, saying nothing about `/canon`,
  `/retrieve-memory` or `slice research`. The state with the most explicit ordered
  checklist has the least guidance.
- **`design show --format prompt` is the DEFAULT**, and `resume --known-fragment`
  lets a caller declare a fragment it already holds so the run reports agreement
  instead of re-sending. The just-in-time delivery lever is already built.
- **No `design_*` MCP tool exists** (`src/mcp_server/tools.rs` registers
  `memory_*`, `review_*`, `dispatch_*`, `observation_record`, `worker_commit`,
  `doctrine_onboard`). The managed run is unreachable from an MCP-only agent.
- **ADR-019 names no specific hymn** and carries no per-asset `fixed` value — it
  mandates only that every projected asset *declare* a customization. Per-asset
  policy lives in `publication/manifest.toml`; the seal's rationale is **DEC-077**
  ("framework prompt content may therefore remain embedded and authoritative"),
  answering **QUE-191**. So reversing the seal is a superseding DEC plus a
  manifest edit — not an ADR change.
- **The incumbent skill contradicts this project's own guidance**:
  `design/SKILL.md:98` says "prefer multiple choice questions when possible";
  `CLAUDE.md` says the opposite for design loops (prose, because the user often
  reframes the question). Live evidence that craft is at least partly *taste*.
- **A shared-test-helper module needs an opt-in boundary, and `tests/common/`
  has none.** Only top-level `tests/*.rs` compile as crates; a subdirectory
  module is inert until a crate declares `mod X;`. So a sibling module
  (`tests/design_fixture/`) is compiled only by crates that opt in, while
  `tests/common/` is already declared by ~30 e2e crates — everything added there
  is compiled by all of them. That asymmetry, not style, is the placement rule.
  Corollary that made it concrete: `common` is already ~60% concern-specific by
  line count (`marker_free_base`, the ISS-028 worker-fork block), each having
  arrived by "two callers, so it's common". Durable:
  `mem.pattern.tests.shared-helper-placement`; residuals IMP-368/IMP-369.
- **A handover finding that asserts a cardinality is an unverified claim, and it
  propagates.** PHASE-07's F-13 said the design-run bootstrap existed in exactly
  one place; there were seven. I relayed the claim and built a three-option
  analysis on it before the owner questioned the premise. Sheet findings read as
  already-verified because they are written by someone who *was* looking — but
  "the only X" is the same vacuous-absence shape this phase already logged twice.
  Re-run the positive control at the moment you rely on it, not when it was
  written. (F-15; friction `019fb0dd`.)
- **Splitting a lifted fixture by *dependency direction* beats splitting it by
  size.** `Fixture` divided cleanly into a CLI-driving bootstrap (needs only
  `doctrine_bin`/`SLICE_DIR`) and model-reading accessors (need the
  `#[path]`-included `design_run` leaf). Only the first is genuinely shared;
  lifting both would have dragged the leaf into every including crate. Rust makes
  the split free — a type defined in a module of the crate can take a second
  inherent `impl` in that same crate, so crate-specific methods stay local with
  no wrapper and no trait.
- **`record-delta` has no faithful representation for a phase interleaved with
  foreign commits.** One row per phase (UPSERT) and two modes: `--commit S`
  records exactly `[S^, S]`, `--start/--end` records a contiguous range. A phase
  spanning three commits with someone else's commit interleaved can be recorded
  precisely-but-incompletely (VTs go UNATTRIBUTABLE) or completely-but-impurely
  (the range sweeps the foreign commit). PHASE-07 chose the range and disclosed
  it. Worth knowing *before* choosing when to commit: keeping a phase's code
  contiguous is a property you can only preserve prospectively. (F-17.)
- **A gate whose components are payload-claimed cannot be proved to be a
  conjunction — and PHASE-12's anti-theatre criterion is what exposed it.**
  Every `Condition` was satisfied by an `EvidenceDeclaration` the caller supplied
  and `apply` bound to a fingerprint. On that mechanism the `reviewing → locked`
  boundary's four components are indistinguishable: four tests each omitting one
  component would all re-prove `gate::advance`'s filter, which
  `src/design_run/tests.rs` already proves, while the attestation / finding /
  acceptance machinery stayed decorative. The four are now **derived** from the
  run's own review state (`DesignSnapshot::review_standing`), and a payload naming
  one is refused at admission (`Refusal::DerivedConditionClaimed`) rather than
  ignored — a caller that believes it cleared a gate and is silently not believed
  learns nothing.

  The generalisable part: **whether a criterion can be *claimed* decides whether a
  test about it can prove anything.** VA-2 ("a test that omits three components
  and asserts refusal proves nothing about which component the gate reads") is
  what forced the question; the design already said clearance is derived
  (DEC-066/DEC-067), but the code had a claimed path for all ten conditions and
  nothing had yet needed the distinction. Six remain claimed, disclosed at
  `Condition::is_derived`, and at least `materialisation-current` is mechanically
  derivable — the asymmetry is stated, not resolved.

- **Whole-run evidence needs a currency rule, and the pure layer cannot hash for
  it.** A section attestation is live while a stored `Fingerprint` equals its
  subject's current one. An integrated review and a user acceptance are about the
  *document*, which has no stored fingerprint. A composite digest looks obvious
  and is unavailable: the shell builds `DerivedInput` **before** `apply` runs the
  batch, so it cannot supply a digest of the post-batch section set, and the pure
  layer may not compute one. The answer is to compare the **set** —
  `ContentCoverage(BTreeMap<DesignId, Fingerprint>)` against
  `SectionGroup::fingerprints()`. Pure, no injection, no staleness window, and it
  catches an added or removed section as well as an edited one. One owner, two
  users (the integrated review and the lock acceptance).

  Corollary worth keeping: `all()` over an empty set is vacuously true, so
  "every section is attested" needs an explicit non-empty guard or an empty draft
  locks.

- **A refusal's `Display` is a load-bearing surface when the caller is a
  process.** `Refusal::GateNotCleared` carried `missing: Vec<Condition>` and
  printed `"N condition(s) outstanding"`, while `gate.rs`'s module doc promised it
  "names every missing condition … an agent that fixes one and retries should not
  have to discover the rest one round-trip at a time". Both claims sat in the same
  file and disagreed for four phases. `refusal.rs`'s own header says the data is
  the contract and "a test asserts on the variant rather than on prose" — true for
  the pure suite, and **false for every black-box e2e test**, which can only see
  `anyhow!("{refused}")` on stderr. When the surface is a subprocess, prose *is*
  the API. Fixed; nothing asserted the old text.

- **An enumerated-vocabulary test is the cheapest guard in this subsystem.**
  `ChangeEvent::ALL` grew 12 → 16 and PHASE-03's
  `every_material_event_kind_persists_a_change_row` failed immediately, naming all
  four unwired events — exactly the "a new event kind nobody sized cannot slip
  past a hand-picked example" contract it was written for. It is why the four new
  events got fixture coverage in the same change instead of shipping unexercised.
  The same shape (`IdKind::ALL` iterated rather than counted) absorbed 4 → 6 kinds
  with no test edit at all — only stale *prose* about the cardinality, in three
  places, none of which was a compile error.

- **The 228-design corpus is the oracle for PHASE-11's legacy import reader, and
  it falsified the a priori reasoning twice.** Every `design.md` in this repo
  came from one template, so "what shapes does legacy prose actually take" is a
  measurement, not a judgement call. Surveyed all `.doctrine/slice/*/design.md`
  — **228 real files; the raw glob returns 453 because the slug symlinks
  double-count every slice**, which is worth knowing before any corpus-wide
  count in this repo is quoted.

  | shape | total | files |
  |---|---|---|
  | heading-like line inside a fence | 161 | **33 (14.5%)** |
  | fence parity wrong at end of document | — | **2** |
  | heading immediately followed by heading | 421 | **198 (87%)** |
  | degenerate heading (`###`, `## ###`) | 0 | 0 |
  | non-blank bytes before the first heading | 0 | 0 |
  | `#` / `##` / `###` / `####+` | 229 / 2117 / 1742 / 56 | 228 / 228 / 207 / 7 |

  What the measurement decided, against reasoning that had gone the other way:
  1. **Import must split at *every* ATX heading, not at `##` only.** The template
     puts an HTML comment and a status blockquote *between* the `#` title and the
     first `##`. Splitting at `##` alone orphans that front matter into an
     unheaded preamble and would refuse essentially the whole corpus. The
     template's shape decides this; the "don't infer nesting from depth" argument
     reaches the same answer but does not establish it.
  2. **A naive fence-parity toggle is measurably wrong.** It desynchronises on 2
     of 228 designs and classifies genuine `##` headings as code — in both cases
     including the Open Questions section (`## 17. Open Questions` in SL-073,
     `## Remaining open questions` in SL-164), which is exactly what `EX-3` must
     locate. Failure is silent: import succeeds, the section is simply absent.
     The reader needs CommonMark's rule — an opening fence of *n* backticks
     closes only on ≥ *n* — plus a refusal when a fence is still open at EOF.
  3. **Contentless sections are the norm, not an edge case** — 87% of designs
     have at least one, because `## 5. Proposed Design` immediately followed by
     `### 5.1` is what the template produces. Import seats them as legal,
     title-bearing, empty-bodied sections. Reviewers should expect ~2 per import,
     not treat them as a defect.
  4. **The two hazards reasoning *did* surface — degenerate headings and unheaded
     preambles — occur zero times.** Keep the guards (import targets arbitrary
     prose), but the phase's risk was never there.

  **Generalisable beyond PHASE-11:** the `VA-2` hostile fixture is this slice's
  own `design.md`, chosen for adversarial *import semantics* (citations in prose
  about citations). It carries none of the three lexical hazards above. A fixture
  adversarial in one dimension is not adversarial in every dimension, and
  "a synthetic fixture proves nothing" can read as licence to skip the synthetic
  cases that are the only ones reaching the real defects.
- **The corpus survey's finding of "two malformed designs" was itself the defect
  — WITHDRAWN, and nothing is owed against the authored corpus.** The survey
  above reported `.doctrine/slice/073/design.md` and `164/design.md` as having
  unbalanced fences. When PHASE-11's reader landed, it swept all 228 designs and
  **refused none**: 073 imports 37 regions ending in `17. Open Questions`, 164
  imports 29 ending in `Retrievability edge-case` — precisely the two sections a
  parity reading swallows. Under CommonMark's rule both documents are
  well-formed. **The parity analysis was what was malformed, not the files.**
  Three consequences: the owner's open question about filing a backlog item is
  answered — there is nothing to file; the D6 "open fence at EOF refuses" rule is
  hardening that never fires on a real design; and the generalisation is that a
  survey run with a *wrong reader* reports the reader's defects as corpus
  defects, which is indistinguishable from the real thing until a correct reader
  exists. The measurement was right about the parity toggle and wrong about the
  files, from the same run.
- **An absence assertion must be made on the surface its criterion is about —
  three instances on this phase, two of them shipped as green.** PHASE-11's five
  tests were authored red-first, and two of them still asserted the wrong thing:
  `!snapshot_text.contains("OQ-9")` and `snapshot_text.contains("QUE-177")`.
  Section bodies are stored **verbatim** (PHASE-14's own lesson: a digest alone
  made re-adoption unable to adopt prose), so both strings are in the snapshot
  whatever import does — the first criterion was *unsatisfiable* and the second
  *vacuous*, and neither was visible until the feature existed to contradict
  them. The fix is to assert through the model, pinning each node to its
  `(section, line)`. Companion to the earlier finding that an empty-collection
  assertion passes vacuously against an unbuilt feature: **absence is not
  falsifiable without a positive control, and a string search over a store that
  keeps prose verbatim cannot tell "is not a node" from "is not in the file".**
- **A zero-hit-grep criterion needs word boundaries — this slice now has two that
  are unsatisfiable as literally worded.** PHASE-11 `VA-3` greps
  `levenshtein|similarity|jaro|fuzzy|ratio|distance` for zero hits over
  `src/design_run/`; it returns 15 on an untouched tree, because **`declaration`
  contains `ratio`** (so does `operational`). Word-bounded it is genuinely
  zero-hit. PHASE-13 carries the other instance. Two occurrences make it a
  pattern in how these criteria get authored, not two typos: a substring grep
  over English prose in a code tree will hit a longer word, and the criterion
  that ships un-run is the one nobody can satisfy later without a finding.
- **Import cannot distinguish a first adoption from a post-loss reconstruction,
  so it labels every import as reconstruction.** Once the runtime snapshot is
  gone, nothing observable separates the two: same document, no run, and no
  markers if it was never materialised. `EX-6` demands the distinction from exact
  resume be surfaced, and the only honest way to satisfy that is to say it on
  *every* `--from-design` — which is what the design means by "import IS the
  new-run/runtime-loss path". Telling them apart would need a durable trace
  outside the runtime tier, and DEC-084 principle 6 forbids inventing one.
- **The execution mode changed at PHASE-14's close: dispatch → solo-in-worktree**
  (owner's call, 2026-07-29). The case, from this slice's own evidence: both
  substantive defects the dispatch arm produced were **distillation** defects —
  PHASE-13's inverted document-order fixture and PHASE-14's truncated scope set
  were born in the worker brief, a lossy re-encoding of criteria that already
  exist in full in `plan.toml`. Add to that the `.doctrine/` wall (eight `VA-NC`
  transcripts relayed as prose and retyped by the orchestrator), a fork gate the
  worker cannot trust (ten IMP-352 false-reds re-run by the coordinator anyway),
  a funnel that models one worker per phase with no path for a partial landing,
  and ISS-274's stale index firing after four separate funnel verbs.
  **What close still needs, and it is the only coupling:**
  `dispatch sync --prepare-review` gates on a complete **conformance registry**
  (`src/dispatch.rs:3474-3488` — *"record-delta the missing phase(s) before
  audit"*). Normally filled from the boundaries ledger by `conclude`;
  `slice record-delta 233 PHASE-NN --commit <oid>` writes the same row directly.
  Since ISS-241 already forces that call by hand on the claude arm, solo drops
  the funnel without dropping anything close depends on. **Do not skip it.**
- **A distilled worker prompt must quote an anti-theatre criterion verbatim, not
  paraphrase its fixture.** PHASE-13's prompt told the worker to declare `sec-11`
  before `sec-2` for the document-order test. `SectionGroup` is id-ordered and
  `"sec-11" < "sec-2"` lexically, so that pairing makes seq order and id order
  **coincide** — the test would have passed against the exact defect `EX-6` exists
  to close. The plan's `VA-3` said the opposite and was right. What produced the
  error is that the sketch carries a *second*, adjacent fixture — the `sec-1` /
  `sec-11` collision case — where "declare `sec-11` first" **is** correct, for an
  unrelated prefix-matching reason. Two fixtures, opposite orders, one paraphrase.
  The worker caught it on the stated authority order (plan > sketch > prompt),
  which is the only reason it did not land. **The authority order is not
  ceremony; it is the thing that survives an orchestrator's confident error.**
- **`record-delta` is required on the claude arm, whatever `/dispatch-agent`
  says.** Measured twice now: after `reap`, `verify-vt` read PHASE-13 `VT-1`
  UNATTRIBUTABLE; after `record-delta --commit <import oid>`, PASS. Only the
  phase's **new** file needed attribution — `tests/e2e_design_state.rs` was
  already attributed from earlier phases, which is why `VT-2` never showed the
  symptom. **A phase that creates no new file will not reveal this gap**, which is
  what makes it easy to skip and expensive to skip. ISS-241 is the cause.
- **A phase is oversized when its deliverable count, not its difficulty, exceeds
  what one worker can hold.** PHASE-06 passed every readability check and was
  still undoable: ten deliverables, eighteen named tests, fifteen sequenced red
  windows, one commit. The tell was not complexity but **separability** — and the
  plan already carried the test, in the 2026-07-28 note that had split the same
  phase once before. Applying an existing split criterion a second time is
  cheaper than discovering mid-flight that a worker stranded its single commit.
  Splitting cost ~1h of plan revision; the failure mode it avoided is an
  amend-heavy landing where the orchestrator finishes the worker's job.
- **A criterion that prescribes both a payload shape and a red kind must have the
  two checked against each other** (PHASE-06 F-2). `VA-5` mandated the removed
  keys ride *alongside a valid `dispose`*; `VA-NC3` mandated the red be a wrong
  acceptance. For the `record` leg both cannot hold — that payload already hit a
  **second** pre-existing refusal (`CheckpointDispositionConflict`), so its red
  is a wrong *refusal reason*. The `adopt_record` leg does red as an acceptance
  and carried the intent. Generalises: prescribing an input and an expected
  failure mode is two claims about the incumbent, and the second is the one
  nobody executes.
- **A migration list derived from one criterion's grep is not the phase's blast
  radius** (PHASE-06 F-4). `EX-12` enumerated the annotation fixtures — and
  undercounted them (three named, four real). But `EX-13(b)` forced every section
  body to open with a heading, and **no criterion anywhere listed a section-body
  fixture site**. One hid a real trap: a test asserting `sha256(b"draft two")`
  that silently depended on the old body bytes. The rule that changes a *data
  shape* has a wider fixture radius than the rule that changes a *field*, and
  enumerating the latter tells you nothing about the former.
- **A design claim about a tool's behaviour is worth nothing until the tool's
  *relevance* is measured, not just its behaviour.** RV-323 derived three
  grammar rules from prettier's formatting of `design.md`. Nobody asked the
  prior question until rev 5: prettier **cannot parse TOML at all**, so the
  authored entity tier was never exposed; markers survive formatting
  byte-identically, so the structure was never at risk; and prettier is **not
  installed in this repo** — no binary, no config, not a dependency of the one
  `package.json`. The threat model was a hypothesis about a *client* project,
  and it was load-bearing for three revisions without ever being stated as one.
  Generalises past this slice: measure whether the hazard reaches you before
  designing against how it behaves.
- **An enumerated oracle fails the same way an enumerated rule does, and the
  third occurrence is the signal to change the instrument, not the list.**
  RV-323 shipped a non-partition heading table twice, then replaced the *rule*
  with a proved-total decision procedure but left the *oracle* an enumeration —
  and that is exactly where round 5's case escaped. The generated differential
  that replaced it found a defect the contest had not reached, and then showed
  the obvious fix for the contested family closed **zero** of 4,224 divergences.
  Reading cannot produce that result; only an instrument with a positive control
  can. → `mem.pattern.design-run.adoption-is-the-parser-readout` is the sibling
  lesson for incumbent claims.
- **An instrument must reproduce every row of the table it is cited for, and any
  placement the result is sensitive to must be pinned in the instrument.** Rev 5
  published a five-row divergence table and shipped a probe implementing three
  of the rules. The two missing rows carried the argument's load-bearing number
  — the obvious repair closing **zero** — so the one figure the reviewer was
  invited to attack was the one figure they could not re-run. Worse, that result
  is entirely sensitive to *where* the guard sits: on the content region it
  closes nothing (the published figure), on the derived title it closes 1,476 or
  2,560 depending on whether whitespace counts. Reconstructing the wrong
  placement reads as "the published number is wrong" rather than "the guard was
  wrong". Completed at `81e2f4e1` **before** round 7; the raiser then re-ran the
  probe, matched all five rows, and did not contest. Cost of finding it late
  would have been a whole review round lost to an objection wrong on the merits.
  Friction observation on `edge` `9cb36d95`.
- **A property you cannot derive must be withdrawn, not approximated.** Chasing
  formatter stability of a derived title is unbounded — closing sequences,
  cascades, whitespace runs, then inline emphasis — and a per-character probe
  showed no charset can close it, because the instability is a *token-pair*
  property. Only rendered-inline-content derivation would, i.e. a Markdown
  parser. So the claim was withdrawn exactly as the id charset's maximality
  claim was under F-4, and the one rule kept was re-justified on **CommonMark**
  rather than on the withdrawn premise. Corollary that felt like a regression:
  a working one-line repair (whitespace collapse, 832 divergences closed) was
  **declined**, because its only justification was the retracted one.

- **Clearance derived from evidence, never stored, collapses DEC-066 and
  DEC-067 into one mechanism.** A gate condition holds iff *live* evidence exists
  for it, and evidence is live iff its subject's current fingerprint still
  matches. There is then no clearance flag for a regression to un-set, and
  `advance` re-derives the whole cumulative set on every forward move — "no
  inherited clearance" becomes structural rather than remembered. Shipped in
  `src/design_run/{facts,gate}.rs`.
- **A cycle checker over `needs` must separate the DFS path from the settled
  set.** One merged visited-set reports every *diamond* as a cycle, and `needs`
  makes diamonds routine (`a→b`, `a→c`, `b→d`, `c→d`). Caught by mutation, not by
  the cycle cases — the two real cycles still refused correctly under the bug.
- **Measure the `cfg(test)`-dead surface before choosing a `dead_code`
  suppression scope.** A module staged a phase early is entirely dead in the
  non-test build but usually only *slightly* dead under `cargo test` — nine items
  of forty-odd here. A module blanket is the reflex and is ~4x too coarse; it also
  masks later dead siblings, which is the SL-059 failure. See
  `mem.pattern.lint.dead-code-module-level-vs-cfg-attr`.
- **VA-NC is better discharged by mutation than by write-test-first ordering.**
  Mutating the code under test proves the assertion discriminates against a
  *specific* wrong behaviour; "it failed before the file existed" proves only that
  the file did not exist. Eight mutations, eight assertion failures, no compile
  errors — the full table is in the PHASE-02 sheet.
- **The privacy precondition is a module-tree obligation, discharged.** Because
  design §5.5 puts projections *and* the serialization contract under
  `src/design_run/`, the sketch's compile-error guarantee only holds while
  `render` stays a sibling of storage and admission. No constants at the module
  root; the invariant is written into `mod.rs` so PHASE-04 inherits it as text.
- `entity::write_body` (`src/entity.rs:685`, SL-230) is the prose-write
  seam — materialise rides it rather than inventing a writer.
- `src/comparison/wire.rs` is the schema+version precedent to ride;
  revision CAS is net-new (no runtime store has one).
- `entity::materialise` has no caller-visible midpoint between id-claim
  and byte-write — DEC-086 needs a ~20-line callback seam.
- Review ledger transfer list: copy `can()`, avoid `LockGuard` + `Baton`.
- Spec descent = 2 new entities + 2 narrow amendments; the "skill
  contract" is ungoverned (`spec-003.md:50`, `spec-010.md:55`).
- RFC-021 crossover: the prompt-pack fork is the hardening risk.
- **A universal guard in front of a recovery path whose purpose is to
  cross that guard makes the path self-refusing** — RV-315 F-19→F-20.
  The exception must be a protocol, never an informal bypass.
- `with_turn`'s guarantee is not portable as worded: it holds a writer
  lock and hashes the file it replaces. A cross-file, multi-effect
  operation can borrow the two-window *shape*, not the "nothing written"
  *promise* — DEC-083/086 order effects before the snapshot.
- Worktree forks do **not** share the parent's `.doctrine/state` — each
  carries its own real dir (verified: `.dispatch/SL-209`, `SL-231`).
  Refutes the thread-6 claim; fixes the evaluation kit to the coord tree.
- `src/install.rs:941` comments `KNOWN_STAGE_LABELS` "Provisional" — the
  stage-hymn rejection rests on axis *meaning*, not registry closure.
  The comment already states both facts and `design` is already a
  member: PHASE-07 must NOT edit the constant.
- ADR-001 tier registration is `.doctrine/adr/001/layering.toml`, not
  `tests/architecture_layering.rs` — the test auto-discovers units and
  loads the toml. Adding a module is an authored `.doctrine/` edit, so
  PHASE-02 is coordinator-only.
- The incumbent `slice design` is `src/slice.rs:174/449/734`, not
  `src/commands/` — design §5.5's home list omits it.
- `src/vtgate.rs` matches raw case-sensitive substrings AND requires
  the `test_file` to be modified by the slice. A VT mandate naming a
  type the design never named, or a two-token CLI invocation, cannot
  pass a correct implementation.
- Detail with citations lives in `research/research.md` § Round 2.
- **No spec→RFC relation edge exists.** `originates_from` is `Tier::Typed` +
  `TypedVerbOnly` (ADR-014 D2), authored only by `revision new
  --originates-from`, REV→RFC; the `references(originates_from)` role is
  scoped `{SL + backlog}` ↔ `{BACKLOG + SL}`. RFC descent is recorded in
  prose + `governed_by → ADR` — precedent PRD-017 / SPEC-026, which descend
  RFC-021 and carry no RFC edge. The reciprocal `related` lives outbound on
  the RFC; SL-233 does not add one (`.doctrine/rfc/**` undeclared, and it
  would make the descent five entities).
- **`spec req add` writes outside the spec directory** — REQ entities land in
  `.doctrine/requirement/<NNN>/` plus a slug symlink. plan.md's stall sweep
  missed the tree entirely; no slice in the corpus has ever declared a
  requirement-tree selector. Generalises: an entity-authoring verb may write
  outside the entity's own directory, so a selector sweep scoped to the
  obvious path under-covers. Backlog/memory candidate.
- Selector glob grammar: `NNN**` is **uncompilable** ("recursive wildcards
  must form a single path component"). Covering an entity dir + its slug
  symlink needs two rows: `NNN/**` and `NNN*`.
- Conformance over a *wide* range flags `slice-233.toml` itself as
  undeclared — circular, since the first selector append cannot be declared
  in advance. Over the `[S^,S]` boundary shape a registry row records, it is
  clean. Do not mistake the wide-range result for a violation.
- `spec new tech`'s scaffold comment teaches the **malformed** `[[source]]`
  identifier form (`identifier = "doctrine/cli"`). Convention is path-form
  identifier + Rust-path module; SPEC-020 was the corpus's sole deviation
  and misreported as covered until 92d46941. Backlog candidate.
- `dispatch next` has no notion of a coordinator-only phase: with PHASE-01
  done and committed it still prescribes `spawn — PHASE-01`. Settled by hand
  for this slice (see below); captured as IMP-346.
- **Review verbs refused in the coordination tree** — `resolve_review_root`
  guarded on `is_linked_worktree`, a blanket test, so every baton-driving verb
  bailed on `dispatch/233`. That strands all three design gates, whose sketches
  live in the coord tree by ADR-012's topology. Fixed on `edge` as ISS-275
  (test the *role*, not the linkage) and flowed in via `refresh-base`; settled
  as DEC-094. `state_dir` is root-derived, so no baton relocation was needed —
  the coord tree keeps its own.
- **A design gate earns its keep — the sketch was wrong in ways I could not see
  from inside it.** RV-320's seven blockers were all substantively valid, and
  two of them killed claims the document was built on: cardinality caps do not
  imply an encoded-byte bound (a character cap is not a byte cap), and exact
  omitted counts do NOT distinguish a small run from a truncated one when the
  selection stage narrows a universe *before* the cap applies — the count
  measures what the cap dropped, never what selection dropped. Generalises
  beyond this sketch: **any bounded projection that filters before it truncates
  must carry global totals, or its truncation signal is honest about the wrong
  quantity.** Also: a worst case asserted from an average ratio is not a worst
  case (3.5 B/token vs ~1.5 for dense JSON is a 2.3x error on the number R1 was
  resting on).
- **A projection bound must never propagate into the record it projects from.**
  Rev 3 applied `ENVELOPE_PAYLOAD_BYTES` to the *stored* change row, truncating
  regression reasons and fingerprints on disk to buy space in a rendering the
  reader may not request. The `ENVELOPE_` prefix already named the scope and the
  violation still survived two revisions and a review round. The store was
  gitignored runtime state bounded by a retention constant — neither committed
  nor multiplied — so the lossy write bought nothing. Fix shape: separate the
  artefacts (store full, render bounded, `--full` reads the store) rather than
  pick one cap that serves both badly. Recorded as
  `mem.pattern.projection.bound-must-not-bind-the-record`.
  **GENERALISED at round 3** — this is one half of a broader rule, and stating
  only this half is what let rev 4 violate it while quoting it. **The layer
  rule:** a constant's prefix declares the layer it binds, and a layer may bind
  only its own artefacts; and **identity and closed vocabularies are bounded at
  admission (refused), never at emission (truncated)** — only gracefully
  degrading prose may be elided when rendered. The second half is the one this
  entry was missing: it catches a *correctly-layered* constant that still
  destroys meaning by trimming an id. Both halves are needed; each catches
  instances the other misses. Sketch § *The layer rule*, PHASE-03 EX-15.
  The memory item needs updating to the general form.
- **A defect class that recurs after its finding is verified is a missing
  mechanism, not repeated carelessness.** F-1 (bounded container with an
  unbounded scalar inside) was disposed fixed and verified, then recurred
  **five** times: `stage_moved`'s reason, `section_fingerprint_changed`'s
  digests, rev 3's unbounded subject id, rev 4's stored reason bounded "to
  `ENVELOPE_REASON_BYTES`", and rev 4's id cap that truncated identity.
  **CORRECTED at round 3 (this entry previously prescribed the wrong fix).**
  It said the mechanical containment check (PHASE-03 EX-14(c)) was "the response
  that works". It was not, and the reason generalises: **a check that saturates
  every scalar *at* its cap proves a container holds its contents, and can never
  detect that a cap is at the wrong layer or is the wrong kind of bound.** It
  verifies the arithmetic while the premise is what's broken. Instances 4 and 5
  both sailed through it — instance 4 was written *inside* EX-14, the criterion
  authored to stop the class.
- **The test of a proposed mechanism is whether it retro-catches the
  recurrences that already happened.** A rule that explains only the newest
  instance is a patch wearing a rule's clothes. The layer rule was accepted
  because it catches all five; that obligation is now written into PHASE-03
  EX-15(e) so the next author cannot skip it. Corollary: state the falsifier
  with the rule — here, a sixth instance the rule does not catch means widen
  the rule, never patch the instance.
- **State the enforcement mechanism's limit, or lose the next round to it.**
  Rounds 3 and 4 were both lost to claiming more enforcement than existed — a
  containment check that could not fail, then a grep that proved sensitivity to
  one spelling. Rev 8 names what Rust privacy does *not* catch (a copied
  literal) and routes it to STD-001 review. An honest limit costs nothing and
  removes the reviewer's best attack.
- **Check that the mechanism you are riding can express the constraint.** Rev 7
  asserted ADR-001's layering test could enforce a rendering-vs-storage
  boundary. `layering.toml` models one axis — altitude — and pure constants
  classify `leaf`, which everything may depend on: it would have *licensed* the
  violation. One file read falsified it. Riding an existing seam is right (DRY),
  but "an existing checker exists" is not "it checks this".
- **Three failed rounds on one finding is a strategy signal, not an effort
  signal.** Rounds 1–3 of RV-320 F-7 all failed the same way: patch the
  instance the reviewer named, ship it, watch the reviewer find the same defect
  somewhere new. The fix was to change *strategy* (derive the mechanism), not
  to read more carefully. The `/rigour` tripwire — "two failed attempts → the
  third must be a different strategy" — is the generic form and it applies to
  review rounds, not just debugging.
- **Deriving a number honestly usually moves it up.** Rev 3 asserted a 220 B
  change row and a ~320 B payload; deriving every term gave 264 B and 145 B,
  and the saturated table went 15,136 → 15,576 B. An asserted figure that
  survives review is not evidence — F-4 and this are the same lesson twice.
- **Id allocation genuinely collides across trees.** PHASE-01 predicted this as
  R-1; it fired. `review new` in the coord tree allocated RV-319, already taken
  in the primary tree by a live SL-236 review that was still untracked and so
  invisible here. `reservation reach = "local"`. The next `refresh-base` merge
  brought their RV-319 in, which would have collided outright. Recovery has no
  `--id` override: leave the collided scaffold in place so the allocator steps
  past it, re-allocate (→ RV-320), then delete the scaffold. **Check the other
  tree before allocating any id from a coordination worktree.**
- Authoring in the coord tree is selector-bound in a way the primary tree is
  not: `.doctrine/review/**`, `.doctrine/knowledge/**` and `.doctrine/backlog/**`
  carry no SL-233 selector, so anything raised there lands undeclared. Slice
  artefacts (the RV) get an exact-path selector declared first, alone;
  everything else (backlog, knowledge, observations) is authored in the primary
  tree on `edge` instead.

**PHASE-03 (2026-07-29), landed `1f90ae7a`.**

- **A test that never reaches the bound cannot fail — and two mandated ones
  didn't.** VA-NC's mutation requirement exposed both.
  `change_log_floor_is_recorded_not_inferred` left the log with zero surviving
  rows, so an inferred floor degenerated to the right answer and the mutation
  passed; `distinct_ids_sharing_a_long_prefix_render_distinguishably` exercised
  only the subject column, which never routes through `render_value`. Both were
  green, mandated, and proved nothing. Fixed; both now red under their mutations.
  This is the strongest available evidence FOR mandatory negative controls on any
  criterion asserting a bound holds — and it is the same insight VA-7 (saturated
  scalars) and VA-8 (ids differing only after `DESIGN_ID_BYTES` − 1) state in
  advance. Full detail + the 19-mutation table in the PHASE-03 runtime sheet.
- **A half-arm binds nothing, and it fails maximally late.** `dispatch arm-spawn
  --slice N` *without* `--phase PHASE-NN` writes a fork record carrying no
  `(slice, phase)`. Nothing warns at arm, cd, or spawn time; the worker runs to
  completion and only then does `worker_commit` refuse `unprovable-fork`. Cost
  here: ~405k subagent tokens before the failure surfaced. **No rebind verb
  exists by design** (zero-rescue — no verb may guess a fork's phase), so the
  only recovery is the fallback live-worktree import, which loses worker author
  attribution. `src/worktree/create.rs:381` already comments on the shape.
- **A worker cannot trust its own gate run.** 10 integration targets fail inside
  any marked fork with `worker fork (signal: marker): refusing authored write` —
  they spawn the binary without `.current_dir()`, so the child inherits the
  marked worktree. Same commit is `check gate` exit 0 / 106 targets / 0 failures
  on the markerless coord tree. `worker_commit`'s gate clears the marker, so the
  real gate is sound; it is *self*-verification that is unreliable. Durable fix
  is `.current_dir(tmp)` + `.env_remove("DOCTRINE_WORKER")` in those 10.
- **The binary-only crate constrains test placement twice, not once.** No
  `[lib]`, no `src/lib.rs` ⇒ `tests/*.rs` can only spawn the binary. That blocks
  EX-11's in-process hook (settled by EX-17, relocating two tests to
  `src/commands/design.rs`) **and** blocks VT-6/VT-7/VT-8's five *rendering*
  assertions, since PHASE-04 owns `show`. Only the first was caught at plan time.
  The lesson generalises: having found a harness constraint, sweep *every*
  mandated test against it rather than fixing the instance that surfaced first —
  the same widen-don't-patch discipline the slice's own layer/provenance rules
  encode.
- **Privacy held exactly as designed, and the stated limit is real.** All three
  `ENVELOPE_*` are bare `const` in `render/mod.rs` (`:53,56,65`); a storage-path
  reference is `error[E0603]: constant ... is private`. The two `_UNDER_TEST`
  re-exports are `#[cfg(test)]`-gated, so absent from a shipped build. But
  EX-16(c)'s named limit fired immediately: a bare `96` was typed three times
  into `render/mod.rs`'s own test where `ENVELOPE_CHANGE_REASON_BYTES` was in
  scope. Privacy cannot catch a copied literal; review did.
- **Entity ids collide across trees, and a review ledger is the worst place for
  it to land. CHECK THE PRIMARY TREE BEFORE ALLOCATING ANY FURTHER RV.**
  `reservation reach = "local"`, so the coordination tree's counter and the
  primary tree's counter cannot see each other. Both handed out **320**: SL-233's
  projection-bounds review (raised 00:32 on `dispatch/233`) and SL-237's design
  review (raised 12:59 on `edge`), twelve hours apart. Nothing warned at
  allocation on either side.
  - **It surfaced at `refresh-base`, not at creation** — the merge conflicted on
    `.doctrine/review/320/review-320.{toml,md}` and halted. That is the earliest
    it *can* surface, which means the window between a duplicate allocation and
    its discovery is however long the coordination branch runs.
  - **Trunk was already inconsistent before the merge.** `edge`/`main` carried
    references meaning *SL-233's* RV-320 — `backlog-275`, observation
    `019fa925…`, and the memory item
    `mem.pattern.projection.bound-must-not-bind-the-record` (`ref = "RV-320"`) —
    while the entity at `320` on main was SL-237's. So the collision silently
    misdirected landed references; the conflict exposed it rather than caused it.
  - **Resolution:** SL-237's was renumbered to **RV-322** (the later allocation,
    and nothing in SL-237's own records cited it). SL-233 keeps RV-320/RV-321.
  - **Renumbering is hand-work, not a verb** — `doctrine reseat RV-320 --to 322`
    fails structurally on *every* review, because `reseat` reads the alias slug
    through the strict meta reader which demands a `status` field, and a review's
    status is derived from its findings and deliberately never stored (ADR-007
    D-C8). Filed as **ISS-277**.
  - **The ritual, for every remaining RV this slice raises** (per-phase reviews,
    PHASE-12's review choreography, and the close-out audit RV): before
    `review new`, list the review ids in the primary tree and take the next id
    above the maximum across *both* trees — `git ls-tree --name-only main
    .doctrine/review/` alongside the coord tree's own. There is no `--id`
    override, so the only lever is allocating from a tree whose counter is
    already ahead. Cheap to check, and the alternative is a hand-renumber of a
    ledger with inbound references.

- **When the subject has no fingerprint, bind the evidence to the *value*.**
  DEC-066 makes evidence live while its subject's stored fingerprint still matches
  — which works for sections, because a section has one. An inquiry node does not,
  and PHASE-10 needed exactly that question about one ("is this proposal still
  answering the obligation it was given?"). The plan's answer was to derive it from
  the change log; the implemented answer is that the delegation **stores the
  obligation as assigned** and compares the whole node for equality. Three lines,
  total, no retention-window case, and it catches a rewritten *question* — which
  the log-based rule structurally could not, because a question change is
  deliberately not a member of the material-change vocabulary. Generalises: the
  fingerprint in DEC-066 is an *optimisation over* comparing the content, so where
  no fingerprint exists, compare the content and do not invent a digest the pure
  layer may not compute.
- **A second source of declarations belongs in the SAME batch, not in a second
  pass.** Accepting a delegated proposal means applying declarations that arrived
  in an earlier submission. The obvious shape is a second `declare` loop after the
  payload's own; the right shape is to concatenate them and run one loop. That buys
  two guarantees for nothing: `Batch::validate` refuses a subject named by both the
  payload and the proposal as the duplicate it genuinely is (instead of an invented
  last-wins order), and the accepted declarations inherit the existing
  subject-determined row order rather than needing a new clause in `apply`'s
  documented ordering contract. A second pass would have had to answer both
  questions badly.
- **Land the whole wire *before* any guard, so the first red is a wrong
  admission.** PHASE-12 could only produce payload-refusal reds (`unknown field
  'concerns'`) and had to reconstruct wrong-admission evidence afterwards as a
  negative control. PHASE-10 inverted the order deliberately: the types, the wire
  and the happy paths landed first with the two guards absent, so the guarded
  tests' first failures were `unexpectedly succeeded: revision 4 stage drafting`
  and a stale proposal accepted *with its map change applied*. That is the red
  `VA-NC` is asking for, obtained in the natural order instead of staged after the
  fact. The cost is one extra build; the discipline is: **write the mechanism, run
  the test, then write the refusal.** It does not help a test that asserts a
  feature *exists* — that red stays structural, and saying so is part of the
  evidence.
- **A per-item `dead_code` gate discloses; a module-wide one masks its siblings.**
  Narrowing `attestation.rs`'s module-wide `#![expect(dead_code)]` to seven
  per-item expects surfaced seven accessors with **no reader anywhere** — not the
  binary, not any e2e suite — including `AcceptanceAttestation::basis`, which is
  the *auditable* half of DEC-088 and is written but never read back. The handover
  predicted this gate would go *unfulfilled* when delegation went live; it did not,
  because delegation moved to its own module, so the residual dead set stayed
  non-empty and a module-wide gate would have stayed fulfilled while hiding all
  seven. **A blanket that is still "correct" is the dangerous case** — it fails
  only when the module is fully live, which is exactly when it has nothing left to
  tell you. IMP-367.
- **A test that finds its subject as "the only X" is asserting an unstated
  uniqueness premise.** `stored_reason_is_byte_identical_at_any_length` located the
  regression reason as *the first `ValueKind::Prose` term in the whole change log*
  — true only while nothing else stored prose. PHASE-10's attribution and refusal
  reasons ended that, and the test failed for a reason unrelated to what it tests.
  The fix is one filter; the pattern to watch is a locator that works by
  elimination rather than by identity. Sibling of the `ChangeEvent::ALL`
  exhaustive-row test, which *is* meant to fail when the vocabulary grows — the
  difference is whether breaking on growth is the point.
- **Fourth instance of "the `VA-` criterion is unrunnable or vacuous as literally
  worded"** (after PHASE-11 `VA-3`, PHASE-13, PHASE-12 `VA-4`): PHASE-10's `VA-3`
  greps for bare `exec`, which matches *execution* in three doc comments and
  `execute_checkpoint` at its definition and its call site — five hits on a clean
  tree, before the phase changed anything. Criteria ids are immutable, so each
  instance becomes a disclosure carried to close rather than a fix. The cheap
  intervention is at authoring time and it is now filed: **execute every runnable
  `VA-` criterion against the tree at plan time** (IMP-365). Word boundaries and a
  positive control belong in the criterion text, not in an execution-time rescue.
- **Fifth and sixth instances, and the sixth is a NEW failure mode — a criterion
  can be jointly unsatisfiable with its own phase's EX.** PHASE-07 `VA-3`
  ("`git diff --stat src/install.rs` — EMPTY") collides with PHASE-07 `EX-1`
  (seal `stage/design`): the seal set's cardinality was asserted by
  `install::tests_hymns::embedded_seal_set_from_live_manifest`, an inline
  `#[cfg(test)] mod` **inside `src/install.rs`**. Sealing flips 1 → 2 and reds
  that test, so making the suite green necessarily diffs the fenced file. No
  ordering avoids it. **In Rust, fencing a file also fences its inline tests** —
  an absence obligation about production code must be scoped to the *item* (as
  the sibling `EX-6` correctly was), never to the file; escalating item-scope to
  file-scope reads as more rigorous while silently annexing the test module.
  Resolved by a minimal edit confined to that test, re-expressed as membership of
  the two sealed slots rather than a bare count. The *fifth* instance is the same
  `VA-3`'s other half: it is **vacuously satisfiable** as worded, because
  `git diff --stat <path>` reads the working tree, so post-commit it is EMPTY
  whether or not the file was touched — the inverse of PHASE-10's false red, and
  pre-running it *conceals* rather than reveals the defect. Intent-faithful form
  is a two-point range: `git diff --stat <base>..HEAD -- <paths>`.
- **A seal is worth a negative control.** "The sealed slot drops its user twin"
  passes trivially if the band never resolved anything or if projection is empty.
  Proven properly at PHASE-07: `stage/design` was temporarily unsealed and the
  embed rebuilt, and the twin-drop assertion then FAILED with both bodies in the
  resolve output. Two more controls make the same test attributable — an
  *unsealed* disk stage hymn does resolve (the EN-2 probe), and install still
  projects the six `expose`d slots as `.md`+`.toml` starters, so a sealed slot's
  absence is the seal's doing and not an empty projection. Seal and expose are
  observably the mirror pair SPEC-023 claims.
- **`prompt check` enforces `KNOWN_STAGE_LABELS`, measured not assumed:** exit 0
  with a known label, **exit 1** with `unknown stage label "..." in slot
  stage/...`. The stage band's happy path had never run before PHASE-07 (zero
  stage hymns on either root), and the disk overlay root (`.doctrine/hymns/`) is
  the cheap place to exercise it — no rebuild, unlike the embed root.
- **The `install/` embed root is `src/asset_source.rs`, not `src/install.rs`.**
  `#[folder = "install/"] struct InstallAssets` lives in `asset_source.rs`;
  `install::asset_text` is a thin delegate. So reading a *new* install asset adds
  no diff to `src/install.rs`, and the RustEmbed re-embed footgun is fixed by
  `touch src/asset_source.rs` — touching `src/install.rs` would be both wrong and
  (at PHASE-07) criterion-violating. design.md:419 and spec-023.md:53 both name
  `install.rs` as the asset-access/loader home, which reads as a conflict and is
  not one.
- **`publication/manifest.toml`'s `customization` is a closed two-value
  vocabulary** (`customizable` | `fixed`, `CustomizationStatus`) — "sealed" is not
  a value. A sealed hymn and a code-owned store with no user override are `fixed`;
  SL-233 authored that manifest's first `fixed` entries. `kind` is likewise closed
  (`template`/`reference`/`guidance`/`integration`), while the `address` *prefix*
  is free-form modulo path-traversal rejection — so a new logical category
  (`design-prompts/`) needs no code change.
- **Repairing a finding reproduces the finding's own defect more often than is
  comfortable — audit the repair, not just the artefact.** Three instances in
  RV-325: the F-7 remedy exempted DEC-104 from the falsifier F-7 had just forced
  into existence (caught as F-10); the F-3 amendment's own warrant table
  reintroduced *"evidence available in principle"*, the construction F-3 killed;
  the F-6 fix extended `VA-4`'s vocabulary by the one kind F-6 named, leaving the
  boot snapshot — a destination the F-4 fix had just created — with no slot. The
  last is F-9's *fix the class, not the instance* recurring inside the repair for
  F-6. **Post-repair self-audit found four defects a four-round external review
  did not.** The cheap version is a sweep for the withdrawn vocabulary plus a
  re-read of each amended criterion against the census it must be total over.
  **Now SIX instances, and rounds 5–6 supply the two clearest.** F-12: the
  round-4 re-anchor fixed the *evidence* (no more intent-reading) and left the
  *consequence* overclaimed, which is the defect `VA-5` exists to catch, inside
  the repair meant to satisfy it. F-13: the F-12 remedy stopped the signal
  over-routing to reclassification **by adding an exit that routes it nowhere**
  (`stand`) — a finding said the evidence supported more than one consequence,
  and the fix added a consequence supported by none. **The pattern has a shape:
  each repair overcorrects along the axis the finding named and lands off it in
  the other direction.** Checking a repair therefore means asking what the fix
  now *over*-permits, not only whether it stopped the cited failure.
  **Now SEVEN, and the seventh has TWO FACES because one repair narrowed two sets
  at once.** Round 6 removed `stand`, and in the same move (i) left **reword** as
  an exit that revises no *placement* — defended by broadening `VA-4`'s noun from
  *placement* to *placement-or-boundary-contract* (F-15) — and (ii) replaced
  `stand`'s **unbounded** notion of anomaly with **three named events**, omitting
  the commonest one (F-14). **Durable rule: when a repair replaces an open-ended
  thing with an enumeration, the enumeration is the new defect surface — ask what
  the open-ended version covered that the list does not.** And the companion:
  **a resolution set is not pre-registered until the CHOICE BETWEEN ITS ARMS is.**
  Naming two admissible outcomes and leaving the selection to judgement is
  route-to-the-cheapest-revision with the routing step renamed; the manoeuvre
  migrates to whichever link is still unpre-registered.
- **A ruling landed in the artefacts that ARGUE it and not in the records that
  GOVERN it — and the two-tier rule made that half-invisible.** RV-325 F-11: the
  set-mode deferral was written into the sketch, the diagram, DEC-104 and
  IMP-373, while **DEC-101's body still assigned `set` to PHASE-08** and PHASE-16
  `EX-7`/`EX-14` still moved it there. Contested, because the repair then amended
  DEC-101's **prose tier only** — `record-101.toml`'s consequences still carried
  the live structured assignment, so `knowledge show` emitted the assignment
  *beside its own withdrawal*. **The two-tier reading rule this project states
  everywhere was broken in the record used to state it.** Durable rule: amending
  a knowledge record means amending **both tiers**, and the check is
  `doctrine <kind> show <ID>` grepped for the withdrawn phrase — never the `.md`
  alone. Second durable half: a **post-repair** sweep for the withdrawn phrase
  found a fifth surface (`sketches/runbook-runner.md`) that neither the finding
  nor the contest cited. Sweep *after* repairing, not before — the repair is what
  tells you the phrase to sweep for.
  **Round 7 supplied the third recurrence and the scope rule.** The round-6 sweep
  ran `deferred to PHASE-08` + `on merit` over `plan.toml`, `sketches/` and the two
  knowledge dirs — a good pattern over a bad scope. It missed `src/` entirely and
  `plan.md` (only `plan.toml` was listed), and `runbook.rs:455` **would have
  matched** had `src/` been in scope. **Durable rule: a claim-withdrawal sweep is
  repo-wide or it is nothing** — `grep -rn` from the repo root, minus `target/`.
  The false claim had already reached shipped source and the plan prose. The
  round-7 sweep, run that way, found **three surfaces neither the finding nor the
  contest cited**, two of them in files an earlier repair had already touched:
  `runbook.rs:1137` (a *third* site in the file), `runbook-runner.md` §9, and this
  file. **Corollary that cost the most:** the F-14/F-15 repair had to amend
  **DEC-104's structured tier**, which carried the whole reopening condition —
  leaving it would have been F-11 leg (a) reproduced inside the repair for two
  other findings. Before repairing a *ruling*, ask which record **governs** it, not
  only which artefacts argue it.
- **A grep for a withdrawn phrase misses its cousins.** F-9's classifier survived
  at five sites; two used *"leaves a trace"*, which asks whether an artefact
  exists afterwards — the same proxy in softer words, and invisible to a search
  for the exact rule.
- **DEC-078 fragment suppression is CALLER-DECLARED.** The digest already rides
  every emission as a `name@digest` header (`prompt.rs:27, :89`); the body is
  omitted only when a caller declares `--known-fragment <name>@<digest>` back. So
  an agent can suppress a fragment by claiming a receipt for bytes it never read,
  and Doctrine cannot distinguish that from a genuine hold. This is the actual
  weak point in every-turn fragment delivery, and it bounds what a mechanism-side
  remedy can be.
- **`review dispose --response` built as a double-quoted shell string silently
  eats backticked spans.** Exit 0, success receipt, damage only on a discarded
  stderr — and the turn-based ledger makes it permanent (`answered` refuses
  rewrites, there is no amend verb, `unlock` only clears a stale lock file). Live
  on F-9. Build the prose in a file and invoke via `subprocess`; read it back
  with `review show --json`, because the receipt is not evidence about what was
  stored. `mem.pattern.doctrine.review-response-shell-backtick-mangling`, IMP-377.
- **A reseat does not sweep gitignored runtime state, and that state is exactly
  what a review campaign reads.** The `DEC-099 → DEC-105` move on this branch
  swept every tracked file, but the 16 runtime phase sheets still carry 13
  `DEC-099` occurrences that now name a different decision. Sheets are the dedup
  corpus's largest single input (506 KB), so the stale id would have been read as
  current by every raiser. The authored tier's eight remaining `DEC-099` mentions
  are narrative *about* the move and are correct as written — which is why a
  blanket rewrite would have been wrong here too. Recorded as trap 0 in
  `prior-findings.md`.
- **The RV venue rule is a convention nothing enforces — hold it yourself.**
  `resolve_review_root` (`src/review.rs:2014`) refuses only a worktree *fork*;
  otherwise it returns the **local** root. `ISS-275`/`DEC-094` made the test the
  role rather than mere linkage precisely so a coordination tree could host
  reviews, which is where ADR-012's topology puts the design gates. So a review
  verb run from coord writes to coord. `RV-341`/`RV-342` landed in the primary
  tree only because the invoking shell happened to be `cd`-ed there — not
  because anything routed them. Pass `-p /workspace/doctrine` explicitly on every
  campaign `review` verb. Caught when `review raise RV-341` from coord failed
  with "review 341 not found at `.dispatch/SL-233/.doctrine/review/341/`" —
  which is also the cheap way to test the claim before trusting it.
- **An entity's body can be tier-split across trees, and the empty side is
  indistinguishable from a legitimately empty body.** `IMP-024`'s nine-section
  method notes (5,508 B) were authored on `edge` after the dispatch fork, so the
  coord tree holds a 353 B scaffold stub. `backlog show IMP-024` from coord
  returns a header and nothing else — no error. The review campaign cites
  `IMP-024` §1/§2/§3/§5 as its method authority and is executed from coord, so
  read it via `-p /workspace/doctrine`.

### Open

<!-- ids + one clause, never a status — query the CLI for current status.
     Settled entries drop at each pass; git holds the history. -->

**THE AUDIT'S CODE-REVIEW LEG IS SCOPED AND RULED — see
`.doctrine/slice/233/review-campaign.md` (authored, 2026-08-02).** It is more
than one agent session: 26,444 lines of churn across 16 phases, 71% never
code-reviewed. The ruling in one line — `RV-320`/`323`/`325` are **design**
reviews and buy zero code coverage, so PHASE-02/06/08 are code-dark *and* carry
the highest design bar; their 30 verified findings become a conformance
checklist. Two ledgers (conformance-to-design; ordinary code review), five
sessions, cheap/top tiering per `IMP-024`. PHASE-01 and PHASE-09 are zero-`src/`
and are not review work at all. Venue rules per artefact kind are in §5 — RV
ledgers mint in the PRIMARY tree, because `ISS-277` means an RV collision cannot
be reseated.

**S0 IS DONE (2026-08-02).** Both ledgers are open in the primary tree —
**`RV-341` Kind A** (conformance to reviewed design, PHASE-02/06/08) and
**`RV-342` Kind B** (the dark phases). Briefs are written. The dedup corpus is
`.doctrine/slice/233/prior-findings.md`, authored on this branch: 34 item lines
built from the 60 verified findings across the six terminal ledgers, 112 items
censused from the runtime phase sheets, 67 from `### Learned`/`### Open` above,
and the open backlog. **Inject it verbatim into every raiser** — that is the
whole point of building it once. Raw censuses are at
`.doctrine/state/slice/233/campaign/s0/` (disposable).

**S1 (KIND A) IS DONE (2026-08-02) — `RV-341` carries three findings**, one
major and two below. Three confined read-only raisers ran, one per phase, at
`PI_THINKING=high`; 30 findings checked across `RV-320`/`323`/`325` and 36
sketch commitments examined. Raw passes at
`.doctrine/state/slice/233/campaign/s1/`. Two things the next session needs:

- **`RV-341` F-1 is the one that matters** — `RV-325`'s authoring gate (F-16
  rule 1, F-17's limb split) requires every 2a step to state its truthful
  completion condition *in its text*. PHASE-08's own three runbooks do (2/2,
  1/1, 3/3); the pre-existing `exploring.toml` is 5/0 and four of its five state
  an act. Their conditions live only in `thin-adapter.md`'s gate table, so
  PHASE-09 collects adherence against conditions never delivered to the subject.
  Disposition may reasonably be that the gate binds the design set rather than
  the shipped asset — **criteria are the owner's**.
- **Eight of `RV-325`'s seventeen findings are UNCHECKABLE against code** (F-6,
  F-7, F-9, F-10, F-12, F-13, F-14, F-15): their responses promised `plan.toml`
  criterion amendments or sketch text, not implementation. That is a real fact
  about the slice's most heavily reviewed gate — 59 rounds bought mostly design
  and criteria movement — and it belongs in the audit's spec-coherence leg,
  where those `plan.toml` and PHASE-09 promises can actually be checked. The
  code-review funnel structurally cannot see them.

**Next is S2** — Kind B, `RV-342`, PHASE-04 + PHASE-16 (the heavy pair, 30 src
files, 6,366 churn); the campaign flags it as the largest and possibly worth
splitting. **`RV-341` has had no top-tier (codex) adjudication pass** — the
cheap tier reported PHASE-06 and PHASE-08 clean, and F-1 came from overturning
one of its HONOURED rows by hand. Treat those two clean reports as unconfirmed.

**LANDING SL-233 IS A SEPARATE PROBLEM FROM AUDITING IT, AND IT IS THE HARDER ONE.**
`dispatch status --slice 233` reports **`sync: not yet run`** and **trunk moved
195 commits past the fork-point**. Shape: 230 commits ours, 195 theirs,
merge-base `e8e7ca73`, **20 files touched on both sides**. `edge` is 9 ahead of
`main` and must be promoted before any integration.

- **SIX DEC ID COLLISIONS — `DEC-099` through `DEC-104` name DIFFERENT DECISIONS
  on each branch. RULED AND PART-EXECUTED 2026-08-02.** Not a content conflict a
  merge can resolve: one id, two allocations that never saw each other. Ours are
  this slice's load-bearing classifications (`DEC-104` *heuristic sets are stage
  framing, not runbook checklists* is the decision the whole PHASE-09 evaluation
  kit exists to falsify); theirs belong to the spike-capsule work (`DEC-104`
  *capsule admission reuses conflict semantics only*). **A naive merge silently
  repoints every citation.** This is `ISS-279` (reservation reach is local; two
  trees allocate the same id unwarned) firing a second time — the first was
  `RV-320`, which the owner moved on `edge` by hand.

  Counts below are **whole-token occurrences over tracked files**, `git grep -w
  -o` per branch — they supersede the earlier line-based 308/104 figures, which
  over-counted. Every row favours the same side, including `DEC-100`, earlier
  flagged as near-even:

  | id | ours | theirs | ruling |
  |---|---|---|---|
  | DEC-099 | 26 | **48** | move OURS → **`DEC-105`** ✅ `b8cfbf3d8` |
  | DEC-100 | **15** | 6 | move THEIRS → `DEC-106` ✅ `791a53923` |
  | DEC-101 | **80** | 24 | move THEIRS → `DEC-107` ✅ `791a53923` |
  | DEC-102 | **69** | 11 | move THEIRS → `DEC-108` ✅ `791a53923` |
  | DEC-103 | **69** | 17 | move THEIRS → `DEC-109` ✅ `791a53923` |
  | DEC-104 | **134** | 12 | move THEIRS → `DEC-110` ✅ `791a53923` |

  **ALL SIX ARE DONE.** Verified after: the branches share decision ids
  `090`–`098` only, and every shared id names the same decision on both.
  `dispatch/233` holds `100`–`105`; `edge` holds `099` and `106`–`110`. Zero
  intersection in the contested range — **the id blocker on integration is
  cleared.** (`DEC-098` differs in *title* across branches — same slug, same
  created date, one decision edited twice. An ordinary merge conflict, not a
  collision.)

  **Status is asymmetric and it corroborates the count:** ours `DEC-099` was
  `superseded` (`DEC-092 → DEC-099 → DEC-100`), so its citations are historical
  narrative; theirs is `accepted` and live — cited by SL-241's in-flight
  `design.md`, an open `RV-340`, two memories, an observation, and the
  spike-capsule fixtures. The dead record moved.

  **A per-tree bulk rewrite of the citations is WRONG BY CONSTRUCTION and cost a
  25-of-70 revert.** `edge` carries SL-233's *own* items — `IMP-370`, `ISS-283`,
  `ISS-284`, `ISS-285`, `ISS-287`, `ISS-289`, `RFC-026`, two memories, an
  observation — citing SL-233's `DEC-099`–`DEC-104`, **interleaved** with SL-241
  items citing the spike-capsule records at the same ids. The tree a citation
  lives in does not tell you which decision it means; classification is per-hit
  and in context. SL-233's own refs to its `DEC-099` were separately repointed
  to `DEC-105`.

  **Slash-compressed id runs defeat any whole-token scan.**
  `DEC-099/101/102/103/104` carries only one `DEC-NNN` token; the rest are bare
  numbers. Two live instances survived a `git grep -w` sweep that reported the
  tree clean. Match `DEC-[0-9]{3}(/[0-9]{2,3})+`. Both defects are on `ISS-292`.

  **`doctor` now reports `DEC-103` unresolved on `edge`, and that is an
  improvement.** Before the move that citation *resolved* — to the wrong record.
  The remaining SL-233 `DEC` refs on `edge` stay unresolved until integration,
  the same split-corpus condition as `ISS-289`/`290`/`291`.

  **`doctrine reseat` moves the entity but NOT the citations — and its own
  dangler report cannot be trusted as the worklist.** It renames the dir, sets
  `id` in the TOML, swaps the alias symlink, then `bail!`s with a list scanned
  from `.doctrine/**/*.md` only. Measured on the `DEC-099` move, that list:
  **missed every `.toml`** — including `record-092.toml`'s `superseded_by` and
  `record-100.toml`'s `supersedes`, the two structured relations that actually
  bind — **missed `src/commands/design.rs`**, **double-counted** every hit
  through the slug alias symlinks, and **listed gitignored runtime sheets**
  (`phases/phase-15.md`) because the `phases` symlink defeats its
  `.doctrine/state` disposability check. Build the worklist with `git grep -l -w`
  over tracked files; use reseat's output only as a cross-check. Raised as
  `ISS-292`.
- **The other 14 overlapping files:** `install/routing-process.md`,
  `plugins/doctrine/skills/handover/SKILL.md`, `src/commands/cli.rs`,
  `src/commands/guard.rs`, `src/state.rs`, `tests/common/mod.rs`,
  `tests/e2e_claude_install.rs`, `tests/e2e_subcommand_help.rs`, and the DEC
  record bodies. Ordinary conflicts; enumerated so nobody re-derives them.
- **THE CORPUS IS SPLIT ACROSS TWO TREES AND AN AUDIT RUN FROM THE WRONG ONE
  DRAWS FALSE CONCLUSIONS.** `ISS-289`, `ISS-290`, `ISS-291`, `IMP-373`,
  `IMP-374`, `IMP-376` were raised BY this slice but authored in the PRIMARY
  tree, so they do not exist on `dispatch/233`: `backlog show` returns nothing
  for all six from the coordination worktree and resolves all six from
  `/workspace/doctrine`. Verified both directions. The unresolved-citation
  warnings `check gate` emits here are that, not corpus rot.
- **The conformance surface the audit must disposition:** 159 conformant,
  **13 undeclared**, **11 undelivered** (`slice conformance 233`). The undeclared
  set includes `src/commands/verify.rs` and `tests/runbook_fixture/mod.rs` —
  real code no selector claims.
- **`verify-vt 233` exits 1 on PHASE-15 `VT-2`** — `ISS-289`, an audit judgement
  deliberately left unrepaired. A disposition for the audit, not a defect to fix.
- **`VA-4` vs `EX-8` round 4 (F-10)** — flagged in PHASE-09, unamended.
  Criteria are the owner's.

**PHASE-09 IS DONE AND THE SLICE IS AT `audit`.** (Superseded: this block previously read "done but for `VH-1`"; the owner accepted it 2026-08-02.) T0–T9 complete at `25350b07`;
**16/16 phases `completed`**. Two things are the owner's:

- **`VH-1`** — *the user accepts the rubric as measuring something they care
  about, before CHR-049 spends a live human-moderated session against it.* Not
  self-certifiable, so PHASE-09 stays `in_progress`. `evaluation/rubric.md` closes
  with the three places to push on if it measures the wrong thing: the five
  classes are RFC-021's and inherited rather than chosen; the four bands make
  `claimed` cheap on purpose; and `adhere` is mechanically checkable over five of
  nine obligations and no more.
- **`VA-4` vs `EX-8` round 4 (F-10) — flagged, deliberately not fixed.** `VA-4`'s
  literal example expects the *adherence* signal (a delivery signal) to revise the
  placement; round 4 ruled that a category error and `VA-4` was never amended. The
  kit satisfies `VA-4`'s **substance** — every signal names a consequence that
  changes something specific, and the signal set names a reachable placement
  revision via the classification signal — without giving the delivery signals a
  placement consequence, which would re-insert the F-10 incoherence to make a
  checklist tick. **Criteria are the owner's**; the phase flagged it rather than
  amending it.

**PHASE-08's `EN-2` IS MET. RV-325 closed terminal at `73139a61e` — 17/17
`verified`, 59 turns, eleven rounds. Nothing on the governance axis is owed.
PHASE-08 is `in_progress`; then PHASE-09.**

- **T7 and T8 are DONE; `VH-1` is with the owner and T4 is still the STOP.**
  Every agent-mode criterion (`VA-1`…`VA-3`, `VA-5`…`VA-7`, `VA-ES`, `VA-G`,
  `VA-NC`, `VT-1`, `VT-2`) is discharged at the final commit `def8b809`. The
  phase stays `in_progress` deliberately: `EX-7` (the handover adapter) is unmet
  and `VH-1` is not self-certifiable.

- **The disposition table found a genuinely undelivered disposition — `:194`.**
  `(a.3)` assigns incumbent `SKILL.md:194` (*"the design doc is canon for design
  intent"*) to the sealed stage hymn, warranted at length in the sketch. **T1's
  checklist had no hymn row**, so it was never delivered: `install/hymns/`
  carried zero changes while eight other `install/` files moved. Shipped at
  `def8b809` with the full re-embed tail. **The durable lesson is about the
  criterion, not the miss:** `VA-4`'s *"a table that does not sum to the
  incumbent is incomplete"* is usually read as an anti-relocation rule, and it is
  also a **delivery audit** — the only artefact that compares the census against
  what is actually on disk, row by row. A task checklist derived by hand from a
  census can silently drop a row; the table cannot, because the row has nowhere
  to hide. Eleven of twelve rows verified delivered, one did not, and nothing
  else in the phase would have caught it.

- **Measured, not asserted (`VA-7(4)`):** the skill loses **157 lines / 7,610
  bytes** (−73%/−75%); the ten-asset shipped corpus gains **72 lines / 3,835
  bytes** (+17%/+18%). 11,445 bytes arrived across nine destinations against
  7,610 leaving, the excess being real duplicated delivery (`:203`, `:115-116`,
  `:144-161` each land twice) plus craft the incumbent only gestured at. *The
  gross corpus grows while the skill shrinks* is now arithmetic.

- **`(d)`'s leaving table mis-files one row.** It accounts for `:59-60` (the
  `## Process (detail)` heading) as retained; the rewrite has no such heading and
  could not — it is scaffolding for the deleted state machine. 2 lines / 21 bytes
  move *retained → deleted*; `(d)`'s "195 lines leaving" is **197**. Neither sum
  is disturbed. Recorded rather than absorbed, since a table that quietly
  reclassifies a row to make itself sum is the defect the rule exists to catch.

- **ISS-291 raised** (primary tree, `ebd0f695`) — `SKILL.compact.md`, 81 lines,
  untouched by this phase and outside `(a.3)`'s census, still teaches the
  eight-stage machine. Correctly out of scope for every `EX-`, but it ships, and
  PHASE-09 `EX-8(a)` wants a *"pre-SL-233 skill baseline"* — which is what this
  file accidentally is. Three options recorded; owner's call.

- **~~BLOCKER~~ SETTLED — the runbook step-id bound. Owner ruled (d); shipped at
  `3516a8b3`; T1 is unblocked and green.** The blocker was framed as two
  defensible bounds disagreeing (a 64 B step id recorded through a 16 B `Label`
  term). It was not. `ValueKind::Label`'s contract is *"a **closed vocabulary**
  label"* and all seven other `PayloadTerm::label(` sites pass an enum's
  `as_str()`; `run.rs:1441` alone passed an open-vocabulary, project-authored
  string. Decisively, `ChangeEvent::StepDischarged`'s **own declared shape**
  (`change_log.rs:215`) already said `(PayloadKey::Step, ValueKind::Token)` — the
  construction site had drifted from its event's schema. So: the call site now
  builds a `token` (bounded by `DESIGN_ID_BYTES` 32, whose provenance is *"the id
  slot in a change payload"*), and `RUNBOOK_STEP_ID_BYTES` is now **derived** from
  that same constant instead of independently chosen, so the ends cannot drift
  apart again. `inquire.knowledge` keeps the id diagram C names for it.
  `EX-14`'s arithmetic is untouched and that was *verified, not argued* —
  `rendered_payload_fits_its_cap_for_every_event_kind` saturates each term at the
  kind its event **declares**, so it has been proving `step_discharged` fits at
  32 B all along. That arithmetic was the scope worry that made option (b) look
  expensive; it was never owed. Detail in the sheet, `F-P08.2`.

- **A handover claim proved false — `e2e_design_review` was RED at `78d00a07`
  (7/74), not green.** The packet and that commit both said *"one RED test and
  nothing else"*. T1's runbook wiring changed behaviour for **three** suites; the
  prior session found and repaired two at their subject and missed the third,
  having verified with targeted runs rather than one full-suite run. Proven
  pre-existing rather than assumed — both changed files were reverted to their
  `78d00a07` content, rebuilt, run to identical failures, and restored. Repaired
  on the seam the suite already had (the seed discharges each edge's step list as
  it reaches it). **Read the general lesson, not just this instance:** a packet's
  test-state claim is what the successor attributes every later failure against,
  and it is cheapest to produce correctly (one `--no-fail-fast` run) and dearest
  to consume when wrong. Recorded as an RFC-011 friction observation.
  Sheet: `F-P08.3`.

- **Two owner rulings on T6, and one of them changes what `VA-3` can run.**
  **`EX-5` is discharged by DELETION, not refresh** (`0b00d789`): nothing
  projects `.doctrine/routing-process.md` any more, so it sat 83 diff-lines
  adrift missing six routing rows, and refreshing it would only re-create a file
  whose sole future is to go stale again. A reader's route is now `doctrine
  library show reference/routing-process.md`. Swept before deleting — `memory/`,
  `install/`, `plugins/`, `AGENTS.md`, `CLAUDE.md` hold **zero** references to
  it; the boot sector's two hits both name `install/routing-process.md`, the
  embed source, which stays; every other mention is this slice's own artefacts or
  a historical record that must not be rewritten. CHR-043 independently proposes
  gitignoring the whole family of such projections.
  **⚠ Consequence for T8: `VA-3`'s grep target list names the deleted file.** Run
  the sweep over the surviving three paths and *say so*. Its *"two real hits,
  both must move"* resolves as: the first moved (T3), the second is gone with its
  file — a stronger discharge than moving it, but it must be stated, not elided.

  **`EX-8` is DEFERRED to CHR-049.** Both distribution channels install from the
  **github remote at `main`** — Claude by plugin, the rest by `npx vercel/skills`
  — so local content is not installable until the slice lands. Regenerating the
  lock now would record a `computedHash` against `sourceType: github` for content
  not on `main`: a hash no consumer can fetch. The harm `EX-8` names (uncommitted
  drift contaminating `git status`) equally cannot arise pre-close, since nothing
  regenerates the lock locally — the tree is clean. **Not amended in `plan.toml`,
  deliberately:** the criterion is right in general and only inapplicable before
  landing; amending it would lose that for the next slice that edits a `SKILL.md`.
  *Method warning:* the lock's `computedHash` is **not** sha256 of the `SKILL.md`
  bytes — that model matched **0 of 35**, including untouched skills. It cannot be
  hand-verified or hand-edited, only regenerated by `npx vercel/skills`.

- **ISS-290 raised — the class defect under both of the above is unguarded.**
  `ChangeEvent::payload_terms()` declares a `(PayloadKey, ValueKind)` shape per
  event and **nothing checks construction against it**: `ordered` reads it for key
  order only, and VA-7's containment test saturates *from* the declared kinds
  rather than comparing them to anything. That is how `F-P08.2` stayed latent
  since PHASE-16. A live sibling remains and is **deliberately unrepaired**:
  `StepDischarged` declares `Outcome` as `Token` while `run.rs:1442` builds a
  `label`. Harmless today, but genuinely ambiguous which side is the error —
  `Outcome` *is* a closed vocabulary, so `Label` may be right and the declaration
  wrong (cf. `CheckpointDisposed`'s `Disposition`) — and picking without a ruling
  entrenches a guess. Sheet: `F-P08.4`.

- **The live surface is now implementation, not epistemics.** The authoring gate
  is written *and applied* (round 11); what remains untested is whether PHASE-08's
  actual assets can carry the nine stated completion conditions when authored.
  That is the first thing that could falsify DEC-104 for real, and it is an
  implementation question.
- **Unchanged and still open**: DEC-102's present-tense overclaim (proposed
  across six rounds, never ruled on); nothing in `design-prompts/` is
  project-overridable (**IMP-375**, flags `design.md §7`); `EX-7`'s handover
  adapter lands in PHASE-08 **unsketched and outside `EN-2`'s gate**; **ISS-289**
  (PHASE-15 completed with an unmet `VT-2`, an audit judgement); **`VH-1` is the
  owner's** and is *stricter* than written, since D10 is reversed.

- **DECISION 1 — `set` mode: SETTLED. Keep `sequence`; withdraw the "on merit"
  claim as false; defer IMP-373 on a warrant that survives inspection.** The
  claim that fell: the sketch said *"the DEC-101 tension is gone rather than
  conceded"* after D14 turned the two coverage-set checklists into fragment
  prose. True premise, bad conclusion — F-5 then settled edge 2 at two steps and
  edge 4 carries three, so **five order-independent steps ship under `sequence`**
  (`run.rs:1369-1377` refuses any discharge not naming the step at the cursor).
  DEC-101's own precondition for `set` — *real instances to design it against* —
  is therefore **met**, and `set` is deferred in spite of that. The surviving
  warrant is **F-5's rule applied to itself**: `set`'s admission is six lines,
  but its **render** — a cursorless runbook under `EX-14`'s token bound — is
  unsketched and outside `EN-2`'s gate, so shipping it in PHASE-08 would hand a
  design choice to implementation. The tension is **conceded and judged cheap**
  at reduced scale, never argued to be meaningful (that is the F-2 move).
  **Repayment is evidence-triggered:** observed `DischargeNotAtCursor` refusals
  (`refusal.rs:198`) against the edge-2/edge-4 runbooks — collected in PHASE-09's
  exercise, *not* the run record, since a refusal aborts the write and the change
  log records only applied changes. Landed in D7's amendment (the diagram
  governs), sketch (a.3), DEC-104's consequences, and IMP-373's body.
- **DECISION 2 — the misclassification signal: SETTLED. Both halves re-anchored
  from intent to observable events.** The old wording asked a moderator to judge
  that a step *"cannot honestly be complete"* and that agents *"treat"* a lens as
  gate-worthy — inferences about a mind, no adjudicator, no rubric anchor, which
  is F-3's criticism of the original discriminator reappearing inside the
  falsifier meant to redeem it. Now: **(i)** *a 2a step was discharged at a turn
  where its subject did not yet exist in run state* — a run-state predicate,
  **scoped** to the steps where it is checkable (selector recording, knowledge
  capture) and **disclosed** as uncollectible for `inquire.scope`'s *"or
  explicitly confirm"* branch, which by design leaves no artefact; **(ii)** *the
  agent explicitly asked whether a lens was complete, or requested a gate on
  one*, **quoted verbatim** — a speech act, not a read of intent. Half (ii)
  cannot be run-state anchored at all (a lens has no discharge event) and was
  **kept rather than dropped**: dropping it leaves the falsifier one-directional,
  able to detect *2a-was-never-a-step* and never *2b-should-have-been-2a*.
  `PHASE-09 EX-8`(d) and `VA-5` both amended (appended, ids immutable).
- **RV-325 round 5 repaired both settlements; F-9/F-10 verified, F-11 and F-12
  raised and integrated.** **F-11 (blocker)** — Decision 1 landed only in the
  sketch and diagram while **DEC-101's body still assigned `set` to PHASE-08**
  and **PHASE-16 `EX-7`/`EX-14` still moved the mode and its cursorless render
  there**. `EN-2` cannot narrow an accepted decision or a discharged criterion by
  describing it differently. DEC-101 amended; `EX-7`/`EX-14` annotated (forward
  assignment only — **neither discharged obligation changed**); `runbook.rs`'s
  `Mode` doc comment carried the same false rationale and is corrected with them.
  PHASE-08 holds no criterion assigning `set`. **F-12 (major)** — Decision 2's
  re-anchor fixed the evidence and left the **consequence overclaimed**: a 2a
  discharge before its subject exists has **three** live causes (no boundary /
  premature attestation / misleading step text), so the raw event proves none.
  Consequence weakened to *reopens the discriminator for that step*, with three
  admissible outcomes (**reword · reclassify · stand**), and the observation now
  requires a **within-run sibling control** — which separates ordinary
  noncompliance from the other two; wording is separated at the reopening, not by
  the observation. CHR-049 is one exercise, so the within-run control is the
  collectible discriminator and cross-run corroboration is opportunistic —
  disclosed, not assumed.
- **Round 6: F-11 CONTESTED and re-fixed, F-12 verified, F-13 raised and
  integrated.** The F-11 contest was correct on both legs. **(a) The round-5
  repair amended DEC-101's PROSE TIER ONLY** — `record-101.toml`'s consequences
  still carried the live structured assignment, so `knowledge show`, which
  synthesizes both tiers, emitted the assignment beside its own withdrawal. The
  two-tier reading rule this project states everywhere, broken in the record used
  to state it. **(b) `PHASE-09 EX-8` was never swept** — it still asserted
  *"IMP-373 … stays deferred on DEC-104's own merits"* (withdrawn at round 5) and
  still listed *envelope-visible digest* as a mechanism-side revision candidate,
  which is vacuous: the digest already rides every emission as the `name@digest`
  header `design resume` emits. Two candidates now, not three. The contest
  **accepted** the completed-phase annotations. **A fifth surface the sweep found
  and neither the finding nor the contest cited:** `sketches/runbook-runner.md`
  §2/§5 still deferred `set` to PHASE-08 — repaired with the rest.
  **F-13 (major)** — `stand` made the falsifier narratable. `VA-4` demands every
  signal name a result that *would revise the placement*; an unconditioned
  `stand` let any firing be explained away as an anomalous run, which is
  route-to-no-revision. **Removed.** Two outcomes now — **reword or reclassify** —
  and the anomaly control moved into the **pre-registered firing condition**.
  **Fourth instance this review of a repair reproducing its own finding's defect**
  — the round-5 fix stopped the signal over-routing to reclassification by adding
  an exit that routes it nowhere. *(Round 7 amended both halves of this repair —
  see the F-14/F-15 entry below. The claim that both outcomes `revise the boundary
  contract` and so satisfy `VA-4` is **withdrawn as false**, and the firing
  condition's three named exclusions were **incomplete**.)*

- **RV-325 round 7 — F-14 and F-15, the seventh instance and its two faces.** The
  round-6 repair narrowed the **exit set** and the **exclusion set** in one move,
  and each narrowing lost something the thing it replaced covered. They had to be
  answered together: separately, either one makes the other worse.
  **F-15 (major)** — reword leaves a 2a step 2a on the same edge, so it revises
  the obligation's *contract*, not its *placement*, and `VA-4`'s noun is
  **placement**. The round-6 defence broadened the noun to make the cheap arm
  qualify — structurally what `stand` was doing. **`VA-4` is not amended**: it asks
  each signal to name *a* result that would revise the placement, and reclassify is
  that result; what would defeat it is an **unreachable expensive arm**, so the
  repair goes to reachability. The routing manoeuvre had migrated to **the
  reopening** — the one link left to judgement while everything around it was
  pre-registered, which is a `VA-5` breach on its own terms. Now pre-registered:
  **reclassify is the default**; **reword must produce candidate step text** naming
  a truthful, run-state-observable completion boundary or it is not available; and
  **reword is one-shot per step**.
  **F-14 (major)** — `stand` covered anomaly unbounded; its replacement covered
  three *named* events and omitted the commonest, **incidental context exhaustion
  or compaction**, which nothing marks. The run record can never supply it
  (Doctrine observes writes, not an agent's context — the same limit already
  disclosed for IMP-373's repayment signal), so the instrument is the **moderator
  protocol**, which is SL-233's to write, not CHR-049's. Condition (b) now excludes
  **any** context boundary, and new condition (d) requires the protocol to have
  **recorded** the context state — absent that record the observation is
  **uncollected, not weighed**. **Default-deny is what makes the condition
  decidable.** Disclosed: it lowers the firing probability, so the kit must report
  its own deny rate.
  **The round-6 warrant lapsed and is replaced, not restated** — *"a resolution set
  whose cheap arm is safe does not need an escape hatch"* fails once the cheap arm
  must be earned. `stand` goes because it was **post-hoc and unconditioned**. One
  principle, twice: *every exit and every exclusion is pre-registered; nothing is
  chosen after the run.*
  **F-11, contested a second time** — two more live surfaces cited
  (`runbook.rs:454-457`, `plan.md:172-180`), and the repo-wide sweep found **three
  more nobody cited**: `runbook.rs:1137` (a *third* site in that file),
  `runbook-runner.md` §9, and this file at the `set`-mode entry. Also amended
  **DEC-104's structured tier**, which carried the whole reopening condition and
  would otherwise have been F-11 leg (a) reproduced inside the F-14/F-15 repair.

- **RV-325 round 8 — F-11 and F-14 verified; the F-15 contest REJECTED, and the
  first contest this review to fail on the text.** The contest read the one-shot
  rule as the *only* route to compulsory reclassification and concluded that at
  N=1 no placement revision is reachable inside CHR-049. It skips the first two
  bullets of the round-7 rule. The first-firing route is *reword-must-produce-an-
  artefact* failing and *reclassify-is-the-default* applying, and `VA-5` (2),
  `EX-8` (2)(a)+(b), `thin-adapter.md:1042-43`, `target-machine.d2:690-92` and
  the cause table all carry it already — *"failing to produce one compels the
  expensive one"*. **What was load-bearing and unstated is the entailment that
  gives the bar teeth: the candidate-text test is unmeetable EXACTLY under cause
  A**, because DEC-104 classifies content 2b *precisely because it has no
  truthful boundary completion*. Text producible ⇒ a truthful observable boundary
  exists ⇒ the step is 2a-eligible and reword is *correct*, not an escape; text
  not producible ⇒ no such boundary ⇒ cause **A** ⇒ reclassify, first firing.
  **The artefact test IS the A-against-C discriminator** the round-5 scale note
  leaves *"settled at the reopening"*. Written now on all four surfaces, marked
  *clarified / no rule change*. The contest's implied remedy — reclassify-always
  — is the overclaim **F-12 removed one round earlier**, so adopting it would
  have been the eighth instance by way of reinstating a defect already fixed.
  **Residual disclosed:** failure to produce candidate text is not proof no
  boundary exists, so default-deny biases toward the **expensive** arm and the
  residual error is a false *reclassification* — the safe direction, and the
  gradient round 7 inverted on purpose.
  **Durable rule:** *before repairing a contest, check whether the contest's own
  reading is available in the text* — an entailment that is load-bearing but
  unstated reads as absent, and a verifier under the same criterion will miss it
  the same way. The fix is to write the entailment, not to change the rule.

- **RV-325 round 9 — F-16 (major), conceded in full. The round-8 clarification
  was a valid route resting on an invalid inference.** F-16: the round-8 bullet
  inferred *candidate text not producible* ⇒ *no truthful boundary exists*, and
  the paragraph directly beneath it conceded that non-production can equally be
  **failure to invent**. Both on the page together — so the reclassify branch
  revised a placement on an **admitted false-positive route**, and disclosing
  that was not repairing it. The rejection of the F-15 contest still stands:
  the first-firing route was real. What was defective is what licensed it.
  **The repair bounds the search and enumerates MOMENTS, not WORDINGS.** Wordings
  are unbounded, so failing to find one licenses nothing; the candidate completion
  *moments* are finite — the run record holds a finite sequence of turns in the
  window at that edge, and at each the obligation's subject is present in run
  state or it is not. That is exhaustive over an enumerable domain and it is
  signal (i)'s own check run once per turn instead of once. The reopening records
  the **subject**, the **turns**, and per turn whether the subject was present:
  some turn present ⇒ that turn *is* a truthful observable boundary ⇒ **reword**;
  no turn present ⇒ reclassify **on the enumeration**.
  **The claim is scoped and the outcome defeasible.** What is shown is *no
  truthful observable boundary within the observed window*, so the
  reclassification carries that scope plus its enumeration, and qualifying
  candidate text produced later reverses it. **Not `stand` restored** — `stand`
  changed no placement and cost no artefact; here the placement moves immediately
  and reversal costs exactly what the reword arm would have. And an
  **uninformative window is reported inconclusive**, not converted into a verdict.
  **F-16's third remedy declined wholesale** — *declare the direction unfalsifiable
  at N=1* — because that makes the expensive arm unreachable again, which is
  **F-15 exactly**; answering F-16 by reversing F-15 would be the eighth instance
  of the pattern, arriving by restoring a defect already removed.
  **Residual:** exhaustive over *moments*, not over *subjects* — which deliverable
  is the obligation's real subject stays the judgement `VA-5` already performs on
  signal (i). Reduced, not eliminated.
  **Durable rule:** *a disclosed defect is still a defect.* Writing "this may be a
  false positive" beside an inference does not license the inference; it documents
  that you knew. Either repair it or route the result to `inconclusive` — the
  disclosure is the third thing, and it is not one of the two.

- **RV-325 round 10 — the F-16 contest conceded, and the whole run-side inference
  abandoned. Owner ruling.** The contest is correct and the error was mine at the
  logical level, not the evidential one: the original signal is sound *because it
  is negative* — subject **absent** proves a step is not complete — and round 9
  inverted it into subject **present** proves a truthful completion moment exists.
  That is affirming the consequent, and enumerating turns exhaustively only asks
  the wrong question at every turn. The selector obligation is the counter-example
  the reviewer cited and it is in our own sketch (`thin-adapter.md:463-474`): it
  commits to *what the code-impact section now commits to*, so a recorded selector
  coexists with an unfinished obligation.
  **The loop was structural, not bad luck.** Rounds 5–10 all tried to infer *"this
  obligation has no truthful completion moment"* from run behaviour. That cannot
  work: **classification is a property of an obligation's text, not of a run.** A
  run tells you how one agent behaved once; it cannot establish a universal
  negative about the obligation. Every cleverer inference had the same defect in
  new clothes, and the cheap patch each round converged on an authoring-time
  property anyway.
  **The remedy — the classification test becomes an authoring gate.** Every 2a
  step states its own completion condition in its text: truthful (satisfying it
  *discharges* the obligation, not starts it) and checkable in run state. An
  obligation whose condition cannot be stated is not a step. Decidable over the
  whole set **before any run**, re-runnable by anyone — a check that can actually
  fail, which is what DEC-104 needed all along. The exercise then collects
  **adherence against the stated condition** (*discharged at a turn where its
  stated condition was not satisfied* — mechanical), and a step-specific result
  means the condition was mis-stated.
  **Owner's framing, which is the durable part:** *we do not build taxonomical
  truth from first principles — that is what design is. Evidence feeds design; the
  owner arbitrates whether the design is good.* And: **design content carries the
  rule; review reasoning belongs in the ledger.** The rounds 8–9 epistemology was
  cut from the sketch and the diagram rather than extended — ~160 lines removed,
  replaced by the gate and a short withdrawal note.

- **RV-325 round 11 (F-17) — a rule is not applied until it has reproduced its
  own assignments.** Round 10 wrote the authoring gate and worked *one*
  obligation. F-17 raised that, and applying the gate over the whole set is what
  found the defect: round 10's *"and checkable in run state"* **contradicted the
  round-3 owner ruling nobody had reopened** — *verification strength is an
  orthogonal axis, not the sorting rule* — which exists to give mechanically
  unverifiable but genuinely mandatory acts somewhere honest to sit.
  **The transferable lesson is the shape of the error, not the ruling.** Round 10
  was arguing against *run inference*; *checkable in run state* rode in as a
  qualifier on the fix, not as a considered reversal of anything. **A qualifier
  attached to a correct ruling can carry a second, unargued ruling** — and it
  survives because everyone is reading the sentence for the part that was
  contested. Applying the rule to the whole population is what makes it visible;
  reading it again does not.
  **Two reductios, both mechanical**: it demotes the two acts round 3 protected by
  name, and it condemns four of `exploring.toml`'s five *already shipped* steps. A
  runbook whose only admissible steps carry mechanical verifiers is a verifier
  list — DEC-101 and DEC-102 gutted.
  **The repair — split the limbs.** Classification asks only *truthful*;
  *state-visible* moves to collection, where it grades a condition's **evidence**
  exactly as a verifier grades a discharge's. Applied over all nine 2a obligations
  (the four this slice relocates **and** `exploring.toml`'s five — a whole shipped
  asset had been excused on the strength of one of its five members carrying a
  verifier): all nine state a condition and survive as 2a; **five of nine** are
  state-visible, and the coverage fraction is now a reported number rather than an
  assumption. Two step texts bought visibility by naming a landing site; knowledge
  capture deliberately did not, because the artefact it would need is F-2's
  ceremony objection returning by the back door.
  **One asymmetry kept on purpose:** the reword arm still demands *checkable in
  run state*, so a firing cannot be answered by rewording a condition **out of
  visibility**. Withdrawing a clause everywhere it appears would have opened that
  door — the sweep is per-site adjudication, not find-and-replace.

- **DEC-102 currently overclaims, and PHASE-08's shape depends on the repair
  chosen.** Its consequences state *"the craft has genuinely moved out of skill
  prose into data"*. Not yet true: its own central example — `SKILL.md:98`
  "prefer multiple choice questions" against this project's `CLAUDE.md`, which
  asks for prose forks — is **inquiry** craft, while PHASE-16 shipped the
  **exploring** runbook, a different edge. Two repairs available: **D12** (add
  the edge-2 runbook, which this slice took) or amend DEC-102 to say the craft
  moves in PHASE-08 (cheaper, not obviously wrong). Named as reviewer attack 2.
- **`EX-7`'s handover adapter lands in PHASE-08 unsketched.** All six `EN-2`
  questions are about the *design* skill; the handover convergence (DEC-058,
  `plugins/doctrine/skills/handover/SKILL.md`, 114 lines / 5,037 bytes) is a
  declared design target of the phase but sits **outside this gate entirely**.
  Disclosed in the sketch, not fixable inside `EN-2`.
- **The `Outcomes` section escaped every triage until arithmetic caught it.**
  `SKILL.md:205-214` was assigned by no question and surfaced only when the
  sketch's (d) had to make 214 lines account for themselves. Evidence about
  method — a triage driven by a question list covers what the list names.
- **`VH-1` is new and PHASE-08 cannot self-certify it.** The owner reads the
  rewritten skill prose and judges whether design prose was pushed down as far as
  practical, bounded by D7/D8/D10. `VA-4` now produces the disposition table it
  is judged against.
- ~~**IMP-373 is off PHASE-08's critical path** (D7)~~ **RE-WARRANTED — see
  DECISION 1 above.** Still off the critical path, but for a different reason
  than D7 gave. Its rendering question (what a cursorless runbook renders under
  `EX-14`'s token bound) is what now carries the deferral, and it has a named
  repayment trigger rather than an open end.
- **DEC-102's present-tense overclaim is still unruled.** *"is therefore fixed
  FOR THIS REPO"* while `SKILL.md:98` is verbatim unchanged. The repair was
  proposed across three rounds and never ruled on; RV-325 round 3 and round 4
  both declined to raise it. The last disclosed defect carrying no disposition.
- **F-2's stored ledger response is stale and F-9's is mangled.** F-2's describes
  the withdrawn granularity bound (DEC-104 replaced it); F-9's lost three
  backticked spans to shell expansion. Both are `verified`/`answered`, so
  `dispose` refuses rewrites. The F-9 correction is appended to F-10's response.
  A round-5 verifier reading either at face value would close falsely.

- **VT-1 is seven of seven**, each observed red on its own seam — VA-NC answered
  empirically rather than argued. Every PHASE-16 criterion is discharged.
- **F12 · DEC-102 does not cite IMP-372 by id** — it names the deferral's carrier
  by kind. Blocked on tree topology, not on judgement: the two records live in
  different trees until the slice lands. Add the citation at reconcile.
- **A regressed step cannot be skipped past — deliberate, asserted, untested.**
  A skip runs no check, so the old `verified` record stays live and the cursor
  stays past it. That refusal is right (waiving a failing required check by
  re-declaring it is verification theatre) but it is reasoning, not a test, and
  it belongs in the ship-and-learn set. Recorded as PHASE-16 sheet finding F9.
- **The fixture-placement decision is SETTLED** (owner, 2026-07-31): a shared
  helper belongs in "a sensible module imported by just its callers, not every
  e2e test". Hence `tests/runbook_fixture/` — not `tests/common/` (~30 crates
  pay) and not `tests/design_fixture/`, which is a *bootstrap* its three
  includers take for `DesignRun`. The interrupted
  `design_fixture::discharge_exploring` was dropped, not relocated.
- **ISS-288 · the shipped verifier resolves `doctrine` on PATH.** Raised at T6.
  Right for a project-supplied verifier, questionable for one Doctrine ships and
  invokes from inside itself; in the jail the PATH binary is stale and lacks
  `verify`. Three options recorded; a `{doctrine}` placeholder looks right and
  wants doing before any discharge persists in anger. PHASE-16 does not depend
  on the answer.

**BOTH FORKS ARE SETTLED (2026-07-31). The design conversation converged and its
product is PHASE-16.**

- **PHASE-16 — Obligation runbook runner**, appended and *placed* between
  PHASE-07 and PHASE-08 in the `plan.toml` array (position is the execution
  authority; ids are names). Design: `sketches/runbook-runner.md`. Decisions:
  **DEC-101** (the primitive, the TOML asset, executable verifiers,
  attestation-as-progression, the trust boundary, the claim limit) and
  **DEC-102** (invariants stay sealed, craft ships overridable — no manifest
  entry is reversed). Both `accepted`, both `shapes SL-233`.
- **Fork 1 → option (ii)**, the generic ordered-obligation primitive with items
  as asset data. Refined past the original three candidates by the owner: the
  verifier is an **arbitrary executable, zero on success**, not a closed Rust
  predicate vocabulary — a closed vocabulary would have reimported the
  recompile-per-check cost that killed option (i). What stays closed is the
  **placeholder** set (`{slice}`, `{run}`, `{repo_root}`, `{step}`).
- **Fork A (asset shape) → TOML**, not a Markdown convention. Step ids are
  explicit and author-assigned, never positional.
- **Fork B (scope) → insert the work now**, rather than shipping a fat design
  skill and deferring. PHASE-16 ships the mechanism plus one runbook
  (`exploring`); PHASE-08 converts the other two prose checklists.
- **Fork 2 → dissolved, nothing is reversed.** The question "is design craft
  doctrine or taste?" was mis-shaped: invariants are doctrine, craft is taste,
  and the assets already split along that line. An asset is sealed when an
  override would make its content *false* rather than merely different. DEC-102
  narrows DEC-077 (which **deferred** override semantics rather than forbidding
  them) instead of superseding it; ADR-019 does not move.
- **The sketch took an external adversarial PROSE review** (codex, read-only,
  deliberately NOT an RV ledger — the owner declined a fifteen-round disposition
  cycle for a pre-implementation catch). Ten findings, **nine stood** on
  independent verification against the source; sketch **§12** is the disposition
  table and the sketch is now **revision 2**. What moved: the gate clause belongs
  on `gate::advance`'s `Condition` set, not `can_advance` (which is const
  edge-membership); a discharge binds the **step-definition digest**, because an
  id solves reference and not equivalence; `sequence`/`set` are two admission
  rules, not one behind a flag; `verify` is an **argv array**, never a shell
  string; validation is **owned**, not inherited from `prompt check`; a top-level
  `verify` must join `FAMILIES`; and `customization` **resolves to nothing**
  today. The one refinement rather than acceptance: the run *does* CAS at
  admission — what is true is that the admit→persist span has no recheck, so
  PHASE-16 keeps subprocesses out of it (ISS-284).
- **Owner's ruling (b), 2026-07-31 — the override seam is deferred, not
  delivered.** The v1 runbook ships **embedded**; DEC-102's delivery claim is
  corrected to identify-and-defer while its *reasoning* stands unchanged.
  **IMP-372** carries the resolution work. The craft still moves out of skill
  prose into data — the `SKILL.md:98` contradiction is fixed for this repo and
  not yet for anyone else.
- **A SECOND external prose review ran over revision 2** (fresh codex session,
  read-only, not the first pass's agent — revision 2 had rewritten precisely the
  parts revision 1 got wrong, and those repairs had been checked by nobody).
  **Eight findings, three structural**, all verified against source before
  acceptance; sketch **§12** now carries both disposition tables and the sketch is
  **revision 3** (`89404556`). What moved: the runbook had **no selector** and is
  now keyed to a **forward edge** beside `boundary_conditions`; the clause is a
  **third derived input** to `advance` (`RunbookStanding`), NOT a new `Condition`,
  which is payload-free with an existential satisfaction test and a refusal that
  cannot name a step; verifier results enter through **`DerivedInput`**, never
  payload — `DerivedDesignFacts` is persisted snapshot state and `request.evidence`
  is refused by `DerivedConditionClaimed`. Also corrected: the section-attestation
  precedent was a **false citation** (the four-part binding is
  `AcceptanceAttestation`'s → **ISS-287**), the digest's canonical encoding was
  unspecified (→ **EX-18**), and `FAMILIES` has a second integration point.
- **Owner's rulings from the second pass.** (a) A stale discharge **warns**, does
  not block, and the runbook requirement is evaluated **per-edge**, not
  cumulatively — the machine's first non-cumulative condition. (b) The runbook
  **must not derive** the incumbent conditions: truth cannot flow from
  user-customisable material into a Doctrine-owned closed vocabulary, and there is
  nothing to reconcile anyway (**ISS-285**). (c) **Ship and correct from
  operation** rather than seek a third pre-implementation pass — spend remaining
  design attention only on what hardens into contract at ship time (sketch §0).
- **PHASE-16's criteria have been amended TWICE**, both times before the phase
  started and before any evidence was recorded against it. Ids unchanged
  throughout; **EX-16/EX-17** appended by the first, **EX-18/EX-19** by the second.
  The pattern is disclosed in `plan.md` and the phase objective rather than
  absorbed: each review found the prior round's repairs resting on false claims
  about the code, which is evidence about the *method*.
- **`set` mode left PHASE-16.** *(Amended round 7, F-11 second contest: this read
  "left PHASE-16 **for PHASE-08**. No shipped user, and no reachable end-to-end
  test under ruling (b)." Both clauses lapsed — `set` belongs to **IMP-373**, and
  the no-shipped-user warrant is withdrawn as false; see the DECISION 1 entry
  above and DEC-101's amendment. The no-reachable-test fact was true of PHASE-16
  and is not a warrant for deferring past it.)* VT-1 is **seven** tests, not eight;
  `editing_a_step_definition_makes_its_discharge_stale` is now explicitly a
  **fixture-based** test over the `#[path]`-included pure model.
- **Still open, deliberately, for `/phase-plan` PHASE-16:** the seventh writer
  act's wire key (`discharge` is the working name; `attest` collides with the
  existing content-bound section attestation); the **digest's canonical encoding
  and version tag** (EX-18 — hardens on ship); and which shipped checks `doctrine
  verify` launches with beyond `research-current`. **Runbook drift is no longer
  open** — the per-step definition digest is the answer. **The stale-after-clear
  rule is no longer open** — see the owner's ruling (a) above; `EX-16` carries both
  halves.
- **PHASE-08's own `EN-2` gate is untouched and still owed** —
  `sketches/thin-adapter.md` plus a non-author RV. PHASE-16 takes no new gate.

**PHASE-08 — the owner replaced `EN-2`'s procedure; this is the record of it.**

- **Owner ruling, 2026-07-30:** the six-question sketch as a *driver* is rejected
  — "(e) in the bin, and the rest are not much guarantee of a useful outcome …
  a setup to spend the rest of the week's tokens without getting this thing
  shipped." Order is now **design conversation first, paperwork back-solved**. The
  sketch and its non-author RV still land (`EN-2` is authored and criteria ids are
  immutable); the difference is it will assert what was established rather than
  drive the establishing. (e) is binnable on evidence: skills are projected by the
  binary's own embed, so an installed skill cannot outrun its binary, and "errors"
  is already served by the typed `Refusal` vocabulary.
- **Stated intent to design against** — move the workflow *and the process implied
  by the prose* out of skill prose into Doctrine, both to relieve the agent of
  executing deterministically and so instruction arrives as **"do this now"**, not
  "be on the lookout for a situation, in case of which …".
- ~~**OPEN FORK 1 — the checklist primitive.**~~ **SETTLED — see above.** Owner: "a parameterised runbook from
  hymn + conventions isn't a terrible idea" (favourable, not yet a decision).
  Candidates: (i) sub-state per step in Rust — max adherence, every prose revision
  needs a recompile; (ii) **a generic ordered-obligation primitive in Rust whose
  items are asset data** — Rust owns "ordered, one at a time, no skipping, visible
  in the envelope", the contents come from an overridable hymn; (iii) richer
  fragments and no new state — cheapest, adherence stays voluntary. (ii) is the
  recommendation on the table: it is DEC-077's own split one level deeper, and one
  primitive retires three prose checklists (explore's five steps, present-design's
  fifteen content items, review's four attack surfaces).
- ~~**OPEN FORK 2 — the customization reversal.**~~ **SETTLED by DEC-102 — nothing is reversed; see above.** Owner assumes all fragments should
  be **hymns, and ultimately end-user overridable**. That reverses what PHASE-07
  shipped: `hymns/stage/design.md` plus the four `design-prompts/*` are the ONLY
  `customization = "fixed"` entries in `publication/manifest.toml`, and
  `stage/design` sits in `[hymns].seal`. Needs a DEC superseding DEC-077's
  "embedded and authoritative" clause; ADR-019 does **not** move. Scope question
  for the owner: does this land in PHASE-08 or its own slice?
- ~~**Unresolved:** is design craft doctrine or taste?~~ **Answered by DEC-102:**
  the question was mis-shaped. Invariants are doctrine, craft is taste. The
  `SKILL.md:98` multiple-choice contradiction gets fixed by moving that craft into
  an overridable runbook, not by unsealing an invariant hymn.

**PHASE-07 — now `completed`; these carry to reconcile, not to execution**

- **VA-3 reconcile item** — jointly unsatisfiable with EX-1, and separately
  vacuous as worded. Both halves in *Learned*; the resolution is disclosed in
  `1c09a319`'s message and sheet F-10. Ids immutable → carries to reconcile.
- **VA-1's `just nix-build` leg is DEFERRED** — host-only, nix absent in the jail.
  Recorded, never silently skipped. EX-7's substance (no new embed root) is
  verified independently: both stores sit beneath the already-grafted `install/`.
- **S1 regression baseline will miss again** — same disposition as PHASE-11/10:
  record at reconcile that the phase landed on a full-suite `check gate` green,
  strictly broader than the targeted diff it substitutes for.
- **Two more staged-but-unwritten surfaces** — `RunHeader.next_obligation`
  (rendered as *elided prose*, so it could not serve as EX-2's closed enum without
  a v1 wire change) and `FragmentGroup`/`run.fragments`. Neither has any writer.
  Same family as **IMP-367**'s seven readerless accessors → one backlog item for
  the family at harvest, not three. Consequence worth noting: the existing
  `known_fragment ... NOT held by this run` line (asserted by
  `tests/e2e_design_projection.rs:762`) compares caller declarations against an
  always-empty group, so it *always* reports not-held.
- **The design-run test fixture wants lifting into `tests/common/`** — test 3
  needs a live run and the only bootstrap is `Fixture` local to
  `tests/e2e_design_projection.rs:47`. The selector already declares the intent
  (`tests/common/**` is a design target noted "shared test helpers";
  `tests/e2e_design_*.rs` is one too). Owner's call pending: lift vs a minimal
  local bootstrap.

**Carried design residuals**

- Eviction-ladder evidence is synthetic by construction — the 24576 B ceiling
  sits far enough above the 15,576 B saturated table that the ladder cannot fire
  on realistic input; sketch § *For the reviewer* item 3.
- Sketch §(e)'s 15,576 B worst case is **arithmetic, not measurement** — PHASE-04
  measured 6576 B on a fixture that saturates every cardinality cap but no byte
  cap, and the ladder's terminal rule is reachable only via `project_within` with
  an artificially tightened budget. Detail in the PHASE-04 sheet `## Findings`.
- PHASE-02 EX-7 cites design R4 (runtime/authored divergence); the authority
  risks are R2 and R12. Substance implemented, citation wrong, criteria
  immutable — a reconcile item, not a code fix.
- ~~**Two spellings reach `Disposition::Adopted`**~~ — **SETTLED by PHASE-06
  `EX-12`** (`564e8775`). `record` / `adopt_record` are off the wire, both
  `(None, Some(_))` arms are deleted, and `(None, _)` is now
  `CheckpointDispositionMissing`. `CheckpointDispositionConflict` went with them
  as unconstructible (see PHASE-06 F-3). Dropped from Open at the next pass.

**PHASE-13 implementation obligation — carriage return, from RV-323 round 7**

> **REASSIGNED 2026-07-29.** This said *PHASE-06* and no longer does. The
> 2026-07-29 split gave the obligation a criterion of its own — **PHASE-13
> `EX-4`**, with a named test (`declare_refuses_carriage_return_on_the_document_boundary`)
> under PHASE-13 `VT-2`. PHASE-06 closed **without** it, correctly: PHASE-06 no
> longer owns admission's document boundary. Do not read the completed PHASE-06
> as carrying an undischarged obligation.

- The title differential's corpus **omits carriage return**, and a targeted run
  shows `Title\r` derives `Title` after formatting. The raiser found this while
  re-running the committed probe, and dispositioned it as
  **implementation-correctable rather than design-invalidating** under the
  owner's terminality bar: formatter stability of the derived title is
  withdrawn, and `CarriageReturnInDocument` is already the document boundary.
  What PHASE-06 owes is **aligning declaration admission with that boundary** —
  the boundary refuses `\r` in a document, so a declared body carrying one must
  not reach derivation by a different door. Recorded here because the raiser's
  argument lived only in the gitignored baton, and the ledger's `--note` is
  ephemeral by construction.

**Verification-mode reachability — now four instances, and a second sub-shape**

- PHASE-04 VA-NC is only partially met (3 of 7 EX-10 tests genuinely red first);
  black-box tests over a not-yet-existing binary cannot be driven test-first.
  With EX-17 and EX-18 this is the **third** instance of a verification mode
  mandated without the phase shape that makes it reachable — per the EX-18 note,
  `/phase-plan` gains that as a check rather than a fourth appended criterion.
- PHASE-05 VA-NC is also partial, and it is a **new sub-shape**. The check *was*
  applied at phase-plan and passed: the command existed, so a black-box red was
  genuinely available, and the sheet ordered the fault seam before the crash
  tests to buy the red. What defeated it was the **task split** — T4 (six-step
  path) and T7 (recovery/resume) were planned apart, and the worker reports they
  are the same code, so the red-first window for the two crash tests never
  existed. The worker instead obtained a red per test by deliberate mutation
  after green, which is arguably *stronger* evidence of non-theatre but is not
  what VA-NC says. **The check needs a second question: can these tasks actually
  be done in this order, or does the code make them one?** Reconcile item.

**Open review**

- **RV-323 — CLOSED, no longer open.** Terminal at round 30, 6/6 verified;
  EN-2's non-author-review half is satisfied and PHASE-06 may start. Two rev-5
  positions survived the round and are now settled rather than pending: the
  derived title's formatter-stability property is **withdrawn**, and requiring
  it would route to a Markdown-inline-parser slice rather than a sixth
  extraction rule; and rev 4's "prettier is idempotent on its own output" is
  **false** with F-1 nonetheless standing on the corrected claim.
- RV-321 — `await=raiser`; codex may still contest F-2's rejected framing half.
  `ROW_FRAMING_BYTES` = 10 is a disclosed conservative reserve and sketch §(e)
  depends on it; do not "fix" it.

**Backlog carried, `originates_from SL-233`**

- IMP-346 — funnel cannot express a non-dispatchable (coordinator-only) phase.
- IMP-347 — stall sweep misses out-of-directory entity writes.
- IMP-349 — `.doctrine/state/slice` literal single-sourced in name only.
- ISS-273 — tech spec scaffold teaches a malformed `[[source]]` identifier.
- ISS-274 — `record-boundary` leaves index and worktree stale; `arm-spawn`,
  `dispatch_import`, `conclude` and `reap` all share the shape.
- ISS-275 — review verbs refuse in the coordination worktree.
- ISS-277 — `reseat` cannot renumber any review (strict meta read demands a
  derived `status`).
- ISS-279 — entity id reservation has local reach; two trees allocate the same
  id unwarned. Realised as the RV-320 double-allocation.
- IMP-352 — ~10 integration fixtures spawn the binary without `.current_dir()`,
  so every marked worker fork false-reds them and no worker can use `check gate`
  as a signal. Per-phase tax; one sweep removes it.
- IMP-353 — `WITHDRAWN_STATUSES` (PHASE-05, the adopt disposition's usable-status
  predicate) is hand-maintained and not derivable from the per-kind vocabularies.
- **IMP-365** — pre-run runnable `VA-` criteria at plan time; the fourth instance
  of a criterion unsatisfiable or vacuous as literally worded (PHASE-10 `VA-3`).
- **IMP-366** — re-emit an outstanding delegation assignment on read; PHASE-10
  emits it only at export (D3's disclosed bound, re-derivable from the snapshot).
- **IMP-367** — remove or reach the seven design-run read surfaces with no reader,
  surfaced by narrowing `attestation.rs`'s module-wide `dead_code` gate.
- **IMP-355** — kind-folder-local `.prettierignore` on first record write, as the
  POL-002-clearing alternative to the installer seeding a root one. Deliberately
  low: the measured exposure is a one-time whole-document re-review, not data
  loss. Filed is not resolved — whether a file inside Doctrine's own tree escapes
  POL-002 is the open question, and the PHASE-06 gate does not decide it.
- `review contest` has no durable rationale field — the raiser's argument for why
  a fix is inadequate lands in gitignored baton state; `raise` and `dispose` both
  write durable prose. **Now captured as an RFC-011 friction observation**
  (`591b1d981`), and it bit again at RV-323 round 5: recovering codex's contest
  reasoning meant tailing `.doctrine/state/review/323/baton.toml`, because the
  finding's authored `detail` still shows round-1 text and `response` still
  shows the answer the contest rejects.

**Knowledge carried**

- QUE-177 — design-run recovery and freshness model.
- QUE-178 — governing specification boundary; **answerable** (PHASE-01 settled it
  as PRD-019 + SPEC-029 + two amendments).
- QUE-179 — minimum useful inquiry-map schema.
- QUE-180 — checkpoint admission and acceptance. **PHASE-05 delivered the
  evidence; the record still needs settling.** Two facts: (a) admission
  (`plan_checkpoints` — kind resolvable, record exists, status not withdrawn) and
  acceptance (DEC-088's attestation, touching only status) are separable and were
  kept apart, which is what let acceptance be optional without weakening
  admission; (b) acceptance had to be *non-declarable* to mean anything — the
  wire has no `status` field, so `accepted` is reachable only by supplying a
  `basis`, and Doctrine derives the binding digest. **Residual to record
  explicitly:** `accepted_status` derives from each kind's own vocabulary, so
  today only `decision` has anything an acceptance unlocks; an acceptance on a
  QUE/ASM checkpoint is journalled and auditable but flips no status. Deliberate
  and conservative — but implicit, and it should not stay implicit.
- QUE-181 — bounded delegation contract. **PHASE-10 delivered the contract and
  the record needs settling.** Four facts worth the record: (a) the contract is
  four acts on one tagged wire field — export / propose / accept / refuse — so
  they are mutually exclusive by construction rather than by a conflict refusal;
  (b) "the coordinator is sole writer" is enforced as a **composition** refusal
  (a `propose` payload may carry no writer act) and not as a role check, because
  v1 authenticates no worker identity and inventing one would launder authority;
  (c) a proposal's currency is exact equality against **the obligation as
  assigned**, which is DEC-066's rule applied to a subject that has no
  fingerprint; (d) the transport is genuinely absent — the delegate's proposal is
  an ordinary `design apply`, so the session boundary is never enacted anywhere,
  including in the tests. Residuals: acceptance applies the proposal's
  declarations (the `EX-1` reading taken, see PHASE-10 A2/F-6), attribution is
  unverified free text, and an outstanding assignment is not re-emitted on read
  (IMP-366).
- QUE-182 — prompt-fragment ownership boundary.
- ASM-004 — incremental records improve prolonged-design continuity.
- ASM-005 — static activation remains during the experiment.
- ASM-006 — existing knowledge kinds fit semantic checkpoints. **PHASE-05
  supports it, with one caveat.** All seven kinds fit with no new wire type
  (VA-2 zero, positive-controlled); the existing scaffold / reservation / status
  / relation seams carried the whole six-step protocol; `record shapes SL-NNN` is
  already legal for all seven. Caveat: EX-4's "usable status" had **no existing
  predicate** — `is_hidden` and `is_terminal` both misclassify (an `answered`
  question is hidden *and* terminal yet is exactly the right adoption target), so
  PHASE-05 added `WITHDRAWN_STATUSES`. Drift guard filed as IMP-353.
- PRD-019 OQ-1 (retrieval of applicable existing records), OQ-2 / RSK-229
  (managed-instruction authority) — carried, non-blocking.

**Dispatch ritual, settled — kept as a pointer**

- Coordinator-only phases (PHASE-09 remaining) execute solo in the coordination
  worktree → `dispatch record-boundary` → flip the sheet; `dispatch next` still
  prescribes `spawn` for them (IMP-346). The boundary row is not bookkeeping:
  `src/ledger.rs` makes `boundaries.toml` the input to `sync --prepare-review`'s
  `phase/<slice>-NN` cut, so a phase without one gets no per-phase review ref.
- Attribution additionally needs a registry row — `slice record-delta 233
  PHASE-NN --commit <import-commit>`. The claude arm derives the registry only at
  `prepare-review`, so mid-drive `verify-vt` reads `UNATTRIBUTABLE` without it.
