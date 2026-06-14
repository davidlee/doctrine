# boot install wires AGENTS.md for non-claude harnesses

## Context

`doctrine boot install` wires the `@.doctrine/state/boot.md` import (the
governance snapshot / "sector") into each harness's committed entry file —
Claude reads `CLAUDE.md`, codex reads `AGENTS.md` (`import_targets`,
boot.rs:378). The set of harnesses comes from `resolve_harnesses`
(boot.rs:390): explicit `--agent` wins, else auto-detect by marker.

The auto-detect carries a `!claude` guard (boot.rs:399): when `.claude/` is
present it refuses to also detect codex, *even when a real `AGENTS.md` exists*.
The intent (comment, boot.rs:386-389; test, boot.rs:1677) was this repo's
pattern — `CLAUDE.md → AGENTS.md` symlink — where `AGENTS.md` is just Claude's
import target via one inode, not a separate codex surface.

The assumption fails when `AGENTS.md` is a **separate file** (a genuine codex
surface in a repo that also has `.claude/`). There, auto-detect picks Claude
only, so `AGENTS.md` never receives the `@`-import and the non-claude harness
boots with no governance snapshot — silently. In *this* repo the bug is masked:
the symlink means the single `CLAUDE.md` write lands on the shared inode, so
`AGENTS.md` appears wired.

The write seam already handles the symlink correctly on its own:
`ensure_boot_import` dedups by **resolved inode** (`resolve_target`
canonicalizes, boot.rs:451-467). So same-inode targets collapse to one write
regardless of detection — the `!claude` guard is redundant for the symlink case
it was written for, and wrong for the separate-file case.

## Scope & Objectives

- Fix auto-detection so a repo with `.claude/` **and** a separate-inode
  `AGENTS.md` wires **both** harnesses (CLAUDE.md + AGENTS.md), while a repo
  whose `AGENTS.md` is a symlink onto `CLAUDE.md` stays Claude-only (one inode,
  one write — this repo's behaviour unchanged).
- The discriminator is inode identity, not mere existence: detect codex when
  `.codex/` exists, or when `AGENTS.md` exists and does **not** resolve to the
  same inode as `CLAUDE.md`.
- Update the pinned test (boot.rs:1673 `resolve_harnesses_auto_detects_by_marker`)
  to cover the new split: symlinked AGENTS.md → Claude-only; separate-inode
  AGENTS.md alongside `.claude/` → both.
- Verify the report/refresh path: codex is import-only (no SessionStart hook,
  boot.rs:766) so additionally detecting it must not perturb hook wiring in this
  repo.

## Non-Goals

- No change to explicit `--agent` resolution (already correct).
- No change to the snapshot content, the hook-merge core, or `import_targets`.
- No new harness kinds; codex/claude only.
- Not touching `skills::resolve_agents` (the sibling detector `resolve_harnesses`
  mirrors) — note any divergence as a follow-up, don't widen scope.

## Affected Surface

- `src/boot.rs` — `resolve_harnesses` (detection), and its unit test
  `resolve_harnesses_auto_detects_by_marker`.
- Possibly a small helper to compare two paths' resolved inodes (may reuse
  `resolve_target`).

## Risks / Assumptions / Open Questions

- **OQ-1 (design crux) — CLOSED:** "review fix #1" is the *one-file-per-harness*
  principle (no double-inline), untouched by this fix. The detection rule is the
  separate piece refined here. SL-011's inquisition confirmed Claude does not read
  AGENTS.md natively, so wiring a distinct AGENTS.md for codex is consistent with
  fix #1. Discriminator: AGENTS.md triggers codex unless it is Claude's inode-alias
  (alias-suppression gated on Claude-detected — see design §3.1).
- **ASM-1:** in this repo `CLAUDE.md` and `AGENTS.md` canonicalize to one inode;
  the symlink branch must be exercised by a test using a real symlink (tmpdir),
  not assumed.
- **R-1:** `resolve_harnesses` and `skills::resolve_agents` share detection
  shape; fixing one may surface the same latent bug in the other (follow-up,
  not in scope here).
- **Sibling (ISS-012):** same `doctrine claude install` / AGENTS.md surface, but
  a different mechanism — a too-broad `.doctrine/agents/*` gitignore that swallows
  an authored AGENTS.md and breaks the worktree classifier. Distinct fix; tracked
  separately. Cross-referenced because both stem from `claude install` treating
  AGENTS.md inconsistently — coordinate if either touches the shared install seam.
- **OQ-2 — CLOSED:** order is Claude-first by construction and not significant
  (inode dedup collapses same-inode targets regardless). Adversarial review
  surfaced + closed a real edge: a CLAUDE↔AGENTS symlink pair with **no
  `.claude/`** must still resolve `[Codex]` — handled by gating alias-suppression
  on Claude-detected (design §3.1, test case 5).

## Verification / Closure Intent

- Unit: separate-inode AGENTS.md + `.claude/` → `[Claude, Codex]`; symlinked
  AGENTS.md + `.claude/` → `[Claude]`; bare AGENTS.md → `[Codex]`; CLAUDE↔AGENTS
  symlink pair, no `.claude/` → `[Codex]`; none → error.
- Behaviour-preservation: this repo's `boot install` still produces a single
  effective write (inode dedup) and unchanged hook wiring.
- `just gate` green; clippy zero warnings.
