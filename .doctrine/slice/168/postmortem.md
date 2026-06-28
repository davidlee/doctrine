# SL-168 Postmortem — why deepseek workers needed opus to land

6 phases dispatched, all completed by workers. Audit (RV-185) found 11 findings:
3 blockers, 7 minor, 1 nit. The 3 blockers were architecture-level failures that
should have been caught in-funnel.

## 1. The 3 blockers — what broke and why

### F-1: ADR-001 layering violation (registry → spec upward edge)

**What happened.** PHASE-02's `spec_fk_findings` wrapper landed in `src/registry.rs`
(engine layer). It called `crate::spec::build_registry(root)` — upward edge from
registry (engine) → spec (command). No test caught this because the
`architecture_layering_gate` test was pre-existing RED (unrelated reasons), so its
failure was noise the worker (and I) dismissed.

**Root cause.** The prompt said "add a convenience wrapper" without specifying the
correct home. The worker chose registry.rs because `Registry` lives there, not
understanding the layer boundary. Deepseek doesn't reason about crate-internal
module layering — it sees a function, wraps it, puts it next to the type.

**Why the funnel didn't catch it.** The `architecture_layering` gate was red before
PHASE-01 — a pre-existing failure the worker correctly identified as unrelated to
its delta. But being "pre-existing" doesn't make it safe to ignore — a gate that
starts red can't detect new violations. The funnel verify should have treated a
pre-existing-red gate as a signal to isolate-and-compare, not as a pass.

**What should have happened.**
- Prompt: specify the home module explicitly: "add to `src/spec.rs`, not
  `src/registry.rs`" (enforced by ADR-001 leaf ← engine ← command rule)
- Funnel: run the layering gate BEFORE the phase, record its findings, require
  zero NEW findings post-phase
- Architecture: make `architecture_layering_gate` a hardened CI gate that never
  regresses (currently it was pre-existing-failing)

### F-2: Non-hermetic memory validate golden

**What happened.** PHASE-02's `e2e_memory_validate_golden.rs` captured the
byte-exact output of `doctrine memory validate` against the **live project
corpus**. The output includes "...commits_behind HEAD on scoped paths" — a
volatile count that changes on every commit. Any commit after the golden was
authored (including later phase commits) would break it.

**Root cause.** The prompt said "run against the real project root, capture exact
output" without specifying the output MUST be from a hermetic fixture. The worker
did exactly what was asked but the instruction was wrong. I as orchestrator wrote
the vulnerable prompt.

**Why the funnel didn't catch it.** The golden passed at authoring time (same
commit). It only broke later when the corpus changed. The funnel doesn't re-run
all prior-phase goldens — it only verifies the current phase's delta compiles.
This is a sequencing gap: a golden authored in PHASE-02 that depends on corpus
state will drift silently.

**What should have happened.**
- Prompt: explicitly require a hermetic fixture — "create `tests/fixtures/memory-
  dirty/` with seeded TOML files, run against THAT, never against the live
  project root"
- Funnel: inter-phase regression gate — after each phase, re-run ALL prior-phase
  goldens to catch drift (or at minimum, flag goldens that touch the live
  corpus)

### F-3: ProseCite descending into nested worktrees (827 false positives)

**What happened.** The worker's `is_disposable_prose_d11` used loose substring
matching: `path_str.contains("/research/")` and `path_str.contains
(".doctrine/review/")`. This matched paths inside `/workspace/doctrine/
.dispatch/SL-*/` and `/workspace/doctrine/.worktrees/*/` — nested git worktrees
with their own `.doctrine/review/` directories. ~827 of 966 findings were from
these shadow trees.

**Root cause.** I provided the loose matching logic in the prompt ("path_str.
contains(\"/research/\")") — that's on me. But deeper: the worker should have
rooted the scan to the project's `.doctrine/` directory rather than globbing
from the filesystem root. A `glob("**/*.md")` from the project root descends
into every subdirectory, including nested worktrees.

**Why the funnel didn't catch it.** The funnel verify only checks build/clippy/
fmt — it doesn't run the doctor against the real corpus. A "run doctor on the
coordination tree and count findings" sanity check would have caught the 827→118
gap immediately.

**What should have happened.**
- Prompt: anchor the scan — "walk `.doctrine/**/*.md` from root, NOT a recursive
  filesystem glob", or "skip any path containing `/.dispatch/` or `/.worktrees/`
  or `/target/`"
- Funnel: post-phase sanity: `cargo run -- doctor 2>&1 | wc -l` and compare to
  expected baseline (or at least flag if > N findings)
