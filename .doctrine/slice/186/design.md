# Design SL-186: Prompt cascade: per-context instruction resolver

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Doctrine agents need instructions from several sources — universal, harness,
model/family, stage (skill/verb insertion points), and role (orchestrator vs
worker) — but only universal (boot snapshot) and, partially, harness (IMP-116)
have a home. Model, stage, and role guidance have nowhere to live. Worse,
**orchestrators burn tokens hand-assembling worker context every spawn** — context
fully determined by `(harness, model, role, arm, stage)`.

Directly on **ADR-011**'s thesis: *mechanism in prose is the design smell → move
it into a CLI verb, identical across harnesses.* Hand-rolled per-spawn assembly is
exactly that prose. The fix is a resolver verb.

This slice builds the NARROW cut of the model shaped in **IMP-155**: a **prompt
cascade** — selectors + composition, not a filesystem winner-takes-one lookup.

## 2. Current State

- **`src/boot.rs`** — `doctrine boot` regenerates `.doctrine/state/boot.md`, a
  *pure projection* of governance (routing, process, guardrails, active
  ADRs/policies/standards, memory signposts) with a content-diff cache key. It
  already owns a `struct Section` assembly seam, an `enum Harness` + `match` seam
  (`parse_harness`/`resolve_harnesses`), and per-harness `@`-import wiring
  (`CLAUDE.md` vs `AGENTS.md`). Governance sections are **entity-derived**, not
  authored markdown files.
- **`.doctrine/agents/`** — holds *agent definitions* only:
  `{universal,claude,pi,codex}/dispatch-worker.md`. Shipped from `install/agents/`.
- **`src/install.rs`** — `rust_embed` `Assets` over `install/`; `install/manifest.toml`
  (schema = `struct Manifest`) drives `build_plan`; `corpus::sync_corpus(root,
  embedded_assets(), …)` already projects embedded → disk (the layered mechanism).
- **`src/globmatch.rs`** — path/glob matching leaf. **`src/commands/`** — command
  layer (ADR-001: command ← engine ← leaf).

No home exists for model/stage/role instruction; no resolver; no snippet corpus.

## 3. Forces & Constraints

- **ADR-011** — mechanism in a CLI verb, harness-identical; per-harness capability
  altitude (floor/ceiling). **ADR-001** — layering: command ← engine ← leaf, no
  cycles (`tests/architecture_layering.rs` gate).
- **Pure/imperative split** (slices-spec) — no disk/clock/env in the pure engine;
  the corpus loader is the thin impure shell.
- **Behaviour-preservation gate** — existing boot/dispatch suites must stay green
  unchanged; boot's entity-derived projection is not disturbed.
- **POL-002** — platform independence from host-project conventions.
- **No parallel implementation** (CLAUDE.md ethos) — ride existing seams
  (`globmatch`, `corpus::sync_corpus`, `install/manifest.toml`, boot `Section`).
- **High-churn hazard** — model→spawn-param identity (OpenRouter-class lists)
  changes weekly; the design must keep that churn *out*.

## 4. Guiding Principles

- **P1 — CSS, not filesystem lookup.** Snippets declare *where they go* (band) and
  *when they apply* (selector); one resolver composes. A snippet lives **once**,
  is matched — never copied per combination. Zero repetition.
- **P2 — Uniform composition.** Every matching snippet concatenates; ordering never
  silently suppresses (except opt-in `replaces`).
- **P3 — Provenance is a layer, not a merge.** Framework and user are two layers of
  the same cascade; user is the outer (last-word) layer. Framework-must-win is
  enforced by **non-exposure** (seal), not precedence.
- **P4 — The corpus is the vocabulary.** No enumerated model/registry. The `model/**`
  tree is sparse, user-authored, self-pruning; unknown models degrade gracefully.
- **P5 — Harness is code, model is data.** A harness is a behavioural arm (enum,
  bespoke wiring); a model is a classification key (a path). Neither is a churny list.
- **P6 — Classify, never map.** The resolver selects guidance by model
  classification and carries enough metadata for downstream domains; it never maps
  a model id to a harness spawn parameter.

## 5. Proposed Design

### 5.1 System Model

