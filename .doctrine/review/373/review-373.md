# Review RV-373 — reconciliation of SL-260

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

<!-- Pre-reading + lines of attack: what this review is probing, the invariants
     it must hold the subject to, and where the bodies are likely buried. Seeded
     at `review new`; the reviewer fills it before raising findings. -->

Mode: **conformance**, self-audit. Surface reviewed: `edge` at `6a40cb706`
(not dispatched — phases landed directly on `edge`; no candidate branch).

Subject: `SL-260`'s shipped convention text — `install/design-prompts/reviewing.md`
(routing section + stand-alone carve-out), `install/review-ledger.md` §4 route
axis, `plugins/doctrine/skills/plan/SKILL.md` transcription pointer — plus the
`PHASE-03` records (`CHR-077`, `QUE-224`, `QUE-225`, `RFC-026` `P10`
settlement, `DEC` status transitions).

Lines of attack:

1. **Shipped text vs binding decisions.** The design is locked, but the
   accepted `DEC` records it minted are tier-2 too. Where shipped text (and the
   design §5.2 drafts it transcribes) disagrees with a `DEC`, that is drift.
2. **Single owner** (scope §9.1 item 3): every normative clause stated once, in
   the fragment; the two pointers non-asserting.
3. **Delivery through the render**, never the source file: `design resume` on a
   run at `reviewing`, `library show`, installed `.claude/skills/plan/`.
4. **`src/` tripwire** at close: commit-scoped, with positive control.
5. **Conformance algebra** (`slice conformance 260`): undeclared / undelivered.
6. **Enforcement honesty** (`CON-006`): no clause implies a check that does not
   exist.
7. **`PHASE-03` apparatus**: chore cites rather than restates; `QUE`s owned by
   the chore; `DEC`s no longer `proposed`.

Evidence run at audit: `doctrine check gate` exit 0; tripwire
`git log 423c181d1..HEAD --grep=SL-260 -- src/` → 0 commits (control: 26);
`design resume 253` (stage `reviewing`) renders the routing section (:127), the
carve-out (:122) and the `DEC-277` residue sentence; `library show
reference/review-ledger.md` carries the axis (:167); installed plan skill
carries the pointer and its `CON-006` clause (:43, :52); all 14 `DEC`s
`accepted`; `CON-006` `active`.

## Synthesis

**Closure story.** `SL-260` delivered what it scoped. The routing convention
has one normative owner, the reviewing fragment. Two non-asserting pointers hang
it at its other firing moments (the `review-ledger.md` §4 route axis and the
`/plan` transcription pointer), and all three cite `CON-006`. Delivery was
verified through rendered output, not source. The `src/` tripwire held across
all 26 slice commits. The trial apparatus (`CHR-077`, `QUE-224`,
`QUE-225`, the `RFC-026` `P10` settlement) cites rather than restates. All
three phases' verification criteria hold at audit.

**The one substantive defect** (`F-1`, major, fixed in `5b9cdf055`). The shipped text
extended `DEC-263`'s narrowed `verify` and its deferral past the design lock
to every "routed finding". Every severe finding carries a route (`DEC-265`), so
the extension reached `route:review` and `route:owner-fix`, which have no
criterion to verify against. The fault started in design §5.2's drafts and was
transcribed faithfully. The locked design and an accepted `DEC` disagreed; the
owner ruled for `DEC-263`. This is the same class as `DEC-277`: design drafts
that are verbatim-transcribed inherit their defects, and the single-owner sweep
cannot catch one that is *present* but wrong. Only a read against the governing
records did.

**Standing risks.**
- The convention is unenforced by construction (`CON-006`). Transcription is
  caught only at slice close, as audit-grade detection.
- `R1` to `R3` stand as scoped. The trial answers *can it operate*, not
  *does it work*.
- Design runs already in `reviewing` (`SL-253`) now receive the convention
  without being trial-eligible (eligibility is by id after this slice lands).
  This is harmless to counting, but `CHR-077` should not mistake such a ledger
  for a trial datum.

**Tradeoffs consciously accepted.**
- A `tests/` allowlist edit on a "no tooling" slice (`F-2`). It was the owner's
  ruling and sits outside the `src/` tripwire.
- The friction observation record stays undeclared. It is instrumentation, not
  slice work.

## Reconciliation Brief

### Per-slice (direct edit)
- **`F-1`** — design.md §5.2 drafts (around :540-544, the raiser paragraph, and
  around :584-586, the §4 axis draft), plus the stand-alone carve-out draft
  (§5.2, "a routed finding's criterion sketch"). Narrow to *instrument-routed*
  exactly as shipped in `5b9cdf055`, including the added sentence that a `review` or
  `owner-fix` finding is verified in the pass as usual.
- **`F-3`** — design.md §5.2: add the post-`verified` residue sentence to the
  accumulation draft and the `CON-006` clause to the plan-pointer draft,
  matching `768dd1bde` and citing `DEC-277`.
- **`F-2`** — the selector registry is load-bearing, so run
  `doctrine slice selector add 260 --intent design-target` for
  `tests/e2e_claude_install.rs`, `.doctrine/backlog/chore/077/**`, the 14
  decision record dirs `.doctrine/knowledge/decision/{263..273,275,276,277}/**`,
  and `.doctrine/knowledge/question/{224,225}/**`, plus their slug symlinks if
  conformance lists them. Mirror them in design §6. Re-run
  `slice conformance 260`: only the observation record should remain
  undeclared.
- **`F-4`** — design.md (:552, :818, :888) and slice-260.md: re-cite
  `reviewing.md:63-64` → `:98-99` and `review-ledger.md:164-166` →
  `:191-192`, or cite by quoted text. Leave plan criteria untouched (immutable).

### Governance/spec (REV)
- None. `DEC-263` stands as written, and `F-1` brought the text into line
  with it.
