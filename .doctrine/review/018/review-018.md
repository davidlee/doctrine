# Review RV-018 — reconciliation of SL-061

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-061 (rewire `/code-review` + `/inquisition` onto the RV
ledger via a shared `review-ledger.md`), driven through the dispatch funnel as
P01 (keystone) → P02+P03 (file-disjoint batch) → P04 (inline). Lines of attack:

- **INV-3 — `/audit` behaviour-preserved by the extraction.** The strongest test
  is the dogfood: this very audit runs on the refactored `/audit` + shipped
  `review-ledger.md`. If a mechanic were lost, the audit could not be driven.
- **Phase EX/VT criteria** (plan.toml): review-ledger.md owns the protocol; the
  three SKILL.md rewrites; `review` plugin retired + marketplace integrity;
  facet-by-target + `--raiser inquisitor` with no new facet / no `src/review.rs`
  diff; IMP-023 closed + follow-up backlog minted; re-embed.
- **Zero production `src`** (D5/VA-2) — only `src/skills.rs` + `src/install.rs`
  test/fixture tier may change; no production behaviour.
- **Green gate** — `just check`/`just gate` clean; clippy zero.
- **Bodies likely buried:** install-time collateral from the P04 re-embed
  (`doctrine claude install` side effects on `.gitignore` / `.doctrine/agents/`),
  and scope-wording drift between "zero production src except `src/skills.rs`" and
  what actually landed.

## Synthesis

**Closure story.** SL-061's thesis landed: the invariant RV-driving protocol now
lives once in `install/review-ledger.md` (auto-shipped to `.doctrine/`), and the
three review skills collapsed to persona + lens + a pointer to it —
`/audit` (refactored, INV-3), `/code-review` (relocated into doctrine core,
the standalone `review` plugin retired), `/inquisition` (rewired, facet-by-target
+ `--raiser inquisitor`, `inquisition.md` retired). The strongest INV-3 evidence
is that **this audit was itself driven on the refactored `/audit` + the shipped
`review-ledger.md`** — a lost mechanic would have made the loop undrivable. The
verb surface backs every consumer's prescribed flow (facets incl. `code-review`/
`reconciliation`, `--raiser`, raise/dispose/verify/contest/withdraw/prime). The
facet enum and `src/review.rs` were untouched (INV-2). Marketplace integrity is
clean (no dangling `review`-plugin reference; the residual `review` strings are
the RV *kind*). All four phases landed as clean, file-disjoint commits through the
dispatch funnel despite heavy concurrent `main` activity (SL-057 close, SL-060
dep/seq, SL-062 authoring) — every batch re-anchored onto the moved coordination
HEAD on a disjointness proof, foreign uncommitted work never commingled.

**Findings (3, all terminal).**
- **F-1 (blocker → fix-now, verified).** The P04 re-embed's `doctrine claude
  install` self-appended `.doctrine/agents/*` to `.gitignore`; I wrongly committed
  it, which (a) RED-ed the worktree classifier invariant test and (b) would have
  blanket-ignored the *authored* `AGENTS.md`. Reverted within the audit
  (`1037154`); derived `dispatch-worker.md` removed; gate green; SL-061's
  zero-production-src invariant preserved. The close-gate teeth did their job —
  this would have refused close had it stayed open.
- **F-2 (minor → follow-up).** Root cause is upstream: the SL-056 agent-install
  surface enforces a too-broad, unclassified ignore. Reassigned to **ISS-012**
  (narrow the ignore to derived outputs + classify in `DERIVED_RUNTIME`, together).
- **F-3 (nit → aligned).** "zero production src except `src/skills.rs`" wording
  undercounts the P01 `src/install.rs` belt test; both are test-tier, the
  substantive invariant holds. Cosmetic, not worth doc churn.

**Standing risks / tradeoffs.** None blocking. The harvest-DRY (IMP-059) and
`/handover` relocation (IMP-060) follow-ups are consciously deferred per design D6.
`--raiser inquisitor` posture is a recorded-but-unfilterable label (D2 traded
queryability away knowingly). Design + governance reconciled; ready for `/close`.
