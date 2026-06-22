# IDE-020: Seed project-orientation memory during install

From a conversation about the boot→overview→orientation chain: when a user
runs `doctrine install`, we should seed a `.doctrine/memory/items/<key>/`
memory with a templated onboarding page that agents load at boot time.

**What it is**: a `signpost` memory in `items/` (user-editable, committed)
whose body starts as an onboarding template — "edit this to describe your
project". The shipped overview memory (`mem.signpost.doctrine.overview`)
already tells agents to look for a local key-based memory; this seeds that
slot.

**Key questions**:
- Should the seeded memory be created via `run_record` (reuses all scaffolding
  but captures install-time git frame — fine, but needs idempotency guard)
- Or via a direct write path in `install.rs` with `anchor_kind = "none"`
  (no git frame at all, but duplicates some template rendering)
- Body content: the seeded `memory.md` should be a meaningful onboarding guide
  template, not a blank page
- The seed template could live alongside other `install/templates/` files

**Implementation surface**:
1. New `[memory.seed]` section in `install/manifest.toml`
2. A loop in `install.rs` execute_plan (or run_forward_steps) that writes
   the memory fileset for each seeded key
3. A seed template (e.g. `install/templates/seed-onboarding.md`)
4. Idempotency check: skip if key symlink already exists
5. Tests: idempotency, content correctness, manifest parse

**Precedent**: `install/manifest.toml` already lists `[dirs].create` entries
for tree scaffolding; this extends that pattern to also seed content.

**Completed in this session (2026-06-22)**:
- `seed_by_key()` in memory.rs — silent, unanchored, reuses all template rendering
- `SeedItem`/`MemorySection` manifest structs + deserialization
- `seed_authoring_memories()` loop in install.rs
- `install/templates/seed-onboarding.md` — guide with editable sections
- 3 tests: creation + unanchored, idempotency, type/status correctness
- 2403 unit tests + integration tests all green, clippy clean
