# SL-015 Implementation Notes

Durable, committed record of decisions/findings that outlive a phase sheet.
Design rationale lives in `design.md`; phase criteria in `plan.toml`; this is for
cross-phase implementation decisions the design didn't pin.

## Cross-phase decisions

- **D-1 — requirement `kind` is template-seeded, then overwritten post-reserve.**
  `ReqKind` (functional|quality) is not carried by `entity::ScaffoldCtx`, and the
  engine must stay unchanged (R6 gate). So `install/templates/requirement.toml`
  seeds a default `kind = "functional"`, and `spec req add --kind` (PHASE-03) sets
  the real value after reservation via an edit-preserving `toml_edit` write —
  exactly the `adr::set_adr_status` pattern (status seeded `proposed`, later
  flipped). Spans PHASE-01 (seed) → PHASE-03 (overwrite). No engine edit.

- **D-2 — staged-landing lint bridge.** A module landing one phase ahead of its
  first production caller (PHASE-01 `requirement.rs` before PHASE-03 `spec req
  add`) is `dead_code` in the bins/lib build. Bridge each pending item with
  `#[cfg_attr(not(test), expect(dead_code, reason = "first caller in PHASE-NN"))]`.
  Bare `#[allow]` is a hard error in this repo — see the recorded pattern memory
  `mem.pattern.lint.expect-not-allow` (`doctrine memory retrieve --query expect
  allow`). PHASE-02 is expected to unwind it as `spec.rs` references the
  requirement types; `expect` makes any stale bridge a build error, so it cannot
  rot.

## Findings

- **F-1 — lint suppression form** captured durably as memory
  `mem.pattern.lint.expect-not-allow` (not repeated here; the storage rule).
