# Implementation Plan SL-178: Close drift-discharge legibility: richer error + skill recipe + shipped memory

Prose companion to `plan.toml`. Narrative only — no queried data lives here
(the storage rule); the phase list, criteria, verification, and links are
authored in the TOML. Use this for the plan's rationale and sequencing.
<!-- Cite entities by padded id (SL-020, REQ-059); phases as PHASE-01,
     criteria as EN-1/EX-1/VT-1/VA-1/VH-1. See .doctrine/glossary.md § reference forms. -->

## Overview

Three independently-shippable legibility fixes (design §1), one shared identifier
— the recipe-memory key. Each surface (binary error, `/close` skill, shipped
master) collapses a slice of the ~4-round-trip close-discovery cost (IMP-202).
The plan splits along those surfaces, ordered so the canonical source ships
before anything points at it.

PHASE-01 ships the master, PHASE-02 the error, PHASE-03 the skill. The design
labels these P2 / P1 / P3; the durable PHASE ids run in landing order, so the
master is PHASE-01. Hold that mapping in mind when cross-reading design §9.

## Sequencing & Rationale

**PHASE-01 first — the master is the pointer target.** The binary error (PHASE-02)
and the skill (PHASE-03) both name the shipped key
`mem.pattern.doctrine.close-drift-discharge-rec`. Until the master ships, that key
is a dangling pointer into unshipped state — a POL-002 violation in the interim
(design R5). So the master lands first and the other two gate on it (their EN-1).

PHASE-01 is not a re-home. The sanctioned master-authoring path is `doctrine
memory record --global`, which suppresses the git born-frame (`repo=""`,
`anchor.kind=none`) and writes into `memory/`, guaranteeing the ADR-002
global-orientation signature for free (design D3, RV-195 F-5). The verb mints a
*new* uid; the local capture `mem_019f075f` is a genuinely different entity (a
repo-local capture, not a platform master) and is **superseded**, not moved. The
stable handle across the uid change is the *key*, which is why PHASE-02's const
references the key rather than the uid.

The body does **not** ship as-authored. A shipped master *is* the platform, so its
body must carry no host-project-local state (design §5.4 body scrub, R6). Two
scrubs (drop the local-memory cross-ref `mem_019ec912`; replace the host backlog
id `ISS-006` with prose) and one conscious tolerance (the SL-165 worked example
stays, re-framed as an explicit Doctrine-development illustration — concreteness
is its pedagogical value). This scrub is PHASE-01's exit gate (EX-2), checked by
VA-1 grepping the body, not just the downstream surfaces.

**PHASE-02 — error + data shape.** `undischarged_drift` already loads each REQ's
authored status and discards it; the change is to keep it (a named struct
`UndischargedReq`, design D1) and spend it in the bail copy. Behaviour-preservation
is the proof: the close-gate refuse/pass behaviour is unchanged — only the error
*payload* and the function's *return type* move (design §5.5 INV). The existing
drift-gate cases keep asserting refuse/pass; their payload assertions switch to
substrings (design F-3/F-4) so the cases survive copy edits.

**PHASE-03 — skill subsection.** Pointer-tier per ADR-005: the condensed clauses,
the `rec new` line, and the key pointer. The worked example is never duplicated —
it has one home (the master). Smallest of the three; gates only on PHASE-01.

PHASE-02 and PHASE-03 are file-disjoint (`src/slice.rs` vs
`.agents/skills/close/SKILL.md`) and both depend only on PHASE-01, so they may run
in parallel once the master ships.

## Notes

- **Embed visibility (design F-2).** `rust-embed` carries `debug-embed`, so the
  debug/test binary reads `memory/` from disk at runtime — a hand-added master is
  live via `./target/debug/doctrine` for PHASE-01's VT/find checks. The PATH
  binary only sees it after reinstall; use the dev binary during PHASE-01.
- **Const ↔ key match (design R4/§5.5 edge).** PHASE-02's const and PHASE-01's
  shipped key must be byte-identical or the error's pointer dangles. Single const;
  PHASE-02 VT-1 asserts the key string is present.
- Line numbers in the design (`slice.rs:831`, `:1275`, …) have drifted; the named
  symbols (`run_status`, `undischarged_drift`, `rec_discharges`, `SLICE_DIR`,
  `expect_close_refused`) are the stable anchors and all resolve in the current
  tree.
- **Supersede, do NOT delete (design tension, pinned).** §5.3 says the local
  capture `mem_019f075f` "is removed"; D3 / §5.4 step 3 / §7 say **superseded**
  (`memory status superseded --by <new-uid>`). D3 is the authoritative revised
  decision (post-RV-195 F-5) — "removed" reads as functionally-inactive, not
  `rm`. PHASE-01 supersedes the capture; it is not deleted (it carries the
  historical refs that still resolve via `memory show`).
- **Same-key shadow check.** After supersede + `memory sync`, the superseded local
  capture and the active shipped master both spell the key. PHASE-01 EX-3's "find
  discovers the master" must confirm `memory find` returns the *master*, not the
  superseded capture (or both) — verify, don't assume the supersede excludes it.
- `--global` verified present on `doctrine memory record` (mint a global
  orientation master: suppress born frame, write into `memory/`) — design D3's
  premise holds.
