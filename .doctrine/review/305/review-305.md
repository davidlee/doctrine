# Review RV-305 — design of SL-228

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Round 3** of the external adversarial review (raiser label `inquisitor`; the
raising arm is **codex / GPT-5.5 via MCP**, fresh thread — no round-1/2 context)
of the SL-228 design — `.doctrine/slice/228/design.md`, post-RV-304 state
(`527e55bd`). **Premise: the ten RV-304 repairs are themselves new, unreviewed
surface.** This trial interrogates the *round-2 repair mechanisms only* — it is
scoped, not a full re-run.

**Scope directive (user-set).** Governance-conformance axes are OUT of this
round: no ADR-001/006/011/012, POL-002, STD-001/002 sweep. Mechanism
correctness only — crash-safety, concurrency, totality, liveness, replay
identity — held to:

- **Requirements**: SPEC-021 REQ-384/385/386/387 (FR-008..011), SPEC-022
  REQ-388/389 (FR-010/011). Per FR: does the twice-repaired mechanism still
  satisfy the statement?
- **Ground truth**: `.doctrine/slice/228/extraction.md` (as-built graph at
  `0603f11f`); claims must verify against the actual code
  (`src/worktree/*`, `src/dispatch.rs`, `src/git.rs`, `src/mcp_server/*`).

Lines of interrogation (the tender parts, priority order):

1. **Claim → bind → act fork protocol + claim lock** (§3, D8 area; the
   RV-304 F-2 remedy, forged through three contests). Crash between claim
   and bind: the residue is a claimed branch ref with no binding — who
   sweeps it, and does gc's busy-lock=active-claimant rule leak it forever
   or reap it under a stale lock? Lock-file residue semantics after
   crash (kernel releases the flock, file remains) — can a dead claim's
   lockfile be distinguished from a live one? Two concurrent spawns of the
   same name; gc sweep vs spawn on *different* names sharing lock scope;
   does the lock also cover gc's ref-deletion arm?
2. **Replay identity restricted to stable input facts** (§2 idempotent
   replay; F-1 remedy, `onto` demoted to provenance). The inverse defect:
   with `onto` out of the identity, can a *genuinely different* import be
   mistaken for a replay (false-positive replay → silently skipped work)?
   Is the stable-fact set per transition actually stable AND sufficient?
3. **`preflight` kind-gate / `attempt` split** (§2/§4; F-3 remedy). Totality
   of the one const table across verb × position; can preflight admit what
   attempt then refuses in a way that strands the funnel mid-verb; is
   `TransitionKind` genuinely constructible pre-act in every arm?
4. **Oracle actionability ladder** (§6; F-4 remedy). Determinism of the
   single prescribed action under ties; starvation in cells the ladder
   doesn't rank; interplay with the RV-303 F-1 reverify loop under
   parallel mid-funnel phases — still livelock-free?
5. **Durable position⊕boundary outrank the sheet post-conclude** (§9; F-8
   remedy). Is the ordered matrix still total and first-match-wins?
   Pre-conclude sheet read errors — what wins then?
6. **Verify untracked-aware proofs** (§5; F-9 remedy). Untracked file
   appearing mid-suite; suite-mutated-bytes refusal interplay; is the
   ignored-path evidence boundary honestly load-bearing anywhere?
7. **Deletion exemption scoped to the validated path-set; reversion arm
   never sleeps** (§7; F-5 remedy). Can a hostile/accidental mass deletion
   inside the validated set still ride the exemption? Escape-env scoping
   still sound after the rescope?
8. **Binding resolver's caller-declared expected fork state**
   (`AtBase`/`Advanced`; F-7 remedy). A caller declaring the wrong
   expectation turns a guard into a heal — what constrains the caller?
9. **§11 verification alignment** — do the VTs actually pin the new
   protocol (claim lock, replay identity, ladder order), or only the
   pre-repair mechanisms?

Out of scope (do-not-re-raise): governance-conformance findings (user-scoped
out); locked D1–D8 absent evidence of defect; RV-300's 7 findings; the 6
internal-pass findings; RV-303's 12 and **RV-304's 10 findings as raised**
(probe whether the *fixes* hold — a broken fix is fresh heresy, a settled fix
is not); SL-225 DEC-003 `worker_commit` validate-skip legs; the subprocess-arm
deferral posture (user-settled scope, REQ-387 may stay `pending`).

**Convergence rule (declared before the round).** If this round yields ≤2
blockers and no repair mints a new mechanism, the design is declared
converged, pending the User's approval. Only a repair introducing genuinely
new machinery earns a further pass, and then only on that machinery.
