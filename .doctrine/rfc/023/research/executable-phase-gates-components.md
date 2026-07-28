# RFC-023 Research: executable phase gates — component survey

Survey of the four components RFC-023 touches — phase sheets, `doctrine check`,
`doctrine.toml`, and the `justfile` — with current shapes, seams, and the
cheapest path to making phase gates executable without per-language intelligence.

## 1. Phase sheets

### 1.1 Two tiers

**Authored plan** (`plan.toml` under `.doctrine/slice/NNN/`). Durable, committed,
diffable. Carries `[[phase]]` arrays with EN/EX/VA/VT/VH criteria. The plan is
the single source; phase status lives elsewhere.

**Runtime phase sheets** (`.doctrine/slice/NNN/phases/phase-NN.{toml,md}`):
- `.toml` — `schema = "doctrine.phase.tracking"`, version 1. Single table:
  `phase`, `status ∈ {planned,in_progress,completed,blocked}`, `started`,
  `completed`, `last_updated`. Append-only progress log.
- `.md` — Disposable scratch: objective expansion, task list, assumptions,
  STOP conditions, risks, decisions, findings. Gitignored by convention but
  here the sheets live in the authored tree (not `.doctrine/state/`).

**Runtime state** (`.doctrine/state/slice/NNN/`) — phase tracking is duplicated
here in some slices; SL-233 has no runtime state yet (not started).

### 1.2 Verification criteria shape (plan.toml)

```toml
[[phase]]
id = "PHASE-03"
verification = [
  { id = "VT-1", test_file = "tests/e2e_design_state.rs",
    keywords = ["watermark"],
    patterns = ['^\s*fn hand_edit_between_applies_is_refused_at_entry\('] },
  { id = "VA-1", text = "RUN: `cargo test --test e2e_design_state -- --list` — all thirteen EX-12 names appear" },
  { id = "VA-NC", text = "NEGATIVE CONTROL: each test was observed RED before green" },
  { id = "VA-G", text = "RUN: `doctrine check gate` — exit 0" },
]
```

Three modes encoded in the id prefix (not a field):
- **VT** — test, automated. Carries `test_file`, `keywords`, `patterns`, `waived`.
  Presence-only check (`vtgate.rs::check_vt` — substring + regex over raw source,
  no execution).
- **VA** — agent check. Free-text `text` (or `expects`). Prose instructions an
  agent runs by hand (e.g. "RUN: `cargo test …` returns nothing").
- **VH** — human acceptance. Same shape as VA, different sign-off.

### 1.3 Current executability

Only VT rows are machine-checked (by `vtgate`), and only for presence — the test
is never run. VA rows are prose. The plan currently has NO executable row type
that spawns a process.

---

## 2. `doctrine check`

### 2.1 Architecture

Pure/impure split (ADR-001):
- `src/verify.rs` (engine) — `resolve_check` (pure, config → CheckPlan),
  `run_suite` (impure, spawns process, returns `SuiteStatus`)
- `src/commands/check.rs` (command shell) — routes subcommands, forwards exit
  codes via `std::process::exit`

### 2.2 Cadences

| Cadence | Default (when `[verification].<key>` absent) | Config key |
|---------|---------------------------------------------|------------|
| `quick` | OWNED no-op ("doctrine check quick: no [verification].quick set — skipping") | `quick` |
| `commit` | `just check` | `commit` |
| `gate`   | `just gate` | `gate` |
| `prove`  | `just prove` | `prove` |

Plus: `regression` (S1 baseline-diff, not a cadence proxy), `plan` (VT shape
lint — `BareTestFile`/`BareKeywords`/`UndeclaredTestFile`).

### 2.3 The POL-002-clean proxy pattern

This is the key mechanism. `resolve_check`:
1. Reads the `[verification]` config from `doctrine.toml`
2. Resolves the cadence to `CheckPlan::{Run(argv),Noop,Empty}`
3. The engine knows only the cadence name; the project owns the command

`run_suite` then spawns `argv` with `cwd = root`, inheriting stdio, no timeout,
and returns `SuiteStatus::Completed { code }`.

**This is the pattern to reuse.** It delegates the actual check logic to the
project's toolchain without the engine knowing the host language.

### 2.4 `check plan`

Runs `check_vt_shape` + `undeclared_test_files` — structural lint, not
execution. Flags plans whose VT rows lack structured mandates or whose
`test_file` paths are undeclared by design-target selectors.

---

## 3. `doctrine.toml`

### 3.1 Location & parser

