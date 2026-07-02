# Design SL-187: Prompt cascade — per-harness delivery & boot integration

<!-- Consumes SL-186's locked contract. Reference forms: entity ids padded
     (SL-186, IMP-155, CHR-033); doc-local refs bare — D1 (§7), R1 (§8),
     INV-D1 (§5.5). "F<n>" cite the codex findings logged in SL-186 §10 +
     this slice's §9. -->

## 1. Design Problem

SL-186 ships the resolver **engine + `prompt` verbs** — inert, no caller. This
slice **delivers** that output to live agents at session start, and decides which
*cache tier* each band lands in. The token win IMP-155 targets (orchestrators stop
hand-assembling worker context) is realised here.

The hazard is blast radius: every change here touches a **live, shared bootstrap
surface** — `.doctrine/state/boot.md` (every session, every harness), the
`doctrine_onboard` MCP tool (every MCP agent), the shipped pi extension, the
memory corpus. The behaviour-preservation gate is therefore the spine of the design.

## 2. Current State

- **`src/boot.rs`** — `doctrine boot` regenerates `.doctrine/state/boot.md` (pure
  governance projection, content-diff cache key via `write_if_changed`); `boot --emit`
  emits the exact bytes to stdout. `enum Harness { Claude, Codex }` + `import_targets`
  (`CLAUDE.md`/`AGENTS.md` `@`-import) + `SessionStart` hook wiring. `generate_pi_extension()`
  (~L1703) emits the pi `before_agent_start` extension. Governance sections are
  entity-derived; the "Onboarding" section is a user-authored footer (`SourceKind::Footer`,
  `.doctrine/boot-footer.md`) that *names* `overview`+`orientation` and says "load next turn."
- **CHR-033 (closed 2026-07-02)** — the pi extension already stdout-emits `doctrine boot
  --emit` via `before_agent_start`, injects byte-identical per turn (**Anthropic prefix
  cache holds across turns**), SYSTEM_APPEND.md removed. Names the SL-187 seam: swap the
  exec'd command to `doctrine prompt resolve <role>`.
- **`memory::collect_all`** (memory.rs ~L2736) — unions `items/` (local) ∪ `shipped/`
  (derived, SL-018/ADR-002), deduped by uid with **local winning**; `filtered_list` layers
  the standard tag filter. So `onboarding`-tag selection across shipped+local needs **no new
  union code**.
- **`doctrine_onboard`** (MCP) — loads the two onboarding memories today.
- **SL-186** — provides `doctrine prompt resolve --role <r> [--harness --model --arm --stage
  --band]` (stdout composition) + `prompt model-keys`. This slice consumes that contract.

## 3. Forces & Constraints

- **ADR-011** — per-harness capability altitude (floor/ceiling); mechanism in a CLI verb,
  harness-identical. **Behaviour-preservation gate** — boot/dispatch/onboard suites stay
  green; only additive deltas. **POL-002** — platform independence.
- **CHR-033 posture** — session-start injection must stay byte-identical per turn to hold
  the prefix cache.
- **Contract dependency, not build dependency** — SL-186's `prompt resolve` interface is
  locked in design; this slice builds against it in parallel (dispatch).

## 4. Guiding Principle — split by cache property

The one idea the whole design turns on: **content that is stable + model-agnostic rides
the token cache; content that is mutable (model-specific) rides the path that busts cache
anyway.** Never put churny content on the cached path, never pay a tool-call round-trip for
stable content.

```
CACHE-STABLE boot sector   MODEL-AGNOSTIC — survives /model swaps
  governance + universal hymns + inlined onboarding memories
  tier-1  SessionStart hook stdout / pi before_agent_start   (cache-hold: probed + CHR-033)
  tier-2  @-import universal boot.md                          (universal-only fallback)
CACHE-BUSTING supplement   MODEL-SPECIFIC
  doctrine_onboard MCP tool = model identification + model band
```

## 5. Proposed Design

### 5.1 System Model — two channels

- **Cache-stable boot sector.** The universal, model-agnostic composition. Delivered
  tier-1 as the per-harness session-start **stdout** stream (`prompt resolve --role
  orchestrator --harness <h>` → `universal snapshot ++ session-stable harness/role hymns`);
  tier-2 as the `@`-imported on-disk `boot.md` (universal-only, for harnesses that can't
  inject). `model` is excluded **on purpose** — it is the one axis that changes mid-session.
