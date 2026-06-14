# ISS-013: skills::resolve_agents has no AGENTS.md/codex auto-detect branch

Surfaced by the SL-063 reconciliation (RV-021 F-5). SL-063 fixed
`boot::resolve_harnesses` to detect codex via an inode-gated AGENTS.md check.
Its sibling `skills::resolve_agents` (src/skills.rs:512-522) mirrors the
detection *shape* but is more limited: explicit `--agent` wins, else `.claude/`
→ `[Claude]`, else error. It has **no** `AGENTS.md`/`.codex/` auto-detect
branch at all.

Consequence: it does **not** carry SL-063's exact separate-file-suppression bug
(it never inspects `AGENTS.md`), but a codex-only repo (`AGENTS.md`, no
`.claude/`) cannot auto-detect under the skills install path — it errors,
forcing an explicit `--agent`. Divergence from `resolve_harnesses`, which now
auto-detects that case.

Out of scope for SL-063 by design §7 ("note any divergence as a follow-up,
don't widen scope"). Decide whether to align `resolve_agents` with the
inode-gated detector or leave the skills path explicit-only by design.

Refs: SL-063 design §7, RV-021 F-5, src/skills.rs:512, src/boot.rs:391.
