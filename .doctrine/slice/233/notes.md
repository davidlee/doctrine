# Notes SL-233: CLI-managed design runs and inquiry maps

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Harvest
<!-- single-copy: updated in place each harvest; ids only, never restated content -->
fresh-as-of: 2026-07-27 · design (RV-315 terminal, awaiting lock) · 7de1f628

### Produced

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
- DEC-092 (authored watermark) — **`proposed`, awaiting user acceptance**.
- Scope B2 amended: descent is exactly 2 new specs + SPEC-019 + SPEC-023.
- Round-2 research packet: 6 `pi-scout` threads, `research.md` refreshed
  and baseline re-stamped. **Gitignored tier** (`.gitignore:48`) — exists
  in the primary working tree only.
- ISS-265, ISS-266 (0364bd5b, f36949f5); RFC-011 case notes x3.

### Learned

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
- Detail with citations lives in `research/research.md` § Round 2.

### Open

- QUE-177 — design-run recovery and freshness model
- QUE-178 — governing specification boundary
- QUE-179 — minimum useful inquiry-map schema
- QUE-180 — checkpoint admission and acceptance
- QUE-181 — bounded delegation contract
- QUE-182 — prompt-fragment ownership boundary
- ASM-004 — incremental records improve prolonged-design continuity
- ASM-005 — static activation remains during the experiment
- ASM-006 — existing knowledge kinds fit semantic checkpoints
- Explicit lock + `slice status SL-233 plan` — user's word; RV-315 is
  no longer a blocker and DEC-092 is accepted
- OQ-1 (retrieval of applicable existing records), OQ-2 / RSK-229
  (managed-instruction authority) — carried, non-blocking