A **snippet** is one `.md` (prose). Its **slot** is `<band>/<label>`; its
**selector** is a set of axis→pattern constraints. One **resolver** takes a
**context vector** and emits ordered, composed markdown.

```
                 ┌──────────── context vector ("the element") ─────────┐
                 │ role · harness · model · arm · stage  (+ project)    │
                 └──────────────────────────────────────────────────────┘
                                      │  resolve(context, corpus)
   corpus = embedded(framework) ⊕ disk(user)   ▼
   match each snippet's selector → order by PRECEDENCE KEY → concat → assembled md

   PRECEDENCE KEY:  band  →  provenance(framework<user)  →  specificity  →  alpha
```

**Bands** (closed registry, fixed order — position, not identity):

```
preamble · harness · model · role · stage · project
```

`role` and `stage` are selector-axes that also *name* a band. Open bands (`model`,
`project`) accept any label; locked bands validate labels against a known set
(stage labels = real doctrine skills/verbs). The **band is the first path segment**
under the corpus root; the **label** is the remaining path — free identity within
the band, and naming it can never move the band (position is fixed).

**Two roots, layered (D1):**

```
install/scriptures/     compile-embedded (rust_embed), framework-authored, the SUPERSET
      │  install-time projection via corpus::sync_corpus — the SEAL FILTER
      ▼
.doctrine/scriptures/   user-customisable, read at runtime; holds only exposed + user snippets
```

The resolver **unions** embedded-framework ⊕ on-disk-user at read time. Provenance
is derived from *which root a snippet came from* — not a flag. `SEALED` framework
content is simply never projected (stays embedded-only) → no user file can exist to
override it → it wins by absence. `EXPOSED` framework content is projected as an
editable starter; a user edit at the same path wins (provenance dominates
specificity). This is doctrine's existing shipped-corpus ⊕ project-overlay pattern.

> **Name.** `scriptures` is provisional (a dir name + one const); rename is cheap
> and late. `canon`/`corpus` are taken (skill / `src/corpus.rs`).

### 5.2 Interfaces & Contracts

**Command (ADR-001 command layer) — `src/commands/prompt.rs`:**

```
doctrine prompt resolve --role <orchestrator|worker>
                        [--harness <name>] [--model <id>] [--arm <subagent|subprocess>]
                        [--stage <skill/verb>] [--band <name>]...
    → assembled markdown to stdout.  Read-only, idempotent, stateless.
    · --band repeatable; absent = all bands applicable to the role's assembly shape.
    · role selects the assembly shape (which bands, base).

doctrine prompt model-keys --harness <name>
    → the model-band leaf keys that EXIST in the corpus for that harness/vendor.
    · The "named set to choose from" for agent self-identification (§5.4).
    · Reflects authored guidance only — NOT a registry. Empty ⇒ don't ask.
```

**Engine (pure) — `src/scriptures.rs`:**

```rust
struct ContextVector { role: Role, harness: Option<Harness>, model: Option<ModelKey>,
                       arm: Option<Arm>, stage: Option<StageKey>, bands: BandFilter }
struct Snippet { slot: Slot, selector: Selector, provenance: Provenance, body: String }
struct Slot { band: Band, label: String }
// Selector: axis→pattern map (path-derived, sidecar-superseded); `replaces: Option<Slot>`.

fn resolve(ctx: &ContextVector, corpus: &[Snippet]) -> String   // pure: match→order→concat
fn matches(sel: &Selector, ctx: &ContextVector) -> bool
fn specificity(sel: &Selector) -> u32   // Σ concrete pinned segment-depths (D3)
```

**Loader (impure shell, command edge):** walk embedded `install/scriptures/**` +
disk `.doctrine/scriptures/**`; derive `Slot`+`Selector` from path; overlay sidecar
`<file>.toml` (supersede per-axis, carries `replaces`); tag `provenance` by source
root. Reuses `corpus::embedded_assets()`, `fsutil`, `globmatch`, `dtoml`.

**Boot integration (`src/boot.rs`):** boot calls `resolve(role=orchestrator, harness=<wired>, bands=all-except-model)` and **appends** the result as a snapshot
tail section. The **model band is omitted** (not baked). Boot's entity-derived
sections are untouched (behaviour-preservation).

