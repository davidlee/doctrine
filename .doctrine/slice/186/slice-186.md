# Prompt cascade: per-role instruction resolver

Realises the model shaped in **IMP-155** (see that item for the full reasoning
and the rejected alternatives). This slice is the **resolver engine + `prompt`
verbs** — inert, no caller. **Delivery** (wiring it into the session-start path,
boot, onboard, memory) is **SL-187**, split off on a blast-radius boundary: this
half is additive and provable on a hermetic fixture corpus; SL-187 mutates live
bootstrap surfaces. Contract-first — the two are dispatchable in parallel.

## Context

Doctrine agents receive instructions from several sources — universal, harness,
model/family, stage/skill insertion points, and role (orchestrator vs worker) —
but only universal (boot snapshot) and, partially, harness (IMP-116) have a home.
Model, stage, and role guidance have nowhere to live, and **orchestrators burn
tokens hand-assembling worker context every spawn** — context fully determined by
`(harness, model, role, arm, stage)`. (That token win is *realised* by SL-187;
this slice makes it *possible*.)

This lands squarely on **ADR-011**'s thesis: *mechanism in prose is the design
smell → move it into a CLI verb, identical across harnesses.* The fix is a
resolver verb.

The model (from IMP-155) is a **prompt cascade** — selectors + composition, not a
filesystem winner-takes-one lookup:

- **Snippet** = one `.md` (prose). Selector from the **path** by default
  (`harness/claude.md` ⇒ `{harness:claude}`); a **sidecar `.toml`** (idiomatic,
  no md front-matter) only when multi-axis / non-default band / `replaces`.
- **Slot** = `<band>/<label>`. **Band** = closed registry, fixes position:
  `preamble · harness · model · role · stage · project`. **Label** = free identity
  within the band. Open bands (`model`, `project`) take any label; locked bands
  validate against known hooks (stage = real skills/verbs).
- **Composition** = uniform append; every match concatenates. Precedence
  `band → specificity → provenance(fw<user) → alpha` (specificity leads with the
  band's own axis; provenance is the equal-specificity tiebreak — post-review flip,
  see design D1/D3). Seal = framework hard-win (resolution-enforced). `replaces`
  (unique-most-specific) is the only other suppressor.
- **Resolver** = `doctrine prompt resolve --role <orchestrator|worker>
  [--harness --model --arm --stage --band]` → assembled markdown to **stdout**
  (read-only). `--role` is the role-axis value AND selects the assembly shape (one
  concept — F15/A). Sibling verbs: `model-keys` (self-ID set), `explain` (precedence
  trace), `check` (corpus integrity).

## Scope & Objectives

1. **Resolver core (pure).** Given a context vector + a snippet corpus, produce
   the ordered, composed markdown. Pure function — no disk/clock/env (per the
   pure/imperative split). Owns: selector matching, band ordering, intra-band
   specificity→alpha, `replaces` suppression. Deterministic (design INV-7).
2. **Corpus loader (shell).** Walk embedded `install/hymns/**` ⊕ disk
   `.doctrine/hymns/**`; derive selector+band from path; overlay sidecar `.toml`;
   tag provenance by root; drop disk twins of sealed slots.
3. **`doctrine prompt` verbs.** Thin shell over 1+2: `resolve` (stdout), `model-keys`,
   `explain`, `check`. Read-only, stateless.
4. **Seed corpus + convention docs.** Enough real snippets (universal/harness/
   model) to prove the world, plus the directory-layout + authoring convention.

## Non-Goals (boundary)

- **Delivery** — session-start injection, boot integration, `doctrine_onboard`,
  onboarding-memory inlining, model-band floor/supplement, pi/hook wiring. That is
  **SL-187**; this slice locks only the `prompt resolve`/`model-keys` *contract* it
  consumes.
- **Agent-definition composition / field-merge.** Definitions (`dispatch-worker.md`:
  name/tools/model) stay their own surface — a **static shell with one injection
  hole** the resolver fills. No per-field merge. Deferred, maybe never.
- **IMP-197's worker snippets** (negative-contract, home-module, hermetic,
  path-anchor). *Authored on top of this world* (IMP-197 downstream). Out of scope
  beyond proving a `role=worker` snippet resolves.
- **Model self-identification transport** (how `--model` reaches the resolver). SL-187.

## Affected surface (coarse — `/design` tightens)

- `install/hymns/**` — NEW seed corpus (`harness/`, `model/`, `role/`, `stage/`
  trees) + convention doc. `install/agents/` stays (defs are a separate surface).
- New command surface `doctrine prompt {resolve,model-keys,explain,check}` — new
  `src/hymns.rs` engine + `src/commands/prompt.rs` (loader) per ADR-001 layering.
- `src/install.rs` / `install/manifest.toml` — seal/expose projection + embedded
  SealSet accessor; a hymns-specific embedded→disk projector (NOT `sync_corpus`).
- `src/main.rs` — wire the `prompt` command.

## Risks / Assumptions / Open Questions

- **OQ-1 Altitude.** ~~Its own ADR/tech-spec?~~ **RESOLVED (user): a tech spec, via
  REV.** The selector + composition semantics are the durable "how" — shaped in this
  slice's `/design`, promoted to a tech spec as a **REV** (ADR-013), settled at
  `/reconcile`. Spec (new vs extend SPEC-011) + parent PRD decided there.
- **OQ-2 Corpus name/root.** RESOLVED: `hymns` (const `HYMNS_ROOT`). Sub-point:
  `doctrine.toml` override vs const-only — leaning const-only.
- **OQ-3 Specificity metric.** Precise "specificity" for path/toml selectors —
  crisp testable rule (design D3: `(band-primary-axis depth, Σ other-axis depths)`).
- **OQ-4 Def↔hymn injection hole** in `dispatch-worker.md` — this slice or follow-up?
  Leaning follow-up (also unblocks IMP-197). Confirm at plan.

## Verification / closure intent

- Resolver core: table-driven goldens — corpus + context vector ⇒ exact assembled
  output, covering band order, specificity>provenance, band-primary-axis, alpha,
  non-match, `replaces` (unique-most-specific; overlap/cycle rejected), missing-tier
  degradation, seal disk-twin drop.
- Loader tests: path→slot/selector, sidecar per-axis supersede, provenance tagging,
  embedded⊕disk union, sealed-twin excluded.
- E2E golden: `doctrine prompt resolve …` over a hermetic fixture corpus.
- `model-keys` reflects only authored keys (full relative). `check`/`validate` flags
  overlapping/cyclic `replaces` + unknown stage labels.
- Layering gate `tests/architecture_layering.rs` green.

## Follow-Ups

- **SL-187** — delivery: boot integration, onboard, onboarding-memory inline,
  per-harness wiring (consumes this slice's contract).
- IMP-197 — author worker snippets on this world (downstream).
- Revisit agent-def composition once the selector engine has proven out.
