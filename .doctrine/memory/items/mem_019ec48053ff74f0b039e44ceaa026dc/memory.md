# Detection identity must agree with the write seam's identity

When a detection/read path and a write path both reason about "is this the
same file?", they must share **one** identity notion. SL-063: `boot.rs`
`resolve_harnesses` auto-detect used a coarse `!claude` existence guard to decide
whether `AGENTS.md` was just Claude's symlink alias — but the write seam
(`ensure_boot_import`) deduped by **resolved inode** via `resolve_target`. The two
disagreed: a separate-inode `AGENTS.md` beside `.claude/` was suppressed by
detection yet would have deduped correctly at write. Fix reused `resolve_target`
for the detection compare (`agents_is_claude_alias = claude && agents.exists() &&
resolve_target(agents) == resolve_target(CLAUDE.md)`) — no new helper, no second
notion of identity.

**Why:** an "is-a-symlink" test is a *proxy* for "same file" — it misses
hardlinks, reverse-direction symlinks, and chains. Inode-equality is the real
property; the write seam already had it, so detection should borrow it, not
reinvent a weaker one.

**How to apply:** when adding a detect/skip branch that mirrors a dedup the write
layer already performs, reuse the write layer's identity helper rather than a
structural shortcut. Gate any alias-*suppression* on the aliased party actually
being claimed (SL-063 gated suppression on `claude`, else a lone symlink pair
with no `.claude/` would wrongly suppress codex). Sibling detectors that share
the *shape* may carry a related-but-distinct gap — check, don't assume identical
(see [[mem.pattern.entity.write-seam-canonicalizes-every-id-axis-the-read-view-does]];
SL-063 R-1 → ISS-013).
