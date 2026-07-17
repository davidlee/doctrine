# Notes SL-214: Knowledge authoring skill

Durable per-slice scratchpad — tracked in git. The place to lift anything from a
disposable phase sheet (`.doctrine/state/.../phase-NN.md`) that must survive
`rm -rf` before the slice close-out audit harvests it.

## Execution summary (2026-07-17, all phases)

- PHASE-01 (a88e35e5): `/knowledge` SKILL.md + D3 pointers in design/consult/
  preflight. All D2 elements; POL-002 sweep clean (ADR-017 gating teaching
  ships as content, no repo-local ids cited); skill-discovery tests + gate
  green. VT-1..4 PASS.
- PHASE-02 (32bf6abc): consult mid-phase — install resolves from GitHub
  (marketplace slug + npx delegate), so F14 reachability needed the push;
  user pushed, install then selected+installed knowledge (byte-identical).
  Routing row + using-doctrine gating sentence landed AFTER install; boot
  regenerated + --check clean. VT-1..2 PASS.
- PHASE-03 (a2d10d04): dogfood via the routed skill — DEC-002
  (capture-vs-harvest boundary, settled proposed→accepted) and ASM-002
  (touchpoints drive population; held, validation = re-census after SL-215),
  both `shapes → SL-215` only; no dep/seq authored by records.

## Design-premise corrections made en route

- design.md mechanics note reconciled pre-plan: embed root is src/install.rs
  since IMP-226 removed src/skills.rs (commit a6750ffa).
- Scope's "zero records" census is stale: ASM-001/DEC-001/QUE-001/QUE-171/
  CON-001 landed post-census (SL-158/ADR-017 era). Dogfood ids are therefore
  -002, and "first records" reads "first authored through the routed skill".
- Distribution premise gap (consult outcome): "install from rebuilt binary"
  is insufficient — the claude marketplace and the npx skills delegate both
  resolve github.com/davidlee/doctrine, so a publish precedes reachability.

## Harvest candidates for audit

- ISS-226 (verify-vt UNATTRIBUTABLE misreports "keyword present"; also:
  attribution keys off the phase delta, so pre-completion runs are
  systematically UNATTRIBUTABLE — arguably correct, message isn't).
- ISS-227 (stale `needs`/`supersede` --help vs SL-158/SL-097 gates).
- Memory corrected: mem.pattern.distribution.skill-refresh-command
  (src/skills.rs → src/install.rs + debug-embed caveat).
- Stale shipped memory (NOT fixed — shipped corpus, out of scope):
  mem.signpost.doctrine.knowledge has six kinds + wrong status vocab
  (src seeds ASM=held, DEC=proposed). Backlog candidate.
- Memory candidate: rust-embed `debug-embed` means NO disk fallback in debug;
  every install/ or plugins/ asset edit needs cargo build before boot/install
  reflects it, and PATH doctrine is an old release binary — use
  ./target/debug/doctrine.

## Outstanding VH gates (for audit)

- PHASE-02 VH-1: human confirms F14 sequencing + boot snapshot reads well.
- PHASE-03 VH-1: human accepts DEC-002/ASM-002 as fair first citizens.
