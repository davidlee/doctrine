# IMP-284: Elicit curation skill over compare elicit JSON queue

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Intent

RFC-019 queue/curator split, curator half: a skill over `doctrine compare
elicit --json` (SL-217 D16 schema v1). Engine picks mathematically productive
questions; the skill picks humanly sensible ones.

Scope (skill text only, no engine code — SL-217 Q1 posture):
- Consume the JSON envelope: sort/filter on the common spine
  (rank/kind/guaranteed_yield/impact/score/reasons/ask), read `yield_basis` /
  `yield_note` semantics per kind.
- Commensurability-in-the-small: skip or reframe pairs a human can't sensibly
  compare; `incomparable` handling per D11 (disclosed zero-yield answer).
- Frame choice + audience fit: which value frame (`equal-effort` /
  `prefer-first`), which session shape; stakeholder vs team offers.
- Session flow: batch size, fatigue, exact answer leg via the `ask` commands
  (`compare record` / `value set`); estimate nomination per D17 (curator
  nominates, engine only discloses the bare-estimate mask).
- Surface language: soften "guaranteed yield" per product-critique #4
  (order-bearing floor, per-answer breakdown), and D5/D15 stability wording.

Distinct from IMP-274 (/triage grooming skill, hand-picks pairs); triage's
pairwise mode can later consume this queue instead.
