# Audit SL-024 — Harden TOML render: escape user free-text through a shared seam

**Mode:** conformance (post-implementation, two phases shipped).
**Date:** 2026-06-08.
**Against:** `design.md` (locked, inquisition-amended A2), `plan.toml`
(PHASE-01/02 EN/EX/VT), ADR-001 (layering), the storage rule, and the
behaviour-preservation gate.

## Evidence

- `cargo clippy` (bins/lib) — **zero warnings**, exit 0.
- `cargo test --bins` — **555 passed, 0 failed**.
- `just check` — exit 0.
- Phase state: PHASE-01 `completed`, PHASE-02 `completed`; `slice phases 024`
  → "Phases up to date"; rollup **2/2**.
- Routing grep: every `render_*_toml` `title`/`slug` splice goes through
  `tomlfmt::toml_string`; the only residual raw `.replace("{{title}}", title)`
  sites are the `render_*_md` body renderers (markdown, unparsed — Non-Goal).

## Findings

### F-1 — PHASE-01 leaf extraction · ALIGNED
**Expected** (EX-1..4): `src/tomlfmt.rs` holds `toml_string`/`toml_array_inner`,
both `pub(crate)`, bodies byte-identical to the memory.rs originals, importing
only the `toml` crate (leaf, ADR-001); `main.rs` declares `mod tomlfmt;`; memory's
two private copies deleted and re-imported; no other module touched.
**Observed:** all present. `src/tomlfmt.rs` imports only `toml`; the two fns are
the verbatim move (D1); `main.rs:24` declares the module; `memory.rs:32` imports
the seam and every call site (`:597/:627/:628/:630–641`) resolves to the leaf.
ADR-001 satisfied — a pure leaf depended on by six command-tier modules, no cycle.
**Disposition:** aligned. No follow-up.

### F-2 — Behaviour-preservation gate (memory byte-identical) · ALIGNED
**Expected** (VT-2, R3, D1): memory's output stays byte-identical; its suite green
**unchanged**. **Observed:** `memory.toml` was already self-quoting, so no template
changed; the escaper moved verbatim with visibility raised only. Memory's suite is
green within the 555. Byte-identity holds by construction (verbatim move + no
template delta). **Disposition:** aligned.

### F-3 — PHASE-02 corpus routing · ALIGNED
**Expected** (EX-1..3): seven templates converted to bare self-quoting tokens
(memory.toml already bare → eight total); five renderers route `title`+`slug`
through the seam, each template+renderer edit a lockstep pair; no raw TOML-literal
splice of user free-text remains. **Observed:** all eight `*.toml` templates carry
bare `slug = {{slug}}` / `title = {{title}}`; zero residual `"{{title}}"`/
`"{{slug}}"` quoted tokens; the five renderers route both fields; the only raw
`.replace` splices left are MD bodies (correct — storage rule). No `""value""`
half-applied edit (R1) anywhere. **Disposition:** aligned.

### F-4 — VT mechanism: new focused tests vs. "extend the four" · ALIGNED (mechanism drift, intent met)
**Expected** (VT-2/VT-3): extend the four existing direct round-trip tests
(`adr`/`slice`/`requirement`/`backlog`) with hostile input, and add one **new**
direct-render test for `spec` (which has no existing direct round-trip; the disk
path via `fresh` would false-red at `<id>-<slug>` symlink creation — inquisition
Charge 1). **Observed:** implementation added **five** new focused
`render_*_toml_escapes_hostile_title_and_slug` tests (one per renderer, including
spec's direct `render_spec_toml` call) rather than extending the four existing
round-trip tests in place. The prescribed *evidence* — adversarial value
re-parses via `meta::Meta` and round-trips verbatim, per renderer — is fully
present; spec correctly calls the private renderer directly, not the disk path.
**Disposition:** aligned. A separate focused test per renderer is better cohesion
than overloading the existing round-trip assertions; the VT intent is discharged.
The drift is mechanism, not coverage.

### F-5 — `]`-is-not-a-quoted-literal-breaker · ALIGNED
**Expected** (design §5.5 / A1): the hostile driver must contain `"` (and ideally
`\`/newline); a `]`-only red is green-already (false red); `]`/`,` breakout is
tested on `toml_array_inner` only. **Observed:** all five renderer tests drive
`a"b\c\nd` / `p"q` (contain `"`,`\`,newline, no bare `]`); `tomlfmt`'s array test
carries `]`/`,`/`"`/newline breakers. The loose `]`-among-title-breakers language
in `slice-024.md` was corrected during the code-review pass (commit 58b1c24).
**Disposition:** aligned.

### F-6 — Lifecycle status divergence (`slice list` ⚠) · HANDOFF TO /close
**Expected:** `slice-024.toml` `status` reflects reality. **Observed:** `slice
list` shows `024 proposed ⚠ 2/2` — the hand-edited `proposed` diverges from the
2/2 phase rollup; ⚠ is SL-009 surfacing exactly the no-lifecycle-transition gap.
**Disposition:** follow-up — reconcile in `/close` (advance `status` to the
terminal value). Not a defect of this slice's implementation.

### F-7 — Slug wound left open (OQ-1) · FOLLOW-UP SLICE (IMP-005 filed)
**Expected** (design OQ-1, scope Q3): escaping secures storage; `--slug`
normalisation is a separate, deferred policy. **Observed:** render-escape closes
the **TOML wound** but not the **slug wound** — a hostile explicit `--slug` now
round-trips the TOML cleanly yet still reaches the `<id>-<slug>` symlink filename
unnormalised. This is a **live risk carried out of SL-024**, not a closed one.
**Disposition:** follow-up slice — **IMP-005** filed
(`normalise-explicit-slug-like-a-derived-slug-at-entity-new`), linked → SL-024,
with the policy choice (silently normalise vs. reject) called out.

### F-8 — `state.rs` phase-sheet splice (OQ-2) · TOLERATED DRIFT
**Expected** (design OQ-2): the runtime phase sheet splice deferred, not folded
in. **Observed:** `state.rs:336` splices `{{name}}`/`{{objective}}` raw — but into
`templates/phase.md` (**markdown**, not the TOML sheet the design framed), so it
is doubly out of scope (gitignored disposable runtime state *and* an unparsed MD
body). Lower stakes than the design implied. **Disposition:** tolerated drift —
consciously deferred, disposable state, no parse risk. Not worth a backlog item;
revisit only if a phase TOML sheet ever moves to template-splice rendering.

## Durable harvest

- The shipped pattern memory `mem.pattern.render.toml-splice-escape-user-values`
  (SL-020 capture) now has its corpus-wide fix landed. Durable refinement worth
  promoting at `/close`: **render-escape secures the TOML *document*, not a
  value's downstream filesystem use** — a `slug` that re-parses cleanly can still
  be hostile as a path component. Captured operationally in IMP-005.

## Closure readiness

All EN/EX/VT criteria for PHASE-01 and PHASE-02 are aligned. Two findings route
forward by design (F-7 → IMP-005; F-8 tolerated). One reconciliation remains for
`/close` (F-6 lifecycle status). Audit-ready.