- **Cache-busting supplement.** `doctrine_onboard` (MCP): model identification + the model
  band. A tool call busts cache regardless, so dynamic model content is free here.

**Why the cached path is genuinely cached (F10).** Not asserted from docs — **CHR-033 already
ships on exactly this**: the pi extension injects a byte-identical string every turn and its
own notes record that the Anthropic prefix cache holds across turns. The claim is probed and
in production, not inferred. Requirement inherited: the sector must stay byte-identical per
turn (INV-D1/D2 guarantee this — it only changes when a committed input changes).

### 5.2 Interfaces & Contracts

**`prompt resolve` (consumed from SL-186) — delivery behaviour added here:**

```
doctrine prompt resolve --role <orchestrator|worker>
                        [--harness <name>] [--model <id>] [--arm <subagent|subprocess>]
                        [--stage <skill/verb>] [--band <name>]...   (--band repeatable)
  → disk:   regenerate the UNIVERSAL boot.md (reuse boot generator, write_if_changed);
            AXIS-INVARIANT — no axis flag alters the on-disk artifact (SL-186 §5.2 INV-7:
            "flags never alter the on-disk artifact"); INV-D1 restates it for this slice.
    stdout: <universal snapshot> ++ <role/harness hymns>.  Idempotent.
(also consumed: `prompt model-keys [--harness <name>]` — the exact `--model` key strings.)
```

**Boot generator (`src/boot.rs`):**
- **Expose** the universal-snapshot generator so `resolve` reuses it (no second projector).
- **Add** a `universal`-band hymns section to the disk snapshot (harness-agnostic authored
  prose, incl. the model-band floor directive). The **entity-derived** producer sections and
  their ordering are untouched; this is a *purely additive* section for those.
- **Change the Onboarding section (not additive — a contract change).** Today
  `("Onboarding", SourceKind::Footer)` sits in the core section table (`boot.rs:104-132`),
  rendered by the `Footer` arm from `.doctrine/boot-footer.md` (`boot.rs:292-297`). This slice
  **replaces** that footer's signpost instruction with the memory bodies inlined in place —
  mutating that one section's producer, not the entity-derived sections. The boot golden for
  the Onboarding section therefore changes **by intent** (see §9); "assembly logic untouched"
  applies only to the entity-derived sections above, not to this substitution.
- **Inline** `onboarding`-tagged memory bodies (via `collect_all` + tag filter). Order:
  **key-ascending with a uid fallback for keyless memories** — reuse the existing `boot_keys`
  sort pattern (`memory.rs:2830`), don't mint a second ordering rule.
- Harness/model/role/stage bands are **never** added to the disk snapshot — stdout-only.

**pi extension (`generate_pi_extension()`) + Claude/Codex `SessionStart` hook:**
- One-line command swap: `doctrine boot --emit` → `doctrine prompt resolve --role
  orchestrator [--harness <h>]` (CHR-033 seam). Byte-identical per turn preserved.

**`doctrine_onboard` (MCP) — a deliberate contract change (not additive):**
- **Mechanism (F-5).** The tool takes no arguments and receives no model id (`tools.rs:327-333`),
  so it cannot *read* the client's model. It instead **teaches self-identification**: emits the
  `prompt model-keys` set + the standing directive for the agent to resolve `prompt resolve
  --band model --model <id>` itself. Doctrine never resolves the model band on the agent's behalf.
- **Drops** the two-memory load (moved into the cached sector). This **removes** the current
  `Onboarding Memories` section — a contract subtraction, not an extension. The onboard e2e
  assertion on that section (`tests/e2e_mcp_server.rs:1083`) changes **by intent** (§9).

### 5.3 Data, State & Ownership

| Surface | Owner | Consumer | When |
|---|---|---|---|
| `.doctrine/state/boot.md` (universal sector) | boot generator | tier-2 `@`-import / stdout base | session start / any resolve |
| session-start stdout stream | `prompt resolve` | tier-1 harness injection | session start |
| `onboarding` tag | framework (shipped memories) + user | boot inline query | boot regen |
| model band | `doctrine_onboard` / agent self-resolve | agent, mid-session | on demand |

- Onboarding designation = the **`onboarding` tag** (checked: ADR-002 orientation class is
  too broad — it covers all ~24 signposts; the footer id-list is unstructured). Tag unions
  shipped+local via `collect_all` (INV-D3).

### 5.4 Lifecycle & Dynamics

