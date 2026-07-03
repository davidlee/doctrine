# Exposed-slot self-replaces projection

Originates from **ISS-206**. Governing design locked by **REV-019** (done,
approved) against **SPEC-023**. Sequenced **before SL-191**, which hand-authors
`role/worker` overlay content and therefore depends on this mechanism to avoid
the very duplication ISS-206 reports.

## Context

An **exposed** hymn slot is meant to let a user *override* the framework default
via a projected editable starter under `.doctrine/hymns/`. SPEC-023's narrative
promised that override ("the user wins the same-slot tiebreak — the legitimate
customisation"); its mechanism did not deliver it. Provenance in the precedence
key is an **ordering** term, never a **suppression** term (`replaces_suppression`,
`src/hymns.rs`), and `replaces` is the sole suppressor (INV-2). So a
framework snippet and its exposed user twin share slot + selector and **both
emit** — the customisation is *appended to* the thing it meant to replace.

`prompt resolve --role worker` reproduces it: `role/worker` emits twice
(Framework rank=1, User rank=2 ★ WINNER — winner is ordering only).

**REV-019** settled the design: deliver override through the **existing**
suppressor — the projector, when it writes an exposed editable starter, also
writes a sidecar `replaces = <own slot>`. Expose becomes the mirror of seal:

- **seal** → drop the *user* twin before matching (framework wins) — delivered.
- **expose** → keep the user twin; it self-`replaces` the *framework* origin
  (user wins) — **this slice**.

`replaces` stays the only suppressor; the precedence key stays ordering-only;
INV-2/INV-3 are untouched (self-slot replacer is unique-top by provenance, the
self-edge is excluded from the cycle graph, the suppression loop keeps the
carrier — all pressure-tested in REV-019 against unchanged `hymns.rs`).

**Two facts about the current state** shape scope (found in preflight):

1. The projector `project_starters` (`src/install.rs`) is **dead code**
   (`#[expect(dead_code, reason = "reserved: SL-187 …")]`) — never called. SL-187
   is `done` but never wired disk-projection of exposed starters. So the fix has
   no live call site to attach to yet.
2. The defect is **corpus-wide**: all 5 exposed slots (`harness/claude`,
   `model/anthropic/claude-sonnet-4`, `model/deepseek/_default`,
   `role/orchestrator`, `role/worker`) have byte-identical on-disk twins, now
   **tracked/authored** (gitignore fixed to include `.doctrine/hymns`), none
   carrying a sidecar → all double. ISS-206 named only `role/worker` (the one
   visible in the baked worker def). The twins are **orphans** — no live producer
   (`project_starters` is dead); seeded by earlier/removed wiring.

So this slice must both **wire the projector** and **emit the `replaces`
sidecar**, then reconcile the existing twins (backfill their sidecars by running
the wired producer), or the symptom does not clear.

## Scope & Objectives

Realise SPEC-023 REQ-322 / REQ-323 / REQ-329 (already amended by REV-019):

1. **Sidecar on projection (REQ-322).** When the projector writes an exposed
   editable starter to `.doctrine/hymns/<band>/<label>.md`, also write a sidecar
   `<label>.toml` carrying `replaces = "<own slot>"`, so the projected overlay is
   a genuine suppressing override, not an appending twin.

2. **Wire the projector.** `project_starters` is currently uncalled dead code
   (SL-187 leftover). Wire it into the install/forward flow so exposure actually
   projects. Without a live call site, (1) has nowhere to land. (Minimal: the
   fix's home is the projector; wiring it is unavoidable to deliver it.)

3. **Seal/expose symmetry, verified.** Assert single-emit on both sides:
   exposed slot → one emit (user, framework suppressed by self-`replaces`);
   sealed slot → one emit (framework, user twin dropped). Cover unedited
   (byte-identical body → dedup) and edited (`B'` → framework `B` suppressed,
   customisation wins) cases.

4. **Reconcile the existing twins.** Run the wired producer to backfill sidecars
   for all 5 tracked exposed twins (independent write-if-absent preserves the
   `.md`, adds the `.toml`); commit the sidecars. Confirm `prompt explain` shows
   **no** exposed slot doubling.

## Non-Goals

- **Implicit same-slot override** (equal-specificity higher-provenance suppresses
  without `replaces`). REV-019 rejected it — mutates the core compose rule to fix
  a projection-specific problem. Out.
- **The hand-authored-no-sidecar general case.** A user hand-creating an exposed-
  slot snippet without a sidecar still doubles — REV-019's documented known gap,
  follow-up only if demand appears. This slice fixes the *projection* path (and
  the one stray twin), not arbitrary hand authoring.
- **Set-valued trait selection** (SPEC-023 FR-004/5/7/9) — that is **SL-192**.
  This slice is engine-independent of it.
- **Worker-contract hymn content, deepseek patterns, bake generalization, funnel
  check-cadence** — all **SL-191**. This slice only makes exposed projection
  single-emit; it authors no hymn content.
- **No `replaces`-semantics change, no precedence-key change, no new band.** The
  engine (`src/hymns.rs`) is unchanged (REV-019 verified); the change is in the
  projector (`src/install.rs`).

## Affected surface

- `src/install.rs` — `project_starters` (sidecar emission + wire into forward
  flow); sidecar write path.
- `.doctrine/hymns/role/worker.md` (+ new sibling `.toml`) — the stray twin,
  reconciled.
- Tests: install/projection unit tests; resolver seal/expose symmetry
  golden/e2e (`prompt resolve --role worker` single-emit).

## Risks / Assumptions / Open Questions

- **OQ — projector wiring scope.** Wiring `project_starters` is arguably
  unfinished **SL-187** delivery, not new ISS-206 work. Minimal wiring (project
  exposed starters at install/forward) is in scope because the fix cannot land
  without a call site; broader SL-187 projection concerns stay out. Confirm the
  minimal cut in `/design`.
- **OQ — sidecar vs existing content on re-project.** `project_starters` skips
  `dest.exists()`. Decide whether re-projection backfills a missing sidecar for an
  already-projected/edited starter, or only writes the pair on first projection.
  Bears on whether the stray twin is auto-fixed or must be removed. `/design`.
- **ASM — engine unchanged.** REV-019 pressure-tested self-`replaces` against
  `hymns.rs`; behaviour-preservation gate is the existing resolver suite staying
  green.
- **ASM — sidecar schema already supports `replaces`.** `Sidecar.replaces`
  (`src/install.rs`) + `parse_slot_ref` exist; no schema change needed.
- **RSK — crane embed-strip.** No new embed root here (sidecars are *written to
  disk*, not embedded), so the nix hollow-asset risk does not apply; confirm.

## Summary

Make **expose** the single-emit mirror of **seal**: the install projector writes
a `replaces = <own slot>` sidecar alongside each exposed editable starter, so a
user overlay suppresses its framework origin through the existing `replaces`
mechanism — no engine change. Wire the currently-dead projector so the fix has a
call site, and reconcile the one stray hand-authored twin so
`prompt resolve --role worker` emits once. Precondition for SL-191's overlay
authoring.

## Follow-Ups

- **SL-191** rides this — its `role/worker` overlay authoring composes cleanly
  only once exposed projection is single-emit.
- **Known gap (REV-019):** hand-authored exposed-slot snippet without a sidecar
  still doubles. File as an issue if demand appears; the implicit-override fix is
  the only thing that would cover it, and it is rejected.
