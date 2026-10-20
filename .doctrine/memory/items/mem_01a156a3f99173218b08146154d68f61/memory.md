Observed during the SL-284 audit (RV-424 F-1). SL-284 deleted the npx skills
delegation and gave pi its own boot adapter. DEC-388 and DEC-389 are the real
governance for that, both accepted after DEC-377, both narrower. By the authority
model's tie-break ("within a tier, the narrower and more recent source wins") they
clearly govern.

But DEC-377 still reads, in full, as a live instruction:

  - "Unregistered harness names keep their npx skills route ... its skills are
    still delegated to `npx skills add --agent <name>`";
  - "the boot adapter: claude → claude, pi → codex, codex → codex".

`doctrine inspect DEC-388` shows only `shapes: SL-284` — no `supersede` edge — and
DEC-377's inbound set is `concerned by: ISS-520`. The design's REV clause table
(sec-6) covered SPEC-009/010/011 and never considered the decision.

The general point: **time-precedence is a rule an agent applies while holding both
records; it is not a fact the corpus records.** The moment the reader holds only
one, the contradiction is invisible and the stale decision wins by default,
because nothing on the entity says it is spent.

Two consequences when planning a change that reverses a decision:

1. Sweep the decision corpus for the behaviour being removed, not just the specs.
   `doctrine search <removed-term> -k all` surfaces the decisions; a spec-only REV
   will miss them.
2. Prefer an explicit edge (`doctrine supersede`) or a REV `modify` row over
   relying on readers to rank by date — and check whether the earlier decision is
   only *partly* falsified. DEC-377's global-no-prune paragraph still stands, so a
   blanket `supersede` would have been wrong: a `modify` row with a per-paragraph
   replacement statement was the correct instrument.


## The instrument is kind-dependent — get this right or the plan stalls

Do not read "use a REV `modify` row" as a general answer. A Revision revises
governance/spec truth only: `src/relation.rs:668` restricts `revises` to
`TargetKind::Kinds(&[SPEC, PRD, REQ, ADR, POL, STD])`. `dec`, `que`, `asm`, `con`,
`evd`, `hyp` and `cpt` are **refused**.

So the split is:

- **DEC / QUE / ASM / CON / EVD / HYP / CPT** → mint a successor
  (`doctrine knowledge new <kind>`), author its facet, `doctrine supersede <NEW>
  <OLD>`, then `doctrine knowledge status <OLD> superseded` as a *separate* step
  (`supersede` writes edges only; skipping the status leaves two live accepted
  records that `doctrine validate` happily calls clean), then re-link any
  `shapes:` edge the predecessor carried.
- **SPEC / PRD / REQ / ADR / POL / STD** → a REV `modify` row.

A change that reverses a decision usually needs **both halves**, because specs
restate the decision in their own words. That is exactly SL-284's case: DEC-377
falsified on two paragraphs *and* SPEC-009 `D4` / SPEC-010 `D2`+`REQ-175` /
SPEC-011 `REQ-480` carrying the same claims. Only the half that matches the kind
goes through each instrument.

Corollary for audits: a `design-wrong` disposition that names an unrunnable
command is worse than no disposition — it looks reconciled and stalls the next
stage. Check the instrument's kind constraint before writing the brief, not after.
