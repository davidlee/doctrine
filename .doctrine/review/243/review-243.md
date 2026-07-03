# Review RV-243 — reconciliation of SL-195

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance-facet audit of SL-195 (installer dual-mode: `--dev` marketplace
source + `.mcp.json` MCP-command portability), 3 phases delivered:

- **P01** `boot.rs` — `.mcp.json` command `${DOCTRINE_BIN:-doctrine}` (POL-002),
  ownership predicate + idempotency comparator migration (RV-241 F-1 blocker).
- **P02** `install.rs`/`cli.rs` — `--dev` flag, marketplace source selection,
  qualified `doctrine@doctrine`, exact presence match, manifest selection rule.
- **P03** `install.rs` — stale-source refresh (R4 probed live).

**Lines of attack:**
1. Path-conformance algebra — did the code touch exactly the design-target
   selectors? (undeclared = scope creep; undelivered = dropped work.)
2. Every `EX-`/`VT-` satisfied with real evidence, not asserted — cross the plan
   criteria against `doctrine check gate` (must be green) and the source.
3. POL-002 invariant INV-1: no absolute host path in any git-tracked file.
   STD-001: the env-form command and marketplace name are single-source consts.
4. Idempotency (SPEC-009, RV-241 F-1): reinstall converges, no bogus rewrite,
   no double-register — the two coupled seams (ownership predicate + no-op
   comparator) both migrated to the env literal.
5. The R4/F-5 deferral: did the live probe's answer land in code *and* canon,
   and is refresh-failure fatal (not swallowed into `skipped_*`)?
6. Verification completeness — are the human-mode `VH-` legs actually run?

**Invariants held:** INV-1 (no tracked abspath), INV-2/INV-3 (refresh-not-skip),
POL-002 (baked ⟺ gitignored), STD-001 (named consts), SPEC-009 (idempotent MCP
plan). Governance edge: `references(concerns) SPEC-009`, `governed_by POL-002`.

## Synthesis

**Closure story.** SL-195 lands clean. `doctrine check gate` is green (clippy
`-D pedantic` zero-warning, full test, fmt, build); `slice conformance 195`
reports **0 undelivered, 3 conformant** (`boot.rs`, `cli.rs`, `install.rs`) —
the code touched exactly its three `design-target` files and nothing else. Every
test-mode criterion is satisfied with running evidence, not assertion: P01
VT-1..4 (boot), P02 VT-1..6 (install), P03 VT-1..2 (install) all execute under
gate. The two RV-241-hardened seams verify: `MCP_COMMAND` (boot.rs:549) is the
sole env-literal source (grep finds no second occurrence — STD-001 F-6), and the
idempotency comparator compares against that const rather than `exec.display()`
(SPEC-009 F-1 blocker closed, `plan_mcp_idempotent_when_current` green on the env
form). POL-002 INV-1 holds — the only tracked abspath breach (committed
`.mcp.json`) is dissolved to the env expansion; gitignored baked surfaces (pi
`mcp.ts`, hooks) stay baked per the *baked ⟺ gitignored* invariant (D2), untouched.

**Standing risks.** R-P3-1 (parser breaks on a future CC `Source:` line-format
drift) is bounded fail-safe: an unrecognised line → `None` → treated as Absent →
`Add` (idempotent no-op), never a mis-refresh. Version-stamped to CC 2.1.198 via
`mem.fact.claude.marketplace-add-overwrites-source`; a CC upgrade is the trigger
to re-probe. R-P2-1 is **closed** — the restore leg proved `marketplace list`
echoes the canonical abspath byte-equal to `fs::canonicalize`, so the `as_arg()`
comparator equality is sound.

**Tradeoffs consciously accepted.** (1) F-1 — the three `VH-1` live-acceptance
legs + OQ-4 are deferred (`tolerated`): no code depends on them, every mechanical
sub-leg is confirmed, and the residual is interactive end-to-end confirmation
needing a live `claude` + a repo move, unsafe in the primary edge tree. Captured
as a backlog chore so it is not lost. (2) F-2 — the R4 probe collapsed the
refresh verb to a single `add` (add overwrites in place on CC 2.1.198); the
destructive `remove`+`add` branch was deliberately **not written** (YAGNI + the
repo's `-D dead_code` gate). Plan EX-4's "handles BOTH branches" is thus
vacuously satisfied — an immutable-criterion artifact of a deferral the plan
itself mandated ("via the probed verb"). VT-2's substantive requirement (a failed
Refresh aborts via `return Err`, never swallowed into `skipped_*`) is honoured on
the live add/Refresh path. No blocker; the close-gate is clear.

**Conformance note.** The 14 `undeclared` paths `slice conformance` reports are
all `.doctrine/**` non-code: other agents' in-flight `backlog/211` + `slice/196`
(not this slice's), plus this slice's own audit-harvest artifacts (memory items,
`slice/195` notes/toml). None are code deliverables under a `design-target`
selector — expected registry noise, not scope creep. No finding.

## Reconciliation Brief

### Per-slice (direct edit)
- **design.md §8 R4** (F-2): drop "determine the refresh verb ... Probe live
  before choosing" — the verb is settled. Record: on CC 2.1.198 `claude plugin
  marketplace add <newsrc>` **overwrites** an existing name's source in place
  (exit 0), so refresh = a single `add`; `marketplace update` only re-pulls
  content at the same path (not a relocation). Cite
  `mem.fact.claude.marketplace-add-overwrites-source`.
- **design.md §8 R7** (F-2): the destructive `remove`+`add` branch does not exist
  on this CC version — there is no destructive window to abort. Rewrite the
  mitigation to: refresh is a single non-destructive `add`; the fatal-on-failure
  guard (`refresh_failure_is_fatal` ⇒ `return Err` on a failed Refresh) is the
  surviving F-5 protection, honoured on the add path. Note plan EX-4's
  both-branches clause is vacuously satisfied (immutable criterion — not edited).

### Governance/spec (REV)
- None. No ADR, policy, standard, or spec (SPEC-009, POL-002, STD-001) diverged —
  all held. No REV required.

## Reconciliation Outcome

### Direct edits applied
- **design.md §8 R4** (RV-243 F-2): replaced the "Impl-time empirical / probe
  live before choosing" open question with the settled refresh verb — a single
  non-destructive `marketplace add` (overwrites in place, CC 2.1.198), citing
  `mem.fact.claude.marketplace-add-overwrites-source`.
- **design.md §8 R7** (RV-243 F-2): rewrote the deferred-verb × swallowed-failure
  mitigation — the destructive `remove`+`add` branch does not exist on this CC
  version; the surviving F-5 protection is `refresh_failure_is_fatal` (failed
  Refresh ⇒ `Err`, never swallowed into `skipped_*`). Noted plan EX-4's
  destructive-abort clause is vacuously satisfied (immutable criterion — not edited).

### REVs completed
- None. Reconciliation Brief carried no governance/spec items.

### Withdrawn / tolerated
- RV-243 F-1: `tolerated` — 3× VH-1 + OQ-4 live-acceptance legs deferred (no code
  dep, mechanical sub-legs confirmed). Harvested to CHR-037. Rationale in finding
  disposition + Synthesis tradeoff (1).

Reconcile pass complete — handoff to /close.
