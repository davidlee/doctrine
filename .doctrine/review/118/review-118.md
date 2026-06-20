# Review RV-118 — reconciliation of SL-128

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

**Mode:** conformance (post-implementation audit of a dispatched slice).

**Review surface (dispatched-slice rule).** Audit opened against the **candidate
interaction branch** `candidate/128/review-001` (tip `a237b890`) — a clean no-ff
merge of impl bundle `review/128` (`717a8f75`) onto `main` (`19b68131`). The two
fix-now repairs (F-1/F-2) landed as an additive commit on the impl bundle
`review/128` → `72527bbd` (close delivers from `review/<N>`, so the fix must ride
the bundle, not the candidate alone), after which the surface was refreshed to
`candidate/128/review-002` (tip `60b45216`), the post-repair surface this review's
final disposition reflects. `review/128`/`dispatch/128` remain immutable evidence
refs in spirit (R2); the additive repair commit appends, it does not rewrite.

**Lines of attack.** SL-128 lands `[dispatch] deliver_to` as the single trunk-ref
source for two seams (close-integration gate + `dispatch sync` READ) plus a read
verb, retiring the `refs/heads/main` literals. The audit probes:

1. **Behaviour preservation (R3).** Default stays `refs/heads/main`; SL-126's
   `trunk_integration` suites must be green *unchanged*. `--integrate` Option
   semantics untouched (I2) — config must not default the write opt-in.
2. **Layering (ADR-001).** `ledger` stays ref-agnostic; the ref is read in the
   shell and passed down. The new impure reader lives in a neutral module
   (`dtoml.rs`, codex F2), not `slice.rs`.
3. **D3 precedence.** explicit `--trunk` › config › default.
4. **Failure ordering (codex F3).** the gate reads `deliver_to` *inside* its own
   `reconcile→done` branch — no hoisted shared parse that would turn malformed
   TOML into a pre-write refusal on every transition.
5. **Prose hygiene (VT-1 P04).** no delivery-path `refs/heads/main` literal or
   step-3a TODO survives in `close/SKILL.md`.
6. **Project gate.** clippy zero-warn, `cargo fmt` clean, tests green — the full
   `just check`/`gate`, not just the cargo subset the dispatch workers could run.

## Synthesis

**Closure story.** SL-128 lands `[dispatch] deliver_to` as the single trunk-ref
source and retires the `refs/heads/main` literals, with the default unchanged — a
behaviour-preserving change. Conformance against design is clean on every
load-bearing axis:

- **R3 behaviour preservation.** SL-126's `trunk_integration` suites are green
  *unchanged* (the only test edits inline the removed `TRUNK_REF` const as the
  literal `"refs/heads/main"` — mechanical, assertions identical). Default resolves
  to `refs/heads/main` via both the `impl Default` and serde absent-key paths
  (EX-2, VT-1 P01).
- **I2.** `--integrate`'s `--trunk`/`--edge` Option semantics are untouched; only
  the READ verb `--show-journal-trunk-oid` relaxed `requires="trunk"` and defaults
  from config. The write opt-in is never config-defaulted.
- **ADR-001 layering.** `ledger` stays ref-agnostic; the ref is read in the shell
  and passed down. The impure `load_doctrine_toml` lives in the neutral `dtoml.rs`
  (codex F2), and `load_conduct` now delegates to it — one reader, no parallel
  config plumbing (DRY).
- **D3 precedence.** explicit `--trunk` › `deliver_to` › default — verified live
  (`dispatch deliver-to`: default `refs/heads/main`, override `refs/heads/release`).
- **codex F3 ordering.** the gate reads `deliver_to` inside its own `reconcile→done`
  branch; `load_conduct` keeps its own read. `doctrine.toml` is read twice on that
  one transition — a conscious cost (F3) to avoid turning malformed TOML into a
  pre-write refusal on every transition. Not a defect.
- **VT-1 P04.** no delivery-path `refs/heads/main` literal or step-3a TODO survives
  in `close/SKILL.md`; the two remaining mentions are documentary (naming the
  default), not command literals — aligned.

**Findings (both fix-now, both terminal).** Two execution-hygiene gaps, neither
touching behaviour or design:

- **F-1 (minor) — bundle not rustfmt-clean.** `cargo fmt --check` failed on 3
  source files + `e2e_dispatch_sync.rs`. Root cause: dispatch workers cannot run
  `just check` (lint-js node_modules gap), so phases verified via `cargo
  clippy`/`cargo test` directly and `cargo fmt` fell out of the loop. Fixed on the
  bundle (`72527bbd`).
- **F-2 (nit) — stale module doc.** `dtoml.rs` still declared the file read lived
  "in the shell" after this slice co-located the impure reader there. Doc corrected
  to name `parse` as the pure leaf and `load_doctrine_toml` as the thin seam.

**Standing risks / tradeoffs accepted.** None for this slice. The double
`doctrine.toml` read on `reconcile→done` is the one consciously-accepted cost
(F3 ordering). Downstream IMP-129 (edge/main split, default flip, promote
workflow) builds on this config but is explicitly out of scope.

## Reconciliation Brief

Both findings were code-hygiene defects dispositioned **fix-now** and remediated
in-audit on the impl bundle (`review/128` → `72527bbd`); neither implicates
design, ADRs, or specs. There is **nothing for `/reconcile` to write** — no
per-slice design.md prose is stale (design matches implementation), and no
governance/spec REV is warranted.

### Per-slice (direct edit)
- _(none)_ — design.md, plan.toml, and slice-128 prose all match the shipped
  implementation; no drift to reconcile.

### Governance/spec (REV)
- _(none)_ — no ADR/spec/REQ finding. ADR-001 layering and SL-126 gate semantics
  are upheld, not amended.

**Handoff note for `/reconcile`.** The substantive reconcile work is confirming
the rollup and resolving **IMP-124** (the originating improvement, resolved on
close per slice scope §6) and threading the IMP-129 follow-up. No write surface is
owed by the audit findings themselves.
