# Review RV-021 — reconciliation of SL-063

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Reconciles SL-063 — inode-gated codex auto-detect — against `design.md` (§3.1
discriminator, §3.3 no-double-wire, §3.4 hook, §5 cases), `slice-063.md` scope,
and `plan.toml` EX/VT. Single phase PHASE-01, implementation committed at
`18df431`. Lines of attack:

1. **Discriminator fidelity (§3.1).** Does `resolve_harnesses` (boot.rs:391)
   implement the inode-gated alias-suppression *exactly* — alias-suppression
   gated on `claude` so a lone CLAUDE↔AGENTS symlink pair (no `.claude/`) still
   wires codex? Reuses `resolve_target`, no new helper, no signature change?
2. **§5 six-case coverage.** All six cases present in
   `resolve_harnesses_auto_detects_by_marker` + `resolve_harnesses_errors_when_none`,
   with **real** symlinks (not assumed), and green?
3. **Behaviour-preservation (EX-3/VT-5).** `import_targets_is_one_file_per_harness`
   + `ensure_boot_import_dedups_same_inode_to_one_write` + install/refresh suites
   pass UNCHANGED — the write seam and one-file rule untouched?
4. **No double-wire (§3.3).** Both-detected unions `[CLAUDE.md, AGENTS.md]`, then
   `ensure_boot_import` dedups by resolved inode → one write per inode?
5. **Hook wiring (§3.4).** Codex import-only — `install_refresh` returns `None`;
   detecting codex never perturbs hook merge?
6. **Doc comment de-stale.** The comment above `resolve_harnesses` matches §3.1,
   not the old coarse `!claude` rule?
7. **R-1 carry-forward.** `skills::resolve_agents` mirror — confirm disposition
   (backlog follow-up), do not widen (§7).

Gate evidence: `just check` exit 0, `cargo clippy` no issues, the 5 detection +
preservation tests green.

## Synthesis

**Closure story.** SL-063 reconciles cleanly against design and governance. The
single-phase fix — replacing the coarse `!claude` existence guard in
`resolve_harnesses` with inode-gated alias-suppression — landed exactly as
designed (§3.1): codex is detected on `.codex/`, or on an `AGENTS.md` that is
not Claude's inode-alias, with the suppression *gated on Claude being detected*
so the adversarial edge (a lone `CLAUDE↔AGENTS` symlink pair with no `.claude/`)
still wires codex. The change reuses `resolve_target` — no new helper, no
signature change (§4 honoured). The doc comment was de-staled to match.

All six §5 cases are pinned with **real** symlinks in
`resolve_harnesses_auto_detects_by_marker` (+ `resolve_harnesses_errors_when_none`)
and pass. The behaviour-preservation gate holds: `import_targets_is_one_file_per_harness`
and `ensure_boot_import_dedups_same_inode_to_one_write` pass unchanged, proving
the write seam and one-file rule are untouched and that §3.3's no-double-wire
(union `[CLAUDE.md, AGENTS.md]` → resolved-inode dedup → one write) still holds.
§3.4 reconciled: codex is import-only (`install_refresh → None`), so detecting
it never perturbs hook merge — and this repo doesn't detect codex anyway, so its
install output is byte-identical. `just check` exit 0, clippy clean.

**Standing risks / accepted tradeoffs.** None blocking. One conscious carry-forward:
the sibling `skills::resolve_agents` (F-5) mirrors the detection *shape* but has
no `AGENTS.md`/`.codex/` auto-detect branch at all — so it does **not** carry the
identical bug, but a codex-only repo can't auto-detect under the skills install
path. Scoped out by design §7 and captured as **ISS-013** (follow-up), not fixed
here. Cross-ref ISS-012 (same `claude install`/AGENTS.md surface, different
mechanism — too-broad `.doctrine/agents/*` gitignore) noted, untouched.

**Verdict:** ledger done, await=none, no `blocker`. Ready for `reconcile` → `/close`.
