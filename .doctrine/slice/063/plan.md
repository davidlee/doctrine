# Implementation Plan SL-063: boot install wires AGENTS.md for non-claude harnesses

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

One bug, one function, one test. The defect is a detection rule, not a write-seam
flaw: `resolve_harnesses` refuses codex whenever `.claude/` is present, even when a
genuine separate-inode `AGENTS.md` is a real codex surface. The write seam already
dedups by resolved inode (`ensure_boot_import`), so the fix is purely in detection —
swap the coarse `!claude` existence guard for the inode-gated alias-suppression of
design §3.1. The change is small enough that decomposing into multiple phases would
add ceremony without isolating any real risk. A single phase carries it.

## Sequencing & Rationale

**PHASE-01 — Inode-gated codex detection.** TDD red/green/refactor on one seam:

1. **Red** — add the failing case first: `.claude/` + separate-inode `AGENTS.md`
   asserting `[Claude, Codex]` (VT-1). Under today's `!claude` guard it returns
   `[Claude]` and fails. This is the bug made executable.
2. **Green** — apply §3.1: delete the guard, add `agents_is_claude_alias` (gated on
   `claude` AND inode-equality via `resolve_target`), detect codex when AGENTS.md
   exists and is not that alias. No signature change, no new helper.
3. **Refactor / extend** — round out `resolve_harnesses_auto_detects_by_marker` to
   all six §5 cases, replacing the old bug-encoding fixture (regular AGENTS.md beside
   `.claude/` asserting `[Claude]`) with the symlink split (VT-2) and the adversarial
   lone-symlink-pair edge (VT-3). The symlink cases use a real
   `std::os::unix::fs::symlink` in a tmpdir, never an assumed inode (ASM-1).

The behaviour-preservation gate is load-bearing here (EX-3 / VT-5): the write seam,
`import_targets`, and the one-file-per-harness rule are untouched, so their suites
must stay green *unchanged*. Any edit forced into them would signal scope leak.

## Notes

- **Verify against a fresh dev build, not the installed jail binary** (EN-2) — the
  cited line numbers and the `resolve_target` seam are confirmed at HEAD; a stale
  installed binary has previously masked source state on this surface.
- **Out of scope, do not widen:** `skills::resolve_agents` mirrors this detection
  shape and likely carries the same latent bug (R-1) — file a backlog item if
  confirmed, don't fix here. Sibling ISS-012 touches the same `claude install`/
  AGENTS.md surface via a different mechanism (gitignore breadth); coordinate only
  if either touches the shared install seam.
