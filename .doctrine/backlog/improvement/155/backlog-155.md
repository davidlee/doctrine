# IMP-155: Per-harness + per-model agent instruction injection

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-155.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

Doctrine agents receive instructions from several sources, but they have no
common model — so guidance is scattered, duplicated, or has nowhere to live:

| Source | Known when | Examples | Current mechanism |
|------|-----------|----------|------------------|
| **Universal** | always | AGENTS.md rules, governance, conventions | AGENTS.md + boot snapshot (sentinel-gated) |
| **Harness** | session init, fixed | "MCP tools available" vs "none"; spawn mode; confinement | Ad-hoc — tool defs injected by harness, no instruction layer |
| **Model / family** | mutable post-boot | "you're DeepSeek — weak at multi-step git"; "no thinking mode" | **Nowhere** |
| **Stage / skill** | at the verb | project end-of-phase / audit / code-review process | **Nowhere** — no user insertion point |
| **Role** | at spawn | orchestrator (self) vs worker (spawned) contracts | Hand-rolled by the orchestrator each spawn |

Two costs bite:

1. **No home** for model, stage, and role guidance — the model tier especially
   has nowhere to go.
2. **Orchestrators burn tokens** hand-assembling worker context every spawn,
   re-deriving what is fully determined by (harness, model, role, arm, stage).

The naïve fix — a directory tree per source with fallback-to-one-winner — was the
original sketch here. It doesn't compose (one winner per tier) and it explodes
combinatorially (harness × model × arm × role). Rejected. See the model below.

## Model: a prompt cascade (selectors + composition)

The insight that dissolves the combinatorics: this is **CSS, not a filesystem
lookup**. Snippets declare *where they go* and *when they apply*; one resolver
assembles them for a given context. A snippet lives **once**, is *matched* by
selector, never copied per combination. Zero repetition falls out.

```
                 ┌─────────── context vector (the "element") ──────────┐
                 │ harness=claude  model=deepseek/v3  role=worker       │
                 │ arm=subprocess  stage=execute      project=doctrine  │
                 └──────────────────────────────────────────────────────┘
                                      │  doctrine prompt resolve(context)
   snippets (each lives ONCE)         ▼
   ┌───────────────┬─────────┬───────────────────────┐    order by band,
   │ snippet       │ band    │ selector              │    then specificity
   ├───────────────┼─────────┼───────────────────────┤    ┌──────────────┐
   │ AGENTS.md     │ preamble│ *                     │──▶ │ assembled    │
   │ harness/claude│ harness │ harness=claude        │    │ prompt for   │
   │ model/deepseek│ model   │ model=deepseek*       │    │ THIS agent   │
   │ worker-negctr │ role    │ role=worker           │    └──────────────┘
   │ audit-hook    │ stage   │ stage=audit           │
   └───────────────┴─────────┴───────────────────────┘
```

### 1. Snippet — one `.md`, selector from the path

A snippet is one markdown file (prose only). Its **selector** comes from its
**path by default** — `harness/claude.md` implies `selector={harness:claude}`,
`model/deepseek/_default.md` implies `selector={model:deepseek*}`. Path handles
the single-axis common case (~90%) at zero config.

A **sidecar `.toml`** (idiomatic — matches slice/adr/backlog pairing; no markdown
front-matter, per project convention) appears **only** when a snippet needs more
than the path gives: multi-axis selector, a non-default band, or `replaces`.

```
worker/negative-contract.md          # selector={role:worker}, band=role
worker/negative-contract.toml        # ONLY when refining:
    # selector = { role = "worker", arm = "subprocess" }
    # band     = "stage"
    # replaces = "stage/..."     # opt-in; see §3
```

### 2. Slots — locked band (position) + free label (identity)

A slot is `<band>/<label>`. The **band is a closed registry** and fixes *where
the snippet lands in the assembled prompt*. The **label is free identity within
the band** — naming it freely can never move it, so freedom is harmless.