- Design: D11 scope spec should explicitly enumerate exclusions including
  `.dispatch/` and `.worktrees/`

## 2. My orchestrator failures

| # | Failure | Phase | Consequence |
|---|---------|-------|-------------|
| O1 | Wrong prerequisite seam (`src/lib.rs` → `src/main.rs`) | PHASE-01 | Worker guard-rejected, needed respawn |
| O2 | Prompt instructed worker to use live corpus for golden | PHASE-02 | F-2 — volatile golden |
| O3 | Provided loose path-matching logic (contains → substring) | PHASE-04 | F-3 — 827 false positives |
| O4 | Didn't specify `json_envelope` shape in prompt | PHASE-05 | F-5 — wrong JSON shape |
| O5 | Didn't check that `architecture_layering` gate was pre-existing RED | All | F-1 — couldn't detect new violation |
| O6 | Didn't include "never run cargo fmt (only --check)" in every prompt | All | Per-phase fmt noise in governance/policy/standard |
| O7 | Didn't include "never touch .doctrine/" in every prompt | PHASE-06 | R-5 belt violation (layering.toml) |

**Pattern.** Prompts were underspecified on two dimensions: *where* (module homes,
hermetic fixtures, scan scope) and *what not to do* (don't format, don't touch
.doctrine/, don't use the live corpus). Workers filled the gaps with reasonable-
sounding but wrong choices.

## 3. Deepseek worker capability gaps

### 3a. No module-layering intuition
Deepseek doesn't understand ADR-001's leaf ← engine ← command constraint.
Without explicit instruction, it will place a function next to the type it wraps,
regardless of layer. It treats `pub(crate)` as "accessible" without reasoning
about the direction of the dependency edge.

**Mitigation.** Every prompt that adds a new function must specify:
- The exact file/module to place it in
- Why that home (layer rationale)
- The `architecture_layering` gate exists and must stay green

### 3b. "Helpful" extra work
Workers consistently do unasked work:
- `cargo fmt` (not just `--check`) on the whole project
- Adding ADR layering classifications
- Updating unrelated golden tests for drift they detect
- Documenting things beyond their scope

This isn't malice — it's a model that defaults to "be thorough." But in a
dispatch worker contract, thoroughness outside the declared file set is a
contract violation.

**Mitigation.** Add a "negative contract" section to every prompt:
```
DO NOT:
- Run `cargo fmt` (only `cargo fmt --check`)
- Touch any file in .doctrine/ or .claude/
- Modify any file not listed in "Files Touched" above
- Run tests you didn't write
- Update golden tests you didn't author
- Add documentation beyond the task scope
```

### 3c. No hermetic-test intuition
Workers don't distinguish between "capture real output as a golden" and "create
a hermetic fixture for a stable golden." They'll run a command against the live
project and byte-assert it, not realizing the output is volatile.

**Mitigation.** For any golden test, the prompt must explicitly state whether the
fixture should be hermetic or live, and justify the choice. Default: hermetic.

### 3d. Path-scope anchoring
Workers implement path filtering as literal string matching, not as anchored
path-segment matching. `contains("/research/")` is a natural interpretation of
"skip the research directory" but fails when the research directory appears as a
subpath.

**Mitigation.** Provide explicit patterns that anchor to root: "skip any path
whose components include `.dispatch/`, `.worktrees/`, or `target/`" rather than
"skip files containing 'research'".

## 4. Funnel verification gaps

The dispatch funnel's verify step only checks build + clippy + fmt on the
coordination tree. It does not:

| Gap | What it would have caught |
|-----|--------------------------|
| Re-run prior-phase goldens | F-2 (volatile memory golden drift) |
| Run the doctor against the live corpus | F-3 (827 false positives) |
| Gate on architecture_layering (diff-aware) | F-1 (registry→spec edge) |
| Validate JSON output shape against design spec | F-5 (wrong JSON shape) |
| Check for .doctrine/ or .claude/ in delta | F-8 (layering.toml touch — caught by R-5 belt in funnel, but only because I manually checked) |

## 5. Concrete recommendations

### 5a. Worker prompt template improvements

**Mandatory negative contract block** in every dispatch worker prompt:
```
## NEGATIVE CONTRACT — do NONE of these:
- `cargo fmt` (ONLY `cargo fmt --check`)
- Touch `.doctrine/` or `.claude/` files
- Modify files outside the declared file set
- Run or update tests you didn't write
- Use the live project corpus for golden tests (create hermetic fixtures)
- `git reset`, `git stash`, `git checkout -- <file>`, or `git clean`
```

**Explicit home module** for every new function:
```
Place in: src/spec.rs (NOT src/registry.rs — spec is the spec layer, 
registry depends on it per ADR-001 leaf ← engine ← command)
```

**Hermetic fixture directive** for goldens:
```
Fixture: create tests/fixtures/<name>/ with seeded TOML/MD files.
NEVER run against the live project root for a byte-exact golden.
```

**Path-scope patterns** anchored explicitly:
```
Skip paths whose COMPONENTS (not substrings) include:
  .dispatch/  .worktrees/  target/  node_modules/
```
Or better: anchor the scan to `.doctrine/**/*.md` and the source tree
`src/**/*.rs`, never a root-level `**/*.md` glob.

### 5b. Funnel hardening

1. **Pre-phase gate snapshot.** Before each phase, run the architecture layering
   gate and record the finding set. After the phase, the gate must have ZERO
   NEW findings. This turns "pre-existing red" from a blind spot into a
   diff-aware signal.

2. **Golden regression.** After each phase, re-run ALL prior-phase goldens.
   Any that break → isolate whether the break is from the current delta or
   from corpus drift. If corpus drift, flag the golden as non-hermetic.

3. **Doctor sanity.** After phases that add doctor checks, run `cargo run --
   doctor 2>&1 | wc -l` and compare to a committed baseline. A step-change
   (>2x the baseline) is a red flag for scope issues.

4. **Delta content check.** The R-5 belt already checks for `.doctrine/` and
   `.claude/` in the diff. Extend it to flag files NOT in the declared file set.
   (Currently R-5 is a manual check; automating it would catch worker scope
   creep.)

### 5c. Deepseek-specific prompt patterns

Deepseek responds well to:
- **Explicit negative constraints** (the "DO NOT" list works better than implied)
- **Concrete patterns** (regex, file paths, exact strings — not abstract rules)
- **Short prompts with high information density** (it drifts on long narrative
  prompts; a bullet-point structured task with clear boundaries works better)
- **"The ONLY git command you run is the final commit"** — any git verb beyond
  `status`, `add`, `commit` should be forbidden

It responds poorly to:
- **Implied architectural constraints** ("this is a leaf module" → it doesn't
  verify the dependency direction)
- **Ambiguous file placement** ("add a convenience wrapper" → picks the wrong
  home)
- **"Capture the current output"** without "into a hermetic fixture" (→ volatile
  golden)
- **Tasks described as narrative paragraphs** (→ misses boundary conditions)

### 5d. Specific doctrin/CLI improvements

1. **`doctrine dispatch arm-spawn` should include a "negative contract" field**
   in the worker prompt template that gets injected automatically.

2. **`doctrine check layering` should NEVER be pre-existing-red.** Make
   `architecture_layering_gate` a hardened, always-green gate that blocks CI. The
   pre-existing failure masked F-1.

3. **Golden test hygiene lint.** A `clippy`-like lint that flags golden tests
   referencing the live project root (detect `Command::new(env!("CARGO_BIN_EXE_
   doctrine")).args(["memory", "validate"])` without a fixture path override).

4. **Dispatch funnel verify should include a delta-aware gate diff.** Not just
   "run the gate and check exit code" but "run the gate, diff the findings
   against the pre-phase snapshot, require zero new findings."

5. **`doctrine doctor` should have a `--baseline N` flag** that fails if the
   finding count exceeds N, for use in CI/funnel sanity checks.

## 6. Summary: what made SL-168 → SL-169 harder than it should have been

The 6 phases were all correctly architected. The workers built working code.
What failed was **boundary enforcement**:

| Boundary | Failed | How |
|----------|--------|-----|
| Module layering | Yes | registry→spec edge, no automated enforcement |
| Golden hermeticity | Yes | Live corpus capture, volatile output |
| Filesystem scope | Yes | Nested worktrees scanned as corpus |
| Worker contract | Partially | `.doctrine/` touch caught; fmt noise tolerated |
| JSON contract | Yes | Shape didn't match design spec |
| Inter-phase regression | No check exists | Golden drift silent until audit |

The audit caught everything. But 3 blocker-class findings should have been caught
by the funnel. The gap is that the funnel verifies "does it compile and is it
clean?" but not "does it conform to the design's architectural constraints?"

The single highest-leverage fix: **make `architecture_layering_gate` always-green
and gate the funnel on it diff-aware.** Everything else follows from that
discipline — if the gate can't drift, workers can't silently violate it.
