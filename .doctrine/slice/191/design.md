# SL-191 Design — Dispatch worker contract: cascade hymns + check-cadence + role/model/stage bake

> **STATUS: LOCKED (design stage).** D1–D5 closed and approved. Targets the
> post-unlock world: **SL-192 (trait-set engine) DONE**, **SL-193 (exposed-slot
> self-replaces) at audit** (obj #3 overlay + twin reconciliation gated on its
> close). Durable decision log + code anchors: `notes.md`. Scope: `slice-191.md`.

## Problem

Deepseek-class dispatch workers make reasonable-but-wrong choices on two axes —
*where* (module home, path matching) and *what-not-to-do* (unasked work). SL-168
shipped three blocker-class defects from this (ADR-001 layering violation,
non-hermetic live-corpus golden, substring path matching), plus unasked work
(whole-tree `cargo fmt`, out-of-set edits, `.doctrine/` touches). The fix is a
hardened worker contract, delivered through machinery that already exists — no
parallel implementation.

## Approach (settled frame)

Ride existing seams:

- **Cascade** (SL-186 resolve + SL-187 delivery, **SL-192** trait-set engine):
  author the contract as **hymns** in `install/hymns` (Framework, shipped) with
  this-repo specifics in `.doctrine/hymns` (overlay, safe post-SL-193).
- **Subagent-def bake** (`src/install.rs`): the claude arm is already wired — the
  install-time `expand_worker_marker` inlines resolved worker hymns into each
  shipped `dispatch-worker.md`. Generalize the resolve from **role-only** to
  **role + declared traits**.
- **`doctrine check` cadence** (SL-163): the owned, POL-002-clean formatter/lint
  seam. Worker contract and funnel invoke `doctrine check`, never a literal
  `cargo fmt`.

## The unlock (why this design differs from the pre-lock draft)

**SL-192 delivered** a set-valued model axis (`ContextVector.model` /
`Selector.model` are `BTreeSet<String>`; repeatable `--model`; conjunctive
selectors; root-wise specificity) but **explicitly fenced out** "how an agent
comes to carry trait keys (def frontmatter, spawner, env heuristics)." That
def-frontmatter → context bridge is **this slice** (the bake). The model band is
now a *trait classification space*, not a vendor-identity path — the honest
SPEC-023 shape, and the one SL-192's engine was built to enable.

**SL-193** (at audit) makes the `.doctrine/hymns` overlay a genuine suppressing
override (self-replaces kills the ISS-206 doubling); it also reconciles the stray
hand-authored `role/worker.md` twin. So obj #3 overlay authoring waits on SL-193
close.

## Locked decisions

### D1 — Model band arm-asymmetric by data, never a baked rule (Q1)

Hymns stay **model/trait-keyed**. The band is selected by what a def actually
declares, NOT by an arm→model map. No "claude=strong / subprocess=deepseek" rule
appears in hymn content or bake logic. A def with **no declared traits resolves no
model band**; a claude worker run under pi declares pi's traits and gets the right
band. Coupling lives in **data** (the def's declaration), not logic. The negative
contract still reaches every arm via the model-agnostic **role** band.

### D2 — Trait-keyed model band, declared via `traits:` frontmatter (Q2, post-unlock)

The model band is a **trait classification space** (SPEC-023):

- **Hymns are trait-keyed:** `install/hymns/model/adherence/low.md` (the §5c
  delivery patterns, correctly homed as low-adherence guidance) — **not**
  `model/deepseek/_default.md`. Patterns reach *any* low-adherence model by
  membership, never just the `deepseek` vendor.
- **The def declares a trait SET in a dedicated `traits:` frontmatter field** —
  `traits = ["adherence/low"]`. Distinct from the harness's `model:` field (which
  pi consumes to *select* the actual model — identity, untouched). Identity and
  classification stay separate. (Overloading `model:` — Option C — rejected:
  re-fuses the two and collides with harness selection.)
