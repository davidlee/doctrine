# Design SL-224: Honest dispatch refusals

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Descends from RFC-016 (zero-rescue dispatch), Cluster 1 / move C. The RFC-011
case-notes analysis (`.doctrine/rfc/011/case-notes-analysis.md`) ranks two
sibling funnel-burners:

- **#4 selector under-declaration** (8+ repros): a phase's delta touches a path
  no `design-target` selector declares, so `dispatch_import` refuses
  `undeclared-scope` — N phases into a dispatch, forcing a refresh-base / re-fork
  loop.
- **#7 empty `Refused.detail`** (4 repros): the MCP `dispatch_import` refusal
  names *no* offending path, forcing a source-dive of `import.rs` /
  `conformance.rs` on every refusal.

Both share one seam: the refusal knows *which* paths offended (the belt computed
them) but drops that knowledge, and the incompleteness that causes the refusal
is invisible until import time. The zero-rescue move: (1) the refusal carries its
own prescription, and (2) the incompleteness is caught **at plan time**, before
dispatch.

## 2. Current State

**The shared predicate already exists.** `conformance::undeclared_paths(selectors,
paths) -> Vec<String>` (`src/conformance.rs:131`) returns the delta paths no
selector declares. Both the CLI and MCP import arms already call it; the import
belt (`classify_import`, `src/worktree/import.rs:106`) gates on its
non-emptiness. Slice assumption **A** ("the undeclared paths are already computed
inside the belt") is **confirmed** — this slice surfaces what the predicate
already knows at two new moments, adding no classifier logic.

**Objective 1 — asymmetric today:**
- The **CLI** arm (`worktree import` → `classify_or_report` →
  `report_undeclared_scope`, `import.rs:146-196`) *already* names the offending
  paths: it prints each path + a per-path `selector add` remediation to stdout
  and bails with `import-refused: undeclared-scope (p1, p2)`.
- The **MCP** arm (`dispatch_import` → `import_compose`, `src/mcp_server/
  dispatch.rs:257-337`) — the one the case-notes hit (SL-210 / SL-213 / SL-222)
  — refuses with `funnel_refused(refusal.token(), String::new())` at
  `dispatch.rs:301`. `Refused.detail` is **empty**. (The MCP arm cannot print to
  stdout — it pollutes the JSON-RPC channel, per the existing warning at
  `dispatch.rs:366`; the diagnosis must ride the structured `detail` field.)

**Objective 2 — the lint surface already exists.** `doctrine check plan <id>`
(`src/commands/check.rs:65,166` → `plan::check_vt_shape`, `src/plan.rs:239`) is a
pure static lint of a slice's authored `plan.toml` VT rows. It flags three
*shape* problems — `BareTestFile` (no `test_file`), `BareKeywords` (`test_file`
but empty `keywords`), `MissingWaiverReason` (opaque waiver) — and exits non-zero
iff any *bare* VT exists (IMP-209). It reads only `plan.toml`; it runs no tests.
It does **not** cross-check a VT's `test_file` against the slice's selectors.

## 3. Forces & Constraints

- **DRY / single-source (STD-001, CLAUDE.md).** The undeclared set and its
  human-facing remediation must have ONE implementation shared by both import
  arms and the lint — no divergence, no re-derived format strings.
- **Behaviour-preservation gate (AGENTS.md).** `check_vt_shape` and
  `classify_import` are shared machinery; their existing suites must stay green
  **unchanged**. New behaviour rides new functions, not edits to the pure core.
- **Pure/imperative split (ADR-001, slices-spec § Architecture).** No clock / rng
  / git / disk in the pure layer. The new formatter and coverage predicate are
  pure; impurity (selector read, stdout, JSON-RPC) stays in the shell.
- **Layering (ADR-001): leaf ← engine ← command, no cycles.** The formatter sits
  in `conformance.rs` (leaf), alongside `undeclared_paths`, callable by both the
  `import.rs` engine and the `mcp_server` shell.
- **Report-and-halt (Non-Goals).** Refusals stay report-and-halt; we make the
  halt self-explanatory, never automatic. No auto-widening of scope.
- **Taxonomy frozen (Non-Goals).** This slice makes the *existing* `design-target`
  vs `scope-relevant` taxonomy legible and checkable; it does not redesign it.

## 4. Guiding Principles

- **A refusal carries its own prescription.** Name the offending paths *and* the
  one-line fix, so a fresh orchestrator resolves the refusal from the message
  alone (the slice's stated closure intent).
- **Catch incompleteness at plan time, not import time.** The plans that produced
  the 8+ repros would be flagged before dispatch, not N phases in.
- **Surface, don't reclassify.** The belt already knows the undeclared set; both
  changes read that same verdict at a new moment.

## 5. Proposed Design

### 5.1 System Model

Both objectives ride the existing `conformance::undeclared_paths` pure predicate
— the same one the import belt gates on. No new classifier logic; two new
surfacing points plus one shared formatter.

```
                 conformance::undeclared_paths   (existing, unchanged)
                        ▲                     ▲
        obj 1 ──────────┘                     └────────── obj 2
   MCP dispatch_import                     check plan <id>
   (Refused.detail)                        (pre-dispatch lint)
                        │
              conformance::undeclared_detail   (NEW pure formatter,
                 ▲            ▲                  shared by both import arms)
        CLI import          MCP import
     (stdout report)       (Refused.detail)
```

Belt (`undeclared_paths`) answers *which paths*. Formatter
(`undeclared_detail`) answers *how to say it*. Lint
(`undeclared_test_files`) answers *would a future import refuse this plan*.

### 5.2 Interfaces & Contracts

**Objective 1 — shared remediation formatter (leaf).** New pure fn in
`src/conformance.rs`, sited next to `undeclared_paths`:

```rust
/// Human-facing remediation block for an undeclared-scope refusal — the offending
/// paths each with the `selector add --intent design-target` fix. SINGLE SOURCE
/// for the CLI stdout report AND the MCP `Refused.detail` (no divergence between
/// arms). Pure: formats a pre-computed undeclared set, reads nothing. Takes the
/// slice id so the emitted `selector add` command is RUNNABLE (F1) — the id is a
/// required leading positional (`selector add <ID> [GLOBS]…`). Does NOT embed the
/// `undeclared-scope` token (F2): the MCP `reason` field carries it, and the leaf
/// cannot reference `Refusal::token()` (engine) without a layering cycle.
pub(crate) fn undeclared_detail(slice: u32, undeclared: &[String]) -> String
```

Output shape (a handful of paths — nowhere near the ~295k-char `worker_commit`
transcript-bloat trap, case-note #24):

```
2 path(s) no design-target selector declares:
  tests/e2e_foo.rs
    remediation: doctrine slice selector add SL-224 tests/e2e_foo.rs --intent design-target --note <why>
  src/view.rs
    remediation: doctrine slice selector add SL-224 src/view.rs --intent design-target --note <why>
```

The CLI arm prepends its standalone header (`import-refused: undeclared-scope — `);
the MCP arm uses the block raw as `detail` (the `reason` field already names the
token). Refactoring `report_undeclared_scope` through this formatter also FIXES a
latent bug in the current CLI hint (`import.rs:161` emits `selector add {path}` —
missing the required id — which would misparse the path as the id; F1).

Consumers:
- **CLI** — `import.rs:report_undeclared_scope` becomes: compute
  `undeclared_paths` → `undeclared_detail(slice, &undeclared)` → `writeln!(stdout,
  …)`, replacing the inline `writeln!` loop. Output is *deliberately changed*, not
  preserved (F7): each remediation line now carries the slice id (F1 fix), and the
  count header drops the "the worker delta touches …" token framing (F2). No
  golden pins this string today (the `run_import_from_worktree` tests only
  `.contains()` token + path), so the change is safe. `classify_or_report`'s
  one-line bail (`import.rs:187-191`) is unchanged.
- **MCP** — `import_compose` (`dispatch.rs:299-302`): the `Err(refusal)` arm
  splits. On `UndeclaredScope`, recompute the undeclared set from the SAME pure
  predicate and pass `undeclared_detail(…)` into `funnel_refused`; every other
  refusal keeps `String::new()`:

```rust
match classify_import(true, true, single_commit, &delta_paths, selectors) {
    Ok(Apply::Ok) => {}
    Err(Refusal::UndeclaredScope) => {
        let paths: Vec<&str> = delta_paths.iter().map(String::as_str).collect();
        let undeclared = crate::conformance::undeclared_paths(selectors, &paths);
        return Ok(funnel_refused(
            Refusal::UndeclaredScope.token(),           // reason carries the token (F2)
            crate::conformance::undeclared_detail(slice, &undeclared),
        ));
    }
    Err(refusal) => return Ok(funnel_refused(refusal.token(), String::new())),
}
```

`Refusal` stays a fieldless `Copy` enum (no payload added — recomputing the set on
the refusal arm is cheap and keeps the widely-used `.token()` / `Copy` contract
intact; D3). On the MCP arm `slice` is already the fn param — no plumbing.

**CLI slice-id plumbing (F4).** The CLI arm does NOT currently carry the slice id
where the remediation is built: `report_undeclared_scope(selectors, delta_paths)`
← `classify_or_report(…)` ← `run_import_fork` / `run_import_from_worktree` ←
`run_import`. To emit a runnable `selector add <ID> …`, thread the resolved
`slice: u32` down that chain (the scope belt is reached ONLY when `--slice` was
supplied, so the id is always present on this arm — pass the unwrapped `u32`, no
`None` hazard). `report_undeclared_scope` then calls `undeclared_detail(slice,
&undeclared)`. This is a ~4-signature change, not a 3-liner — PHASE-01 owns it.

**Objective 2 — selector-coverage lint (engine + command).** A *sibling* pure fn
to `check_vt_shape` in `src/plan.rs`, with its own finding type:

```rust
/// A VT whose mandated `test_file` no design-target selector covers — the plans
/// the import belt would later refuse `undeclared-scope`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VtSelectorFinding {
    pub phase_id: String,
    pub vt_id: String,
    pub path: String,
}

/// VT `test_file` paths not covered by any design-target selector. Reuses
/// `conformance::undeclared_paths` (the SAME predicate the import belt gates on),
/// so a flagged plan's DECLARED test_file is one the belt would refuse if the
/// phase's git delta touches it in that path form (F6: the lint reads authored
/// `test_file` strings; the belt reads the actual `B..fork` git delta — same
/// predicate, different inputs, so "exactly refuse" holds only under path-form
/// agreement + no other undeclared touched paths). Waived VTs and VTs with no
/// `test_file` are skipped (nothing to write / already `BareTestFile`). EMPTY
/// `selectors` ⇒ EMPTY result — mirrors the belt's no-op-when-empty scope leg
/// (A3, import VA-3); guards the `undeclared_paths(empty, …) == all` footgun.
pub(crate) fn undeclared_test_files(plan: &Plan, selectors: &[String]) -> Vec<VtSelectorFinding>
```

**`run_check_plan` restructure (F1 — must-fix).** The current body early-returns
`Ok(())` the moment `check_vt_shape` is empty (`check.rs:173`) — which is exactly
obj-2's primary case (a shape-CLEAN plan whose `test_file` is uncovered). Left as
is, coverage would never run and the lint would silently no-op. The fix hoists the
selector read + coverage check ABOVE the clean short-circuit, which now fires only
when BOTH sets are empty:

```rust
let plan  = crate::slice::read_plan(&slice_root, id)?;
let sel   = crate::slice::selectors(&root, id, Some(crate::slice::SelectorIntent::DesignTarget))?;
let shape = crate::plan::check_vt_shape(&plan);            // unchanged fn
let cover = crate::plan::undeclared_test_files(&plan, &sel);
if shape.is_empty() && cover.is_empty() {
    // "all VT rows carry structured mandates and design-target-covered test_files"
    return Ok(());
}
// render shape table (existing) + coverage table (new) …
let hard = bare_count > 0 || !cover.is_empty();           // MissingWaiverReason stays soft
if hard { std::process::exit(1) }
```

Render — per-VT table line, then the remediation footer routed through the SAME
`undeclared_detail` formatter (F2/F5 — the runnable, id-bearing hint has ONE
source; no re-inlined `selector add` string):
```
  PHASE-03    VT-2   UndeclaredTestFile  — test_file `tests/e2e_foo.rs` not a design-target selector

2 path(s) no design-target selector declares:
  tests/e2e_foo.rs
    remediation: doctrine slice selector add SL-224 tests/e2e_foo.rs --intent design-target --note <why>
```
(the footer is `undeclared_detail(id, &distinct_undeclared_test_file_paths)`).

### 5.3 Data, State & Ownership

Read-only over authored state. Obj 1 reads the fork delta (already gathered) +
the slice's design-target selectors (already read at `dispatch.rs:246`). Obj 2
reads the authored `plan.toml` + design-target selectors via the existing
`slice::selectors` reader. No writes; no runtime-state mutation; no new files.

### 5.4 Lifecycle, Operations & Dynamics

- **Obj 1** fires at import time on the MCP funnel arm — the refusal now carries
  the remediation, so the orchestrator acts on the message rather than
  source-diving.
- **Obj 2** fires at `doctrine check plan <id>` — the pre-dispatch gate an
  orchestrator (or `just`) runs before spinning up workers. A plan whose VT
  `test_file` isn't design-target-covered is refused here, before the phase that
  would trip the import belt is ever dispatched.

### 5.5 Invariants, Assumptions & Edge Cases

- **A3 (empty-selector guard).** `undeclared_test_files` returns empty when
  `selectors` is empty — guard *before* calling `undeclared_paths`, which would
  otherwise flag every path (`conformance.rs` test at :499). Mirrors the import
  belt's VA-3 no-op-when-empty.
- **Waived VTs** — skipped (not going to be written; `test_file` needn't be a
  selector). Same posture as `check_vt_shape`.
- **VT with no `test_file`** — skipped by the coverage check; already caught as
  `BareTestFile` by the shape check.
- **Glob selectors** — `undeclared_paths` already honours globs (`conformance.rs`
  test at :509), so a `tests/**` design-target selector legitimately covers
  `tests/e2e_foo.rs` with no false flag.
- **Path form (F6).** The lint compares the AUTHORED `test_file` string against
  the AUTHORED selector strings — both authored by the same hand in the same
  slice, so they share a form by construction (no git-vs-authored skew the belt
  faces). A non-canonical `test_file` (leading `./`, absolute) would miss a
  matching selector — but that same string is what the runtime VT gate greps, so
  a mis-formed `test_file` is already a pre-existing authoring error the lint
  surfaces rather than introduces. No normalization layer added (keeps the
  predicate honest — same matcher the belt uses).
- **VT `test_file` under `.doctrine/`/`.claude/` (F8, NIT).** Such a path would be
  flagged undeclared and prescribed a selector that can NEVER clear (the belt's
  tier reject precedes the scope leg, `import.rs:122-129`). Left un-guarded: a VT
  whose `test_file` is a governance-tier path is itself malformed (a phase does
  not test into `.doctrine/`), so the futile prescription is a symptom, not a case
  worth special-casing (YAGNI).
- **Coupled non-VT paths** (goldens in other binaries, caller migrations) remain
  the orchestrator's judgement at import (`mem.pattern.dispatch.import-scope-belt-
  omit-slice-for-coupled-paths`) — obj 2 scopes to a VT's OWN mandated
  `test_file`, the narrow case that genuinely SHOULD be a design-target selector,
  so it does not over-reach into that coupled-path territory.
- **A2 (VT `test_file` is the only structured plan touch-target).** EN/EX criteria
  are prose; `test_file` is the sole machine-readable path a plan declares. Obj
  2's "other plan-declared touch targets" has no other referent in today's model.

## 6. Open Questions & Unknowns

None open. Both slice OQs are resolved:
- *"Where does the plan-time lint live?"* → `check plan` (D1).
- *"Lint severity — hard or advisory?"* → hard, non-zero exit (D1).

## 7. Decisions, Rationale & Alternatives

- **D1 — obj 2 lint lives in `check plan`, hard (non-zero exit).** Reuses the
  existing pre-dispatch lint surface and its non-zero-exit contract; a VT
  `test_file` off the design-target set is the same category of broken-for-
  dispatch as a bare VT. *Alt (advisory-only)* rejected: an advisory ignored 8
  times is the status quo. *Alt (`slice phases` at materialize time)* rejected:
  selectors often aren't declared yet at phases time, so A3's empty-set case
  would false-flag every VT.
- **D2 — obj 1 detail = paths + per-path remediation, via a shared
  `undeclared_detail` formatter.** Serves the zero-rescue closure intent (refusal
  = prescription) and costs nothing extra — the CLI already builds this text; we
  factor it so both import arms AND obj-2's lint footer share one source
  (post-inquisition: the lint reuses `undeclared_detail` rather than re-inlining
  the hint — F2/F5). *Alt (paths only)* rejected: leaves the `--intent
  design-target` recall — itself a recurring case-note trap — on the orchestrator.
- **D3 — sibling `undeclared_test_files` fn + `VtSelectorFinding` type, over
  folding a variant into `VtShapeProblem`/`check_vt_shape`.** Keeps the two
  concerns cohesive (shape-completeness needs only the plan; coverage needs
  selectors) and leaves `check_vt_shape`'s plan-only tests byte-for-byte green
  (behaviour-preservation). Cost: a few duplicated render lines in
  `run_check_plan`. *Alt (unified enum + one render loop)* rejected: forces
  selectors into `check_vt_shape`'s signature and stretches "shape" to mean
  "coverage".

## 8. Risks & Mitigations

- **R1 — the two import arms drift in what `detail` says.** Mitigated by the
  single `undeclared_detail` formatter; a divergence would require editing one
  call site to stop using it (visible in review).
- **R2 — `check plan` reads selectors it didn't before → a new failure mode
  (unreadable selectors).** `slice::selectors` already propagates read failures
  (strict, as the import gate relies on); `check plan` inherits that — a corrupt
  selector store fails the lint loudly rather than silently disabling coverage.
- **R3 — hard exit on undeclared `test_file` blocks a legitimately-globbed
  test.** Mitigated: `undeclared_paths` honours globs (5.5), so a directory
  selector covers its files. The only hard-blocked case is a genuinely
  undeclared literal `test_file` — exactly the target.

## 9. Quality Engineering & Validation

- **VT (obj 1, pure)** — unit over `undeclared_detail`: golden string for a
  2-path set (count header + per-path remediation, id present in each command).
  This is the load-bearing coverage of the detail format.
- **VT (obj 1, wiring)** — integration over `import_compose` via the existing
  git-fixture harness (`dispatch.rs:~812`) — NOT a pure unit: the git rev-parse /
  merge-base gather precedes `classify_import`, so reaching the `UndeclaredScope`
  refusal needs a real fork with a delta (F3). This is the SAME test currently at
  `dispatch.rs:873` (`…refuses_before_compose_and_leaves_the_tip`) that asserts an
  EMPTY `String::new()` detail — obj-1 FLIPS it to assert the detail NAMES the
  offending path (F3/#3). Not a behaviour-preservation breach (that gate covers
  the pure `classify_import` verdict, unchanged); this is the expected update to
  the wiring assertion. Note (F9): the keyword floor (`undeclared_detail`,
  `UndeclaredScope`) matches production code in the same file, so this test body
  must actually assert on the non-empty detail — the keyword grep alone is
  vacuous.
- **VT (obj 2) fixture completeness (PLAUSIBLE).** `run_check_plan` now reads the
  slice doc via `slice::selectors` (`read_slice`), which it did not before. The
  obj-2 `check plan` test fixture MUST scaffold a complete `slice-NNN.toml` (with
  the selector rows), not a bare `plan.toml` — else `selectors` errors and the
  lint fails on setup rather than logic. Production `check plan <id>` always runs
  against a real slice that has the doc, so this is a test-authoring constraint,
  not a runtime regression. The pure `check_vt_shape` tests (in `plan.rs`, no
  fixture) are untouched.
- **VT (obj 2)** — unit over `undeclared_test_files`: a VT `test_file` not in the
  passed selectors ⇒ one finding; a covered `test_file` ⇒ none; empty selectors
  ⇒ none (A3); a glob-covered `test_file` ⇒ none.
- **VT (obj 2)** — `check plan` exit-code test over a fixture slice with an
  undeclared VT `test_file` (mirrors the IMP-209 exit tests): exits non-zero and
  names the path.
- **Behaviour-preservation** — `check_vt_shape` and `classify_import` suites stay
  green unchanged.

Closes when the obj-1 detail golden and the obj-2 lint tests land green and the
memory-blind refusal (detail names the path; `check plan` flags the plan) is
demonstrable.

## 10. Review Notes

Internal adversarial pass (author), integrated above:

- **F1 (correctness, integrated).** `slice selector add <ID> [GLOBS]…` takes the
  slice id as a required leading positional. The existing CLI remediation hint
  (`import.rs:161`) omits it — the emitted command would misparse the path as the
  id and fail. `undeclared_detail` now takes `slice: u32` and emits a runnable
  command; the refactor fixes the latent CLI bug as a bonus.
- **F2 (STD-001 / layering, integrated).** The leaf formatter no longer embeds the
  `undeclared-scope` token — the MCP `reason` field carries it, and a leaf
  reference to `Refusal::token()` (engine) would be a layering cycle. Header is
  the bare count line; the CLI arm prepends its own standalone header.
- **F3 (verification precision, integrated).** The obj-1 wiring test is
  integration over `import_compose` (needs the git fixture), not a pure unit; the
  pure `undeclared_detail` golden is the load-bearing format coverage.
- **A2 re-confirmed.** `Criterion` is `{ id, text }` (prose); `test_file` lives on
  `VerificationCriterion` — so a VT `test_file` is the only structured touch-path a
  plan declares. Obj 2's scope is exact.

External Opus inquisition (design + plan), integrated above:

- **F1 (MAJOR → fixed).** `run_check_plan`'s clean short-circuit
  (`check.rs:173`) fired before the coverage check — obj-2's primary case
  (shape-clean, uncovered `test_file`) would silently no-op. Restructured: read
  selectors + compute coverage BEFORE the short-circuit; exit clean only when
  both sets empty (§5.2). This was the load-bearing find.
- **F2 (MAJOR → fixed).** Obj-2's remediation footer had re-inlined the F1 bug
  (`selector add <path>` — no id). Both objectives now route their remediation
  through the single `undeclared_detail(id, paths)` formatter — the runnable,
  id-bearing hint has ONE source (also resolves the F5 DRY/STD-001 overclaim).
- **F4 (MINOR → fixed).** The CLI arm doesn't carry the slice id at the
  remediation site; it threads `slice: u32` through
  `run_import → classify_or_report → report_undeclared_scope` (§5.2). The prior
  "collapses to a 3-liner" framing understated it; PHASE-01 owns the plumbing.
- **F6 / F7 / F8 / F9 (MINOR/NIT → integrated)** — path-form honesty (§5.5),
  changed-not-preserved CLI output (§5.2), governance-tier `test_file` non-guard
  (§5.5), and the vacuous-keyword caveat on the wiring test (§9).
- **PLAUSIBLE (fixture regression → addressed)** — obj-2 test fixtures scaffold a
  full slice doc (§9); no runtime regression (production slices always carry it).

Verdict (inquisitor): sound to implement once F1 and F2 are resolved — both now
integrated.
