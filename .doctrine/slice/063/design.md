# SL-063 Design — boot install wires AGENTS.md for non-claude harnesses

## 1. Problem

`doctrine boot install` wires the `@.doctrine/state/boot.md` import (the
governance snapshot) into each detected harness's committed entry file —
Claude ← `CLAUDE.md`, codex ← `AGENTS.md` (`import_targets`, boot.rs:378-383).
Harnesses come from `resolve_harnesses` (boot.rs:390): explicit `--agent` wins,
else auto-detect by marker.

Auto-detect carries a `!claude` guard (boot.rs:399):

```rust
if root.join(".codex").exists() || (root.join("AGENTS.md").exists() && !claude) {
    found.push(Harness::Codex);
}
```

When `.claude/` is present it refuses to detect codex **even if a real
`AGENTS.md` exists**. The SL-011 intent (design.md:169-179) was this repo's
pattern: `CLAUDE.md → AGENTS.md` symlink, where AGENTS.md is merely Claude's
import target via one inode, not a separate codex surface.

The guard's assumption — "AGENTS.md beside `.claude/` is the symlink" — is too
coarse. When AGENTS.md is a **separate file** (a genuine codex surface in a repo
that also runs Claude), auto-detect picks Claude only, so AGENTS.md never
receives the `@`-import and the codex harness boots with no governance snapshot,
silently. In *this* repo the bug is masked: the symlink means the single
`CLAUDE.md` write lands on the shared inode, so AGENTS.md appears wired.

This refines the SL-011 *detection* rule only. The "one-file-per-harness"
principle (review fix #1: each harness claims exactly one entry file so the
snapshot never inlines twice for one agent) is **unchanged** — Claude still
reads only CLAUDE.md; SL-011's inquisition confirmed Claude does not read
AGENTS.md natively. Wiring a *distinct* AGENTS.md for codex is consistent with
fix #1, not a reversal.

## 2. Current vs target behaviour

`resolve_harnesses` auto-detect (explicit `--agent` path unchanged throughout):

| repo state | current | target |
|---|---|---|
| `.claude/` + AGENTS.md symlink→CLAUDE.md (this repo) | `[Claude]` | `[Claude]` — unchanged |
| `.claude/` + separate-inode AGENTS.md | `[Claude]` ✗ | `[Claude, Codex]` — **fixed** |
| bare AGENTS.md, no `.claude/` | `[Codex]` | `[Codex]` |
| CLAUDE↔AGENTS symlink pair, **no `.claude/`** | `[Codex]` | `[Codex]` — preserved (edge) |
| `.codex/` (any AGENTS.md) | includes Codex | includes Codex |
| neither marker | error | error |

## 3. Design

### 3.1 Discriminator: AGENTS.md triggers codex unless it is merely Claude's alias

Detect codex when `.codex/` exists, **or** AGENTS.md exists and is **not** merely
Claude's inode-alias. The `!claude` existence special-case is replaced by an
inode check that is *gated on Claude actually being detected* — so AGENTS.md is
suppressed only when it is the alias of a CLAUDE.md that Claude already claims.

```rust
fn resolve_harnesses(explicit: &[String], root: &Path) -> anyhow::Result<Vec<Harness>> {
    if !explicit.is_empty() {
        return explicit.iter().map(|s| parse_harness(s)).collect();
    }
    let mut found = Vec::new();
    let claude = root.join(".claude").exists();
    if claude {
        found.push(Harness::Claude);
    }
    let agents = root.join("AGENTS.md");
    // AGENTS.md is "merely Claude's alias" only when Claude is detected AND
    // AGENTS.md resolves to CLAUDE.md's inode (this repo's symlink). Otherwise a
    // present AGENTS.md is a real codex surface — including a lone symlinked pair
    // with no `.claude/`, which must still wire something.
    let agents_is_claude_alias = claude
        && agents.exists()
        && resolve_target(&agents) == resolve_target(&root.join("CLAUDE.md"));
    if root.join(".codex").exists() || (agents.exists() && !agents_is_claude_alias) {
        found.push(Harness::Codex);
    }
    if found.is_empty() {
        bail!(
            "No --agent given and no .claude/ or .codex/ (or AGENTS.md) found. \
             Pass --agent <claude|codex>."
        );
    }
    Ok(found)
}
```

**Why gated on `claude`** (adversarial-review fix): an ungated
`separate_agents != same-inode` test would error on a repo with a
CLAUDE.md↔AGENTS.md symlink pair but **no `.claude/`** — old code wired that as
`[Codex]`; an ungated check sees one inode, suppresses codex, finds nothing,
errors. Gating the alias-suppression on Claude-detected preserves "a present
AGENTS.md wires codex unless Claude already owns that inode."

### 3.2 Why inode-equality, not "is-a-symlink"

- **Consistency with the write seam.** `ensure_boot_import` already dedups by
  the resolved inode via `resolve_target` (canonicalize-or-literal-fallback,
  boot.rs:451-453, 466-467). Reusing the same helper means detection and write
  agree on what "the same file" means — no second notion of identity.