- **Absent `traits:` → empty set → no model band** (mirrors D1). A `traits`-less
  def (today's claude worker) resolves role band only.
- **Bake reads `traits:`:** `resolve_worker_role_body` widens to
  `ContextVector { role: Worker, model: BTreeSet<traits>, bands: Only([Role, Model]) }`;
  membership fires all declared trait guidance in one resolve. The marker
  `{{ prompt resolve --role worker }}` stays a literal but becomes a **sentinel**
  = "resolve my full worker context (role + declared traits)"; the bake reads
  frontmatter, not marker args. (Parametric marker carrying `--model` rejected:
  two sources → drift.)

**No stage band** (unchanged by the unlock): hermetic-fixture +
component-anchored-path directives are worker-**role** conduct, not phase-**stage**
conduct → they live in `role/worker`. `bands: Only([Role, Model])`, never Stage.
No `--stage` threading; no `install/hymns/stage/*`.

### D3 — Trait population: `adherence/low` only (this slice)

Populate the one trait tree that has real content. The §5c delivery patterns are
squarely low-adherence guidance (a loose model needs them regardless of code
capability). Universal/pi def declares `traits = ["adherence/low"]`.
`capability/*` trees stay unpopulated — a demonstrated extension point, authored
when capability-differentiated content genuinely exists (YAGNI: no invented
`capability/code/high.md`). The set-engine's composition is still exercised
(adherence-low ∪ role-worker); the two-trait *intersection* is SL-192's proven
capability, not something SL-191 must manufacture a second trait to justify.

### D4 — Required-trait coverage lint: dual-site over a shared predicate (SPEC-023 OQ-3)

A typo'd trait (`adherance/low`) hits absent-key semantics → empty set → **no
model band → the worker silently loses its delivery contract** — exactly the
silent-wrong harm this slice fights. Guard it:

- **Core (engine, `src/hymns.rs`, DRY):** `traits_covered(declared: &BTreeSet<String>,
  corpus) -> Vec<String>` — pure function returning declared trait keys that match
  **no** model-band selector in the corpus. One implementation.
- **Bake-time (`src/install.rs`):** `resolve_worker_role_body` runs it on the
  def's `traits:` → **install-time hard error**. The backstop — a contractless
  worker cannot ship.
- **`prompt check` (`src/commands/prompt.rs`):** enumerate embedded defs via
  `embedded_agent_defs()`, parse each `traits:`, run the predicate → **author-time
  finding** into `check_corpus`'s `Vec<String>`. Catches the typo at edit, before
  install. Rides the existing `check_def_marker_present_role_unresolvable`
  precedent (`prompt check` is already def-aware).
- **Direction:** def→corpus only (declared-but-uncovered = silent-contract-loss).
  The reverse (dead hymn: a selector pinning a trait no def declares) stays out —
  inert, not harmful.
- **Boundary:** checks **embedded** (framework-shipped) defs. On-disk
  `.doctrine/agents/**` hand-edited `traits:` linting → follow-up (parallels
  SL-193's deferred hand-authored-overlay gap).

Layering (ADR-001): the predicate is pure over (traits, corpus) → engine; both
bake and `prompt check` call down. No cycle. Resolves SPEC-023 OQ-3 at *both* the
`prompt check` home it named and the bake backstop.

### D5 — Import reliability gate: reject-and-halt + base-clean precondition (Q3)

Worker instruction alone is unreliable (a worker forgets its `check quick`). The
funnel needs a belt:

- **Base-clean precondition:** run `doctrine check` (commit/gate cadence) on the
  **base** before `arm-spawn` (and on `main` before branching). A formatter-clean
  base is what makes the worker's `check quick` touch *only its own delta* (fixes
  the SL-168 root cause: fmt on an unformatted base spilled outside the delta),
  and makes the post-import check delta-scoped (given a clean base, only the
  worker's files can be dirty).
- **Reject-and-halt import gate:** on import, run `doctrine check <cadence>` on
  the post-import tree; red (unformatted **or** lint-fail) → **halt, report to
  orchestrator, don't land.** The orchestrator (sole writer) decides —
  re-dispatch, fix, escalate.

Auto-fix rejected: (1) partial — only repairs formatting; lint-logic reds (clippy,
the ADR-001 layering violation that was the SL-168 blocker class) can't be
auto-fixed and must halt anyway; (2) hides the compliance signal the belt exists
to surface (and corrupts RFC-011's worker-compliance instrumentation); (3)
inconsistent with the funnel's "report-and-halt, never auto-merge" ethos and
ADR-012 sole-writer (orchestrator lands-or-rejects, never rewrites the delta).
With the base-clean precondition, reject fires only on genuine contract breach —
which is what we want surfaced.

## Content ownership cut (POL-002)

| Content | Home |
|---|---|
| Touch only declared file set (subsumes whole-tree `cargo fmt` as out-of-set) | `install/hymns/role/worker.md` |
| No `.doctrine/`/`.claude/` writes; only git verb = final commit; no unauthored-test edits | `install/hymns/role/worker.md` |
| Hermetic goldens (never byte-assert live-corpus output) | `install/hymns/role/worker.md` |
| Component-anchored paths (principle) + owned dirs `.dispatch/`/`.worktrees/` | `install/hymns/role/worker.md` |
| State module home + layer/dependency rationale per new fn (DIRECTIVE only) | `install/hymns/role/worker.md` |
| Run `doctrine check quick` per-edit / `check commit` pre-commit | `install/hymns/role/worker.md` |
| §5c delivery patterns (explicit negatives, concrete-over-abstract, high-density) | `install/hymns/model/adherence/low.md` |
| `cargo fmt`, `target/`, `node_modules/`, ADR-001 layer names, Rust module homes, `architecture_layering` gate | `.doctrine/hymns/**` overlay (client habits) |

The rule that dissolves most host specifics: express the contract on the owned
"constrained writer / declared file set" primitive — "no `cargo fmt`" is a special
case of "no out-of-set edit," so it ships host-agnostic and stronger.

## Code impact (design-target touch-set)

**Content:**
- `install/hymns/role/worker.md` — full negative contract (table above).
- `install/hymns/model/adherence/low.md` — §5c delivery patterns.
- `.doctrine/hymns/**` — this-repo overlay (cargo/`target/`/ADR-001-layer/Rust-home);
  authored/reconciled post-SL-193.

**Logic:**
- `src/hymns.rs` — `traits_covered` pure predicate (engine, DRY core).
- `src/install.rs` — `resolve_worker_role_body` gains a `traits: &BTreeSet<String>`
  param (populates `model`, adds `Band::Model`); the **call site** (`:1713`) parses
  the def's `traits:` frontmatter and passes it. SL-192 already made the context
  set-valued (`model: BTreeSet::new()`, `Only([Role])`), so this is populate-set +
  widen-band, **not** a struct migration. Coverage predicate → install-time hard error.
- `src/commands/prompt.rs` — `check_corpus`/`Check` enumerates `embedded_agent_defs()`,
  parses `traits:`, runs predicate → findings.
- `src/dispatch.rs` — reject-and-halt post-import check gate in the import belt.
- `plugins/doctrine/skills/dispatch/**`, `plugins/doctrine/skills/dispatch-agent/**`
  — base-clean `doctrine check` before arm-spawn/branch; funnel check-cadence beats.

**Def declaration:**
- `install/agents/pi/dispatch-worker.md` (+ installed universal twin) — add
  `traits = ["adherence/low"]`. Claude def: no `traits:` (role band only).

## Verification alignment

- **Resolve/bake:** `prompt resolve --role worker --model adherence/low` composes
  role+adherence; bake inlines both into the baked def. Behaviour-preservation:
  `expand_worker_marker` unit tests + subagent-def drift test green (drift pins
  `name:` — unaffected; `resolve_worker_role_body` tests migrate to set-valued
  context, outcomes preserved for the role-only / trait-less case).
- **Lint:** def declaring an uncovered trait → bake hard-error test; `prompt check`
  flags the same (new test, rides `check_def_marker_present_role_unresolvable`).
- **Content gate:** host-agnostic grep over `install/hymns` — no `cargo`/`target/`/
  `just` literals (POL-002).
- **Funnel:** base-clean check before handoff; unformatted imported delta **halts**
  (not auto-fixed) — test; lint-red delta halts.
- **E2E dispatch:** compliant worker → contract received, `check quick` run,
  formatted in-set delta lands; non-compliant (unformatted) delta halts at import.

## Invariants & boundary conditions

- **Absent `traits:` → empty set → no model band** (D2, mirrors D1). A trait-less
  def resolves role band only, byte-identical to today's role-only resolve.
- **`traits_covered` is pure** over (traits, corpus) — no disk/clock/env
  (pure/imperative split); it is why the same predicate serves bake and check.
- **Reject-and-halt keeps ADR-012 sole-writer intact** — the orchestrator lands or
  rejects, never rewrites the worker delta.
- **Behaviour-preservation gate** — existing resolver + `expand_worker_marker`
  *outcomes* preserved for the trait-less / singleton case; struct-literal churn
  from the set-valued signature is mechanical, reviewed by intent.
- **crane embed-strip** — the new `install/hymns/model/adherence/low.md` rides the
  existing `install/` RustEmbed root (already grafted); validate `just nix-build`
  (host).

## Sequencing (hard dependencies)

- **SL-192 DONE** — the set engine is shipped and closed; SL-191's trait selectors
  ride it. Unblocked.
- **SL-193 at audit** — obj #3 `.doctrine/hymns/**` overlay authoring + the stray
  `role/worker.md` twin reconciliation wait on SL-193 **close** (the self-replaces
  projection must land first, or the overlay doubles).
- **⚠ Verification contamination (adversarial F2).** The stray *untracked*
  `.doctrine/hymns/role/worker.md` already on disk makes `prompt resolve --role
  worker` emit **twice** today (ISS-206). SL-191 enriches the Framework
  `install/hymns/role/worker.md` and verifies it via that very resolve — so SL-191's
  role-band resolve verification is **contaminated until SL-193 reconciles the
  twin**. Therefore the dependency is stronger than "obj #3 only": SL-191's
  *execution* (any phase that asserts `prompt resolve --role worker` output) waits
  on SL-193 close, OR removes the stray twin as an explicit precondition. Only the
  pure-unit work (the `traits_covered` predicate, `model/adherence/low.md` content,
  bake signature) is genuinely SL-193-independent. Phase ordering must sequence the
  role-resolve verification behind SL-193.
- `after: SL-192, SL-193` recorded.

## Adversarial review (internal pass, 2026-07-03)

- **F1 — layering cycle on `prompt check`→`embedded_agent_defs`. CLEARED.**
  Checked: `commands/prompt.rs` already imports `crate::install::` (one-directional);
  `install.rs` does not import `commands::prompt`. No ADR-001 cycle. `embedded_agent_defs`
  rides the existing edge.
- **F2 — role-resolve verification contaminated by the stray un-reconciled twin
  until SL-193 closes. STANDS, integrated** into Sequencing (execution, not just
  obj #3, waits on SL-193 close / stray-twin removal).
- **F3 — churn overstated. FIXED.** SL-192 already set-valued the context; the bake
  change is a `traits` param + populate-set + widen-band, recorded in Code impact.
- **F4 — marker sentinel smell (minor, plan-time).** The literal
  `{{ prompt resolve --role worker }}` reads as role-only but now resolves role +
  traits. Decide at plan whether to rename `WORKER_RESOLVE_MARKER` (blast radius:
  both shipped defs + constant) or keep it with a code comment. Non-blocking.
- **F5 — precision (minor).** (a) The base-clean invariant "only the worker's files
  can be dirty" is exact for **formatting**; for **lint** it means "the delta caused
  any new red" (whole-crate clippy) — same responsibility, looser phrasing. (b)
  `traits_covered` must decide "covered" via the **engine membership matcher**
  (a declared key fires ≥1 model snippet), not string equality — so `adherence/_default`
  covers a declared `adherence/low`. Reuses `hymns.rs` match semantics (DRY).
- **F6 — watch-item (not a defect).** With only `adherence/low` populated, SL-191
  exercises cross-band *union* (adherence ∪ role), not SL-192's headline within-band
  *intersection* targeting — which ships with no live consumer until `capability/*`
  populates. Acceptable (YAGNI); noted so the gap is visible.

## Open code-details (resolve at plan time, non-blocking)

- Whether `resolve_worker_role_body` has existing YAML-frontmatter parsing to read
  `traits:` or needs a small parse helper (defs already carry `model:`/`name:`
  frontmatter the bake reads — likely a seam exists).
- Whether the subprocess arm (`/dispatch-subprocess`, `src/dispatch.rs` fork path)
  needs the base-clean beat too, or the shared funnel already covers it.
