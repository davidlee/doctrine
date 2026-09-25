# Review RV-393 — code-review of SL-266

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Pre-close code review of SL-266 PHASE-01..04 (feat/fix commits `3ba03030e`,
`3e745780f`, `a51c614bb`, `d4c6f8ba9`, `78d8e0b44`, `c47451f1a`, `3cdceba18`,
`6d9bf2db0`, `94fb3870b`). Depth: full process (subsystem, pre-close). Raiser
`opus-reviewer`. RV-392's F-1..F-3 (PHASE-03) are not re-raised; their fixes
were re-read and hold.

Governing: `design.md` sec-2..sec-7 (DEC-303..DEC-310, DEC-307 as amended at
VH-1); ADR-001; STD-001; STD-003; SPEC-029 REQ-433; PRD-019 REQ-417/REQ-425.

Lines of attack:

1. **Design conformance.** The envelope carries the map at `Full` only; the tree
   reads only the envelope; config is lazy for other readers; the relay line is
   last, fires only on a map change in relay mode, never on replay/adopt/sidecar.
2. **Pure/imperative split.** No disk, clock, git or tty in `design_run::*`
   (tree, config, `map_changed`, `relay`); the shell passes width, colour,
   titles, selection, delivery.
3. **Renderer edge cases.** Unicode width, wrap and drop rules, overflow
   guarantee, empty map, deep/narrow, cycles and absent parents, blocked vs
   open counting, cursor/pin/stale-cursor suffix colouring across wraps.
4. **`map_changed` semantics.** Does `InquiryMap` equality capture every map
   change and nothing else (legacy blocking set, traversal)? Are all snapshot
   writers covered (start, apply pipeline, adopt, materialise)?
5. **Run scan.** One read; disclosure of every failure; mtime/tie rules.
6. **Coupling and duplication.** New helpers vs existing seams; vocabulary
   spelt once; stringly-typed crossings between envelope and renderer; new
   dependency of the design writes on `doctrine.toml`.
7. **Tests.** Behaviour vs theatre; golden brittleness; VT-1..VT-12 coverage.
8. **Prompt text.** Mode-neutral; no dangling reference to the retired
   hand-built listing.

## Synthesis

- **Overall:** solid
- **Synopsis:** SL-266 renders the whole inquiry map as a tree over the
  `Detail::Full` turn envelope, through one path (`tree_lines`) shared by
  `design tree` and `show --format tree`. The pure renderer's placement (cycles,
  absent parents), wrap/drop/overflow rules, colour-additive output and header
  counts are sound and tested for behaviour. `map_changed` compares the
  `InquiryMap` state, and the relay line fires last, only in relay mode and only
  on a map change — never on replay, adopt or sidecar. Five findings, all
  verified at `fd3c53453`. F-1 (doc comment moved onto the wrong fn), F-2
  (`slice_ref` is now the one spelling) and F-5 (the condition's "that listing"
  now reads "that tree") were fixed. F-4 was fixed by `dtoml::design_entry`,
  so the design writes now depend only on `[design]` and on valid TOML syntax,
  with red/green e2e and unit coverage. F-3 was part-fixed (`State::word`
  derives from `InquiryLifecycle::as_str`); the typed `MapNode` fields that
  would delete the renderer's unreachable string fallbacks are follow-up
  IMP-486. Residuals: `dtoml::parse`'s doc still claims to be "the ONLY
  `doctrine.toml` parser", which `design_entry` makes untrue (a one-line doc
  nit). The design reconciliation items stay open for `/reconcile`: sec-3
  rule 2 against DEC-307's railed drop, and the sec-4 scan skip rules. RV-392's
  scan fixes hold.
- **Haiku:** *One table, not six —
  / the tree now waits on its own
  / key, and names its view.*
