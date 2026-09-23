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