### 5.3 Data, State & Ownership

| Surface | Owner | Consumer | When |
|---|---|---|---|
| `install/scriptures/**` (+ sidecar `.toml`) | framework (committed) | resolver (embedded) | compile |
| `.doctrine/scriptures/**` | user (+ projected starters) | resolver (disk) | runtime |
| `install/manifest.toml` seal/expose section | framework | installer (`sync_corpus`) | install |
| assembled markdown | resolver | agent / boot.md tail | on demand / boot |

- **Provenance** = source root (embedded vs disk). Not stored, derived.
- **Seal** = manifest projection rule; resolver never sees it (it just reads
  whatever exists in each root).
- Path → default `Slot`+`Selector`; sidecar `.toml` supersedes **per-axis**
  (declared axes win; undeclared fall back to path). No folder-level axis manifests.

### 5.4 Lifecycle, Operations & Dynamics

**Baked bands (boot):** every band except `model` bakes into `boot.md`, riding
boot's content-diff cache key. Regenerated by `doctrine boot`; the standard
freshen-then-clear ritual applies.

**Live model band (D5) — capability altitude:**
- **Floor (in scope, works everywhere incl. Claude `/model`):** the baked
  harness/preamble band carries a **standing directive** — *"your model guidance is
  not baked; identify your model (`doctrine prompt model-keys` offers the set) and
  run `doctrine prompt resolve --band model --model <id>`; re-resolve on change."*
  Agent-driven, always in context, degrades gracefully (unknown model ⇒
  universal-only).
- **Ceiling (deferred follow-up, per harness):** harnesses with an init/on-change
  seam (pi env) auto-inject. Not core — incremental like boot delivery (SL-119).

**Worker spawn:** orchestrator knows the target model → `resolve(role=worker,
model=…, arm=…, stage=…)` at spawn → band included fresh, no staleness. The
orchestrator stops hand-rolling context (the token win).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** Band order is fixed; label never changes position.
- **INV-2** Every matching snippet appears exactly once, ordered by the precedence
  key; only `replaces` suppresses.
- **INV-3** `replaces` × provenance: a **user** snippet's `replaces=<slot>` may
  suppress lower-specificity snippets in that slot **including framework** (user is
  the higher provenance). A **framework** snippet's `replaces` may only suppress
  framework (never a user snippet — user is never lower). Sealed content, having no
  user counterpart, is unaffected.
- **INV-4** Missing tier ⇒ no output for it; nothing errors (graceful degradation).
- **INV-5** Pure engine: no disk/clock/env; loader is the only impurity.
- **Edge — non-match ≠ override:** `harness=claude` snippets simply don't match a
  pi context; that's absence, not suppression.
- **Edge — equal specificity, same band+provenance:** alpha on full slot path
  (deterministic); a genuine tie is a "merge them" smell.
- **Edge — unknown stage label** in a locked band ⇒ `doctrine check` / `validate`
  flags it; use `project` band for genuinely new stages.

## 6. Open Questions & Unknowns

- **OQ-1 — Corpus name.** `scriptures` provisional. RESOLVE before install wiring
  lands (cheap to change until then).
- **OQ-2 — Stage-label vocabulary source.** The locked set of valid `stage/` labels
  = the shipped skill/verb names. Where is the authoritative list read from (a
  const, the skills manifest)? Design detail for the validator.
- **OQ-3 — Migration timing.** Do the existing `agents/*/dispatch-worker.md` defs
  migrate to `scriptures/role/worker/…` in *this* slice, or after? Leaning:
  retire `install/agents` and move them here (keeps one corpus), but the static
  shell injection-hole (`{{ resolve … }}`) is minimal — confirm at plan.

## 7. Decisions, Rationale & Alternatives

- **D1 — Two roots, layered, provenance>specificity, seal=non-exposure.**
  Rejected: fully-materialised install (clobbers user edits, can't distinguish
  provenance). Chosen: layered union (matches boot/memory corpus; user surface is a
  clean diff of intent).
- **D2 — Path→default slot+selector; per-file sidecar supersedes per-axis; seal on
  `install/manifest.toml`; no folder-axis manifests.** Cohesion split by
  consumer×time: resolver wants locality (path+sidecar), installer owns set-shaped
  seal (manifest, already exists). Rejected: selector-always-in-toml (noisy),
  folder-axis manifests (non-local surprise, nested-precedence ambiguity).
