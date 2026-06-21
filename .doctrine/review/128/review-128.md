# Review RV-128 — reconciliation of SL-014

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Surface reviewed:** `refs/heads/candidate/014/audit-001` (dispatch candidate
`cand-014-audit-001`, tip `233ce192`), merge of `refs/heads/review/014`
(dispatch impl-bundle).

**Lines of attack:**

1. **PHASE-01 — `boot --emit` CLI flag.** Does `build_and_render` serve both
   `regenerate` and `run_emit` without double-render? Are stdout bytes identical
   to on-disk bytes? Is `--emit`/`--check` mutual exclusion enforced?

2. **PHASE-02 — Codex hook install.** Does `install_codex_hook` write the
   expected `SessionStart` entry shape into `.codex/hooks.json`? Is it
   idempotent? Do foreign hooks survive merge? Does malformed JSON trigger
   `PrintedFallback` with the correct `CODE_HOOKS_REL` path and `boot_emit`
   snippet? Is the stale-exec-path refresh path tested? Does the ownership
   predicate matrix (10 rows) correctly distinguish our command from boot,
   sync, stamp, shell wrapper, and foreign-tool commands?

3. **DRY extraction — `install_hook_to_file`.** Are both Claude and Codex arms
   single-line delegates? Is `install_claude_hook` behaviour unchanged?

4. **`PrintedFallback` struct variant.** Are all match arms updated (compiler-
   enforced via non-exhaustive enum)? Does `annotate_fallback` correctly map
   path+snippet for both hook files?

5. **Trust surfacing + spike coexistence.** Do trust instructions print on
   `Wired`/`Refreshed` and stay silent on `None`? Does `check_spike_coexistence`
   detect >1 `SessionStart` entry with the codex matcher after write? Is it
   suppressed on `dry_run`?

**Evidence gathered:**
- `cargo clippy` — zero warnings
- `cargo test` — 2235+ tests green, zero failures
- Code review against `design.md` D1–D8 and `plan.toml` VT criteria

## Synthesis

SL-014 ships two clean phases with zero design drift and full test coverage.

**PHASE-01 (`boot --emit`)** is a textbook extraction: `build_and_render` is
factored once, shared by `regenerate` and `run_emit`, and both callers get the
same bytes to their respective sinks (disk and stdout). The `--emit`/`--check`
mutual exclusion is enforced at the clap level. Two tests confirm determinism
and byte-identity.

**PHASE-02 (Codex hook install)** rides the existing `plan_hook` merge core
unchanged — the `.codex/hooks.json` JSON structure is identical to
`.claude/settings.local.json`, so no merge logic was duplicated. The key
additions are:

- `HookSpec::boot_emit` constructor and `is_doctrine_emit_command` ownership
  predicate (suffix-strip ` boot --emit`, shared `is_doctrine_program` — same
  pattern as the three existing predicates, pairwise-disjoint).
- `install_hook_to_file` extraction — DRY: both `install_claude_hook` and
  `install_codex_hook` are one-line delegates. Behaviour-preserving for the
  Claude callers (confirmed by `claude_hook_unchanged_after_dry_refactor`).
- `PrintedFallback` struct variant `{hook_file, snippet}` — compiler-enforced
  exhaustive match arms in `wire()`, matched with `..` rest patterns by
  external callers (`skills.rs`, `corpus.rs`, `install.rs`). `annotate_fallback`
  correctly threads path+snippet from the pure plan through the imperative
  installer, matching the `install_mcp` pattern.
- Trust instructions print on `Wired`/`Refreshed` with the correct
  `[dry-run]` tag suppression. Spike coexistence detection counts >1
  `SessionStart` entries with the codex matcher after write, suppressed on
  `dry_run` (no file written → no spike possible).

Seven new tests cover: expected entry shape, idempotency, foreign
preservation, malformed fallback, stale refresh, full ownership matrix
(10 rows), and Claude-behaviour-unchanged. The weak spot is VT-7/VT-8
(trust-instruction and spike-warning stdout assertions) — tolerated as
low-value stdout sniffing of simple format strings on exercised code paths
(F-1).

**Standing risks:**
- The existing spike `.codex/hooks.json` entry (shell wrapper) in this repo
  will fire concurrently with the canonical hook — the warning prints but
  removal is manual (per design D8).
- Codex project-layer trust is advisory-only — cannot be automated.

**Tradeoffs accepted:**
- The `@`-import guard block remains in `AGENTS.md` — harmless dead content
  for codex (~160 bytes). Removing it would cross into the Claude import
  path; out of scope.
- `PrintedFallback` annotation is duplicated in `install_mcp` (inline) vs
  `install_hook_to_file` (via `annotate_fallback`). The duplication is
  superficial — `install_mcp` uses a different fallback snippet function
  and the MCP plan is not a `HookSpec`, so a single helper would need
  generics or trait plumbing with no net clarity gain.

## Reconciliation Brief

### Per-slice (direct edit)

No per-slice edits needed — the implementation conforms to design without
modification.

### Governance/spec (REV)

No governance or spec edits needed — no ADR, standard, policy, or spec
drift surfaced.
