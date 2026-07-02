# SL-180 — Design: Selector conformance hardening

Design-time dry-run (IMP-204) + import-belt scope-creep refusal (IMP-199), both
wired onto the existing pure conformance algebra.

## 1. Current vs target behaviour

**Today.** `conformance::compute(selectors, actual)` (the pure three-cell
set-algebra, SL-147 D6) runs in exactly one place: `slice conformance <id>` at
audit time, folding the recorded boundary registry into `actual`. The import
belt (`worktree::import::classify_import`) rejects only `.doctrine/` / `.claude/`
prefixes — it has no selector awareness.

**Target.** The same pure primitive gains two new callers:

- **IMP-204** — `slice conformance <id> --against <range> [--strict]`: fold a git
  rev-range into `actual` instead of the registry; report undeclared paths;
  nonzero exit under `--strict`. The design-time under-declaration gate.
- **IMP-199** — `worktree import … [--slice <id>]`: when `--slice` is given, the
  belt refuses `UndeclaredScope` if the worker delta touches any path no
  `design-target` selector covers. No override flag; declare-then-retry via
  `slice selector add` is the ledger (see §7).

## 2. Pure core (`conformance.rs`) + layering

Both surfaces need one new pure question: *which of these paths does no selector
match?* Factor the matching `compute` already performs:

```rust
/// Compile selectors once; a None pattern (invalid glob) matches nothing.
fn compile(selectors: &[String]) -> Vec<(String, Option<Pattern>)> { … }

/// Paths matched by NO selector, in input order. Shared by the import belt
/// (IMP-199) and available to any path-list caller.
pub(crate) fn undeclared_paths(selectors: &[String], paths: &[&str]) -> Vec<String>
```

`compute` is refactored to call `compile` + the shared per-path match — **no
behaviour change**; the existing 6 `compute` tests are the freeze.
`undeclared_paths` is the belt's minimal need: names only, no `Status`/`Verb`
synthesis.

### Layering (ADR-001 — load-bearing)

- `conformance` is **engine** (→ `globmatch` leaf only).
- `worktree::import` is **engine** → may depend on `conformance` (engine→engine,
  no cycle: `conformance` imports no worktree). ✅
- `worktree::import` must **not** read `slice` — that is engine→command (upward,
  a hard gate violation). `worktree::coordinate` is command-tier *precisely* to
  legitimise its `slice::run_phases` call; `import` does not get that licence.

Resolution: the **pure** `classify_import` takes `selectors: &[String]` as
data (added parameter). The **selector read** (`--slice <id>` → design-target
strings) lives in the **command shell** `worktree::mod::run` (command tier —
sibling to `slice`; the sub-classification `worktree::import = engine` is the
reason the read must NOT sink into `import.rs`; `mod.rs` falls to the umbrella
`worktree = command`, the same licence `coordinate` uses for `slice::run_phases`).

**Tangle proof (corrected — the fitness gate, not a hand-wave).** The layering
gate extracts edges at **top-level-module granularity** and stores them in a
**deduplicated set** (`tests/architecture_layering.rs` — `first_segment`,
`extract_edges_submodule_files_map_to_parent`, `extract_edges_deduplicates`).
Every file under `src/worktree/` maps to the single unit `worktree`; a
`crate::slice::…` reference from *any* of them yields the edge `("worktree",
"slice")`. That edge **already exists** via `coordinate.rs`'s
`crate::slice::run_phases`. Adding `crate::slice::selectors(…)` in `mod.rs`
produces the *same* pair → the set collapses it → **command `tangle_baseline`
does not grow, no `accepted_violation` needed.** (My first draft argued "slice
imports no worktree" — that was false: `slice → review → worktree` exists
transitively. The real safety is edge-dedup, confirmed by running
`cargo test --test architecture_layering` green at baseline.) The PHASE-02
`just gate` obligation stands as the final proof.

Engine stays pure of slice; the one place that knows "slice → selectors" is the
command dispatcher.

