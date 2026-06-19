# Notes SL-100: Memory lifecycle verbs and agent UX hardening

## Closure summary

**Driven by:** `/dispatch` (4 workers, serial funnel)
**Audited:** RV-089 (2 findings — 1 blocker fixed, 1 aligned)
**Integrated:** `cand-100-close-001` merged onto main at `2ef18387`

**Delta:** 8 files, +1822/-34 lines
- `src/tag.rs` — leaf-tier tag normalization extracted from backlog
- `src/backlog.rs` — imports from tag.rs, local copy removed
- `src/memory.rs` — three pure cores + IO shells (tag, status, edit) + 52 tests
- `src/main.rs` — MemoryCommand::{Tag, Status, Edit} CLI wiring
- `plugins/doctrine/skills/` — record-memory, retrieve-memory updated; reviewing-memory, dreaming new

**Gate:** 1837 tests pass, clippy zero warnings

**Standing risks:**
- Two-write non-atomic supersede (relation before status — benign)
- No transition-legality matrix (any→any, vocab-gated)
- Trunk drift during dispatch: main moved 4+ commits ahead of fork-point; candidate merge resolved cleanly

**Harvested:**
- `mem.pattern.skills.yaml-frontmatter-colons` — YAML colons in SKILL.md descriptions break parsing
- `mem.pattern.dispatch.cherry-pick-loses-unstaged-edits` — orchestrator manual fixes after cherry-pick must be re-staged

**Follow-ups:**
- `--lifespan ""` to clear (deferred, hand-edit TOML for now)
- Scope-array append semantics (deferred, pass full array for now)
