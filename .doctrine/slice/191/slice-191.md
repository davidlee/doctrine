# Dispatch worker contract: cascade hymns + check-cadence + role/model/stage bake

Originates from **IMP-197** (SL-168 postmortem §3/§5a/§5c/§5d.1). Home RFC-005
(dispatch funnel integrity).

## Context

Deepseek-class dispatch workers fill underspecified prompts with
reasonable-but-wrong choices, underspecified on two axes — *where* (module home,
path matching) and *what-not-to-do* (unasked work). SL-168 shipped three
blocker-class defects from this: an ADR-001 layering violation (wrong module
home), a non-hermetic live-corpus golden, and substring (not component-anchored)
path matching. Workers also do unasked work — whole-tree `cargo fmt`, out-of-set
edits, `.doctrine/` touches, stray git verbs.

The delivery machinery to fix this already exists and must be **ridden, not
reinvented** (no parallel implementation):

- **Prompt cascade** (SL-186/187): `doctrine prompt resolve` composes per-context
  hymns from `install/hymns/` (Framework, shipped) + `.doctrine/hymns/` (User
  overlay). Bands: preamble/harness/model/role/**stage**/project.
- **Subagent-def bake** (`src/install.rs`): `WORKER_RESOLVE_MARKER =
  "{{ prompt resolve --role worker }}"` + `expand_worker_marker` inline the
  resolved worker hymns into the shipped `dispatch-worker.md` subagent def at
  install time. This is how static worker contract reaches a claude-arm worker —
  the claude arm is already wired; the marker just resolves `--role worker` only.
- **`doctrine check` cadence proxy** (SL-163): the owned, POL-002-clean
  formatter/lint seam. Resolves argv from the owned `[verification]` config
  (`quick`/`commit`/`gate`); help: *"never carries a host convention as
  correctness (POL-002)."* In this repo `check quick` runs `cargo fmt` via
  `just check`. Workers do not run it reliably today.

## Scope & Objectives

Harden the worker contract by authoring it into the cascade in **doctrine-owned,
host-agnostic terms** (POL-002), generalising the bake to deliver the full
context, and wiring the owned `doctrine check` cadence into both the worker
mandate and the funnel — so an unformatted / out-of-set delta cannot land.

1. **Worker-contract hymns** — `install/hymns/role/worker.md` (Framework,
   host-agnostic), grounded on the owned "constrained writer / declared file set"
   primitive:
   - NEGATIVE CONTRACT: touch only the declared file set (this *subsumes*
     whole-tree `cargo fmt` as an out-of-set edit); never write `.doctrine/` or
     `.claude/`; the only git verb is the final commit; do not run/update
     unauthored tests.
   - Hermetic-fixture directive for goldens (never byte-assert live-corpus output).
   - Component-anchored path matching (match path *components*, never substrings);
     owned dirs `.dispatch/`/`.worktrees/` named; host dirs (`target/`) deferred to
     the overlay.
   - State a module **home + layer/dependency rationale** for every new function
     (the *directive* is static; the per-fn specifics are dynamic — phase plan).
   - Run `doctrine check quick` per-edit (repeated, non-final) and
     `doctrine check commit` before the final commit — the owned lint/fmt seam,
     never a literal `cargo fmt`.

2. **Trait-keyed delivery patterns** — `install/hymns/model/adherence/low.md`
   (§5c): explicit negative constraints, concrete patterns over abstract rules,
   high-density bullets. The model band is a **trait classification space**
   (SPEC-023, delivered by SL-192), NOT a vendor-identity path — this content is
   *low-adherence* guidance and reaches any loose model by membership, never just
   `deepseek`. The worker def declares its trait set in a dedicated `traits:`
   frontmatter field (`traits = ["adherence/low"]`), distinct from the harness's
   `model:` identity field. Absent `traits:` → no model band. Only `adherence/low`
   is populated this slice; `capability/*` trees stay an unpopulated extension
   point (design D3).

3. **This-repo specifics** — `.doctrine/hymns/` overlay (User / `project` band),
   NOT the shipped corpus: `cargo fmt`/`target/`/`node_modules/`, ADR-001
   `leaf ← engine ← command` layer names, Rust module homes, the
   `architecture_layering` gate. POL-002 explicitly sanctions these as client
   habits in the overlay.