## 3. IMP-199 — import belt (`worktree/import.rs` + `worktree/mod.rs`)

Pure classifier gains selectors as data:

```rust
pub(crate) enum Refusal {
    HeadMoved, TreeUnclean, MultiCommit, DoctrineTouch, ClaudeTouch,
    UndeclaredScope,                       // NEW — token "undeclared-scope"
}

pub(crate) fn classify_import(
    head_at_base: bool,
    tree_clean: bool,
    single_commit: bool,
    delta_paths: &[String],
    selectors: &[String],                  // NEW — design-target set; EMPTY ⇒ leg no-op
) -> Result<Apply, Refusal>
```

Belt order — the scope leg appends **after** the existing prefix legs
(`.doctrine/` / `.claude/` reject first, unconditionally):

```rust
if !selectors.is_empty() {
    let refs: Vec<&str> = delta_paths.iter().map(String::as_str).collect();
    if !conformance::undeclared_paths(selectors, &refs).is_empty() {
        return Err(Refusal::UndeclaredScope);
    }
}
```

**Empty-selectors = no-op** is the backward-compat hinge: `--slice` absent ⇒
shell passes `&[]` ⇒ leg skipped ⇒ today's behaviour verbatim. It also means a
slice with *zero* design-target selectors cannot be scope-checked at import —
correct (nothing declared to check against); IMP-204 is where under-declaration
gets caught before code exists.

**Shell plumbing:**

- `worktree import` gains `--slice <id>` (`Option<u32>`,
  `value_parser = slice::parse_cli_id` — admits `SL-097` / `097` / `97` via the
  shared `governance::parse_entity_ref`; no new parser).
