# Notes SL-286: Codex MCP registers at user level with an absolute path

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-10-24 · close · 84213d245+

### Produced

- Plan: plan.toml / plan.md, five phases (924373d8f, 8a768671a). Plan-time
  constraint: codex_mcp imports no command module; callers pass exec, so the
  command tangle baseline holds (see plan.md § Sequencing).
- PHASE-01 done — rootless `serve --mcp` (d81d63eaf). Adds `root::find_opt`,
  `root::DOCTRINE_DIR`, `root::is_doctrine_project`, and the `ProjectRoot`
  resolution in mcp_server. Behaviour change: an explicit `--path` without
  `.doctrine/` is now rootless. Gate green; negative control observed red.
- PHASE-02 done — `src/codex_mcp.rs` pure plan + consent/report text,
  `fsutil::find_on_path`; codex vocabulary + `is_wrapper_form` moved out of
  boot, boot suites unchanged (23a963447..3112de99c). Gate green at 3112de99c;
  VT-1..3 PASS; negative controls red. Per-phase pi `review` (max): 10
  findings — 8 fixed (a742bc319), F5 rejected (design example binds text),
  F2 → DEC-412 item 9 by user ruling. Boundary carries foreign 4aea4ff19
  (IMP-407) mid-range — audit disposes it.
- PHASE-03 done — codex_mcp `execute` shell: `CodexCli`/`ProcessCodex`,
  `Consent`, `CodexUserOutcome`, `remove_wrapper`, new report lines;
  `fsutil::write_atomic_through_link` (40f2c57d0..49e58e65e). Gate green at
  49e58e65e; VT-1/VT-2 PASS; negative controls red. Per-phase pi `review`:
  10 findings — F1 blocker (my ruling put symlink write-through on shared
  `write_atomic`) fixed 4717b1f4e; 7 more fixed 49e58e65e; F6, F9 rejected.
- PHASE-04 done — install and boot install run the codex user MCP leg; boot's
  project-wrapper arm retired (ad689e094..224fb8fc1). `doctrine_cmd` pins a
  never-created CODEX_HOME first (ad689e094). New seams: `codex_mcp::read_plan`
  (one gather), `ask_consent`, `preview`; `Leg::CodexUserMcp`;
  `tests/common::link_doctrine_into` (PHASE-05 fake-codex tests reuse it).
  Gate green at 224fb8fc1; VT-1..3 PASS; VA-1 greps hold; negative controls
  red; real-binary smoke with a fake codex confirmed dry-run / -y / decline.
  Per-phase pi `review`: F1 (relative HOME read the project file as the user
  entry) fixed; F2/F3 comments fixed; F4, F5 rejected; F6 already fixed.
- PHASE-05 done — codex e2e under `tests/common::ProposingEnv`
  (44abc2777..9f802dc6a); gate green at 9f802dc6a; VT-1 PASS; T5 negative
  control red. Per-phase pi review: F1–F4, F6 fixed (9f802dc6a), F5 rejected.
  VH-1 PASS (user report 2026-10-24): no EVD-032 recurrence. VH-2 → EVD-034:
  codex never shows the rootless instructions; ASM-014 invalidated (DEC-405
  holds — the empty tool list is the only signal).
- Minted: ISS-533 — MCP server replies to JSON-RPC notifications (pre-existing).
- Design-time decisions DEC-403..DEC-411; EVD-033; RV-427 (concluded).
- Audit: RV-428 (reconciliation, concluded, 13 findings verified; brief in
  review-428.md). Fixed in audit: PHASE-04 VT-3 keyword regression (from
  9f802dc6a), stale codex_mcp comment in ADR-001 layering.toml.

### Learned

- (memory mem_01a08024f2a57f60b7606dbd24b48acf appended) `doctrine check gate` does not run `slice verify-vt`: a refactor that moves an
  asserted literal can fail a VT keyword check under a green gate (RV-428 F-1).

- mem.pattern.delegation.pi-writable-in-worktree — calibration appended
  (subtree write roots, judgement no-answer turns, transcript budget);
  IMP-522 update (0e99dbcbf): repo-root writes, cargo + git in the write form;
  PHASE-03 update: ceiling on function+tests runs, shared-seam ruling check.
  PHASE-04 update: review ceiling recovery, gate-only convention suites,
  retirement + retarget in one commit. PHASE-05 update: fixture PATH order,
  argv-separated fakes, swap-the-command negative control.
- EVD-034 — codex hides MCP initialize instructions (ASM-014 invalidated).
- EVD-033 — `codex mcp add` replaces the whole table; `mcp get` shows the
  effective entry (ASM-015 invalidated).

### Open

- Reconciled (84213d245): design.md folds, selectors, README, REV-076 (REQ-484
  rewritten, REQ-485 introduced), DEC-323/332 → DEC-406, DEC-325 → DEC-407.
  The DEC-412..414 and reconcile items below are now landed.

- DEC-412 — PHASE-02 design deltas (FileRead, destination split, blocking
  project entries, degraded/removal-only text); reconcile folds into design.md
  and amends PHASE-02 EX-4. Item 8 wording user-approved.
- DEC-413 — PHASE-03 execute deltas (project path param, write-time re-check,
  opt-in symlink write-through, new report lines); reconcile folds into
  design.md. Items 7, 8 user-approved.
- DEC-414 — PHASE-04 wiring deltas (one gather in codex_mcp, exec before the
  plan, consent seam, dry-run heading, leg mapping, boot install Ok on Refused,
  relative HOME → NoHome); reconcile folds into design.md.
- RSK-234 · IDE-069 · ISS-528 (partially resolved by DEC-406) · ISS-533 ·
  IMP-497 (live seam re-verified by VH-1; post-write `codex mcp get` assertion not done).
- Reconcile: REV for REQ-484 (SPEC-011) + rootless-serve requirement;
  supersede DEC-323, DEC-332 → DEC-406 and DEC-325 → DEC-407; README rewrite.

## Design review passes

- **RV-427** (2026-10-23/24): adversarial pass by pi (quotient `review` profile,
  deepseek-flash, max thinking), run in place of codex (budget). Five rounds,
  20 findings, all verified; the final round raised none.
- **Further pass: none needed before lock.** The bake rule (DEC-404) was the
  only area still producing findings late, and the PATH-identity replacement
  closed it with two small fixes that verified clean. What a further pass would
  probe, if wanted at audit: the consent ordering against install's existing
  `Proceed?` flow in the real code (DEC-381); the `codex_mcp` module boundary
  against ADR-001 layering; and the VH items (EVD-032 recurrence check,
  ASM-014 disclosure visibility).