**Unstale (INV-D2).** Every `prompt resolve` regenerates the universal disk snapshot
(`write_if_changed`) — unconditionally, but idempotently: the snapshot is a **deterministic
projection of committed inputs**, so it is a no-op unless an input actually changed. A
worker-spawn resolve therefore rewrites nothing under stable governance; when governance *did*
change, refreshing the shared fallback for the next reader is correct, not "perturbing."

**Concurrency (F12) — benign, self-healing, no lock.** `write_if_changed` is a lock-free
read-compare-atomic-rename (`write_atomic` = "last rename wins"). A stale writer that reads
old inputs and lands last leaves `boot.md` **at most one resolve-cycle behind** — never torn
(atomic rename), and self-healed by the very next resolve (every spawn / session start
re-derives from current inputs). `boot.md` is a **convenience projection, not authority**
(entity files are truth), so a sub-second lag during active human governance-editing is not a
correctness defect. A lock would over-engineer a self-healing cache. (Optional paranoid guard:
skip the write when disk mtime is newer — deferred, YAGNI.)

**Tier-2 tolerance is explicit, not accidental (F-3).** A *non-injecting* harness reading the
disk `boot.md` during the ≤1-cycle stale window gets governance at most one resolve-cycle old.
This is a **consciously accepted tolerance** (D8), not a supported-path guarantee: tier-2 is
best-effort universal-only fallback for harnesses that cannot inject; the stale window is
bounded, self-heals on the next session, and no correctness invariant rests on tier-2 freshness
(entity files are authority). Injecting harnesses (tier-1) never read the disk artifact for
delivery, so they are unaffected.

**Model band — floor + supplement, no true ceiling (F14).**
- **Floor (both tiers):** the universal-band standing directive — self-identify, `prompt
  resolve --band model --model <id>`, re-resolve on change. Best-effort, degrades gracefully.
- **Supplement (better floor, NOT a ceiling):** `doctrine_onboard` hands the agent the
  `model-keys` set and the resolve directive so the agent self-identifies (the tool has no
  model id to read — F-5). Honest correction to the earlier "ceiling" label — a tool the agent
  *may* invoke is not a guaranteed mechanism; it is a nicer-UX floor. A **true** ceiling
  requires the harness to *mechanically* inject the model band, and even a session-start hook
  is not a *mid-session* ceiling (model is mutable, hook fires once/session). No correctness
  invariant rests on the model band regardless — it is fine-tuning; stale-tuning ⊆ graceful
  degradation.

**Onboarding inline (INV-D3).** Boot selects `onboarding`-tagged memories via `collect_all`
(items ∪ shipped, local-wins) and inlines their bodies into the cached sector, ordered
key-ascending with a uid fallback for keyless memories (the `boot_keys` pattern). Collision (a
local memory sharing a shipped uid) resolves local-wins **by design** — but note `collect_all`
skips the shipped twin **silently** (`memory.rs:2736`: no debug-log facility, print is denied),
so this is not observable at runtime. Observability, if wanted, is a `doctrine check` collision
diagnostic (D7); this design documents the behaviour rather than claiming runtime visibility.
The set is small-by-construction (human-tagged), with a soft warning budget (D6).
`doctrine_onboard` correspondingly sheds the memory load.

### 5.5 Invariants & Edge Cases

- **INV-D1** The on-disk `boot.md` is **always the universal, model-agnostic composition**
  (governance + universal hymns + inlined `onboarding` bodies) — **axis-invariant**:
  `--role`/`--harness`/… extend the *stdout* stream only, never the disk artifact. Rationale:
  the shared `@`-import contract **and** cache stability (model content would churn the prefix
  cache on `/model`; it rides the cache-busting `doctrine_onboard` path instead).
- **INV-D2** Every `prompt resolve` regenerates the universal snapshot (`write_if_changed`),
  unconditionally but idempotently (deterministic projection of committed inputs). Not a lock;
  stale-by-at-most-one-cycle under concurrency, self-healing; `boot.md` is projection, not
  authority.
- **INV-D3** `onboarding`-tag selection unions shipped + local (`collect_all`, local-wins),
  ordered key-ascending with uid fallback for keyless (the `boot_keys` pattern); collision is
  intended local-wins (skipped silently at the `collect_all` layer, documented here, not
  runtime-observable — D7); model-agnostic (no model content ever inlines).
- **INV-D4** Session-start injection is byte-identical per turn (CHR-033 cache-hold posture);
  it changes only when a committed input changes.