4. **Bake generalization** — `src/install.rs`: `resolve_worker_role_body` reads
   the def's `traits:` frontmatter → set-valued `ContextVector { role: Worker,
   model: BTreeSet<traits>, bands: Only([Role, Model]) }`, so the new trait-keyed
   model content lands in the baked def (**not** `--role worker` only). The marker
   stays a literal but becomes a *sentinel* = "resolve my full worker context." No
   stage band, no `--stage` threading (folded into role — design D2). Load-bearing:
   without it, (2)'s trait content is authored but never delivered to a claude-arm
   worker.

5. **Required-trait coverage lint** (SPEC-023 OQ-3, routed here from SL-192) — a
   shared pure predicate `traits_covered(declared, corpus)` in `src/hymns.rs`,
   called at **two sites**: the bake (`src/install.rs`) as an install-time hard
   error, and `prompt check` (`src/commands/prompt.rs`) as an author-time finding
   over embedded defs. Catches a typo'd trait → empty set → silent loss of the
   worker's delivery contract. Def→corpus direction; embedded defs only (design D4).
   `prompt check` additionally runs the **full-context resolver** per def and
   asserts the `Model` band is present when `traits:` is non-empty — proving the
   bake actually delivers traits, not merely that they exist in corpus (design C3).
   The bake reads `traits:` via a **new dedicated agent-def frontmatter parser**
   (none exists today — the bake does byte-level marker replace only).

6. **Funnel check-cadence wiring** — `/dispatch` orchestrator + import belt:
   - **base-clean precondition (non-mutating prove-clean):** *assert* the base is
     clean before `arm-spawn` (and `main` before branching) with a non-mutating
     check (`fmt --check` / dry gate), never a fix-in-place — a mutating check on
     the shared base has no owner and re-creates the spill. Pre-existing red is a
     base defect (distinct from worker findings); real cleanup is an operator-owned
     commit (design D5/C1). A clean base makes the worker's `check quick`
     delta-scoped (fixes the SL-168 root cause).
   - **reject-and-halt import gate:** run a `doctrine check` cadence on the
     post-import tree; a red (unformatted / lint-fail) delta **halts and reports**,
     never lands and never auto-fixes (belt, not self-report — design D5).

7. **Corpus knowledge refresh** — the hymn READMEs (`install/hymns/README.md`,
   `.doctrine/hymns/README.md`) are stale (model band as "model-family notes";
   exposed slots winning by provenance tiebreak) — rewrite to the SPEC-023
   trait-space + SL-193 self-`replaces` model. Plus a **shipped concept memory**
   (`memory/mem.concept.doctrine.hymn-cascade`) capturing the authoring model
   (trait-keyed band, bands registry, seal/expose, `replaces`, precedence), which
   installs to every doctrine project (design C4/C6).

## Non-Goals

- **Per-fn home-module + layer rationale content** — that is *per-phase*, lives in
  the phase plan / distilled `prompt:`, not a static hymn. Only the static
  *directive* ships here.
- **ISS-206 / exposed-slot self-replaces** — the same-slot Framework+User twin
  duplication and its projection fix are **SL-193** (at audit). Obj #3's overlay
  authoring + the stray `role/worker.md` twin reconciliation *ride* SL-193's close,
  they are not re-solved here.
- **Stage band** — folded into `role/worker`; no `install/hymns/stage/*`, no
  `--stage` threading. The hermetic-fixture + path directives are role conduct.
- **`capability/*` trait trees** — deferred extension point; only `adherence/low`
  populated (no invented capability content).
- **On-disk `.doctrine/agents/**` `traits:` linting** — the coverage lint checks
  embedded (framework-shipped) defs only; hand-edited on-disk def linting is a
  follow-up (parallels SL-193's deferred hand-authored-overlay gap).
- **Reverse dead-hymn lint (corpus→def)** — split to **IMP-242**. A `model/**`
  hymn no def can match is dead prose, but has no live trigger this slice (our one
  hymn is wired) and its only unique catch needs a second trait root that D3 defers.
- **Auto-fix import** — rejected; the import gate halts-and-reports, it never
  rewrites the worker delta (ADR-012 sole-writer; design D5).
- **Postmortem §5b/§5d.2–5 funnel/CI hardening** beyond the check-cadence wiring —
  `architecture_layering` always-green gate, delta-aware gate diff, golden
  regression harness, `doctor --baseline`. Different surface (verification/CI),
  not the worker-prompt axis. Note as follow-ups.
- **New formatter config key** — unnecessary; `doctrine check` (SL-163) already is
  the owned, POL-002-clean seam.
- **Runtime `prompt resolve` at spawn** for static content — static contract
  reaches the worker via the install-time subagent-def bake; the orchestrator
  supplies only per-phase specifics.

## Affected surface

- `install/hymns/role/worker.md`, `install/hymns/model/adherence/low.md`
  (Framework corpus). No `install/hymns/stage/*` (stage folded).
- `install/hymns/README.md`, `.doctrine/hymns/README.md` (rewrite to trait-space /
  self-`replaces` model — currently stale).
- `memory/mem.concept.doctrine.hymn-cascade` (new shipped concept memory).
- `.doctrine/hymns/**` (this-repo overlay: cargo/target/ADR-001 specifics; rides
  SL-193 close).
- `src/hymns.rs` (`traits_covered` pure predicate).
- `src/install.rs` (new agent-def frontmatter parser; `resolve_worker_role_body`
  gains a `traits` param + `Band::Model`; bake coverage hard error + call site).
- `src/commands/prompt.rs` (`prompt check`: coverage finding + full-context
  resolve/`Model`-band assertion over embedded defs).
- `install/agents/pi/dispatch-worker.md` (+ installed universal twin) — declare
  `traits = ["adherence/low"]`.
- `plugins/doctrine/skills/dispatch/**`, `plugins/doctrine/skills/dispatch-agent/**`
  (funnel check-cadence beats).
- Dispatch import belt in `src/worktree/import.rs` (reject-and-halt import-time
  check gate; corrected from `src/dispatch.rs` at plan time — codex F3).
- Rendered subagent defs (`.claude/`/`.pi/`/… `dispatch-worker.md`) — build/install
  output, verified not hand-edited.

## Risks / Assumptions / Open Questions

- **RESOLVED (design D2) — stage band.** Folded into `role/worker`; no stage band,
  no `--stage`. The bake resolves `Only([Role, Model])`, model band keyed off the
  def's `traits:` frontmatter.
- **RESOLVED (design D4) — required-trait lint (SPEC-023 OQ-3).** In scope as a
  dual-site coverage lint (bake hard error + `prompt check` finding) over a shared
  `traits_covered` predicate; def→corpus direction; a declared-but-uncovered trait
  is an **error** (silent-contract-loss), embedded defs only.
- **OQ — ownership cut per contract line.** Confirm the whole negative contract is
  expressible on the owned "declared file set / touch only what you're told"
  primitive, pushing all Rust specifics to the overlay. Believed mostly yes;
  finalised when `role/worker.md` content is authored.
- **Sequencing** — **SL-192 done** (trait engine shipped+closed; trait selectors
  unblocked). **SL-193 at audit** — obj #3 overlay + twin reconciliation gated on
  its close; Framework hymns, bake, lint, and funnel beats do not depend on it.
- **ASM** — POL-002: `install/hymns` ships only doctrine-owned universal contract;
  `cargo`/`target/`/ADR-001 specifics live in `.doctrine/hymns`. `doctrine check`
  is the sanctioned formatter seam (no host command in shipped code).
- **ASM** — behaviour-preservation gate: extending the bake marker must keep the
  existing `expand_worker_marker` unit tests + subagent-def drift test green.
- **RSK** — bake generalization touches the shared install render path; a hollow
  RustEmbed asset ships no compile error (crane strips non-rust). Any new
  `install/hymns/stage/*.md` rides the existing `install/` embed root — validate
  `just nix-build` (host).
- **RSK** — funnel base-clean beat adds a `doctrine check` run to the hot dispatch
  path; must not double-run or slow the funnel materially.

## Verification / Closure intent

- Enriched hymns resolve host-agnostic in `install/hymns`; `prompt resolve
  --role worker --model adherence/low` composes the role + trait contract.
- `resolve_worker_role_body` (widened) inlines role+traits into the baked
  `dispatch-worker.md`; existing `expand_worker_marker` unit tests + drift test green.
- Coverage lint: a def declaring an uncovered trait → bake hard error + `prompt
  check` finding (tests).
- `install/hymns` contains no host command (`cargo`/`target/`/`just`); a grep
  gate / review confirms POL-002.
- Funnel runs `doctrine check` at base-handoff and import; an unformatted imported
  delta halts-and-reports, never lands (test).
- Live/dry dispatch run: a worker receives the contract, runs `doctrine check
  quick`, lands a formatted in-set delta.
