# deliver_to config as single trunk-ref source

## Context

Origin: IMP-124. The trunk delivery ref for dispatch is hardcoded as
`refs/heads/main` in two places that must agree:

- the **close-integration gate** (SL-126) — `src/slice.rs:447 const TRUNK_REF`,
  fed to `ledger::trunk_integration` on the `reconcile → done` crossing;
- **close skill prose** — `plugins/doctrine/skills/close/SKILL.md` passes
  `--trunk refs/heads/main` to `dispatch sync --integrate` and
  `--show-journal-trunk-oid` (lines 74, 94–95).

SL-126 deliberately took the scope-containing path (design OQ-1 option (b): read
the ref self-describing from the journal trunk row) to avoid opening a config
surface, and flagged a real config slot as the right way for later. The
`TRUNK_REF` doc-comment names IMP-124 as that generalisation. This slice lands
it: a `[dispatch] deliver_to` key in `doctrine.toml`, defaulting to
`refs/heads/main`, consumed as the single source of truth by both seams.

The config table already exists (`src/dispatch_config.rs::DispatchConfig`, wired
through `dtoml.rs` as `doc.dispatch`) — this adds one field and threads it to the
two consumers. With the default unchanged the change is behaviour-preserving.

Downstream: IMP-129 (separate `edge` authoring branch from `main` landing zone)
names this config as its foundation — it later flips the default and adds a
promote workflow. That is a separate drive; this slice ships standalone with the
default at `refs/heads/main`.

## Scope & Objectives

1. **Config field.** Add `deliver_to: String` to `DispatchConfig`
   (`src/dispatch_config.rs`), `#[serde(default = …)]` → `refs/heads/main`.
   Kebab-case key `deliver-to`, default-tolerant, unit-tested alongside the
   existing keys (absent → default; present → override).

2. **Consumer A — close-integration gate (primary, option (a)).** Replace the
   `src/slice.rs` `TRUNK_REF` literal with a read of `doc.dispatch.deliver_to`
   through the existing config-reading shell seam (`run_status` already loads
   `doctrine.toml` for conduct). `ledger` stays ref-agnostic — the ref is still
   passed in.

3. **Consumer B — `dispatch sync` CLI (option (a)).** Make `--trunk` **optional**
   on `dispatch sync` (`--integrate`) and `--show-journal-trunk-oid`; when
   omitted, default to `doc.dispatch.deliver_to`. Update close skill prose to
   drop the `refs/heads/main` literal and the step-3a TODO.

4. **Standalone read verb (option (b), no prose plumbing).** Expose the resolved
   `deliver_to` ref via the CLI (e.g. `doctrine dispatch deliver-to` /
   config-get) so agents doing git by hand can query the trunk ref. Per the
   decision, this is offered as a convenience; close prose does NOT route through
   it (it uses the optional-flag default from (3)).

5. **Resolve IMP-124** on close.

## Non-Goals

- The `edge`/`main` bifurcation, default switch, or `trunk promote` workflow
  (IMP-129) — out of scope; default stays `refs/heads/main`.
- Any change to `ledger`'s ref-agnostic contract or the SL-126 gate semantics
  (it still refuses `reconcile → done` on unintegrated dispatched code) — only
  the ref *source* changes, not the gate behaviour.
- Validating that `deliver_to` names an existing/wellformed ref — git surfaces a
  bad ref at use; no new validation surface here unless design says otherwise.

## Affected Surface

- `src/dispatch_config.rs` — new field + tests
- `src/slice.rs` — `TRUNK_REF` const → config read (gate seam)
- `src/dispatch.rs` / `src/main.rs` — `--trunk` made optional, defaulted from config
- `plugins/doctrine/skills/close/SKILL.md` — drop literal + TODO
- new read verb wiring (location TBD in design)

## Risks / Assumptions / Open Questions

- **R1 — `--trunk` currently required.** Making it optional must not break
  existing callers; confirm the arg-parsing change and the default-resolution
  site in design.
- **A1 — field type is raw `String` ref** (e.g. `refs/heads/main`), matching the
  literal shape, not a typed enum.
- **A2 — single config-read seam.** Both the gate and the sync verb resolve from
  the same `DispatchConfig`; no parallel config plumbing (ADR-001 layering — read
  in the shell, pass down).
- **OQ-1 — verb shape for (b).** Subcommand under `dispatch` vs a generic
  `config get`. Design decides; keep it a thin read.
- **OQ-2 — sync arm precedence.** When `--trunk` IS passed explicitly, does it
  override config, or is the flag retired entirely? (Lean: explicit flag wins,
  config is the default — preserves escape hatch.)

## Verification / Closure Intent

- Unit tests on `DispatchConfig`: absent `deliver-to` → `refs/heads/main`;
  present → override (parity with existing key tests).
- Gate behaviour-preserving: SL-126's existing `trunk_integration` suites stay
  green unchanged with the default ref.
- `dispatch sync` with `--trunk` omitted resolves to config; with it present
  honours the explicit value (per OQ-2 resolution).
- Read verb returns the resolved ref.
- close SKILL.md no longer contains a `refs/heads/main` literal or the step-3a
  TODO; IMP-124 resolved.

## Follow-Ups

- IMP-129 — `edge`/`main` separation builds on this config.
