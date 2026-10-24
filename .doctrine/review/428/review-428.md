# Review RV-428 — reconciliation of SL-286

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Mode: conformance, self-audit. Surface: primary tree on `edge` at 08e6f1b74
(not dispatched — no candidate branch). Last code commit 9f802dc6a.

Lines of attack:

1. Mechanical conformance (`doctrine slice conformance 286`): dispose every
   undeclared / undelivered lead.
2. Evidence: `doctrine check gate` and `doctrine slice verify-vt 286` re-run, not
   taken from the handover.
3. Design conformance of `src/codex_mcp.rs` + install/boot wiring against
   design.md sec-leg / sec-execute / sec-report, net of the accepted deltas
   DEC-412/413/414. Invariants: bake rule (DEC-404), consent default No and asked
   only on change, declined = zero calls and zero writes, degraded prints the
   command (DEC-408), retirement only after consent, override untouched,
   PATH-dependent foreign entry holds incomplete (DEC-406), report lines
   (DEC-407), explicit-install-only (DEC-411), `-y` help text.
4. Rootless `serve --mcp` (DEC-405) and ADR-001 layering of the new module.
5. Governance and docs owed by scope §4: REQ-484 REV, rootless requirement,
   DEC supersessions, README.

First-pass reads were delegated to pi (quotient `judgement`); every finding
raised here was spot-checked against the source before raising.

## Synthesis

The implementation conforms. The gate is green (re-run at audit) and every VT
passes after F-1. No behavioural defect was found; VH-1 passed (no EVD-032
recurrence) and VH-2 produced EVD-034.

- **One evidence regression (F-1, fixed).** A PHASE-05 DRY refactor moved the
  skipped-line literal into `tests/common`, silently breaking PHASE-04 VT-3's
  keyword check. The gate does not run `verify-vt`, so nothing flagged it and the
  handover reported all-pass. Fixed by naming the text in the assertion message.
- **Design lags code (F-2, F-9..F-13).** Three DECs of accepted deltas plus five
  small unrecorded readings — all cases where the code is the better statement
  (daemon hint after any write; empty `url` is Malformed; codex_mcp is leaf-only,
  tighter than the design's ADR-001 row; ASM-014 invalidated; precise rootless
  notice). All go to design.md by direct edit.
- **Registry drift (F-6, F-7).** Three intended files undeclared, one no-op
  selector. Fixed through `slice selector`, not prose.
- **Governance and docs owed by scope (F-3, F-4, F-5).** REQ-484 still describes
  the retired `sh` wrapper; no rootless requirement exists; three superseded DECs
  are still accepted; README documents the old entry and `sh` dependency. These
  are planned reconcile work, not escapes.
- **F-8 aligned.** The undeclared `.doctrine/` paths are bookkeeping; foreign
  4aea4ff19 is an IMP-407 doc commit from another session on shared `edge`.

Standing risks: RSK-234, IDE-069, ISS-528 (partially resolved by DEC-406),
ISS-533 (pre-existing; SL-286 did not change notification handling — the
`notifications/initialized` arm was moved verbatim). The `codex mcp add`
whole-table replacement (EVD-033) remains a user-visible tradeoff, mitigated by
listing dropped keys in the consent prompt.

Process gap worth noting: `verify-vt` keyword checks can regress under a green
gate. A refactor that moves an asserted literal should re-run `verify-vt`.

## Reconciliation Brief

### Per-slice (direct edit)

- F-2: design.md sec-leg / sec-execute / sec-report / sec-code-impact — fold
  DEC-412 (items 1–9), DEC-413, DEC-414. DEC-412's PHASE-02 EX-4 amendment stays
  recorded in the DEC; plan.toml is not edited.
- F-6: `doctrine slice selector add 286` (design-target) for `src/root.rs`,
  `src/catalog/mod.rs`, `tests/e2e_worktree_provision.rs`; mirror as code-impact
  rows in design.md.
- F-7: `doctrine slice selector rm 286 src/commands/serve.rs`; drop the no-op
  code-impact row.
- F-9: design.md:142 — the daemon hint follows every write (user entry or project
  retirement).
- F-10: design.md project table — Override needs a non-empty string `url`; empty
  or non-string `url` is Malformed.
- F-11: design.md:7 — codex_mcp is leaf-only (fsutil, toml_edit, std); drop
  "→ boot for resolve_exec/Harness". (layering.toml comment already fixed.)
- F-12: design.md:197/296/311/371 — ASM-014 invalidated by EVD-034; the empty
  tool list is the only signal (DEC-405 holds).
- F-13: design.md rootless notice shows both variants; design.md:355 names
  `worktree provision` (→ `install::regenerate`).
- F-4 (scope §4): README.md:159-176 — remove the `sh` snippet and dependency
  (DEC-409); document the user-level entry and its scope, the consent prompt and
  `-y`, the project-level override slot, codex's per-folder trust prompt, and the
  codex CLI as a declared dependency.

### Governance/spec (REV)

- F-3: REQ-484 (SPEC-011) → REV modify: user-level registration via
  `codex mcp add` with consent, absolute PATH bake (DEC-404/406/407/408/411).
- F-3: SPEC-011 → REV add requirement: rootless `serve --mcp` completes the
  handshake with no tools and a disclosed cause (DEC-405).
- F-5: `doctrine supersede` DEC-323 and DEC-332 by DEC-406; DEC-325 by DEC-407.

## Reconciliation Outcome

### Direct edits applied

- design.md: DEC-412/413/414 folded into sec-leg, sec-execute, sec-report,
  sec-code-impact (F-2); daemon hint follows every write (F-9); empty or
  non-string `url` is Malformed (F-10); codex_mcp leaf-only row (F-11); ASM-014
  invalidated by EVD-034 throughout (F-12); both rootless notice variants and
  `worktree provision` wording (F-13); code-impact rows for root.rs, catalog,
  e2e_worktree_provision added and the serve.rs row dropped (F-6, F-7 mirror).
  Out-of-band edit on the locked run (mem.pattern.reconcile.edit-design-out-of-band).
- Selector registry: `slice selector add 286 --intent design-target` src/root.rs,
  src/catalog/mod.rs, tests/e2e_worktree_provision.rs; `slice selector rm 286
  src/commands/serve.rs` (F-6, F-7). Conformance now: undelivered 0, conformant
  17; undeclared are only the `.doctrine/` records F-8 aligned.
- README.md codex paragraph rewritten: user-level entry and scope, consent and
  `-y`, override slot, per-folder trust prompt, codex CLI dependency, `sh` removed
  (F-4).

### REVs completed

- REV-076 (`reconcile-sl-286`): done — REQ-484 rewritten to user-level
  registration; REQ-485 introduced as SPEC-011 FR-014 (rootless `serve --mcp`);
  SPEC-011 prose (scope summary + MCP registration paragraph) updated. All three
  rows landed by hand (F-3). Narrative in revision-076.md.

### Supersessions

- DEC-406 supersedes DEC-323 and DEC-332; DEC-407 supersedes DEC-325; all three
  set `superseded` (F-5).

### Withdrawn / tolerated

- F-8 aligned (bookkeeping paths; foreign IMP-407 commit). None tolerated.
