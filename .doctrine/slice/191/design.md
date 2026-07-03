# SL-191 Design — Dispatch worker contract: cascade hymns + check-cadence + role/model/stage bake

> **STATUS: IN PROGRESS (design stage, pre-lock).** Q1 locked (D1). Q2/Q3 open.
> Compacted mid-`/design` at the "Ask clarifying questions" stage. Do NOT treat
> as final; sections below the line are provisional until the open forks close.
> Durable decision log + code anchors: `notes.md`. Scope: `slice-191.md`.

## Problem

Deepseek-class dispatch workers make reasonable-but-wrong choices on two axes —
*where* (module home, path matching) and *what-not-to-do* (unasked work). SL-168
shipped three blocker-class defects from this (ADR-001 layering violation,
non-hermetic live-corpus golden, substring path matching). The fix is a hardened
worker contract, delivered through machinery that already exists.

## Approach (settled frame)

Ride existing seams — no parallel implementation:

- **Cascade** (SL-186 resolve + SL-187 delivery): author the contract as **hymns**
  in `install/hymns` (Framework, shipped) with this-repo specifics in
  `.doctrine/hymns` (overlay).
- **Subagent-def bake** (`src/install.rs`): the claude arm is already wired — the
  install-time `expand_worker_marker` inlines resolved worker hymns into each
  shipped `dispatch-worker.md`. Generalize it from **role-only** to carry the model
  band too (and stage iff Q2 keeps it).
- **`doctrine check` cadence** (SL-163): the owned, POL-002-clean formatter/lint
  seam. Worker contract invokes `doctrine check`, never a literal `cargo fmt`.

## Locked decisions

### D1 — Model band arm-asymmetric in effect, never as a baked rule (Q1)

Hymns stay **model-keyed**. The model band is selected by the model a def actually
targets — NOT by an arm→model map. No "claude=strong / subprocess=deepseek" rule
appears in hymn content or bake logic. A def with **no pinned model resolves no
model band**; a claude worker run under pi declares pi's model and gets the right
band. Coupling lives in **data** (the def's declared model), not logic.

Consequence: the deepseek delivery patterns (§5c) reach whatever def targets
deepseek (universal/subprocess), and are absent from a model-unpinned claude def —
by data, not by a conditional. The **negative contract still reaches every arm**
via the model-agnostic **role** band.

---
## PROVISIONAL BELOW — pending Q2/Q3

### D2 — Where the bake learns (model, stage) [Q2 — OPEN, recommendation only]

**Recommendation (unconfirmed):**
- **Model from the def's frontmatter `model:`** — single source shared with the
  harness's own model selector, so the resolved band always matches the model the
  worker runs. Marker becomes a sentinel = "resolve my full worker context."
  Rejected: parametric marker carrying `--model` (two sources → drift).
- **No stage band.** Hermetic-fixture + component-anchored-path directives are
  worker-**role** conduct, not phase-**stage** conduct → fold into `role/worker`,
  drop `stage` from the bake. Smallest coherent change.

If confirmed, the bake change is: widen `resolve_worker_role_body`'s ContextVector
to set `model` (from frontmatter) + `bands: Only([Role, Model])`; no `--stage`
threading; no `install/hymns/stage/*`.

### D3 — Import reliability gate [Q3 — OPEN, not yet discussed]

Worker instruction alone is not reliable (a worker forgets). Candidate: an
import-time `doctrine check` gate on the delta — reject unformatted/lint-red, vs
auto-`check`+re-import. Funnel wiring in the import belt.

### Content plan (provisional)

- `install/hymns/role/worker.md` (Framework, host-agnostic): NEGATIVE CONTRACT
  (declared file set; no `.doctrine/`/`.claude/`; only git verb = final commit; no
  unauthored test edits), hermetic goldens, component-anchored paths (+ owned
  `.dispatch/`/`.worktrees/`), state-home+layer-rationale directive, run `doctrine
  check quick` per-edit / `check commit` before commit (regression-relevant suite
  per `mem.pattern.dispatch.worker-prompt-run-full-suite`).
- `install/hymns/model/deepseek/_default.md`: §5c delivery patterns.
- `.doctrine/hymns/**` overlay: `cargo`/`target/`/ADR-001-layer/Rust-home specifics.
- `src/install.rs`: generalize the bake context (D2).
- `/dispatch` skills + `src/dispatch.rs`: base-clean `doctrine check` before
  arm-spawn; import-delta check gate (D3).

### Code impact / design-target selectors (provisional)

High-confidence targets recorded now as `design-target` (certain regardless of
Q2): `install/hymns/role/worker.md`, `install/hymns/model/deepseek/_default.md`,
`src/install.rs`. Conditional (held pending Q2): `install/hymns/stage/*`. Funnel
targets (`src/dispatch.rs`, `/dispatch` skills) firm once D3 shape is chosen.

## Verification alignment (provisional)

- `prompt resolve --role worker --model deepseek/…` emits the full contract;
  host-agnostic grep gate over `install/hymns` (no `cargo`/`target/`/`just`).
- Generalized `expand_worker_marker` inlines role(+model) into the baked def;
  existing bake unit tests + drift test green (behaviour-preservation gate).
- Funnel: base-clean check before handoff; unformatted imported delta rejected.
- Live/dry dispatch: worker receives contract, runs `doctrine check quick`, lands
  a formatted in-set delta.

## Open questions (tracked)

- **OQ-1 (Q2):** model-from-frontmatter vs parametric marker; stage band or fold
  into role.
- **OQ-2 (Q3):** import gate — reject vs auto-fix+reimport.
- **OQ-3:** does a spawned claude subagent run a SessionStart `prompt resolve`
  (SL-187 seam), or is the baked def the sole delivery? (Bears on whether any
  runtime resolve is even available on the claude arm.)
