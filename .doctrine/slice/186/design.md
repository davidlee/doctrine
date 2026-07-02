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
                 │   (role = orchestrator|worker; also selects the shape)│
                 └──────────────────────────────────────────────────────┘
                                      │  resolve(context, corpus)
   corpus = embedded(framework) ⊕ disk(user)   ▼
   match each snippet's selector → order by PRECEDENCE KEY → concat → assembled md

   PRECEDENCE KEY:  band  →  specificity  →  provenance(framework<user)  →  alpha
```

**Specificity dominates provenance** (revised post-review, D3/D1): a framework
*exact-model* snippet outranks a user *vendor-default* — corrective, model-specific
guidance is where it belongs and must get the last word. Provenance is only a
**tiebreak at equal specificity** — i.e. when the user edits the *same slot* as an
exposed framework default, the user wins (the legitimate customisation). A user's
*broader* snippet never silently buries a framework's *narrower* one. Seal (below)
is the framework's hard-win, orthogonal to this ordering.

**Bands** (closed registry, fixed order — position, not identity):

```
preamble · harness · model · role · stage · project
```

`role` and `stage` are selector-axes that also *name* a band. Open bands (`model`,
`project`) accept any label; locked bands validate labels against a known set
(stage labels = real doctrine skills/verbs). The **band is the first path segment**
under the corpus root; the **label** is the remaining path — free identity within
the band, and naming it can never move the band (position is fixed).

**Within-band specificity** leads with the **band's own namesake axis**, then the
sum of other pinned axes — lexicographic `(band_axis_depth, Σ other_axis_depths)`.
So in the `model` band, `model=anthropic/claude-sonnet-4` (model-depth 2) outranks a
`model=anthropic` snippet that piles on `harness`+`role` (model-depth 1, others 2) —
axis-count can't bury exact-model. The primary axis is *the band's own*, so no
arbitrary global axis ranking is introduced.

**Two roots, layered (D1):**

```
install/hymns/     compile-embedded (rust_embed), framework-authored, the SUPERSET
      │  install-time projection (embedded → disk); the SEAL SET decides what is exposed
      ▼
.doctrine/hymns/   user-customisable, read at runtime; holds only exposed + user snippets
```

The resolver **unions** embedded-framework ⊕ on-disk-user at read time. Provenance
is derived from *which root a snippet came from* — not a flag. `EXPOSED` framework
content is projected as an editable starter; a user edit at the same slot wins (the
equal-specificity provenance tiebreak).

**Seal is resolution-enforced, not merely un-projected (review finding 1).** A
non-projected sealed slot could still be *shadowed* by a user who hand-creates the
matching path under `.doctrine/hymns/`. So the loader consults an **embedded seal
set** (the manifest's projection list, compiled in) and **drops any disk snippet
whose slot is sealed** before matching. Sealed framework content therefore wins
unconditionally — by active exclusion of disk twins, not by hoping the path stays
empty. This is the one place the resolver *is* seal-aware; everything else derives
provenance from the source root. (This mildly revises the earlier "resolver never
sees seal" claim — it sees exactly the sealed-slot set, nothing more.)

> **Name.** `hymns` (const `HYMNS_ROOT`). `canon`/`corpus` are taken (skill /
> `src/corpus.rs`). Bands parse *relative* to the root, so the name is a mount
> point — a `doctrine.toml` override is a trivial later addition (OQ-1); default-only
> const for now (STD-001 single-source, YAGNI on the knob).

### 5.2 Interfaces & Contracts

**Command (ADR-001 command layer) — `src/commands/prompt.rs`:**

```
doctrine prompt resolve --role <orchestrator|worker>
                        [--harness <name>] [--model <id>] [--arm <subagent|subprocess>]
                        [--stage <skill/verb>] [--band <name>]...
    → disk:   regenerate the UNIVERSAL .doctrine/state/boot.md (write-if-changed);
              axis-INVARIANT — flags never alter the on-disk artifact (INV-7).
      stdout: <universal snapshot> ++ <hymns for the role>.  Idempotent.
    · --role is the role-axis value (orchestrator|worker) AND selects the assembly shape
      (which bands, envelope) — one concept, one name (F15/A); axis flags refine within.
    · --band repeatable; absent = every band the shape includes — never `model` (live, §5.4).