- **D3 — Specificity = scalar sum of pinned concrete segment-depths; alpha tiebreak.**
  Rejected: lexicographic per-axis tuple (forces an axis-priority ranking that every
  new axis re-opens). Scalar sum needs no axis priority; cross-axis ties fall to
  alpha (only within a band, already a smell).
- **D4 — Separate pure engine (`src/scriptures.rs`); boot calls it for baked bands.**
  Rejected: generalize boot into a scriptures assembly — boot's governance sections
  are entity-derived (not files) and the rewrite risks the behaviour gate.
- **D5 — Model band live via `--band` filter; floor=baked standing directive
  (scope), ceiling=per-harness auto-inject (deferred).**
- **D6 — No cache.** Boot's content-diff key covers baked bands; on-demand resolves
  are cheap, pure, stateless. (Confirmed: doctrine hot-loads far larger entity sets
  per page view without caching.)
- **Registry boundary — no model registry (P4/P5/P6).** The corpus is the sparse,
  self-pruning vocabulary; self-ID reflects it (`model-keys`); model→param mapping
  and env auto-detect are out (harness domain / optional ceiling).

## 8. Risks & Mitigations

- **R1 — Accidental model registry** (churn magnet). *Mit:* P4/P6 fence; no
  `models.toml`; `model-keys` reflects the corpus, never enumerates. Guard in review.
- **R2 — Boot regression** from the appended tail. *Mit:* behaviour-preservation —
  boot's existing sections untouched; the tail is additive; snapshot goldens updated
  deliberately, existing boot suites otherwise green.
- **R3 — Two-root confusion** for authors (why isn't my edit winning?). *Mit:*
  provenance>specificity is documented; `resolve` is inspectable; seal is explicit
  in the manifest.
- **R4 — Band/label validation drift** (stage vocab). *Mit:* validator reads one
  authoritative list (OQ-2); `doctrine check` covers it.
- **R5 — Scope creep into agent-def field-merge.** *Mit:* Non-Goal fence; defs are
  static shells with one injection hole, no per-field merge.

## 9. Quality Engineering & Validation

- **Engine goldens (table-driven, pure):** `(corpus, context) → ordered slot list /
  assembled md`, covering band order, provenance layering, specificity ascending,
  alpha tiebreak, non-match, `replaces` (both provenance directions), missing-tier
  degradation, seal (embedded-only wins).
- **Loader tests:** path→slot/selector derivation; sidecar per-axis supersede;
  provenance tagging by root; embedded⊕disk union.
- **`specificity()` unit table:** the D3 examples pinned exactly.
- **E2E golden:** `doctrine prompt resolve …` over a hermetic fixture corpus
  (framework + user), asserting exact output. `doctrine prompt model-keys` reflects
  only authored keys.
- **Boot behaviour-preservation:** existing boot suites green unchanged; one new
  golden for the appended baked-bands tail; model band demonstrably absent from
  `boot.md`.
- **Layering gate:** `tests/architecture_layering.rs` stays green (command ← engine
  ← leaf; no cycle).

## 10. Review Notes

_(adversarial pass pending — §Adversarial review)_

## Code Impact (design-target)

- **`src/scriptures.rs`** — NEW pure engine (`resolve`, `matches`, `specificity`,
  types).
- **`src/commands/prompt.rs`** — NEW command (`resolve`, `model-keys`) + the impure
  loader (embedded⊕disk walk, sidecar overlay).
- **`src/boot.rs`** — call the engine for baked bands, append tail section; omit
  model band.
- **`src/install.rs` / `install/manifest.toml`** — seal/expose projection section;
  `sync_corpus` projects `scriptures/`.
- **`install/scriptures/**`** — NEW seed corpus (universal/harness/model/role/stage
  examples) + convention doc; **retire `install/agents/`** (migrate defs in as
  static shells — OQ-3).
- **`src/main.rs`** — wire the `prompt` command.
- **Tests** — `src/scriptures.rs` unit + goldens; e2e prompt-resolve golden; boot
  golden update.
