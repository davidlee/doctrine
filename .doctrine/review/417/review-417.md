# Review RV-417 — reconciliation of SL-278

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Conformance audit of SL-278 (Review read surface) on the primary tree, `edge`,
commits 593f3ca8d..76af61401 + 26a5d7eaf (PHASE-01..05, capsule-driven, no
dispatch candidate). Held to the locked `design.md` (DEC-361..DEC-370) and
`plan.toml` EX/VT/VA criteria; governance ADR-001 (layering), ADR-024 (no
repo-private ids in shipped text), STD-001 (single-sourced vocab), STD-003 (no
silent skip).

Lines of attack:
1. One view: table, CLI JSON and MCP all render one `ReviewView`; no parallel
   shape survives (`review.finding` leak gone; MCP summary mode deleted).
2. Depth × selector × filter semantics match DEC-362/363/367: `-f`, `--full`,
   `--open`, `-s`, `-S` ANDed; census and `show` share one predicate.
3. Tolerant scan (`scan_reviews`): an unreadable review is disclosed (stderr +
   JSON `unreadable`), never silently skipped (STD-003).
4. MCP `review_show` / `review_findings` arguments mirror the CLI; refusals
   typed; legacy `view`/`format` ignored per design sec-4.
5. Guidance (PHASE-05) tells the truth about the new surface; EX-2 mirror
   regeneration.
6. Conformance algebra: undeclared `plugins/doctrine/skills/{plan,reconcile}`,
   `tests/e2e_subcommand_help.rs`, two memories; undelivered `.agents/skills/*`.

Evidence: `doctrine check gate` — 25 unit failures, all in the known jail
git-guard set (dispatch 7, ledger 10, slice 8; SL-276 territory); integration
suites run separately: 4050 pass, 1 known fail
(`e2e_dispatch_candidate_admit_rejects_unproven_tip`); `just test-bridge`
22/22. Logs under `.doctrine/state/slice/278/audit/`.

## Synthesis

SL-278 delivers its design: one engine-owned `ReviewView` rendered by the
table, CLI JSON and MCP; `review show` depth/selector/filter semantics per
DEC-362/363/367; a shared finding predicate behind `show` and the new
`review findings` census; a tolerant scan that discloses unreadable ledgers.
The adversarial pass (codex, gpt-6-sol high) found the projection, filters,
live output and audit-skill wording clean.

What it found sat at the edges, not the core:
- F-1 (blocker): the new MCP tool text cited repo-private ids. Fixed, and the
  class is now guarded across every tool (`tool_text_cites_no_repo_private_id`);
  the residue that predated the slice was cleared so the guard holds.
- F-3: shipped guidance still claimed the index hides routes. Fixed in the
  reference doc, the plan skill master and one memory.
- F-5: STD-003 disclosure was only proved at the projection; now proved at the
  process boundary for CLI stderr, CLI JSON and MCP.
- F-4: the design named the gitignored skill mirrors as the edit surface.
  Design-wrong → reconcile (selector registry + design prose).
- F-2: the installed mirrors lag the masters. Tolerated: `doctrine install`
  sources skills from the published repo, so they cannot be current before
  SL-278 is pushed. Close-time action recorded in notes.md.

Standing risks: the full gate cannot go green in the jail (25 git-guard unit
failures + one e2e, all pre-existing, SL-276); integration suites were run
separately (4050 pass). VA-1 token budget is over (~294 table / ~497 JSON vs
~200) and accepted by the user. ISS-518 (unknown finding id → -32603) and
CHR-177 (CLI help example ids) are open follow-ups outside this slice.

## Reconciliation Brief

### Per-slice (direct edit)
- F-4 — selector registry (load-bearing; what `slice conformance` reads):
  `doctrine slice selector rm` the design-targets
  `.agents/skills/{plan,reconcile,audit}/SKILL.md`; `doctrine slice selector
  add` design-target `plugins/doctrine/skills/plan/SKILL.md`,
  `plugins/doctrine/skills/reconcile/SKILL.md`, `tests/e2e_subcommand_help.rs`,
  `.doctrine/memory/items/mem_019fbc7514097c42aa042ab7bb2206c6/`,
  `.doctrine/memory/items/mem_01a0d91d7a0d75f1853c505d08237451/`. The audit's
  own fixes also touch `src/mcp_server/tools.rs` (already declared) and the
  two e2e suites (declared).
- F-4 — design.md sec-6/sec-7 (human mirror): masters are
  `plugins/doctrine/skills/*`; `.agents/skills/*` are installed mirrors,
  regenerated from the published repo after landing (F-2), never edited.

### Governance/spec (REV)
- none.