- **Edge — non-injecting harness:** tier-2 `@`-import gets universal governance + universal
  hymns only (no harness hymns) — accepted degradation (the shared file can't carry
  harness-specific prose).

## 6. Open Questions

- **OQ-2 — Claude/Codex hook: single combined emit vs two.** One `prompt resolve` call (base
  from the reused generator) vs chaining. Leaning single (matches the pi one-liner). Safe to
  settle in `/plan` — both shapes hit the same seam; no design fork.

(OQ-1 resolved → D6. The remaining OQ is a plan-level shaping detail, not a design fork.)

## 7. Decisions

- **D1 — Split by cache property; model off the cached sector.** The cached boot sector is
  model-agnostic so it survives `/model` without busting the prefix cache; model-specific
  content rides the cache-busting `doctrine_onboard`. Rejected: model on the cached sector
  (churns cache every model change); a baked per-harness `boot.md` tail (harness prose on a
  file two harnesses `@`-import).
- **D2 — Unconditional idempotent unstale; no lock.** `write_if_changed` + deterministic
  projection makes unstale free under stable governance and self-healing under concurrency
  (F11 dismissed: worker-spawn rewrite is a no-op unless inputs changed, and correct when they
  did; F12 → wording precision, not a lock).
- **D3 — Onboarding memories inlined into the cached sector via an `onboarding` tag over
  `collect_all`.** Single-source (memories stay the source); retires the footer round-trip.
  Rejected: ADR-002 orientation class (too broad); footer id-list (unstructured).
- **D4 — `doctrine_onboard` = model floor-supplement, not a ceiling (F14).** Honest
  terminology; no correctness rests on the model band. True mid-session ceiling deferred
  (needs an on-model-change seam).
- **D5 — pi/hook delivery is a one-line command swap (CHR-033 seam).** Reuse the existing
  stdout-inject extension; don't author a new mechanism.
- **D6 — Onboarding-inline budget: soft warning, no hard cap (was OQ-1, F-7).** The set is
  small-by-construction (human-tagged); a hard byte/count refusal would over-engineer. A
  `doctrine check` warning at a threshold N is the ceiling. `/plan` inherits this stance, not
  an open question.
- **D7 — Collision visibility is documentation + an optional `doctrine check` diagnostic, not
  runtime (F-6).** `collect_all` skips shipped uid-twins silently and the repo denies print;
  local-wins is documented here and, if surfaced, via a `doctrine check` collision warning —
  the design does not claim runtime observability it cannot deliver.
- **D8 — Tier-2 disk fallback is best-effort, stale-window tolerated (F-3).** A non-injecting
  harness may read ≤1-cycle-old governance from disk `boot.md`; bounded, self-healing, no
  invariant rests on it. Consciously accepted, not a freshness guarantee. Rejected: a lock or
  version guard (over-engineering a self-healing convenience projection).

## 8. Risks & Mitigations

- **R1 — Boot regression** (live surface). *Mit:* boot's generator is *reused, not rewritten*;
  the **entity-derived** sections + their ordering are untouched (those goldens stay green). Two
  intentional deltas: an additive universal-hymns section, and the Onboarding footer→inline
  substitution (§5.2). The Onboarding + universal-hymns goldens churn **once, by intent** — this
  is not "suites green unchanged" for boot; it is scoped, reviewed golden churn (§9).
- **R2 — Cache-hold broken** by non-deterministic injection. *Mit:* INV-D2/D4 keep the stream
  byte-identical per turn; CHR-033 already validates the posture; a golden asserts stability.
- **R3 — Onboarding-inline bloat / surprise.** *Mit:* deterministic order, documented
  local-wins, small-by-construction; optional budget (OQ-1). A user tagging a huge memory is a
  `doctrine check` warning candidate.
- **R4 — Onboard contract change** for existing MCP agents. This is a **deliberate contract
  change, not additive**: it adds the model-band self-ID guidance and **removes** the
  `Onboarding Memories` section (moved to the cached sector). *Mit:* the removed payload is not
  lost — it relocates to the always-present cached sector; the onboard e2e assertion on that
  section (`tests/e2e_mcp_server.rs:1083`) is **updated by intent** (§9), not preserved.

## 9. Quality Engineering & Validation

