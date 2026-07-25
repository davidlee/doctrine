# SL-228 — notes

## Harvest

fresh-as-of: plan authored, slice `plan` status (RV-305 round-3 closed, design converged, User-approved) · head `b627cb02`

### Produced
- `extraction.md` (committed `0603f11f`) — as-built funnel state machine, crux
  verdicts, per-verb invariant table. Design input; stays useful post-close as
  the D7 artifact's ancestor.
- `design.md` — D1–D8 locked with User; internal pass (6 findings) + **external
  RV-303 codex inquisition integrated**: 12 findings (8 blockers), 2 rounds,
  5 second-round contests all upheld, 12/12 verified terminal. Commits
  `ccc8fbae` (round 1), `16d18e8b` (round 2), `1b716673` (synthesis).
- **RV-303** — closed ledger, facet design, raiser inquisitor (codex/GPT-5.5);
  synthesis carries the closure story + standing risks.
- **RV-304** — closed round-2 ledger (fresh codex thread over the post-RV-303
  design; premise: the 12 repairs are unreviewed surface). 10 findings
  (7 blockers), 10/10 confirmed against code, all fix-now, 10/10 verified
  terminal; F-2 took three upheld contests (clobbering write → TOCTOU →
  spawn–gc race), landing the claim→bind→act fork protocol under a per-name
  crash-released claim lock. Commits `be7f3abb`, `0ce0c62e`, `dbcebc22`,
  `527e55bd`.
- **RV-305** — closed round-3 ledger (scoped codex pass over the RV-304 repair
  surface only; governance axes user-excluded; convergence rule declared in
  brief). 2 findings: F-1 blocker (worker_commit crash replay unreachable past
  the AtBase belt) took two upheld contests — belt-gated provenance-indifferent
  adoption, then the one-commit heal rule; F-2 minor (stale-token gloss vs
  modulo-record gate). 2/2 verified terminal; **design declared CONVERGED**
  (12 → 10 → 2 decay). Commits `e3bce6f3`, `5a755f0d`, `3e100b3e`, `09aecf7a`.
- **Design approved by User** (post-RV-305); `/plan` run.
- `plan.toml` + `plan.md` (commit `b627cb02`) — 7 phases refining design §12:
  move E (01 reads, 02 guard/ISS-234), move A split (03 machine+record+
  run_suite, 04 fork-binding protocol, 05 gates+verify+receipt), 06 oracle+
  skills, 07 OQ-5 benchmark. All VT rows carry structured mandates
  (verify-vt: zero UNCHECKABLE). Phase sheets materialised; status → `plan`.
- `design-target` selectors recorded (15).
- Scope `## Follow-Ups` reconciled to design (OQ-2/NEW-OQ-A/NEW-OQ-B → D1/D6/D7).

### Learned (durable sinks already hold these)
- The reverse-diff resync trap (`reset --keep` broken when ref advanced under
  checkout) is memory-pinned; design §5/R3 rides the `restore`-based idiom —
  now per-path proof-gated (RV-303 F-6).
- Two-altitude finding (sub-funnel has no `select_guidance` node) — extraction
  §3; drove D4's "new oracle, not carve-out" framing.
- A record that rides a commit's own tree cannot store that commit's oid —
  Class-1 rows carry input facts only; output identity is post-hoc provenance
  (RV-303 F-2, design §3).
- Sheet-first conclude ordering was the real ConcludeIncomplete window; the
  fix is CAS-first with the sheet as trailing projection (RV-303 F-5, §4/§9).
- A "record the act durably" repair has three successively deeper failure
  modes — write ordering, overwrite clobber, then check-then-act races
  (incl. against the sweeper) — the stable shape is claim (atomic primitive
  you already own, here the branch ref) → bind → act, with the
  cleanup path taking the same claim (RV-304 F-2's three contests, design §3).
- Replay identity must be built from a retry's *recoverable* facts only —
  any freshly-sampled input (a live tip) in the identity key turns the
  idempotence promise into a refusal (RV-304 F-1, design §2).
- A replay promise is void if the verb's own belts refuse before the replay
  comparison runs — reachability is part of the contract (RV-305 F-1);
  adoption of unprovenanced work is sound only when gated by the same content
  belts the first-path enforces (belts are the authority, not authorship);
  and a heal that lands prefix rows in a separate commit from its own
  transition forges the induction that keeps ungated rows out — heal is
  all-or-none, one CAS commit (design §3/D8).
- Three-round review decay (12 → 10 → 2 findings) with a pre-declared
  convergence rule + scoped round-3 brief worked well as a stopping protocol.
- CLI friction for RFC-011: `explore` is a boot-spine group, not a CLI
  subcommand (use top-level `doctrine inspect`); requirement statement text is
  unreachable via CLI (`spec req list` shows `prose —`) — lives in
  `.doctrine/requirement/NNN/requirement-NNN.toml`; `review status` takes only
  RV refs (no `--slice` filter) (case-notes appended).

### Open
- **Plan approval gate**: plan authored + committed; User signalled dispatch —
  flip `ready` at handover, then `/phase-plan` PHASE-01 under `/dispatch`.
- Plan-phase items deliberately deferred (unchanged): OQ-5 benchmark harness
  shape; hook script asset path (selector added when picked);
  `NextCore.command` binary-name rendering; REQ-287 prose mapping (ship-time
  REV per Non-Goals); stale-baseline derivation exactness (§5 — algorithm
  sketch, R3-load-bearing; PHASE-05 must pin it before landing verify).
- VT keyword floors deliberately minimal where naming is free (PHASE-04 VT-1
  `claim`, VT-4 `lock`) — `/phase-plan` appends stronger mandates once file
  shapes exist.
