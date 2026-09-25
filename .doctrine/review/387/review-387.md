# Review RV-387 — reconciliation of SL-265

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance audit of SL-265 (the kind-blind `doctrine show` router) against its
locked design (sec-1..sec-8; DEC-295..DEC-299) and plan PHASE-01..PHASE-04.
Dispatched slice: `dispatch/265` at `f952dc9a1`, prepared `review/265` at
`d96f4d8fb`, coordination worktree removed. **Reviewed surface:** the admitted
interaction candidate `cand-265-review-001` (`34aeec3ac`, the no-ff merge of
`review/265` onto trunk `refs/heads/main` `2464bbfd6`) — its worktree carries the
slice's code, so all binary/test evidence below was gathered there.

Lines of attack:

- **The contract** (sec-2): resolve through `kinds::parse_resolvable_ref` →
  `kinds::canonical_id` → the kind's own `run_show`; the router renders nothing;
  the unrouted-prefix refusal lives at the dispatch site, not in `route()`.
- **Fidelity** (sec-7, VT): for every prefixed ref a kind's own `show` accepts,
  `doctrine show <REF>` is byte-identical to the per-kind invocation in both
  formats — one fixture per `KINDS` row, both sides the same binary twice.
- **Refusals** (sec-2 case rule, DEC-297): unknown prefix / dangling ref /
  ambiguous bare id each fail with the resolver's own error text.
- **Totality** (sec-3): `every_kinds_row_routes` iterates `KINDS` (not
  `ALL_KINDS`) with a negative control; `Route` dispatch is wildcard-free.
- **Surface** (sec-4): `Command::Show`, the `explore` family, the `guard.rs`
  `Read` arm, census 56→57, and `CommonShowArgs::format()` as the single home of
  the `--json` shorthand (four inline sites collapsed, no behaviour change).
- **Guidance** (sec-1/sec-7): `install/using-doctrine.md` verb table +
  read-entity paragraph, `install/routing-process.md` Guardrails, the regenerated
  boot snapshot, and the guardrails execution test with its anti-vacuity floor.
- **Governance** (sec-5): `REV-062` introduce `FR-006` on `SPEC-013`, `REQ-482`,
  a check-bound `Verified` coverage cell, and the hand-landed spec prose +
  `responsibilities` entry.
- **Mechanical conformance** leads; each cell dispositioned. `doctrine check
  gate` and the regression half weighed against the base `B = 2464bbfd6`.

## Synthesis

The slice delivers what the design locked. `src/commands/show.rs` resolves a ref
through `kinds::parse_resolvable_ref`, canonicalises it with
`kinds::canonical_id`, then delegates — `route()` names every `KINDS` row through
the `kinds::` prefix constants and returns `Option<Route>`, and the dispatch is a
wildcard-free `match Route` whose `None` arm is the unrouted-prefix refusal at the
dispatch site. Nothing about a kind's rendering changed.

The central property holds black-box. On the admitted candidate surface
(`34aeec3ac`) `tests/e2e_show_equivalence.rs` drives both sides of the comparison
through the same built binary twice and finds all 24 numbered prefixes
byte-identical in `--format table` and `--format json`; no router repair was
needed. `tests/e2e_show_refusals.rs` confirms the unknown-prefix, dangling-ref and
ambiguous-bare-id refusals each carry the resolver's own error text (spot-checked
live: `bare id \`265\` is ambiguous — matches SL-265, REQ-265, …`; `unknown kind
prefix \`ZZ\``). The surface is as designed: `show` sits in `explore` in both
`--help` and `--help --boot-map`, the census assertion reads 57 (updated, not
relaxed), `guard.rs` classifies it `Read`, and `CommonShowArgs::format()` is the
single home of the `--json` shorthand with all four inline sites collapsed
without behaviour change.

Guidance and governance carry. The verb table and read-entity paragraph in
`install/using-doctrine.md` and the Guardrails sentence in
`install/routing-process.md` name the router, and the new
`every_command_named_by_guardrails_is_accepted_by_the_binary` test parses the
Guardrails paragraph and executes each command it names, behind an anti-vacuity
floor. PHASE-03's runtime-state VA was re-derived on the review surface rather
than trusted: `doctrine boot` writes, `doctrine boot --check` reads clean, and
the snapshot's guardrail carries `doctrine show <REF>`. `REV-062` is
`done`/`approved` with its single introduce row landed; `REQ-482` is SPEC-013's
`FR-006`; the coverage cell is check-bound to `e2e_show_equivalence` and reads
`verified`; and `spec-013.md` § *Uniform command grammar* + § *Responsibilities*
plus `spec-013.toml`'s structured `responsibilities` all name the top-level
router. `slice verify-vt 265` is 14/14 PASS on the review surface.

