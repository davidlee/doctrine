# Implementation Plan SL-124: Hook-stamp install reliability: heal stale SubagentStart matcher, sanitize (deleted) exec path, prune dead stamp hooks

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

SL-124 fixes ISS-011's two install-writer defects (design.md): **A** a stale
SubagentStart matcher never healed, **B** a `(deleted)`-poisoned exec path baked
into hook commands plus the dead duplicates it spawns. The work splits cleanly into
two independently-verifiable units along the defect boundary:

- **PHASE-01 — exec-path sanitize (B-path).** A single validated resolver
  (`strip_deleted` / `pick_exec` / `resolve_exec`) and the reroute of all seven
  `current_exe()` bake sites. Pure helpers + thin shells; no merge-core change.
- **PHASE-02 — merge normalize (A + B-prune).** Poison-tolerant ownership and the
  `plan_hook` rewrite that converges each event array to one canonical doctrine-sole
  entry.

## Sequencing & Rationale

PHASE-01 first, for two reasons. It is the smaller, lower-risk change (it never
touches the shared merge core that boot/sync ride), so it lands a clean foundation
under the behaviour-preservation gate. And it makes `resolve_exec` produce clean
exec paths, so when PHASE-02's normalize heals an entry toward `spec.command`, that
command is already poison-free — the two halves of Defect B cooperate (sanitize
stops new poison at the source; poison-tolerant normalize reconciles legacy poison
already on disk). The phases are functionally independent, but this order minimises
risk and keeps each phase's proof self-contained.

PHASE-02 is the single coherent merge-core surgery — splitting it would leave a
half-rewritten core. Its load-bearing constraint is the **behaviour-preservation
gate**: the existing boot/sync/stamp hook tests must pass with *no edits to their
bodies* (VA-1). That gate is why normalize is position-preserving (insert at the
first owned hook's execution slot, not the array tail) and why the existing
order-tolerant assertions stay green — verified during design against
`plan_session_hook_refreshes_on_path_change_preserving_foreign` and
`install_claude_hook_wires_boot_and_sync_as_two_entries`.

Both phases are TDD red/green/refactor. The verification ids trace the design's
closure intent: VT-1 (heal), VT-2/3/4 (prune-converge), VT-5/6 (non-clobber +
order), VT-7 (shared-core proof across specs), VT-8 (idempotency).

## Notes

- Five external codex passes hardened the design before planning; the order-bound
  (PHASE-02 VT-6) and the `entry_has_foreign_hook` survival predicate (VT-4) are
  direct products of that review — see design.md § Review log.
- No spec/requirement registry edits here (v1). The slice already links
  `specs SPEC-009`, `requirements REQ-289`.
