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
sibling to `slice`; `slice` imports no `worktree`, so no cycle and no
tangle-baseline growth). Engine stays pure of slice; the one place that knows
"slice → selectors" is the command dispatcher.

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
      Some(id) => slice::design_target_selectors(&root, id)?,
      None => vec![],
  };
  ```
  and threads `&selectors` into `run_import(…, &selectors)`, which forwards to
  both `run_import_fork` / `run_import_from_worktree` → `classify_import`. The
  arms keep their own `root::find` for git work (idempotent same-root resolve);
  the slice read stays up in the command dispatcher.

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
  let selectors = design_target_selectors(&root, id)?;   // shared helper
  let actual    = actual_from_range(&root, &range)?;      // new: one diff, folded
  let result    = conformance::compute(&selectors, &actual);
  render_conformance(&cid, &result)?;
  if strict && !result.undeclared.is_empty() {
      bail!("{cid}: --strict: {} undeclared path(s) vs {range}", result.undeclared.len());
  }
  ```

**DRY reuse:** `fold_name_status_line`, `conformance::compute`,
`render_conformance` — all verbatim. `actual_from_range` is the only new fn
(`git diff --name-status <range>` → fold loop → `BTreeMap`).
`design_target_selectors` is the shared selector read; the inline design-target
filter in `conformance_outcome` (registry path) is refactored to call it too →
**one selector-read source, three callers** (registry conformance, dry-run,
belt shell).

## 5. Verification alignment

| Mode | Where | Assertion |
|---|---|---|
| VT | `conformance.rs` | `undeclared_paths`: unmatched subset returned in input order; empty selectors ⇒ all undeclared; glob absorbs; literal exact-only |
| VT | `import.rs` (pure) | `classify_import` w/ selectors: undeclared ⇒ `UndeclaredScope`; `.doctrine/` refuses *before* the scope leg (precedence); empty selectors ⇒ no-op `Ok`; conformant ⇒ `Ok` |
| VT | `import.rs` (integ) | `run_import_from_worktree --slice`: undeclared path ⇒ `import-refused: undeclared-scope` + path list; conformant ⇒ applies (reuses `init_repo` helpers) |
| VT | `slice.rs` | `--against` folds a range & flags undeclared; `--strict` ⇒ nonzero |
| VA | both suites | existing import belt tests + 6 `compute` tests **green unchanged** (behaviour-preservation gate: empty-selectors == old belt; `compute` refactor is pure-preserving) |

Maps 1:1 onto the slice's stated 3×VT + 1×VA.

## 6. Code-impact (design-target selectors)

```
src/conformance.rs                                    # undeclared_paths + compile extract
src/worktree/import.rs                                # UndeclaredScope, classify_import sig, shell thread
src/worktree/mod.rs                                   # --slice arg, selector read, plumbing
src/slice.rs                                          # --against/--strict, design_target_selectors, actual_from_range
plugins/doctrine/skills/dispatch-agent/SKILL.md       # add --slice <id> to import invocation
plugins/doctrine/skills/dispatch-subprocess/SKILL.md  # same (subprocess arm)
```

Literal paths (design names each file); test code rides the same modules
(inline `#[cfg(test)]`). `.agents/skills/` is derived output of `just reinstall`
(`npx skills add .`), never hand-edited — `plugins/doctrine/skills/` is source.

**Phasing** (slice order holds): PHASE-01 = IMP-204 builds `design_target_selectors`
(shared) + `actual_from_range` + CLI. PHASE-02 = IMP-199 adds `undeclared_paths` +
belt + `--slice` plumbing + skill edits. The shared selector read lands in 01,
reused by 02 — no rework.

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

## 8. Risks

- **False positives at import.** Mitigated by IMP-204 closing the gap *before*
  import, and by declare-then-retry (§7) rather than a hard dead-end.
- **Double root resolve** in the belt path (shell + arm). Benign — idempotent;
  keeps the slice read out of engine.
- **Skill dual-root drift.** Editing only `plugins/doctrine/skills/` relies on
  `just reinstall` to propagate to `.agents/skills/`; an un-reinstalled dev tree
  runs the old skill and never passes `--slice`. Acceptable — install hygiene,
  not a code defect.

## 9. Open questions

None blocking. `--slice` is optional (opt-in) — enforcement decision sits at the
dispatch call site; a future slice may promote it to required once every import
path routes through dispatch.