- `worktree::mod::run` resolves it once — **the sole slice→selector read,
  command-tier**:
  ```rust
  let selectors = match slice {
      Some(id) => slice::selectors(&root, id, Some(SelectorIntent::DesignTarget))?,
      None => vec![],
  };
  ```
  and threads `&selectors` into `run_import(…, &selectors)`, which forwards to
  both `run_import_fork` / `run_import_from_worktree` → `classify_import`. The
  arms keep their own `root::find` for git work (idempotent same-root resolve);
  the slice read stays up in the command dispatcher.

  `slice::selectors(root, id, intent_filter)` is **one** reader (F6 disposition):
  `None` ⇒ the full union (subsumes today's `selector_paths`), `Some(intent)` ⇒
  filtered. It replaces the bespoke `design_target_selectors` and the inline
  design-target filter in `conformance_outcome` — one selector-read seam, three
  callers (registry conformance, dry-run, belt shell).

**Belt firing in practice (F4 disposition).** `--slice` is optional only to keep
the verb backward-compatible for *ad-hoc manual* imports. Inside dispatch the
belt is NOT inert: PHASE-02 edits the dispatch-agent skill (§6) — the arm that
actually invokes `worktree import` — so the orchestrator *always* passes
`--slice <id>` on the coordination-root import. The subprocess arm carries scope
via `slice record-delta` (no import invocation), and `--slice` is CLI-threaded to
*both* import arms, so subprocess is covered automatically if it ever gains one
(reconciled, RV-233 F-1). So the only
imports that skip the scope check are hand-run ones outside the dispatch funnel —
out of the scope-creep threat model (a human running `worktree import` by hand
owns the consequence). Promoting `--slice` to required is the §9 open question,
deferred until every import path routes through dispatch.

**Refusal legibility (the ledger's usability).** Today `bail!("import-refused:
{}", token)` prints only the token. For `UndeclaredScope` the human needs the
*paths* to `selector add`, so the shell re-derives and prints the list before
bailing:

```
import-refused: undeclared-scope
  the worker delta touches paths no design-target selector covers:
    src/foo.rs
    src/bar.rs
  declare (if in scope):  doctrine slice selector add SL-180 <path> --intent design-target --note "<why>"
  then re-run import.
```

**Why re-derive rather than a payload-carrying refusal (F7 disposition).**
`Refusal` stays a `Copy` unit enum (every other variant is unit; `token()` stays
trivial). Rather than widen it to `UndeclaredScope(Vec<String>)` — which forces
the whole enum off `Copy` — the shell re-computes the list with the **same** pure
`conformance::undeclared_paths(selectors, delta_paths)` it already fed the
classifier. Same function, same inputs ⇒ identical output; there is no second
implementation to drift. The cost is one extra pure pass over an already-gathered
path list (negligible), bought against keeping the refusal type a flat `Copy`
token set.

## 4. IMP-204 — design-time dry-run (`slice.rs`)

Extend the existing verb — no parallel subcommand:

```rust
Conformance {
    id: u32,                                          // parse_cli_id (unchanged)
    #[arg(long)] against: Option<String>,             // NEW — git rev-range
    #[arg(long, requires = "against")] strict: bool,  // NEW — undeclared ⇒ nonzero exit
    path: Option<PathBuf>,
}
```

`--strict` `requires` `--against` (clap-enforced): strict is the design-time
gate's fail mode, meaningless against the registry.

Source switch in `run_conformance`:

- `against: None` → today's path verbatim (`conformance_outcome` → registry fold
  + degrade ladder `Unavailable` / `Incomplete`).
- `against: Some(range)` → **skip registry + completeness gate** (a range has no
  completeness notion):
  ```rust
  let range = require_rev_range(&range)?;                 // F8: must contain ".."
  let selectors = slice::selectors(&root, id, Some(SelectorIntent::DesignTarget))?;
  let actual    = actual_from_range(&root, range)?;       // new: one diff, folded
  let result    = conformance::compute(&selectors, &actual);
  render_conformance(&cid, &result)?;
  if strict && !result.undeclared.is_empty() {
      bail!("{cid}: --strict: {} undeclared path(s) vs {range}", result.undeclared.len());
  }
  ```

**Range-syntax guard (F8 disposition).** `--against` rejects a bare ref: the
value must contain `..` (`A..B` two-dot or `A...B` three-dot), else a clear
`--against expects a rev-range (A..B), got '<x>'`. This kills the single-ref
footgun (a bare `main` diffs against the *working tree*, silently folding
uncommitted changes) at the boundary rather than in prose.

**quotePath hardening (F3 disposition).** `actual_from_range` runs
`git -c core.quotePath=false diff --name-status --no-renames <range>` — the SAME
non-ASCII hardening the import belt already documents (`import.rs:183-191`).
Without it, a non-ASCII path is C-quoted and mis-folds / mis-prints. A non-ASCII
VT pins it (§5). Note: the *existing* registry fold (`conformance_outcome`,
slice.rs:~2260) lacks this too — a pre-existing latent gap; PHASE-01 hardens the
new path and, as a one-line adjacent fix, the registry fold as well (same file,
behaviour-preserving for ASCII paths, which is all the current tests exercise).

**DRY reuse:** `fold_name_status_line`, `conformance::compute`,
`render_conformance` — all verbatim. `actual_from_range` is the only new fold fn.
`slice::selectors(…, Some(DesignTarget))` is the shared selector read (F6); the
inline design-target filter in `conformance_outcome` is refactored to call it →
**one selector-read seam, three callers** (registry conformance, dry-run, belt
shell).

## 5. Verification alignment

| Mode | Where | Assertion |
|---|---|---|
| VT | `conformance.rs` | `undeclared_paths`: unmatched subset returned in input order; empty selectors ⇒ all undeclared; glob absorbs; literal exact-only |
| VT | `import.rs` (pure) | `classify_import` w/ selectors: undeclared ⇒ `UndeclaredScope`; `.doctrine/` refuses *before* the scope leg (precedence); empty selectors ⇒ no-op `Ok`; conformant ⇒ `Ok` |
| VT | `import.rs` (integ) | `run_import_from_worktree --slice`: undeclared path ⇒ `import-refused: undeclared-scope` + path list; conformant ⇒ applies (reuses `init_repo` helpers) |
| VT | `slice.rs` | `--against` folds a range & flags undeclared; `--strict` ⇒ nonzero; bare-ref (no `..`) ⇒ range-syntax error (F8); **non-ASCII path folds/prints verbatim** (F3 quotePath) |
| VA | `compute` suite | the 6 `compute` tests stay **green unchanged** — the `compile` extract is pure-preserving (textually untouched) |
| VA | belt suite | existing `classify_import` / `run_import` tests keep their **verdicts** (empty-selectors == old belt); call sites gain a mechanical `&[]` / `&selectors` arg. Behaviour-preserving, not text-frozen — `classify_import` is worktree-local, not the entity engine, so the signature change is sanctioned |

Maps onto the slice's stated 3×VT + 1×VA (VA split here into the two suites it
touches, to be honest about which is text-frozen vs verdict-frozen).