`.doctrine/doctrine.toml` at the repo root. Parsed by `src/dtoml.rs` — a single
shared reader (`load_config`). The `VerificationConfig` is a subsection.

### 3.2 `[verification]` table

```toml
[verification]
command = ["just", "gate"]          # project-default base argv for VT checks
quick   = ["just", "quick"]         # override: `doctrine check quick`
commit  = ["just", "check"]         # override: `doctrine check commit`
gate    = ["just", "gate"]          # override: `doctrine check gate`
prove   = ["just", "prove"]         # override: `doctrine check prove`
regression = ["cargo", "test", "--no-fail-fast"]  # S1 regression suite
default-source = "stdout"           # where to match output
timeout-secs = 300                  # per-check timeout

[verification.aliases]
gate = ["just", "gate"]             # named aliases a VT check can reference
unit = ["cargo", "test"]
```

Fields in `VerificationConfig` (serde, kebab-case):
- `command: Option<Vec<String>>` — default base argv
- `quick/commit/gate/prove/regression: Option<Vec<String>>` — per-cadence overrides
- `aliases: BTreeMap<String, Vec<String>>` — named argvs
- `default_source: Option<MatchSource>`, `timeout_secs: Option<u64>`

### 3.3 `[dispatch]` table

```toml
[dispatch]
authoring-branch = "refs/heads/edge"
verify-suite = "gate"               # default: "gate" (the strictest cadence)
```

`verify-suite` controls which cadence the dispatch funnel's `dispatch verify`
runs. Default `"gate"`, resolved via `CheckKind::from_key`.

### 3.4 Why it matters

The `[verification]` table is already the single declaration point for
*project-level* check commands. If phase-gate execution follows the same pattern,
a new `[verification.aliases]` entry (e.g. `phase-vt = ["just", "check"]`)
or a `command` field directly on the VT/VA row is all the config needed — zero
new engine intelligence.

---

## 4. The `justfile`

### 4.1 Current recipes (the cadence targets)

```makefile
quick: fmt lint validate                        # per-edit
check: fmt lint lint-js build validate test     # fast inner loop (root package only)
gate:  fmt lint lint-js build validate test-all # full workspace (incl. cordage)
prove: fmt-check lint                           # non-mutating clean assertion
```

Where `test` = `cargo test` (root only), `test-all` = `cargo test --workspace`,
`validate` = `doctrine prompt check && doctrine doctor`.

### 4.2 Why `just` is the right execution layer

`just`:
- Is language-agnostic — recipes name whatever commands the project uses
- Already the project convention (POL-002-safe: the engine proxies `just`,
  not `cargo`)
- Allows the project to compose gate logic without engine changes
- Has exit-code semantics (recipe fails if any step fails)
- Is already declared in `[verification]` config

### 4.3 The gap for per-phase gates

A `just gate` runs the *whole workspace* — slow. There is no per-phase variant.
A per-phase gate could be:
- A `just` recipe parameterized by phase (e.g. `just phase-gate PHASE-03`)
- Or a standalone recipe per phase (`just phase-03`)

But baking per-phase recipes into the justfile couples the justfile to the
plan — the plan would need a way to declare which just recipe gates its phase.

---

## 5. The vtgate (VT existence check)

### 5.1 Current mechanism

`src/vtgate.rs` — pure engine. `check_vt` takes a `VerificationCriterion`, a
`read_file` injector, and a `modified_files` set. It:
1. Short-circuits on `waived`
2. Skips if no `test_file`
3. Fails if file missing
4. Marks `Unattributable` if file not in slice's modified set
5. Greps for `keywords` (raw substring) and `patterns` (line-anchored regex)
6. Returns `Pass` if all present

**It never runs a test.** Presence only. Comment text satisfies keywords.

### 5.2 The non-goal: per-language comment stripping

The module doc explicitly states comment-stripping is out of scope (POL-002):
comment syntax is host-language convention. The gate is already at its
POL-002-clean ceiling for presence checks.

---

## 6. Ideas: cheapest path to executable phase gates

These are candidate directions, kept concise per the assignment.

### Idea A — Executable VA rows (cheapest)

Add two optional fields to `VerificationCriterion`:

```toml
{ id = "VA-1", command = ["just", "check"], expect_exit = 0 }
```

A new `doctrine slice verify-va <id>` verb (or a mode on the existing `verify-vt`
codepath) would:
1. Scan the plan for VA rows with `command`
2. Proxy-execute each through `run_suite` (the existing runner)
3. Exit non-zero if `expect_exit` mismatches

