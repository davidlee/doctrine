# Review RV-401 — reconciliation of SL-270

Adversarial-review ledger. Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Conformance audit of SL-270 (PHASE-01 six-route set, PHASE-02 lock route
check), reviewed on `edge` at d64596cfc (not dispatched; no candidate branch).

Lines of attack:

- Code matches design sec-2 (route set) and sec-3 (predicate, refusal, remedy).
- `unrouted_severe` fails closed on unknown severity/status and never
  double-lists a finding already in `undisposed_blockers` (RV-400 F-7, F-11).
- ADR-001 layering: `design_run` (leaf) receives opaque labels only.
- VA-1 sweep re-run; residual route-set statements in governance go via REV-065.
- `slice conformance` undeclared cells explained.
- Carried items: RV-400 F-1 (gate coverage, owner decision), F-5, F-7;
  DEC-334 build floor reachable at close.

Evidence: `doctrine check gate` green; VT-1..VT-7 present and passing;
VA-1 sweep re-run clean (hits in reviewing.md, gate.rs, memory, vocab.rs and
tests are legacy/retired mentions; ADR-007 and SPEC-032 via REV-065).

## Synthesis

SL-270 shipped what its design locked. PHASE-01 replaced `owner-fix` with
`dedupe` and `refresh` in `Route`, `ROUTES`, the CLI and MCP descriptions, the
goldens, `reviewing.md`, `review-ledger.md` and the teaching memory. The MCP
schemas followed through `ROUTES` with no edit (VT-6). Stored legacy routes
still read (VT-7). PHASE-02 added `unrouted_severe` beside
`undisposed_blockers`: pure, fail-closed on unknown severity or status (VT-3),
and it never double-lists (VT-2, RV-400 F-7 control observed and verified
here). It reaches the leaf `design_run` as opaque labels (ADR-001 held), and the
lock refuses and then clears end to end (VT-4). `doctrine check gate` is green.

Divergences are prose and governance lag, not code: sec-3's remedy (F-1), the
REV-065 governance set (F-2), and sec-4's install obligation (F-4). F-5 records
why the conformance undeclared cell is non-empty.

Standing risks:

- **Coverage gap** (F-3): the check reads the current pass once, at lock.
  Majors left open past lock, superseded passes and post-lock dispositions
  escape it. Owner chose follow-up: IMP-498, sequenced after CHR-082.
- **Stale served binary** (F-4): until the PATH doctrine is rebuilt, agents see
  no `--route` at all, and CHR-082's build floor holds every eligible slice.
- **Provisional axis** (DEC-268): if CHR-082 kills RFC-026 P10, the check and
  the route vocabulary go with it.
- RV-400 F-5 stays `answered`. Its control obligation is hosted by CHR-082's
  window-open step, not by an SL-270 phase.

## Reconciliation Brief

### Per-slice (direct edit)
- F-1 — design.md sec-3, "The refusal": the answered-status remedy becomes
  `review amend <RV> --finding F-n --route <route> --response … --note …`.
- F-4 — design.md sec-4, last paragraph: "install the result" becomes a rebuilt
  PATH doctrine (or a pinned `DOCTRINE_BIN`) plus `doctrine install`, so that CLI
  help and the MCP `review_dispose` schema both serve six routes.

### Governance/spec (REV)
- F-2 — apply REV-065: ADR-007 D-C5 and SPEC-032 list the six routes, and
  SPEC-032 names the design-run lock as the one reader that an unknown route
  gates.

## Reconciliation Outcome

### Direct edits applied
- design.md sec-3: the answered-status remedy now names `--response …` (F-1).
- design.md sec-4: the obligation to CHR-082 names a rebuilt PATH doctrine (or
  a pinned `DOCTRINE_BIN`) plus `doctrine install` (F-4). Both are out-of-band
  edits to a locked run, by design.

### REVs completed
- REV-065 (`six-routes-and-the-lock-route-check`): done. ADR-007 D-C5 and
  SPEC-032 list the six routes, and SPEC-032 names the design-run lock as the
  one reader that gates on route (F-2). Narrative in revision-065.md.

### Follow-up / tolerated
- F-3: follow-up, IMP-498 (after CHR-082).
- F-5: tolerated; conformance undeclared cell explained in the disposition.

Reconcile pass complete; hand off to /close.