doctrine prompt model-keys [--harness <name>]
    → the FULL relative model keys that EXIST in the corpus (e.g. `anthropic/claude-sonnet-4`,
      `deepseek/_default`), one per line — the exact strings `--model` accepts.
    · The "named set to choose from" for agent self-identification (§5.4).
    · Reflects authored guidance only — NOT a registry. Empty ⇒ don't ask.

doctrine prompt explain --role <r> [axes…]
    → precedence trace: per slot, which snippets matched, who won, why
      (band→specificity→provenance→alpha). The cascade's debugger (R3).

doctrine prompt check
    → corpus integrity: sealed slots present & unshadowed, selectors parse, sidecars name
      real bands, `replaces` unique-most-specific. Feeds `doctrine check` (R4/INV-3).
```

**Model-key grammar (finding 9).** A model key is the snippet's path *relative to
`model/`*: `<vendor>/<segment…>`. The context `--model <id>` is matched left-to-right
by path segment against the tree; `_default.md` at any level is the wildcard tail for
that level. `model-keys` emits full relative keys (never bare leaves), so an agent
passes back an unambiguous string. No canonicalisation of vendor spelling is done —
see §5.4 for why that's deliberate, not a gap.

**Engine (pure) — `src/hymns.rs`:**

```rust
struct ContextVector { role: Role, harness: Option<Harness>, model: Option<ModelKey>,
                       arm: Option<Arm>, stage: Option<StageKey>, bands: BandFilter }
// Role = orchestrator | worker — the role-axis value; also selects the assembly shape (F15/A).
struct Snippet { slot: Slot, selector: Selector, provenance: Provenance, body: String }
struct Slot { band: Band, label: String }
// Selector: axis→pattern map (path-derived, sidecar-superseded); `replaces: Option<Slot>`.

