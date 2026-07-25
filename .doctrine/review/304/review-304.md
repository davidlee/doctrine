# Review RV-304 — design of SL-228

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Round 2** of the external adversarial inquisition (raiser label `inquisitor`;
the raising arm is **codex / GPT-5.5 via MCP**, fresh thread — no round-1
context) of the SL-228 design — `.doctrine/slice/228/design.md`, post-RV-303
state (`1b716673`). **Premise: the twelve RV-303 repairs are themselves new,
unreviewed surface.** This trial interrogates the *revised* mechanisms, not a
re-run of RV-303.

Held to:

- **Requirements**: SPEC-021 REQ-384/385/386/387 (FR-008..011), SPEC-022
  REQ-388/389 (FR-010/011) — all `pending`, minted by REV-032. Per FR: does the
  repaired mechanism still satisfy the statement?
- **Ground truth**: `.doctrine/slice/228/extraction.md` (as-built graph at
  `0603f11f`); repairs must verify against the actual code.
- **Governance**: ADR-001 (layering), ADR-006/012 (worktree/dispatch topology),
  ADR-011 (mechanism-over-prose), POL-002 (platform independence), STD-001
  (no magic strings), STD-002 (naming).

Lines of interrogation (soft spots named up front, honestly):

1. **F-1 repair** — tree-identity-modulo-funnel-record conclude gate under
   *parallel mid-funnel phases*: another phase's import between verify and
   conclude touches source paths → refusal — is the refusal loop
   livelock-free (`next` says reverify → import again → …)?
2. **F-2 repair** — `Import{fork_tip, onto}` replay semantics: is `onto`
   stable across a lost-ref-race retry? (Retry composes onto a NEW tip →
   payload mismatch → `already-imported` refusal instead of replay?)
3. **F-4 repair** — DispatchRecord slice+phase binding: durable across the
   crash classes D8 claims? Is the binding provable from git facts, or
   trusted-local?
4. **F-6 repair** — stale-baseline derivation ("parent of the oldest
   not-yet-materialized funnel-provenance commit"): precise under multi-hop
   advances and interleaved non-funnel commits?
5. **F-7 repair** — fail-closed hook: bootstrap ordering (hook installed
   before first funnel commit?); does `dispatch commit`'s own child commit
   pass the hook (escape-env scoping)?
6. **§5 renumber** (now 5 steps) — internal reference consistency.
7. **D6 wording** — "persistence in the engine tier" vs the layering map's
   `dispatch = command`: materially misleading?
8. §6 oracle under parallel phases; §9 ordered-matrix edge cells; §11
   verification gaps opened by the repairs.

Out of scope (do-not-re-raise): locked D1–D8 absent evidence of defect;
RV-300's 7 findings; the 6 internal-pass findings; **RV-303's 12 findings as
raised** (probe whether the *fixes* hold — a broken fix is fresh heresy, a
settled fix is not); SL-225 DEC-003 `worker_commit` validate-skip legs.