Findings are four. Two are handed to reconcile: the selector registry for the new
requirement (`F-1`, minor) and a stale conformance prediction in design sec-5
(`F-3`, nit). The two slice-own authored files are expected undeclared (`F-2`,
aligned). The gate's lone red is a pre-existing environment defect, unrelated to
the slice and now owned (`F-4`, major → `ISS-483`).

Standing risks and accepted tradeoffs:

- The router ASCII-uppercases the prefix unconditionally before resolution. That
  is deliberately a **superset** of every per-kind parser (`spec`'s is
  case-strict), so `doctrine show` accepts a few refs a kind's own `show`
  refuses. The byte-equivalence property is therefore stated over *prefixed* refs
  a kind accepts, not over a case rule (`DEC-296`, `RV-384` `F-25`); a bare id is
  outside it by design, since it resolves across kinds and refuses as ambiguous
  (`DEC-297`).
- The primary worktree's `.doctrine/state/boot.md` is stale until this code lands
  and the primary binary is rebuilt; regenerating it earlier would advertise
  `doctrine show` for a binary that refuses it. It is gitignored runtime state, so
  nothing to carry in the diff — `/close` (or the next session) regenerates it
  after landing, which is the documented integration order.
- `doctrine check gate` red on the reserve suite (F-4). Accepted here only because
  it is provably pre-existing at the base and orthogonal to the slice; it is
  captured as `ISS-483` rather than normalised away.

## Reconciliation Brief

### Per-slice (direct edit)

- **`F-1`, selector registry** (load-bearing verb):
  `doctrine slice selector add 265 '.doctrine/requirement/**' --intent design-target`
  — the REQ-482 subtree is the only undeclared *deliverable* path. Mirror: design
  sec-6's code-impact table already names "a new `REQ`"; no prose edit beyond
  `F-3`.