- **Behaviour-preservation gate — scoped honestly.** *Stays green unchanged:* the **dispatch**
  suite, and boot's **entity-derived** section goldens (governance projection + ordering). *Changes
  by intent (reviewed golden/assertion churn, not regression):* (a) boot's **Onboarding** golden
  (footer instruction → inlined memory bodies) and a new **universal-hymns** golden; (b) the
  **onboard** e2e assertion `tests/e2e_mcp_server.rs:1083` (the `Onboarding Memories` section is
  removed). The gate is: no *unintended* delta beyond these three enumerated changes; model band
  demonstrably absent from `boot.md`.
- **Onboarding inline (INV-D3):** golden with a **shipped** + a **local** `onboarding`-tagged
  memory — both bodies inline, key-ascending order with uid fallback; a local memory sharing a
  shipped uid inlines the local body; no model content.
- **Onboard:** `doctrine_onboard` emits the model-band self-identification guidance (offers
  `model-keys` + resolve directive), no longer the memory bodies; its e2e assertion is updated
  to the new contract.
- **Cache-hold (INV-D4):** the injected sector is byte-identical across two turns on unchanged
  inputs.
- **pi/hook swap:** the emitted extension/hook command is `prompt resolve --role orchestrator`;
  session start still injects.

## 10. Review Notes

Carved from SL-186 after the codex re-pass (SL-186 §10). Dispositions on the delivery-half
findings (F10–F16):
- **F10 (cache claim unproven)** — **dismissed**: CHR-033 ships the byte-identical-per-turn
  cache-hold; user has probed it. Cited, not hedged (§5.1).
- **F11 (unstale buys nothing / perturbs)** — **dismissed**: deterministic projection +
  `write_if_changed` ⇒ no-op unless inputs changed; correct when they did (D2/§5.4).
- **F12 (concurrency race)** — **accepted as wording precision**: not a lock; self-healing,
  stale-by-≤1-cycle, projection-not-authority (INV-D2/§5.4).
- **F13 (inline guardrails)** — **accepted**: deterministic order, documented local-wins,
  small-by-construction + optional budget (INV-D3, R3, OQ-1).
- **F14 (onboard "ceiling" category error)** — **accepted**: relabelled floor-supplement;
  true ceiling needs mechanical inject (D4/§5.4).
- **F16 (scope creep)** — **accepted**: this slice *is* the split — the live-surface delivery
  half, separated from SL-186's inert engine on a blast-radius boundary.

**Codex inquisition on this (delivery) design — RV-210, 2026-07-02.** 7 charges, all disposed
`design-wrong` and reconciled into this artifact (no code defect — the design *prose* was the
heresy):
- **F-1** (contract quote narrowed axis-invariance) → §5.2 quotes SL-186's full flag list +
  "flags never alter the on-disk artifact" verbatim; `model-keys` added.
- **F-2** (footer→inline mislabelled "additive/untouched") → §5.2, R1: reframed as an
  intentional Onboarding-section contract change; "untouched" scoped to entity-derived sections.
- **F-3** (tier-2 stale-window implicit) → D8 + §5.4: explicit accepted best-effort tolerance.
- **F-4** (onboard memory-drop breaks e2e; "green unchanged" false) → §9, R4: reframed as a
  deliberate contract change; the e2e assertion is updated by intent, not preserved.
- **F-5** (onboard model-id mechanism fictional) → §5.2, §5.4: tool has no model id; it teaches
  self-identification (emits `model-keys` + directive), never resolves the band itself.
- **F-6** (silent collision + unsorted scan vs deterministic/observable claims) → §5.2/§5.4/
  INV-D3/D7: order = key-else-uid (`boot_keys` pattern); collision documented, not claimed
  runtime-observable.
- **F-7** (byte budget a bare OQ) → D6: soft `doctrine check` warning, no hard cap.

## Code Impact (design-target)

- **`src/boot.rs`** — expose universal-snapshot generator; add universal-hymns section; inline
  `onboarding`-tagged memory bodies (retire footer instruction); pi extension + `SessionStart`
  hook command swap to `prompt resolve`.
- **`doctrine_onboard` MCP handler** — emit model band; drop memory load.
- **Memory data** — tag shipped `overview` + `orientation` `onboarding` (`install/memory/**`).
- **Tests** — boot goldens (universal-hymns section, Onboarding footer→inline, model-band
  absent), onboarding-inline golden (shipped+local, key-else-uid order), **updated** onboard
  e2e (`tests/e2e_mcp_server.rs:1083`: `Onboarding Memories` removed, model-band guidance
  present), cache-hold stability, pi/hook command.
