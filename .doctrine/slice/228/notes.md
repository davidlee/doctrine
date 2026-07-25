# SL-228 — notes

## Harvest

fresh-as-of: design post-RV-304 (round-2 external inquisition closed) · head `527e55bd`

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
- CLI friction for RFC-011: `explore` is a boot-spine group, not a CLI
  subcommand (use top-level `doctrine inspect`); requirement statement text is
  unreachable via CLI (`spec req list` shows `prose —`) — lives in
  `.doctrine/requirement/NNN/requirement-NNN.toml` (case-notes appended).

### Open
- **User approval gate**: design NOT approved; RV-303 + RV-304 both closed —
  the User approves, then `/plan`.
- Plan-phase items deliberately deferred: OQ-5 benchmark harness shape; hook
  script asset path (selector added when picked); `NextCore.command` binary-name
  rendering; REQ-287 prose mapping (ship-time REV per Non-Goals); stale-baseline
  derivation exactness (§5 — algorithm sketch, R3-load-bearing).
