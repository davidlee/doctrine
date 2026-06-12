# Review RV-011 — reconciliation of SL-053

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Conformance reconciliation of SL-053 (terminal output polish: comfy-table
listings + owo_colors) against `design.md` (D1–D7, F-1…F-8), `plan.toml`
(PHASE-01/02/03 EN/EX/VT), and the pure/imperative + behaviour-preservation
governance. Lines of attack:

- **Determinism (D6)** — is piped output byte-stable terminal-vs-pipe? Does
  `render_table` actually call `force_no_tty()`, and is the `custom_styling`→`tty`
  edge neutralised?
- **Line shape (D7)** — no leading space, no trailing whitespace on every line;
  is the `NOTHING`-preset trap avoided?
- **Purity (D3)** — does colour capability stay in the impure shell with only a
  `bool` crossing into the pure render layer? No `if_supports_color`?
- **F-4** — does `ByValue` read the row's semantic status, never the decorated
  emitted cell (slice `done ⚠`, review `open (await …)`)?
- **Piped-plain invariant** — do the black-box goldens stay green UNCHANGED after
  PHASE-02 (colour must not leak into pipes)?
- **Surface coverage** — all 8 colour+layout kinds painted; priority left
  monochrome; signatures unchanged for direct `render_table` callers.

## Synthesis

**Verdict: audit-ready, no blocker.** Every PHASE-01/02/03 exit and verification
criterion is met; `just check` is green (verified on the combined coordination
tree after each phase import, plus a touch+re-run to defeat the shared-jail-target
false-green). The four findings are all design-doc reconciliations — the code is
correct throughout; `design.md` was the lagging artifact in two of them.

**What landed.** `render_table` now delegates all layout/measurement to
comfy-table (`ContentArrangement::Disabled` + `force_no_tty()` — D6, byte-stable
terminal-vs-pipe, spike-proven), minimalist ` │ ` interior separators with no
outer frame/rules, and the old hand-rolled width/pad maths deleted — the single
layout authority. Colour rides strictly behind the `render_columns` seam:
capability resolved once in the impure shell (`tty::stdout_color_enabled`,
`var_os`+isatty) and injected as a `bool` through `ListArgs` (D3 — no
`if_supports_color`); `ColumnPaint<R>` (`Fixed` id hues, `ByValue` status-by-row),
one shared `status_hue`, bold headers; 8 kinds painted, priority left monochrome.
Piped output stays byte-for-byte plain (colour defaults false), so all goldens —
black-box and in-crate — stayed green UNCHANGED in PHASE-02. Follow-ups captured:
IMP-039 (deferred-colour surfaces) and IMP-040 (`--color` flag, D5).

**Findings (4, all minor/nit, all verified).**
- **F-1 (design-wrong)** — D7's pad-zeroing mechanism is insufficient on its own:
  comfy-table fills short last-column cells to column width, so the impl adds a
  per-line `trim_end()`. design.md §5/D7 reconciled.
- **F-2 (design-wrong)** — design §6 golden inventory listed only the
  `tests/e2e_*.rs` files; the shape change also tripped in-crate goldens in
  `src/{backlog,boot,governance,slice,spec}.rs` and `e2e_backlog_list_order`.
  design.md §6 reconciled. (This omission cost two dispatch file-set escalations.)
- **F-3 (aligned)** — `unsafe_code=forbid` blocked the `set_var` test, so
  capability split into a pure `color_enabled(no_color, is_tty)` seam behind the
  thin shell (mirrors `git::trunk_tree_ish`). Strengthens the purity boundary;
  design intent honored. §4's sketch is illustrative.
- **F-4 (aligned)** — colour resolved once via `ListArgs.color`, not 11 inline
  reads. Cleaner, equally pure, intent preserved.

**Standing risks / consciously accepted.**
- **Wide-char alignment (latent, design §6).** Width moved `chars().count()` →
  display-width (unicode-width via `custom_styling`). No current golden seeds a
  wide cell, so the re-baseline hides no present alignment change — but the day a
  golden seeds a `done ⚠` slice row, alignment shifts. Noted, not blocking.
- **Uncoloured status tokens.** backlog `resolved`/`closed`, memory
  `superseded`/`retracted`/`archived`/`quarantined`, spec `deprecated`/
  `superseded` fall to `status_hue`'s `None` default — conservative, no per-kind
  override invented, consistent with the shared-map-no-duplication constraint. If
  colouring these is later wanted it is a clean follow-up.

**Dispatch-process notes (not findings — orchestration record).**
- **Wrong-base avoided.** Mid-PHASE-01 an external mover advanced `main` two
  SL-049 backlog commits (file-disjoint `.doctrine/backlog/*.toml`). The funnel's
  `S^==B` belt + branch-point guard caught it; the in-flight worker delta was
  re-anchored onto the new HEAD (patch applied clean, file-disjoint) — no
  session↔coordination divergence entered any `B..S`.
- **RSK-1 honored.** PHASE-01 landed as two coordination commits — logic
  (`b701de4`) separate from golden re-baseline (`5b28112`) — so shape churn cannot
  mask a content regression (EX-5). The funnel's one-commit-per-batch rule is a
  concurrency invariant that does not bind a serial batch of one; HEAD moved only
  by the orchestrator's sequential commits.
