# DEC-110: Capsule admission reuses conflict semantics only

Supersedes [[capsule-admission-derived-commit]] (DEC-106), which solved the
wrong problem by preserving machinery it should have questioned.

**Decision.** RT-2's reuse mandate binds **conflict and staleness semantics**,
not transport or staging. Operator ruling, 2026-08-01.

| | |
|---|---|
| **Reuse, mandatory** | `candidate create`'s 3-way merge and `Conflicted` refusal, `admit`'s OID pin, integrate's CAS-on-trunk. RT-2's worked example — result #1 lands, result #2 goes stale — is genuinely subtle and already modelled exactly. Re-deriving it is the disqualifying redo. Probe rows H10, H16. |
| **Reuse as pure logic** | The import belt's *predicate* — forbidden-path (`.doctrine/`, `.claude/`) and undeclared-scope — via `classify_import` / `conformance::undeclared_paths`, without the worktree preconditions. |
| **Do not rebuild** | Coordination-branch staging, `prepare-review` projection, journal-row-as-precondition, the fork-binding gate. |

## Why DEC-106 was wrong

It reached for `worktree import --fork` because it was an existing verb, and then
designed around that verb's constraints — inventing a derive-to-single-commit
stage to satisfy a `S^ == B` precondition.

But that precondition is an artifact of the verb's *shell*, not of conformance.
The existing code says so: the MCP arm calls the same pure belt with
`head_at_base` and `tree_clean` hardcoded true, because "the compose is
working-tree-free onto the coord tip, so neither a coord HEAD position nor a
dirty coord tree is a precondition" (`src/mcp_server/dispatch.rs:462`).
Conformance is `diff B..S --name-only`, which does not care how many commits lie
between.

**Consequences of dropping the verb:** the single-commit constraint dissolves, so
the derivation stage disappears and DEC-106's F3 finding ("no belted
multi-commit admission path exists") stops constraining the target model — it
remains true of the *current* verbs and is reported as such. The worker commits
freely and nothing squashes it.

That the simplification dissolved a finding rather than working around it is
the signal the direction is right.

## The provenance gate is a finding, not scaffolding

Stage 4 today requires a Verified journal row (REQ-316). The rig does **not**
stand up journal machinery to satisfy it. The capsule model does not lack
provenance — it carries a different and arguably stronger proof: **pinned OID +
verify-capsule attestation + ancestry from a contracted base**, where the
journal row proves only that a staging ritual completed.

Re-grounding REQ-316 on that proof is post-spike REV work. Building scaffolding
to feed the old gate would have guaranteed we never discovered the gate was the
wrong shape.

## Related

- [[landed-state-append-only]] — the invariant the simplification buys.
- RFC-025 `red-team.md` RT-2; `probe-specs.md` DQ-1, rows H10/H16.
- QUE-200 — the ingestion-mechanism question this feeds.