- **`F-3`, `design.md` sec-5**: the closing paragraph ("…has no `src/` selector
  and reports as `undeclared` in slice conformance — expected, disposed
  `aligned`") misstates the reading. Replace it with the actual one: the REV +
  `.doctrine/spec/tech/013/**` deliverable is glob-covered by declared
  design-target selectors and reports **conformant**; the new requirement subtree
  is the undeclared path, covered by `F-1`'s selector; `.doctrine/slice/265/`
  files are slice-own and expected.

### Governance/spec (REV)

- **`REQ-482` status `pending → active`** (design sec-5, PHASE-04 `EX-6`). Author a
  `REV` carrying a `status`-action row on `REQ-482` and land it with
  `revision apply` (SL-256 `REV-055` precedent). Evidence already in hand: the
  coverage cell is check-bound (`doctrine coverage verify 265` → `verified`,
  `touched_paths` anchored) and the requirement's scope is witnessed by the
  admitted candidate's VT-1/VT-2 (14/14 PASS).

### Not reconcile's — owners already exist

- **`F-4`** → `ISS-483` (backlog issue; reserve-suite env isolation). Nothing to
  write here — and see the post-audit correction below: the ambient trigger has
  since been removed, so the gate reads green and only the hermeticity defect
  remains.
- **`F-2`** → aligned; no write owed.
- **`CHR-078`** already owns the out-of-scope guidance sweep (canon/walkthrough
  skills, `authority-model.md`, shipped memories).

### Close-time bookkeeping

- Regenerate the primary worktree's boot snapshot after the code lands and the
  primary binary is rebuilt (`doctrine boot`; `.doctrine/state/boot.md` is
  gitignored, so it is an act, not a diff).
- Observations already recorded this session: `01a0d851…` (coord-teardown phantom
  staged deletion), `01a0d84a…` (`coverage record` never populates
  `touched_paths`).

## Post-audit Correction (operator action, 2026-09-25)

`F-4` was raised on a live ambient condition — the jail exporting
`DOCTRINE_RESERVATION_FALLBACK=1`. The operator has since removed that export from
`flake.nix` (`67df42ec8`), and it was never load-bearing: the committed
`.doctrine/doctrine.toml` reserves locally (`[reservation] reach = "local"`,
`allow-local-fallback = true`), verified by running an id-reserving verb with the
variable unset. Effect on this ledger (the findings are append-only and stay as
raised — this is the record of what changed after):

- `F-4`'s observed failure no longer reproduces; a fresh-jail `doctrine check
  gate` is green. The disposition stands (`follow-up` → `ISS-483`), whose scope is
  now the suite's hermeticity alone, not a red gate.
- Nothing else in the brief moves: `F-1`/`F-3` (reconcile) and the `REQ-482`
  `pending → active` REV are unaffected.
- Close-time bookkeeping stands (regenerate the primary's boot snapshot after
  landing).

## Reconciliation Outcome

### Direct edits applied

- **Selector registry** (`RV-387` `F-1`): `doctrine slice selector add 265
  '.doctrine/requirement/**' --intent design-target`. The three `REQ-482` paths
  (slug symlink + toml + md) now report conformant.
- **`design.md` sec-5** (`RV-387` `F-3`): the closing paragraph states the real
  reading — the REV + `.doctrine/spec/tech/013/**` deliverable is glob-covered and
  **conformant**; the requirement subtree carries the new selector; the remaining
  undeclared paths are the slice's own bookkeeping. Direct edit, out of band on
  the locked run (`mem.pattern.reconcile.edit-design-out-of-band`).
- `slice conformance 265` after the writes: 0 undelivered, and undeclared down to
  `.doctrine/slice/265/coverage.toml` + `slice-265.toml` (`F-2`, aligned).

### REVs completed

- **`REV-063` (`reconcile-sl-265`) — done.** One `status` row targeting
  `REQ-482`, auto-landed by `revision apply` (`REC-117`): `pending → active`.
  Covers design sec-5 `EX-6`. Evidence in hand: the check-bound coverage cell
  re-derives `verified` (`doctrine coverage verify 265`) and the requirement's
  scope is witnessed by the admitted candidate's VT set. Rationale + reconcile
  narrative in `revision-063.md`.

### Dispatch landing (close step 3a)

- `main` was promoted from `edge` first (`git fetch . edge:main`) so the landing
  zone was current. Admitted `close_target` candidate `cand-265-close-001`
  (`330b247f0` — the no-ff merge of `review/265` `d96f4d8fb` onto trunk).
- The merge needed **one hand resolution**, in `.doctrine/slice/265/slice-265.toml`:
  the row `F-1` added sat adjacent to the two rows `review/265` also adds, so git
  could not auto-resolve a file whose "ours" is a superset. Resolved to ours and
  adopted via `dispatch candidate ingest` (validated as a faithful `(base, source)`
  3-way).
- `dispatch sync --slice 265 --integrate --trunk refs/heads/main` advanced `main`
  to `330b247f0`. Both post-integrate checks passed: no phantom reverse-diff (the
  working tree was byte-identical before and after; the dirty files present are
  another agent's, unchanged), and the journal trunk row equals the trunk ref
  (`330b247f0` = `330b247f0`).
- `edge` fast-forwarded to `330b247f0`, so the primary tree carries the audited
  code, the rebaked `install/` embed, and `REQ-482`.

### Close pre-check

- `doctrine slice verify-vt 265` from the primary tree: **14/14 PASS**.
- `doctrine check gate`: **green on the landed commit** `330b247f0`, run in a
  detached worktree at exactly that commit (with the gitignored RustEmbed root
  `web/map/dist` linked in). The env was normalised to match the operator's
  `flake.nix` change. Note: the **shared** primary tree's gate was red from a
  concurrent agent's uncommitted `src/design_run/*` work (E0061 at
  `run.rs:1568`), unrelated to SL-265 — recorded as observation `01a0d86a…`.
- Boot snapshot regenerated from the **rebuilt** primary binary (the stale binary
  had rendered one without `show`); `doctrine boot --check` clean, and the
  snapshot's guardrail names `doctrine show <REF>` with `show` in `[explore]`.

### Withdrawn / tolerated

- `RV-387` `F-2`: aligned (slice-own bookkeeping files) — no write owed.
- `RV-387` `F-4`: follow-up → `ISS-483`; its ambient trigger was removed by the
  operator (`flake.nix` `67df42ec8`). Not a blocker.

Reconcile pass complete; hand off to `/close`.