```
BANDS (fixed order):  preamble · harness · model · role · stage · project

  preamble  universal base (AGENTS.md, governance)          locked
  harness   mechanism: tools, spawn, confinement            locked
  model     behaviour / constraints                         open  (any label)
  role      orchestrator- vs worker-specific contracts      locked
  stage     skill/verb insertion points (audit, execute…)   locked (known hooks)
  project   catch-all user guidance, last word              open  (any label)
```

- `role` and `stage` double as **selectors that also name a band**.
- **Open** bands (`model`, `project`) accept any label. **Locked** bands validate
  the label against a known set (stage labels = real doctrine skills/verbs). A
  project extending "end of audit" selects `stage=audit` (a known hook); a
  brand-new stage doctrine lacks → not allowed, use `project`.
- `doctrine check` validates: band registered; locked-band labels known.

### 3. Composition — uniform append; specificity orders, never suppresses

**Every matching snippet concatenates.** Being on claude XOR pi is *non-match*
(pi snippets simply don't match), not override. Within a band, specificity only
decides **order**: general first, specific last, so the specific snippet gets the
last word by proximity. `model/deepseek/_default` and `model/deepseek/v3` **both**
emit — they don't replace each other.

```
within a band:  specificity ascending  →  alpha (tiebreak only)
```

Alpha breaks ties among equal-specificity snippets — deterministic, zero config.
No `order:` field (two order-dependent snippets are a smell — merge them).

**Escape hatch, opt-in, rare:** a snippet may declare `replaces = "<slot>"` to
suppress lower-specificity snippets in that slot. Default append; opt-in replace.

### 4. Resolver — one command, two callers

```
doctrine prompt resolve <context vector>  →  assembled markdown

  harness @ boot:     resolve --role orchestrator --harness claude --model opus-4.8
                        → boot.md   (role selects the assembly shape)
  orchestrator @ spawn: resolve --role worker --model deepseek/v3 --arm subprocess --stage execute
                        → ready-to-paste worker prompt
```

`role` selects the assembly shape. The orchestrator **stops hand-rolling worker
context** — it asks the resolver. That is the token win in the Problem above.

### 5. Model band stays live (never baked)

Harness is fixed at boot; **model can change post-boot** (slash command). Only the
orchestrator's *own* model guidance is affected (workers get `--model` fresh at
spawn). So: **every band except `model` bakes into boot.md; the `model` band is
never baked** — the agent (or a harness hook) re-runs `resolve --band model
--model <new>` on a model swap. Idempotent, read-only, stateless.

Model self-identification (how the `--model` value reaches the resolver — harness
env injection vs agent self-declare) is a resolver-CLI detail, deferred to design.

### 6. Scope: NARROW — instructions only

The resolver assembles **instructions** (prose) only. Agent **definitions**
(`dispatch-worker.md`: name/tools/model — structured fields) are **out of scope**
and stay their own surface. Definition composition would need per-field merge
(union tools? override model?), a *different* combination kind from markdown
append — not free uniformity, and probably not worth it.

Instead: an agent definition is a **static shell with one injection hole** the
resolver fills.

```
claude/dispatch-worker.md   (static shell, hand-authored per harness×subagent)
  name/tools/model + {{ doctrine prompt resolve --role worker … }}   ← one hole
```

The **selector engine** is designed to be reusable later (a definition could one
day be selector-matched too); the **field-merge is deferred, maybe never**. This
mirrors the 197 move (§Related).

## Deferred to design/slice

- Resolver CLI surface + cache keying (context vector → assembled markdown).
- Final directory layout under `.doctrine/agents/` (or a new `prompts/` root).
- Model self-identification path (§5).
- Phasing.

## Related

- **IMP-197** now runs `after` IMP-155 (edge inverted). 197 shrinks to *authoring
  worker-selected snippets* (negative-contract, home-module, hermetic,
  path-anchor) **on this world** — the proof the cascade works, not a blocker.
- **IMP-116** — pi APPEND_SYSTEM.md; the harness band composes into that pipeline.
- **ADR-011** — harness-agnostic orchestrator / capability altitude. This extends
  the concept from the infrastructure layer to the instruction layer.
- `.doctrine/agents/claude/dispatch-worker.md` — existing per-harness agent
  definition (a static shell candidate, §6).
