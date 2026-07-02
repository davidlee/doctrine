# Review RV-212 — code-review of SL-188

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

SL-188 unifies three CLI id-form conventions (prefixed-only, bare-only, both)
into one: accept both `SL-123` and `123` everywhere. Three commits:

1. Phase 1: `value_parser = parse_cli_id` on SelectorCommand + dispatch CLI structs
2. Phase 2: `parse_resolvable_ref` replacing `parse_canonical_ref` in relation,
   facet, tag, dep_seq, supersede, map verbs
3. Fix: existence check in `parse_resolvable_ref`, memory-link regression fix

**Lines of attack:**

- `parse_resolvable_ref` scans all KINDS for bare numbers — correctness of the
  first-match-wins scan in a corpus with pervasive cross-kind id overlap.
- Memory-link branch in `relation.rs` — the syntactic parse + resolution check
  split; verify control flow is correct for free-text vs. entity targets.
- `ensure_ref_resolves` redundancy with `parse_resolvable_ref` post-merge.
- `validate_focus` in `map.rs` — bare numbers accepted without resolution check.
- Test suite invariant: all existing tests pass unchanged (claimed 2,909 green).

## Synthesis

**Overall:** revision-required

**Synopsis:**

SL-188 fixes the single most expensive recurring friction in the RFC-011 case
notes — the three-way id-form split. The mechanical changes (Phase 1: clap
value_parser additions, Phase 2: parse_canonical_ref → parse_resolvable_ref)
are clean and correctly unified on the existing `parse_entity_ref` pattern.

The new `parse_resolvable_ref` function correctly adds existence validation to
the parse step — an improvement over the old two-step parse-then-resolve
pattern. However, its bare-id scan is **silently wrong** in a corpus where
cross-kind id overlap is pervasive (F-1, blocker). Bare `188` silently resolves
to SL-188 despite IMP-188, REQ-188, and RV-188 also existing — the first match
wins with no error. Bare `1` matches 17 entities. The function must detect
ambiguity and reject with an actionable list of colliding entities.

The changes also surfaced a latent test dependency: the e2e dep_seq test used a
non-existent ADR-001 to exercise the work-like gate, which now fails at the
existence check instead (F-2, blocker). The test must seed the entity.

`ensure_ref_resolves` has a redundant second existence check — dead code that
survived the refactor (F-3, major). Remove it.

`validate_focus` intentionally defers resolution to downstream (F-4, minor) —
tolerated given map serve is a dev tool.

The memory-link branch's dual-parse heuristic is correct but fragile (F-5,
nit) — captured as IMP-229 for future refactoring.

**Standing risks:**

- F-1 is the only correctness risk. Until fixed, bare ids are dangerous in any
  corpus with cross-kind id overlap (i.e., all real doctrine repos).
- No other correctness issues found. The parse_resolvable_ref pattern is sound;
  the ambiguity gate is the missing piece.

**Haiku:**

bare number spoken —
seventeen voices answer,
but the first one wins.
