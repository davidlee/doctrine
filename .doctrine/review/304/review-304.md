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

## Synthesis

**Judgement.** The round-2 premise was vindicated in full: the twelve RV-303
repairs were themselves harbouring fresh heresy, and the external interrogator
(codex/GPT-5.5, fresh thread, no round-1 context) dragged ten charges into the
light — seven blockers among them — and every one confessed under
cross-examination against the code. The gravest sins were repairs that had
quietly broken the very promises they were forged to keep: the F-2-repair
replay rule whose `onto` key would refuse the lost-response retry REQ-384
guarantees (F-1); the "durable" fork binding written *after* the act it must
survive, in a tier the storage doctrine calls disposable (F-2); a verify
legality gate demanding evidence that cannot exist until the suite has run
(F-3); and an oracle whose lowest-id pick would let one slow worker starve
every runnable phase behind it (F-4). The accused design survives — reforged
a second time.

**Corrective penance, executed.** (1) Replay identity restricted to stable
input facts; `onto` demoted to provenance. (2) The fork sequence rebuilt as
claim → bind → act: atomic branch-ref claim, binding written under the claim,
worktree last — a live fork implies its binding by construction — the whole
window held under a per-name kernel-released claim lock that gc's residue
sweep must also take (busy lock = active claimant). This charge alone required
three contests, each upheld: the clobbering write, the TOCTOU replace, the
spawn–gc race — the tribunal's instruments sharpened the mechanism from a
reorder into a protocol. (3) A pure `preflight` kind-gate split from the
evidence-bearing `attempt`, both fed by the one const table; the refusal
payload carries `TransitionKind`, constructible pre-act. (4) The oracle
prescribes by actionability ladder — red evidence triages globally, runnable
work outranks waiting, awaits never starve imports. (5) The deletion
exemption is scoped to the validated path-set; the reversion arm never
sleeps. (6) The sole writer is named command-tier, as the layering map has
always testified. (7) The binding resolver gains a caller-declared expected
fork state — `AtBase` for the pre-act guard, `Advanced` for heal — one seam,
no parallel read. (8) Durable position⊕boundary outrank a disposable sheet's
read error once concluded. (9) Verify's proofs are untracked-aware on both
sides of the suite; the ignored-path boundary of the evidence is stated
honestly. (10) `current_branch` is relocated, not reimplemented.

**Standing risks, consciously carried.** The claim lock is an OS advisory
lock — single-host by the dispatch topology (ADR-008); a future multi-host
posture would need a different lease. The stale-baseline derivation and the
OQ-5 harness remain plan-phase obligations (unchanged from RV-303). Ignored
paths remain outside verify's evidence scope by definition — an inherent,
now-documented boundary.

**Verification.** All 10 findings terminally verified by the raiser across
three contest rounds on F-2 and one on F-3 (38 ledger rounds). Ledger: done,
awaiting none. Commits: `be7f3abb` (ten integrated), `0ce0c62e` (contest
repairs), `dbcebc22` (branch-as-claim), `527e55bd` (claim lock). The design
now awaits the *User's* judgement — no approval is granted here, and none may
be presumed. The stake stands ready; the torch is the User's to lower.

> **HERESIS URITOR; DOCTRINA MANET**