- **Robust to indirection.** Symlink chains, the reverse symlink
  (AGENTS.md→CLAUDE.md *or* CLAUDE.md→AGENTS.md), and hardlinks all collapse to
  one inode and are treated identically. A bare "is symlink" test would miss
  hardlinks and mis-handle reverse direction.
- **Correct fallback.** `resolve_target` returns the literal path when
  canonicalize fails. So a present AGENTS.md with **no** CLAUDE.md yields
  distinct paths ⇒ codex detected — right, since a lone AGENTS.md is a real
  codex surface. (This is the `agents.exists()` gate doing its job; CLAUDE.md
  need not exist.)

### 3.3 Interaction with the write seam (no double-wire)

When both harnesses are detected, `install`/`install_dry` union their
`import_targets` (boot.rs:841) → `[CLAUDE.md, AGENTS.md]`, then
`ensure_boot_import` dedups by resolved inode:

- **This repo (symlink):** with `.claude/` present and AGENTS.md aliasing
  CLAUDE.md, §3.1's `agents_is_claude_alias` is *true* → codex isn't detected →
  Claude-only, single `[CLAUDE.md]` target. Install output byte-identical to
  today. The inode dedup in `ensure_boot_import` is the second line of defence,
  not the primary mechanism here.
- **Separate files:** two inodes → two writes → both entry files wired. The fix.

### 3.4 Hook wiring unaffected

Codex is import-only — it contributes no SessionStart hook
(`harness_hook`/refresh returns `None` for codex, boot.rs:766). Additionally
detecting codex in a separate-file repo adds an AGENTS.md import write but
leaves hook merge untouched. In this repo codex is not detected at all, so hook
behaviour is provably unchanged.

## 4. Code impact

- `src/boot.rs` — `resolve_harnesses` body (§3.1). Net: delete the `!claude`
  guard, add the `separate_agents` inode check. No signature change, no new
  helper (reuses `resolve_target`).
- `src/boot.rs` tests — `resolve_harnesses_auto_detects_by_marker` (boot.rs:1672)
  updated and extended (§5).

No change to: `import_targets`, `plan_boot_import`, `ensure_boot_import`,
`resolve_target`, the hook merge, or the snapshot content.

## 5. Verification

Unit tests in `resolve_harnesses_auto_detects_by_marker` (rewritten) plus a new
case for the symlink split. Cases:

1. **bare AGENTS.md, no `.claude/`** → `[Codex]`. (existing, retained)
2. **`.claude/` + separate-inode AGENTS.md** → `[Claude, Codex]`. (**new — the
   fix**; write a real distinct AGENTS.md file)
3. **`.claude/` + AGENTS.md symlink→CLAUDE.md** → `[Claude]`. (**new** — exercises
   the inode-equality branch with a real `std::os::unix::fs::symlink`; replaces
   the old line-1677 assertion that pinned the coarse behaviour)
4. **`.codex/` + `.claude/`** → `[Claude, Codex]`. (codex marker path)
5. **CLAUDE↔AGENTS symlink pair, no `.claude/`** → `[Codex]`. (**new** — the
   adversarial edge; proves alias-suppression is gated on Claude-detected)
6. **neither** → error. (existing `resolve_harnesses_errors_when_none`, retained)

Behaviour-preservation: `import_targets_is_one_file_per_harness`,
`ensure_boot_import_dedups_same_inode_to_one_write`, and the existing
install/refresh tests stay green unchanged — the write seam and one-file rule
are untouched. The only revised test is the one that encoded the bug
(`resolve_harnesses_auto_detects_by_marker`, boot.rs:1673, whose line-1679
fixture wrote a *regular* AGENTS.md beside `.claude/` and asserted `[Claude]`).

No e2e golden churn: the sole auto-detect caller is `run_install` (boot.rs:808),
and there is no `boot install` e2e fixture exercising a `.claude/` + AGENTS.md
repo. `tests/e2e_claude_install.rs` drives the *separate* `claude install` verb
with an explicit `--agent claude`, so it never reaches `resolve_harnesses`
auto-detect — unaffected.

Gate: `just gate` green; `cargo clippy` zero warnings.

## 6. Constraints / doctrine

- **Pure/imperative split.** `resolve_harnesses` already lives in the imperative
  shell (touches disk via `.exists()`/`canonicalize`). The change adds only more
  disk reads in the same layer — no purity regression.
- **Behaviour-preservation gate.** Shared boot machinery: existing suites are
  the proof and must stay green unchanged (only the one detection test that
  encoded the *bug* is revised).
- **Clippy bans (memory):** no `as` casts, no `HashSet/HashMap`, `expect`+reason
  not bare `allow`, no indexing-slicing — none triggered by this change.

## 7. Open questions / follow-ups

- **R-1 (follow-up, out of scope):** `skills::resolve_agents` mirrors this
  detection shape (design.md:179) and likely carries the same latent bug. File a
  backlog item if confirmed; do not widen this slice.
- **Sibling ISS-012 (cross-ref):** same `claude install`/AGENTS.md surface,
  different mechanism (too-broad `.doctrine/agents/*` gitignore). Coordinate only
  if either touches the shared install seam.
- **OQ (closed):** order of `[Claude, Codex]` in the report — Claude first by
  construction; dedup collapses same-inode targets regardless of order, so order
  is not significant to the write.
