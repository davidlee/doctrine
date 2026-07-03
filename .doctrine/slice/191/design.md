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
  - **Prove declared→delivered (adversarial C3).** Coverage alone ("the trait
    exists in corpus") does *not* prove the bake actually feeds traits — the
    existing marker check is role-only (`prompt.rs:321`), so a bake that parses
    `traits:` but drops them (forgets to widen `bands`) evades it. So `prompt
    check` must, per embedded worker def, run the **same full-context resolver the
    bake uses** (not the role-only helper) and assert: marker present, full-context
    resolve succeeds, and when `traits:` is non-empty the resolved context's band
    filter **includes `Model`**. This closes the "trait hymn authored but never
    baked" gap the coverage predicate alone leaves open.
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

- **Base-clean precondition — NON-MUTATING prove-clean (adversarial C1).** Before
  `arm-spawn` (and on `main` before branching), *assert* the base is clean with a
  **non-mutating** check (`fmt --check` / dry `gate` semantics), never a
  fix-in-place. Rationale: a mutating `doctrine check` on the shared base would
  reformat unrelated files, and that write has **no owner** — it re-creates the
  exact spill this gate exists to prevent, and blames the worker for ambient debt.
  So the base gate only *proves* cleanliness; if it fails (pre-existing red), that
  is reported **as a base defect, distinct from any worker finding**, and real
  cleanup is a separate **operator-owned** commit (or an isolated preparatory
  worktree), never folded silently into a worker's delta. A clean base is what
  makes the worker's `check quick` touch only its own delta and the post-import
  check delta-scoped (fmt-exact; for lint, "the delta caused any new red").
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
- **`install/hymns/README.md` + `.doctrine/hymns/README.md`** (adversarial C4/C6) —
  both stale: they describe the model band as "model-family / exact-model notes"
  and claim exposed slots win by the provenance tiebreak — contradicting SPEC-023's
  **trait-space** contract and SL-193's **self-`replaces`** override. Rewrite to the
  post-unlock authoring model so the shipped corpus stops teaching obsolete rules.
- **`memory/mem.concept.doctrine.hymn-cascade/`** — a **shipped** concept memory
  (embedded `memory/` corpus, `src/corpus.rs:44`; installs to every doctrine
  project) capturing the hymn/cascade authoring model: trait-keyed model band,
  bands registry, seal/expose symmetry, `replaces` suppression, precedence. The
  memory-tier peer of the README rewrite — supersedes the stale mental model in
  the corpus knowledge tier.

**Logic:**
- `src/hymns.rs` — `traits_covered` pure predicate (engine, DRY core).
- `src/install.rs` — **new agent-def frontmatter parser (adversarial C2).** The
  bake does **not** parse def frontmatter today — it reads embedded bytes + does a
  literal marker replace (`:1716`, `:900`); the only frontmatter parser is
  skill-specific (`:1252`). So this slice adds a dedicated agent-def frontmatter
  parser (schema: `traits` optional list, `model` cascade-ignored) with negative
  tests (malformed / absent `traits` / unknown keys). `resolve_worker_role_body`
  then gains a `traits: &BTreeSet<String>` param (populates `model`, adds
  `Band::Model`); the **call site** (`:1713`) parses `traits:` and passes it.
  SL-192 already made the context set-valued (`model: BTreeSet::new()`,
  `Only([Role])`), so the *resolver* change is populate-set + widen-band, **not** a
  struct migration — but the frontmatter parse is genuinely new surface. Coverage
  predicate → install-time hard error.
- `src/commands/prompt.rs` — `check_corpus`/`Check` enumerates `embedded_agent_defs()`,
  parses `traits:`, runs predicate → findings.
- `src/worktree/import.rs` — reject-and-halt post-import check gate in the import
  belt (`classify_import`/`run_import_from_worktree`; NOT `src/dispatch.rs` — codex
  F3 at plan time corrected this anchor).
- `plugins/doctrine/skills/dispatch/**`, `plugins/doctrine/skills/dispatch-agent/**`
  — base-clean `doctrine check` before arm-spawn/branch; funnel check-cadence beats.
- `src/commands/check.rs`, `src/verify.rs`, `justfile` — the non-mutating
  base-clean **`prove`** cadence (fmt-`--check` + lint, never auto-fix) built for
  D5's gate; `src/corpus.rs` — the embedded `memory/` root touch carrying the
  hymn-cascade concept memory.

> **Reconciled post-audit (RV-242 F-1, 2026-07-04).** The delivered P05/P06 surface
> above (`src/commands/check.rs`, `src/verify.rs`, `justfile`, `src/corpus.rs`, the
> `memory/mem.concept.doctrine.hymn-cascade` symlink + target) was under-declared in
> the original design-target selector set; the selector registry and this mirror
> were updated to match, and the stale `src/dispatch.rs` selector was removed (the
> import belt landed in `src/worktree/import.rs`, as noted above). `slice
> conformance 191` is now 0 undeclared / 0 undelivered.

**Def declaration:**
- `install/agents/pi/dispatch-worker.md` (+ installed universal twin) — add
  `traits = ["adherence/low"]`. Claude def: no `traits:` (role band only).

## Verification alignment

- **Verification verb (adversarial C7): prefer `prompt explain` + bake unit/e2e,
  not `prompt resolve`.** `prompt resolve` has a **side effect** — it regenerates
  `.doctrine/state/boot.md` before emitting (`prompt.rs:118`→`boot.rs:316`), so it
  depends on boot writeability + snapshot freshness and fails read-only. Slice
  verification uses `prompt explain --role worker …` (pure precedence trace) and
  pure bake unit/e2e tests; `prompt resolve` is used only in live/dry dispatch, not
  as a correctness oracle.
- **Resolve/bake:** `prompt explain --role worker --model adherence/low` traces the
  role+adherence composition; bake inlines both into the baked def. Behaviour-preservation:
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

## External adversarial pass (codex / GPT-5.5, 2026-07-03)

Confirmed the internal verdicts: F1 layering clean (checked `boot.rs:828`
orchestrator-only + `install.rs:900` role-only — no hidden cascade coupling), F2
correctly scoped, and the `traits:`↔`model:` separation genuinely clean today
(no second source feeds the cascade). New findings triaged:

- **C1 (major) — D5 base-clean ownership hole. ACCEPTED** → base-clean is now a
  **non-mutating prove-clean** gate; cleanup is operator-owned; pre-existing red
  reported as a base defect (D5, above).
- **C2 (major) — no agent-def frontmatter parser exists. ACCEPTED** → dedicated
  parser added to scope; the false "seam exists" claim corrected (Code impact).
- **C3 (major) — `prompt check` too weak to prove the bake uses traits. ACCEPTED**
  → `prompt check` runs the full-context resolver + asserts `Model` band when
  `traits:` non-empty (D4, above).
- **C4 (major) — dead-hymn reverse lint. DEFERRED → IMP-242.** No live trigger in
  this slice (our one hymn is wired); its only unique catch needs a second trait
  root / capability hymn that D3 defers; the author-layer drift it guards is
  already covered by C3 + the README rewrite + the shipped memory. Filed as an
  `improvement`, triggered when the trait corpus grows.
- **C5 (minor) — SL-192 dependency framing. ACCEPTED (= internal F6).** SL-192 is
  **done**, so the `after:` is satisfied; with one trait, SL-191 exercises
  cross-band *union*, not within-band intersection — the dependency is
  retrospectively soft. Kept as sequencing-satisfied, not reopened.
- **C6 (minor) — marker sentinel + stale READMEs. ACCEPTED** → README rewrite in
  scope (Content); marker rename deferred to plan (Open code-details).
- **C7 (minor) — `prompt resolve` is stateful. ACCEPTED** → verification prefers
  `prompt explain` + bake tests (Verification, above).

## Open code-details (resolve at plan time, non-blocking)

- **[RESOLVED by adversarial C2]** ~~Whether frontmatter parsing exists~~ — it does
  **not**; a dedicated agent-def frontmatter parser is now explicit scope (Code
  impact). The earlier claim that "defs already carry frontmatter the bake reads"
  was wrong (the bake does byte-level marker replace only).
- Whether the subprocess arm (`/dispatch-subprocess`, `src/dispatch.rs` fork path)
  needs the base-clean beat too, or the shared funnel already covers it.
- Marker rename (C6): rename `WORKER_RESOLVE_MARKER` to state the sentinel contract,
  or keep the literal with a loud comment at `install.rs:44`/`:918`. Blast radius:
  both shipped defs + the constant. Decide at plan.
