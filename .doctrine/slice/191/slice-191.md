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

2. **Deepseek delivery patterns** — `install/hymns/model/deepseek/_default.md`
   (§5c): explicit negative constraints, concrete patterns over abstract rules,
   high-density bullets. Model band is legitimately universal (the model is the
   model regardless of client).

3. **This-repo specifics** — `.doctrine/hymns/` overlay (User / `project` band),
   NOT the shipped corpus: `cargo fmt`/`target/`/`node_modules/`, ADR-001
   `leaf ← engine ← command` layer names, Rust module homes, the
   `architecture_layering` gate. POL-002 explicitly sanctions these as client
   habits in the overlay.

4. **Bake-marker generalization** — `src/install.rs`: resolve the worker
   subagent-def marker for **role + model + stage**, not `--role worker` only, so
   the new model/stage hymn content actually lands in the baked def. Load-bearing:
   without it, (1)'s path/hermetic stage content and (2)'s deepseek content are
   authored but never delivered to a claude-arm worker.

5. **Funnel check-cadence wiring** — `/dispatch` orchestrator + import belt:
   - **base-clean precondition:** run `doctrine check` (commit/gate cadence) on
     the base before `arm-spawn` (and on `main` before branching), so a worker's
     `doctrine check quick` only ever touches its own delta — the formatter-clean
     base that makes the worker fmt mandate safe (fixes the SL-168 root cause:
     `cargo fmt` on an *unformatted* base spills outside the delta).
   - **import reliability gate:** run a `doctrine check` cadence on the imported
     delta so an unformatted / lint-red delta cannot land (belt, not self-report).

## Non-Goals

- **Per-fn home-module + layer rationale content** — that is *per-phase*, lives in
  the phase plan / distilled `prompt:`, not a static hymn. Only the static
  *directive* ships here.
- **ISS-206** — prompt-cascade same-slot Framework+User twin duplication; split
  off, filed, its own resolver-semantics fix.
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

- `install/hymns/role/worker.md`, `install/hymns/model/deepseek/_default.md`,
  new `install/hymns/stage/<execute|dispatch>.md` (Framework corpus).
- `.doctrine/hymns/**` (this-repo overlay: cargo/target/ADR-001 specifics).
- `src/install.rs` (`WORKER_RESOLVE_MARKER` / `expand_worker_marker` + the render
  call site).
- `plugins/doctrine/skills/dispatch/**`, `plugins/doctrine/skills/dispatch-agent/**`
  (funnel check-cadence beats).
- Dispatch import belt in `src/dispatch.rs` (import-time check gate).
- Rendered subagent defs (`.claude/`/`.pi/`/… `dispatch-worker.md`) — build/install
  output, verified not hand-edited.

## Risks / Assumptions / Open Questions

- **OQ — stage band label & threading.** `stage/execute` vs `stage/dispatch`? Does
  the bake resolve a stage at all, and does it thread `--model` for a fixed worker
  model, or stay model-agnostic (model band pulled at the worker's own
  session-start via SL-187's seam)? Central `/design` fork.
- **OQ — ownership cut per contract line.** Confirm the whole negative contract is
  expressible on the owned "declared file set / touch only what you're told"
  primitive, pushing all Rust specifics to the overlay. Believed mostly yes.
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
  --role worker --model deepseek/... --stage ...` emits the full contract.
- `expand_worker_marker` (generalised) inlines role+model+stage into the baked
  `dispatch-worker.md`; existing bake unit tests + drift test green.
- `install/hymns` contains no host command (`cargo`/`target/`/`just`); a grep
  gate / review confirms POL-002.
- Funnel runs `doctrine check` at base-handoff and import; an unformatted delta is
  rejected at import (test).
- Live/dry dispatch run: a worker receives the contract, runs `doctrine check
  quick`, lands a formatted in-set delta.