fn resolve(ctx: &ContextVector, corpus: &[Snippet], sealed: &SealSet) -> String  // pure: filter→match→order→concat
fn matches(sel: &Selector, ctx: &ContextVector) -> bool
fn specificity(band: Band, sel: &Selector) -> (u32, u32)   // (band_axis_depth, Σ other) — D3, band-primary
```

`resolve` takes the `SealSet` (embedded, passed in — keeps the engine pure) and drops
disk-provenance snippets whose slot is sealed before ordering (finding 1).

**Loader (impure shell, command edge):** walk embedded `install/hymns/**` +
disk `.doctrine/hymns/**`; derive `Slot`+`Selector` from path; overlay sidecar
`<file>.toml` (supersede per-axis, carries `replaces`); tag `provenance` by source
root. Reuses `fsutil`, `globmatch`, `dtoml`. **NOT `corpus::sync_corpus`** — that is
memory-specific (`MEMORY_SHIPPED_DIR`, `memory.{toml,md}` uids), not a general
projector (finding 7); the hymns walk/overlay is its own code (a generic embedded⊕disk
projector may be extracted later, but is not assumed to exist).

**Output = stdout only.** `prompt resolve` emits assembled markdown to stdout; it is
read-only and does not itself write disk. **How that output reaches live agents** — the
session-start stdout injection, the on-disk universal `boot.md` fallback, the two-channel
cache split, the model-band floor/supplement, onboarding-memory inlining, and the pi/hook
wiring — is **SL-187** (the delivery slice), split off on a blast-radius boundary (this
engine is inert until SL-187 wires it). This slice locks the *contract* SL-187 consumes:
the verb signatures above, the composed-markdown shape, and the model-agnostic emit.

### 5.3 Data, State & Ownership

| Surface | Owner | Consumer | When |
|---|---|---|---|
| `install/hymns/**` (+ sidecar `.toml`) | framework (committed) | resolver (embedded) | compile |
| `.doctrine/hymns/**` | user (+ projected starters) | resolver (disk) | runtime |
| `install/manifest.toml` seal/expose section | framework | installer + resolver (as embedded SealSet) | install / runtime |
| assembled markdown | resolver | agent (via SL-187 delivery) | on demand |

- **Provenance** = source root (embedded vs disk). Not stored, derived.
- **Seal** has two consumers: the **installer** uses it to decide what to project;
  the **resolver** carries the same set (compiled in) to drop disk twins of sealed
  slots (finding 1). It is the *only* seal-aware point in resolution.
- Path → default `Slot`+`Selector`; sidecar `.toml` supersedes **per-axis**
  (declared axes win; undeclared fall back to path). No folder-level axis manifests.
- **Agent definitions stay their own surface** (`install/agents/**` — frontmatter +
  external `subagent_type` contract), NOT migrated into `hymns/` (finding 7). A def
  is a static shell that may carry a `{{ prompt resolve … }}` injection hole; the
  file itself is not a hymn snippet.

### 5.4 Lifecycle, Operations & Dynamics

**Corpus load → resolve.** The loader walks embedded `install/hymns/**` ⊕ disk
`.doctrine/hymns/**` once per invocation (no cache — D6), producing the `Snippet` corpus +
`SealSet`; `resolve(ctx, corpus, sealed)` filters → matches → orders → concatenates to
markdown. Stateless and pure; the same `(corpus, ctx)` always yields the same bytes (a
determinism the delivery slice relies on for its cache-hold).

**Callers (contract — realised in SL-187).** Two shapes select via `--role`:
`--role orchestrator` (session-start base) and `--role worker` (spawn envelope, +model +arm
+stage). The resolver just composes; *when/where* each is injected is SL-187.

**Self-identification is not a hidden registry (finding 5).** The `model-keys` verb exists so
an agent can resolve the mutable `model` band on demand without doctrine maintaining a churny
model table. No alias/normalisation table exists: the agent already holds a fuzzy
self-description ("Claude Sonnet 4"); `prompt model-keys` hands it the *small, corpus-reflected*
set of exact keys, and the agent (an LLM, good at this) picks the nearest in one shot. A
mismatch degrades gracefully (broader `_default` or universal-only). Declining to normalise is
the deliberate anti-churn stance (P4/P6), not an unsolved gap. **No correctness invariant may
rest on the model band** — it is fine-tuning; stale-tuning ⊆ graceful degradation (the delivery
mechanics + floor/supplement live in SL-187).

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** Band order is fixed; label never changes position.
- **INV-2** Every matching snippet appears exactly once, ordered by the precedence
  key; only `replaces` suppresses.
- **INV-3** `replaces` is legal **only on the unique most-specific active snippet for
  its slot** (finding 4). Two active `replaces` targeting one slot, a `replaces` that
  is not the top of its slot, and any `replaces` cycle are **authoring errors** caught
  by `doctrine check`/`validate` — never silently ordered by alpha. Given uniqueness,
  the winner suppresses all lower-precedence snippets in the slot; a **user** replacer
  may thus suppress framework, a **framework** replacer may not reach a user snippet
  (user is never lower under `band→specificity→provenance`). Sealed slots have no disk
  twin, so their `replaces` only ever acts within framework.
- **INV-4** Missing tier ⇒ no output for it; nothing errors (graceful degradation).
- **INV-5** Pure engine: no disk/clock/env (SealSet passed in); loader is the only impurity.
- **INV-6** A disk snippet whose slot is in the SealSet is dropped before matching
  (seal hard-win, finding 1).
- **INV-7** `resolve` is deterministic: the same `(corpus, ctx, sealed)` yields byte-identical
  output. (SL-187's cache-hold depends on this.)
- **Edge — non-match ≠ override:** `harness=claude` snippets simply don't match a
  pi context; that's absence, not suppression.
- **Edge — equal specificity:** provenance breaks it (framework<user), then alpha on
  full slot path (deterministic). A same-provenance equal-specificity tie is a "merge
  them" smell.
- **Edge — unknown stage label** in a locked band ⇒ `doctrine check` / `validate`
  flags it; use `project` band for genuinely new stages.

## 6. Open Questions & Unknowns

- **OQ-1 — Corpus name / config.** Name RESOLVED: `hymns`. Open sub-point: expose
  the disk root as a `doctrine.toml` override, or const-only? Leaning const-only
  (root-relative bands make a later override trivial).
- **OQ-2 — Stage-label vocabulary source.** The locked set of valid `stage/` labels
  = the shipped skill/verb names. Where is the authoritative list read from (a
  const, the skills manifest)? Design detail for the validator.
- **OQ-3 — Def↔hymn wiring (revised post-review).** Agent defs stay in
  `install/agents/**` (finding 7); they are *not* migrated into `hymns/`. Open: does
  *this* slice add the `{{ prompt resolve … }}` injection hole to `dispatch-worker.md`,
  or is that a follow-up once the resolver lands? Leaning follow-up — keep this slice
  to the resolver + corpus; wiring defs to it is a clean next step (also unblocks
  IMP-197). Confirm at plan.
- *(Delivery/boot-subsumption open questions moved to **SL-187**.)*

## 7. Decisions, Rationale & Alternatives

- **D1 — Two roots, layered; `band→specificity→provenance→alpha`; seal =
  resolution-enforced hard-win.** Rejected: fully-materialised install (clobbers user
  edits, can't distinguish provenance). Chosen: layered union (matches boot/memory
  corpus; user surface is a clean diff of intent). **Revised post-review (findings
  1,2):** specificity now dominates provenance (provenance is the equal-specificity
  tiebreak) — a user's broad snippet can't bury a framework's model-specific one;
  user still wins the *same slot*. Seal is enforced by dropping disk twins, not by
  hoping the path stays empty.
- **D2 — Path→default slot+selector; per-file sidecar supersedes per-axis; seal on
  `install/manifest.toml`; no folder-axis manifests.** Cohesion split by
  consumer×time: resolver wants locality (path+sidecar), installer owns set-shaped
  seal (manifest, already exists). Rejected: selector-always-in-toml (noisy),
  folder-axis manifests (non-local surprise, nested-precedence ambiguity).
- **D3 — Specificity = `(band-primary-axis depth, Σ other-axis depths)`, lexicographic;
  then provenance, then alpha.** Revised post-review (finding 3): a plain scalar sum
  let a shallow-model + extra-axes snippet outrank an exact-model one. Leading with the
  **band's own namesake axis** fixes it *without* a global axis ranking (the primary
  axis is always the band's — non-arbitrary). Rejected: full global lexicographic tuple
  (reopens axis-priority per new axis); plain scalar sum (the finding-3 footgun).
- **D4 — Separate pure engine (`src/hymns.rs`); a thin command shell.** Rejected: generalize
  boot into a hymns assembly — boot's governance sections are entity-derived (not files) and
  such a rewrite risks the behaviour gate. The engine stays pure; the verb is a thin shell
  over loader+engine. (Its *consumption* by boot delivery is SL-187.)
- **D6 — No cache.** On-demand resolves are cheap, pure, stateless. (Confirmed: doctrine
  hot-loads far larger entity sets per page view without caching.)
- **Registry boundary — no model registry (P4/P5/P6).** The corpus is the sparse,
  self-pruning vocabulary; self-ID reflects it (`model-keys`); model→param mapping
  and env auto-detect are out (harness domain).
- *(Delivery decisions — cache-property split, unstale, onboarding inline, onboard
  floor-supplement, `--role` delivery wiring — are **SL-187** D1–D5.)*

## 8. Risks & Mitigations

- **R1 — Accidental model registry** (churn magnet). *Mit:* P4/P6 fence; no
  `models.toml`; `model-keys` reflects the corpus, never enumerates. Guard in review.
- **R2 — Two-root / ordering confusion** for authors (why isn't my edit winning?).
  *Mit:* `band→specificity→provenance` documented; a user edit wins only at the *same
  slot* (broad-shadows-narrow no longer surprises); `resolve` is inspectable; seal is
  explicit in the manifest and enforced.
- **R3 — Band/label validation drift** (stage vocab). *Mit:* validator reads one
  authoritative list (OQ-2); `doctrine check` covers it.
- **R4 — Scope creep into agent-def field-merge.** *Mit:* Non-Goal fence; defs are
  static shells with one injection hole, no per-field merge.

## 9. Quality Engineering & Validation

- **Engine goldens (table-driven, pure):** `(corpus, sealset, context) → ordered slot
  list / assembled md`, covering band order, **specificity>provenance** (framework
  exact beats user vendor-default), **band-primary-axis** (exact-model beats
  shallow-model+extra-axes), same-slot provenance tiebreak, alpha, non-match,
  `replaces` (unique-most-specific; overlap/cycle rejected), missing-tier degradation,
  **seal disk-twin drop**.
- **Loader tests:** path→slot/selector derivation; sidecar per-axis supersede;
  provenance tagging by root; embedded⊕disk union; sealed disk-twin excluded.
- **`specificity()` unit table:** the D3 examples pinned exactly (incl. finding-3 case).
- **Validate tests:** overlapping/cyclic `replaces` and unknown stage labels are
  authoring errors.
- **E2E golden:** `doctrine prompt resolve …` over a hermetic fixture corpus
  (framework + user + a sealed slot with a user twin), asserting exact output.
  `doctrine prompt model-keys` reflects only authored keys, as full relative keys.
- **Layering gate:** `tests/architecture_layering.rs` stays green (command ← engine
  ← leaf; no cycle).

*(Delivery-side validation — boot behaviour-preservation, onboarding-inline goldens,
cache-hold — lives in **SL-187**.)*

## 10. Review Notes

**Codex (GPT-5.5) adversarial pass — 2026-07-02.** 9 findings; 8 valid. Integrated:
- **F1 (crit)** seal was defeatable by a hand-created disk twin → seal is now
  resolution-enforced (embedded SealSet drops disk twins; INV-6, §5.1/§5.2).
- **F2+F3 (maj)** provenance-dominates + scalar-sum buried framework model-specific
  guidance → precedence flipped to `band→specificity→provenance→alpha` with
  band-primary-axis specificity (D1/D3). *User-approved reversal.*
- **F4 (maj)** multiple `replaces` nondeterministic → unique-most-specific rule +
  validate rejection (INV-3).
- **F5 (maj)** "no registry" overclaimed → self-ID honesty: no maintained table,
  agent one-shot fuzzy-match vs reflected set, harness-inject ceiling (§5.4).
- **F6 (maj)** live-model floor = silent drift → model band is advisory-by-construction;
  no correctness invariant rests on it (§5.4).
- **F7 (maj)** `sync_corpus` reuse fiction + def co-location → own projector; defs stay
  in `install/agents/**` (§5.2/§5.3, code-impact).
- **F8 (min)** boot not byte-stable → dissolved by the delivery rework: hymns ride the
  stdout emit, never the shared `boot.md`. **The rework itself now lives in SL-187.**
- **F9 (min)** grammars locked: model-key = full relative; stage vocab source (OQ-2).
- *Survived:* the ADR-001 module split (pure engine + thin command) — unchanged.

**Codex (GPT-5.5) re-pass on the delivery rework — 2026-07-02.** 7 findings; F10–F14 all
concerned the delivery half → **carried to SL-187** (dismissed F10/F11, wording-fix F12,
accepted F13/F14; see SL-187 §10). **F15 (min)** `--context` overloaded → renamed **`--role`**
(one concept: role-axis value + assembly shape; F15/A). **F16 (maj)** scope creep → **this
split** — SL-186 = the inert resolver engine; SL-187 = the live-surface delivery.

## Code Impact (design-target)

- **`src/hymns.rs`** — NEW pure engine (`resolve`, `matches`, `specificity`,
  `SealSet`, types).
- **`src/commands/prompt.rs`** — NEW command (`resolve`, `model-keys`, `explain`,
  `check`) + the impure loader (embedded⊕disk walk, sidecar overlay, seal-twin drop).
  Emits assembled markdown to stdout; read-only.
- **`src/install.rs` / `install/manifest.toml`** — seal/expose projection section +
  embedded SealSet accessor; a hymns-specific embedded→disk projector (NOT
  `sync_corpus`, which is memory-only).
- **`install/hymns/**`** — NEW seed corpus (universal/harness/model/role/stage
  examples) + convention doc. **`install/agents/` stays** (defs are a separate
  surface; `{{ resolve }}` injection hole is a follow-up — OQ-3).
- **`src/main.rs`** — wire the `prompt` command.
- **Tests** — `src/hymns.rs` unit + goldens; e2e prompt-resolve golden.

*(Delivery-side code impact — `src/boot.rs` generator/inline, `doctrine_onboard`, memory
tagging, pi/hook wiring — is **SL-187**.)*