**Verification obligation (layering):** PHASE-02 must run `just gate` after the
`worktree::mod → slice` edge lands. The edge is reasoned safe (slice imports no
worktree ⇒ the edge is in no SCC ⇒ command `tangle_baseline` cannot grow, and
command→command needs no `accepted_violation` entry), but the fitness gate is
the proof, not this argument.

## 6. Code-impact (design-target selectors)

```
src/conformance.rs                                    # undeclared_paths + compile extract
src/worktree/import.rs                                # UndeclaredScope, classify_import sig, shell thread
src/worktree/mod.rs                                   # --slice arg, selector read, plumbing
src/slice.rs                                          # --against/--strict, slice::selectors, actual_from_range, quotePath fold
plugins/doctrine/skills/dispatch-agent/SKILL.md       # add --slice <id> to import invocation
```

> **Not a target: `dispatch-subprocess/SKILL.md`** (reconciled, RV-233 F-1). The
> subprocess arm records the boundary via `slice record-delta` — it has **no**
> `worktree import` invocation to annotate — so there is no skill edit to make.
> `--slice` is CLI-threaded to *both* import arms (`--fork` / `--from-worktree` in
> `worktree::mod`), so subprocess is covered automatically if it ever gains an
> import. It was declared a design-target selector at design time (undeliverable
> by design); the selector was dropped at reconcile so conformance reads clean.

Literal paths (design names each file); test code rides the same modules
(inline `#[cfg(test)]`). `.agents/skills/` is derived output of `just reinstall`
(`npx skills add .`), never hand-edited — `plugins/doctrine/skills/` is source.

**Phasing** (slice order holds): PHASE-01 = IMP-204 builds the shared
`slice::selectors(…, intent)` reader + `actual_from_range` (quotePath-hardened) +
`--against`/`--strict` CLI. PHASE-02 = IMP-199 adds `undeclared_paths` + belt +
`--slice` plumbing + skill edits + the `just gate` layering proof. The shared
selector reader lands in 01, reused by 02 — no rework.

## 7. Override policy — declare-then-retry is the ledger

No `--allow-undeclared` bypass. There is no import-time ledger surface today; a
silent bypass would leave zero durable trace. The system already has the right
ledger — the **selectors themselves**. On `UndeclaredScope` the orchestrator
(sole writer, on the coordination branch) runs:

```
doctrine slice selector add <id> <path> --intent design-target --note "<why in scope>"
```

then re-imports — now conformant. The selector upsert IS the durable, committed,
diffable, audit-reviewed ledger entry (surfaced via F-7 with its `note`). Two
legit outcomes fall out: worker genuinely crept ⇒ reject/redo, never declare;
legit compile-fallout IMP-204 missed ⇒ declare with a reason, retry. Deliberate
by construction — creep is never one keystroke away.

