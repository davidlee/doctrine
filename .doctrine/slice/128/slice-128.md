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

3. **Consumer B — `dispatch sync` READ stage only (α hybrid).** Relax
   `--show-journal-trunk-oid`'s `requires="trunk"` so it defaults `--trunk` from
   `deliver_to`. **`--integrate` is left UNCHANGED** — its `--trunk`/`--edge`
   Option semantics are load-bearing (absent `--trunk` = edge-only projection, a
   live tested path); config must NOT default the write opt-in.

4. **Read verb `doctrine dispatch deliver-to` (option (b)).** Thin stdout read of
   the resolved `deliver_to`. Load-bearing, not merely a convenience: it is how
   close's `--integrate` *write* line names the trunk without a literal (since (3)
   cannot overload absent-`--trunk`), and it also serves hand-driven git work.

5. **Close prose.** Drop ALL delivery `refs/heads/main` literals (codex F1):
   `candidate create --base` (line 68), `--integrate --trunk` (74), the verify
   read (95) and `git diff` compare (96) — each via the verb or config default —
   plus the explanatory text (102) and step-3a TODO (105–107). The git.rs fork-base
   *auto-resolver* (concept #1) has no literal here; it stays sealed.

6. **Resolve IMP-124** on close.

## Non-Goals

- The `edge`/`main` bifurcation, default switch, or `trunk promote` workflow
  (IMP-129) — out of scope; default stays `refs/heads/main`.
- Any change to `ledger`'s ref-agnostic contract or the SL-126 gate semantics
  (it still refuses `reconcile → done` on unintegrated dispatched code) — only
  the ref *source* changes, not the gate behaviour.
- Validating that `deliver_to` names an existing/wellformed ref — git surfaces a
  bad ref at use; no new validation surface here unless design says otherwise.

## Affected Surface

- `src/dispatch_config.rs` — new `deliver_to` field, hand `impl Default`, tests
- `src/dtoml.rs` — neutral impure `load_doctrine_toml` reader (codex F2); drop the
  `expect(dead_code)` on the `dispatch` field (R5); round-trip test
- `src/slice.rs` — `TRUNK_REF` const → config read inside the `reconcile→done`
  gate branch (codex F3 ordering); `load_conduct` → delegating wrapper
- `src/main.rs` / `src/dispatch.rs` — `--show-journal-trunk-oid` default from
  config (relax `requires="trunk"`); `--integrate` UNCHANGED; new `dispatch
  deliver-to` verb
- `plugins/doctrine/skills/close/SKILL.md` — drop all delivery literals + TODO

## Risks / Assumptions / Open Questions

- **R1 — `--trunk` currently required.** Making it optional must not break
  existing callers; confirm the arg-parsing change and the default-resolution
  site in design.
- **A1 — field type is raw `String` ref** (e.g. `refs/heads/main`), matching the
  literal shape, not a typed enum.
- **A2 — single config-read seam.** Both the gate and the sync verb resolve from
  the same `DispatchConfig`; no parallel config plumbing (ADR-001 layering — read
  in the shell, pass down).
- **OQ-1 — verb shape (RESOLVED, design D5).** `doctrine dispatch deliver-to`
  subcommand, not a generic `config get`.
- **OQ-2 — precedence (RESOLVED, design D3).** Explicit `--trunk` › `deliver_to`
  config › default; `DOCTRINE_TRUNK_REF` env stays base-only (it resolves a
  commit-ish; delivery needs a writable ref).
- **R5 — `expect(dead_code)` on `DoctrineToml.dispatch`** fires once read live;
  remove the attr when wiring the gate (caught in adversarial review).

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
- PR/remote **delivery mode** — a future `[dispatch] delivery-mode = "merge" |
  "pull-request"` (+ remote/refspec); `deliver_to` becomes the PR base. The gate's
  "integrated?" predicate goes async there.
- **Base+delivery unification** — folding `deliver_to` into the fork-base
  auto-resolver (concept #1, `src/git.rs::trunk_tree_ish` / the
  `origin/HEAD→main→master` ladder) is an ADR-006 D3 amendment; deferred to
  whoever wants one trunk identity (likely IMP-129).
