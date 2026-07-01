# Prompt cascade: per-context instruction resolver

Realises the model shaped in **IMP-155** (see that item for the full reasoning
and the rejected alternatives). This slice is the buildable NARROW cut.

## Context

Doctrine agents receive instructions from several sources — universal, harness,
model/family, stage/skill insertion points, and role (orchestrator vs worker) —
but only universal (boot snapshot) and, partially, harness (IMP-116) have a home.
Model, stage, and role guidance have nowhere to live, and **orchestrators burn
tokens hand-assembling worker context every spawn** — context fully determined by
(harness, model, role, arm, stage).

This lands squarely on **ADR-011**'s thesis: *mechanism in prose is the design
smell → move it into a CLI verb, identical across harnesses.* The hand-rolled
per-spawn context assembly is exactly such prose. The fix is a resolver verb.

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
- **Resolver** = `doctrine prompt resolve --context <orchestrator|worker>` → refreshes
  the universal on-disk `boot.md` (write-if-changed, context-invariant) **and** emits
  `universal ++ context hymns` to stdout. Two callers: harness @ session start
  (`--context orchestrator`), orchestrator @ spawn (`--context worker`, +model +arm
  +stage). `--context` names the assembly shape (first-class, not sugar).
- **Delivery = two channels by cache property** (per-harness altitude, ADR-011).
  *Cache-stable boot sector* (MODEL-AGNOSTIC): governance + universal hymns + **inlined
  onboarding memories** — rides the token cache. tier-1 claude/codex hook + pi
  `before_agent_start` extension (session-stable harness hymns); tier-2 `@`-import the
  universal `boot.md` (universal-only). *Cache-busting supplement* (MODEL-SPECIFIC): the
  MCP `doctrine_onboard` tool — model identification + model band, where dynamic content is
  free (a tool call busts cache anyway). Harness-/model-specific prose **never** touches the
  shared `boot.md`.
- **Live model band** = never in the cached sector (cache stability). Floor: a **universal**
  standing directive to self-identify + re-resolve `--band model`. Ceiling: `doctrine_onboard`
  (model-id + model band) — **in scope**.
- **Onboarding memories → cached sector.** boot inlines the bodies of memories carrying the
  `onboarding` tag (across **shipped + local** corpora), retiring the footer round-trip;
  `doctrine_onboard` sheds the memory load. Tag the shipped `overview`+`orientation` memories.

## Scope & Objectives

1. **Resolver core (pure).** Given a context vector + a snippet corpus, produce
   the ordered, composed markdown. Pure function — no disk/clock/env (per the
   pure/imperative split). Owns: selector matching, band ordering, intra-band
   specificity→alpha, `replaces` suppression.
2. **Corpus loader (shell).** Discover snippets under the agents tree; derive
   selector+band from path; overlay sidecar `.toml` where present.
3. **`doctrine prompt resolve` verb.** Thin shell over 1+2. Read-only, idempotent,
   stateless. Emits assembled markdown to stdout for a given context vector.
4. **Delivery.** `prompt resolve` unstales the universal disk `boot.md` (reusing boot's
   generator) and emits `universal ++ context hymns` to stdout; disk stays universal +
   model-agnostic (cache-stable), hymns for a harness ride stdout only. Wire tier-1
   delivery: claude/codex SessionStart hook + pi `before_agent_start` extension. Full
   boot-subsumption deferred (OQ-4).
5. **Model ceiling + onboarding via `doctrine_onboard`.** Extend the MCP onboard tool:
   model identification + emit the model band (cache-busting side); **drop** its memory load.
   Inline `onboarding`-tagged memory bodies into the cached boot sector (union shipped +
   local); tag the shipped `overview`+`orientation` memories; retire the footer round-trip.
6. **Seed corpus + convention docs.** Enough real snippets (universal/harness/
   model) to prove the world, plus the directory-layout + authoring convention.

## Non-Goals (the NARROW boundary)

- **Agent-definition composition / field-merge.** Definitions (`dispatch-worker.md`:
  name/tools/model) stay their own surface — a **static shell with one injection
  hole** the resolver fills. No per-field merge (union tools / override model).
  The selector engine is *designed* to be reusable there later; the merge is
  deferred, maybe never.
- **IMP-197's worker snippets** (negative-contract, home-module, hermetic,
  path-anchor). Those are *authored on top of this world* (IMP-197 now `after`
  IMP-155). Out of scope here beyond proving a `role=worker` snippet resolves.
- **Model self-identification transport** (how `--model` reaches the resolver —
  harness env vs agent self-declare). A resolver-CLI detail; may fold in at design
  or defer.

## Affected surface (coarse — `/design` tightens)

- `.doctrine/agents/**` — new `harness/`, `model/`, `role/`, `stage/` snippet trees
  + convention (exact root TBD at design: reuse `agents/` vs new `prompts/`).
- Boot pipeline — `src/boot.rs` generator reused (expose universal-snapshot fn; add
  universal-hymns disk section); no rewrite of entity-derived sections (SPEC-011).
- New command surface `doctrine prompt {resolve,model-keys,explain,check}` (+ its
  engine/leaf modules per ADR-001 layering).
- Per-harness delivery wiring — claude/codex SessionStart hook + the pi
  `before_agent_start` system-extension (extend the existing one).

## Risks / Assumptions / Open Questions

- **OQ-1 Altitude.** ~~Is the cascade model durable enough to warrant its own ADR
  or tech-spec?~~ **RESOLVED (user): a tech spec, via REV.** The selector +
  composition semantics are the durable "how" — but NOT front-authored. Shaped in
  this slice's `/design`, then promoted to a tech spec as a **REV** (ADR-013),
  settled at `/reconcile` (reconciliation is the sole writer of governance/spec
  truth). Which spec (new vs extend SPEC-011) + parent PRD decided at that point.
- **OQ-2 Corpus root.** Reuse `.doctrine/agents/` (already holds agent defs) or a
  dedicated `prompts/` root? §6/§Non-Goals keep defs separate — shared root risks
  conflating the two surfaces.
- **OQ-3 Resolver output & caching.** Context-vector → assembled-markdown cache
  keying (the token win depends on not recomputing). Design detail.
- **OQ-4 Boot-subsumption.** `prompt resolve --boot/--check` could replace `doctrine
  boot` entirely. Deferred follow-up; this slice reuses boot's generator, keeps the verb.
- **OQ-4 Specificity metric.** Precise definition of "specificity" for path- and
  toml-derived selectors (segment depth? axis count?) — needs a crisp, testable rule.
- **ASM-1** boot already has a composition/injection seam (IMP-116, IMP-159
  boot-footer) the resolver can ride rather than duplicate.

## Verification / closure intent

- Resolver core has table-driven golden tests: given corpus + context vector ⇒
  exact assembled output, covering band order, specificity, alpha tiebreak,
  non-match, `replaces`, missing-tier graceful degradation.
- `doctrine prompt resolve` end-to-end golden on a hermetic fixture corpus.
- Boot emits the composed bands; model band demonstrably *not* baked.
- Behaviour-preservation: existing boot/dispatch suites stay green.

## Follow-Ups

- IMP-197 — author worker snippets on this world (now downstream).
- Revisit agent-def composition once the selector engine has proven out.