**Cost**: one new field on an existing struct, one new CLI verb, zero new
per-language intelligence. The `command` is project-declared (POL-002) —
it calls `just phase-03` or `cargo test --test e2e_foo` as the author chooses.

This is the same proxy pattern as `doctrine check`, applied to a different
config source (the plan, not `doctrine.toml`).

### Idea B — VT mode that actually runs

Extend VT rows with a `command` field (or alias to `[verification.aliases]`):

```toml
{ id = "VT-1", test_file = "src/foo.rs", keywords = ["fn bar"],
  command = ["cargo", "test", "--", "bar_test"], expect_exit = 0 }
```

When `command` is present, `check_vt` runs the presence check AND proxy-executes
the command. When absent, behaviour is unchanged.

The `command` field already exists on `VtCheck` in the coverage system — it's
used for `coverage verify`, not for phase gates, but is the same shape.

### Idea C — Declare phase gate recipe in plan.toml

Add a `gate` field to `[[phase]]`:

```toml
[[phase]]
id = "PHASE-03"
gate = ["just", "check"]   # or ["just", "phase-gate", "PHASE-03"]
```

`doctrine slice verify-phase SL-233 PHASE-03` would proxy-run it through
`run_suite`. The engine knows nothing about what the command does.

This is slightly coarser than per-criterion execution but simpler — one command
per phase, same proxy pattern.

### Idea D — Make `doctrine check gate` phase-aware

Instead of new plan fields, let the plan declare a named check alias:

```toml
[verification.aliases]
phase-03 = ["cargo", "test", "--test", "e2e_design_state"]
```

Then every `VA-G` row already says "RUN: `doctrine check gate`" — if the project
configures `gate` to include the per-phase alias, the existing VA row becomes
executable without new fields.

The weakness: VA rows are still prose-attested. A mechanical VA runner would
need a way to find the command, and putting it in `doctrine.toml` aliases
decouples it from the plan that carries the criteria.

### Idea E — `just` recipe per phase, auto-discovered

Convention: a phase `PHASE-03` implies a `just phase-03` recipe. The engine
auto-discovers it — no config, no new fields. Fails clearly if the recipe
doesn't exist.

```makefile
phase-03:
  cargo test --test e2e_design_state
```

This is zero-config but couples the plan to the justfile by convention. It's
also POL-002-clean (the engine only knows cadence names; the justfile carries
the host conventions).

### Comparative assessment

| Axis | A (executable VA) | B (VT runs) | C (phase gate) | D (alias VA) | E (auto-discover) |
|------|-------------------|-------------|----------------|--------------|-------------------|
| New fields on plan | `command`+`expect_exit` on VA | `command`+`expect_exit` on VT | `gate` on phase | none (alias in doctrine.toml) | none |
| New CLI verbs | `verify-va` | none (extends existing) | `verify-phase` | none (reuses `check`) | `verify-phase` |
| Per-language intelligence | none | none | none | none | none |
| POL-002 risk | zero (project-declared command) | zero | zero | zero | zero |
| Granularity | per-criterion | per-criterion | per-phase | per-cadence | per-phase |
| Config locality | plan (good) | plan (good) | plan (good) | doctrine.toml (weak) | convention (weak) |
| Backward compat | additive | additive | additive | additive | additive |
| Rides existing `run_suite` | yes | yes | yes | yes | yes |

**A** (executable VA rows) is the cheapest path: one field on an existing struct,
one new CLI subcommand, rides the existing proxy infrastructure 1:1. **B** is a
natural complement once A exists. **C** is coarser but even simpler. **E** is the
absolute minimax — zero new schema, purely conventional — but fragile.

## 7. References

- `src/verify.rs` — `resolve_check`, `run_suite` (the proxy pattern to reuse)
- `src/commands/check.rs` — `dispatch`, `run_check_plan`
- `src/plan.rs` — `Plan`, `PlanPhase`, `VerificationCriterion`, `check_vt_shape`, `undeclared_test_files`
- `src/vtgate.rs` — `check_vt`, `check_phases` (presence-only gate)
- `src/dtoml.rs` — `.doctrine/doctrine.toml` parser, `VerificationConfig`
- `src/dispatch_config.rs` — `verify_suite` field
- `src/coverage.rs` — `VtCheck` (the alias/command shape reused in coverage)
- `.doctrine/doctrine.toml` — project verification + dispatch config
- `justfile` — `quick`, `check`, `gate`, `prove`, `validate` recipes
- `.doctrine/slice/233/plan.toml` — representative 12-phase plan with VT/VA/VH
- `.doctrine/rfc/023/rfc-023.md` — the RFC itself (Directions A–H)
