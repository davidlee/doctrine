# Prompt cascade: per-harness delivery & boot integration

Delivers the resolver world built in **SL-186** to live agents. SL-186 ships the
`doctrine prompt` engine + verbs (inert, no caller); this slice wires them into
the session-start path across harnesses and folds the model band + onboarding
memories into the right cache tier. Split from SL-186 on a **blast-radius**
boundary: SL-186 is additive/inert (own goldens), this slice mutates the **live,
shared bootstrap surfaces** (`boot.md`, the onboard MCP tool, the pi extension,
memory) — a distinct behaviour-preservation gate. Contract-first: buildable in
parallel with SL-186 against the locked `prompt resolve` interface.

## Context

Agents receive instructions at session start via per-harness seams: Claude/Codex
`SessionStart` hooks and the shipped **pi `before_agent_start` extension**
(CHR-033: already stdout-emits `doctrine boot --emit`, injects byte-identical per
turn so the **Anthropic prefix cache holds across turns**; SYSTEM_APPEND.md
retired). CHR-033 names this slice's seam explicitly: *"change the exec'd command
from `doctrine boot --emit` to `doctrine prompt resolve <role>`."*

The token win IMP-155 targets (orchestrators stop hand-assembling worker context)
is realised **here** — SL-186 only makes it *possible*.

## Scope & Objectives

Delivery splits by **cache property** (this is the load-bearing idea):

```
CACHE-STABLE boot sector   MODEL-AGNOSTIC — rides the prefix cache, survives /model
  governance + universal hymns + inlined onboarding memories
  tier-1  SessionStart hook stdout / pi before_agent_start   (probed cache-hold, CHR-033)
  tier-2  @-import universal boot.md                          (universal-only fallback)
CACHE-BUSTING supplement   MODEL-SPECIFIC — free on a path that busts cache anyway
  doctrine_onboard MCP tool = model identification + model band
```

1. **`prompt resolve` disk+stdout behaviour.** `resolve` unstales the *universal*
   on-disk `boot.md` (reuse boot's generator, `write_if_changed`) and emits
   `universal ++ role hymns` to stdout. Disk artifact is **axis-invariant** (INV-D1):
   `--role`/`--harness` extend stdout only, never disk. Unstale is unconditional but
   idempotent — a deterministic projection of committed inputs, so a no-op unless an
   input changed; self-healing under concurrency (INV-D2).
2. **Boot generator changes (`src/boot.rs`).** Expose the universal-snapshot generator
   for `resolve` to reuse; add a **universal-band hymns** section to the disk snapshot
   (harness-agnostic); harness/model/role/stage bands are stdout-only, never baked.
3. **Onboarding memories → cached sector.** Inline the bodies of `onboarding`-tagged
   memories into the universal disk snapshot via `memory::collect_all` (items ∪ shipped,
   local-wins) + a tag filter — retiring the footer "load these next turn" round-trip.
   Deterministic order (by memory key); local-wins collision is intended + documented;
   set is small-by-construction (INV-D3). Tag the shipped `overview` + `orientation`.
4. **Model band delivery.** Floor: a **universal** standing directive (rides the cached
   sector, both tiers) — self-identify + `prompt resolve --band model`. Supplement (better
   floor, not a guaranteed ceiling): `doctrine_onboard` does model identification (offers
   `model-keys`) + emits the model band on the cache-busting side; **drops** its memory
   load (now inlined). No true mid-session ceiling without an on-model-change seam.
5. **Per-harness wiring.** pi: one-line command swap in `generate_pi_extension()`
   (`boot --emit` → `prompt resolve --role orchestrator`, CHR-033 seam). Claude/Codex
   `SessionStart` hook: same swap. File fallback for harnesses that can't inject.

## Non-Goals

- **The resolver engine, loader, corpus, and `prompt` verbs** — those are SL-186.
  This slice consumes the locked `prompt resolve` / `model-keys` contract, does not
  build it.
- **Full boot-subsumption** (`prompt resolve --boot/--check` replacing `doctrine boot`).
  This slice *reuses* boot's generator and leaves the verb standing.
- **On-model-change auto-inject** (the true model-band ceiling) — needs a per-harness
  on-change seam; deferred.
- **`{{ resolve }}` injection hole in agent defs** — SL-186 OQ-3 follow-up.

## Verification / closure intent

- **Behaviour-preservation (the gate):** existing boot + dispatch + onboard suites stay
  green; the *only* disk delta is one additive universal-hymns section (goldens churn once).
- Boot golden: universal-hymns section present; model band demonstrably **absent** from
  `boot.md`.
- Onboarding-inline golden: a **shipped** + a **local** `onboarding`-tagged memory both
  inline (INV-D3); local body wins a shared uid; deterministic order; no model content.
- `doctrine_onboard` no longer emits the memory bodies (moved to the sector); emits the
  model band.
- Cache-hold: confirm the injected sector is byte-identical across turns (CHR-033 posture).

## Follow-Ups

- Full boot-subsumption under `prompt` (SL-186 OQ-4).
- On-model-change ceiling per harness.