**Note convention (F5 disposition).** The `--note` is not optional flavour — it
is the ledger's *justification* field and MUST state why the path is in scope
(e.g. `"compile fallout: CLI wiring for the new --against flag"`). The declare
records the *widening*, not the refusal event; the note is what makes that
widening auditable at `/audit` (a bare/empty note on a design-target selector is
a reconciliation smell the auditor should challenge). This is the accepted limit
of the "selectors-as-ledger" posture (§ inquisition F5): we record intent-to-
declare with a reason, not a per-refusal event log — the lighter surface chosen
over a bespoke import-refusal ledger kind.

## 8. Risks

- **False positives at import.** Mitigated by IMP-204 closing the gap *before*
  import, and by declare-then-retry (§7) rather than a hard dead-end.
- **Double root resolve** in the belt path (shell + arm). Benign — idempotent;
  keeps the slice read out of engine.
- **Skill dual-root drift.** Editing only `plugins/doctrine/skills/` relies on
  `just reinstall` to propagate to `.agents/skills/`; an un-reinstalled dev tree
  runs the old skill and never passes `--slice`. Acceptable — install hygiene,
  not a code defect.
- **`--against` single-ref footgun.** Now *guarded* — §4 F8: `--against` rejects
  a value without `..`. Residual: a valid-but-wrong range (`HEAD~5..HEAD`) still
  sweeps unrelated commits; that is operator responsibility, not a code defect.
- **Layering claim is reasoned AND checked.** §2's tangle proof is verified by
  edge-dedup semantics + a green baseline `architecture_layering` run; §5's
  `just gate` in PHASE-02 is the final proof after the edge lands.

## 9. Open questions

None blocking. `--slice` is optional (opt-in) — enforcement decision sits at the
dispatch call site; a future slice may promote it to required once every import
path routes through dispatch (§3 F4).

## 10. External inquisition disposition (GPT-5.x, codex mcp)

A hostile external pass ran against this doc + the actual source. All 8 findings
triaged and integrated:

| # | Claim | Severity (GPT / actual) | Disposition |
|---|---|---|---|
| F1 | dry-run checks selectors, not the design.md Code-impact table | blocker / minor | **Reconciled.** Selectors ARE the machine-readable Code-impact projection (design-skill doctrine); slice scope wording aligned. Selectors-as-authority is explicit; widening is the audited ledger (§7). |
| F2 | `worktree::mod → slice` grows command tangle_baseline | blocker / none | **Resolved.** Gate extracts edges at top-level-module granularity, deduplicated; `("worktree","slice")` already exists via `coordinate.rs`. My original justification was wrong; corrected in §2, baseline `architecture_layering` green. |
| F3 | `actual_from_range` misses `core.quotePath=false` | major / valid | **Accepted** (§4). Hardened + non-ASCII VT; adjacent registry fold hardened too. |
| F4 | optional `--slice` ⇒ belt inert / silent bypass | major / partial | **Clarified** (§3). Dispatch skills always pass `--slice` (in-scope); only ad-hoc manual imports skip — out of threat model. |
| F5 | declare-then-retry ledger is hand-waving | major / minor | **Strengthened** (§7). `--note` mandated as the justification field; accepted limit of the lighter surface. |
| F6 | `design_target_selectors` duplicates `selector_paths` | minor / valid | **Accepted** (§2/§4). Replaced with one intent-filtered `slice::selectors(root, id, intent)`. |
| F7 | make refusal carry `Vec<String>` payload | minor / declined | **Declined w/ rationale** (§3). Keep `Refusal: Copy`; shell re-derives via the same pure `undeclared_paths` — no drift. |
| F8 | raw `--against` surface dangerous | minor / valid | **Accepted** (§4). Range-syntax guard (must contain `..`). |

Net: two "blockers" were reviewer errors (F1 misread the resolved semantics; F2's
tangle claim inverted the gate's actual edge granularity — but the *demand to
verify rather than assert* was correct and surfaced a genuinely wrong
justification in my draft). Three real hardening wins (F3, F6, F8). No finding
survives as blocking; design is safe to proceed to `/plan`.
