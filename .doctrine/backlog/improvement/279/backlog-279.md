# IMP-279: Per-component gauge scope

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Refactor SL-213's projection gauge from global corpus-H to per-component
scope (design.md §3 P8's rejected variant; RV-266 F-3 adjudicated the
shipped global-H as prototype-validated, this item is the reconsidered
refactor).

Compatibility analysis (from RV-266 follow-on discussion, 2026-07-11): of
the 22 validated scenarios only S2 has ≥2 disjoint components — the
divergence surface is nearly unconstrained by design evidence. Per-component
preserves P10/P11/P9, satisfies P12 locality STRICTLY (unscope and
strengthen the property test), and honours P8's original "centred" wording
(global-H compresses shallow islands into the low band — S2 pendant mean
0.6, not centred on DEFAULT).

The one real behavioural change to adjudicate at design time: in a mixed
corpus an anchor-free island currently gets P7 flat DEFAULT + P5 ladders
(losing a judgement never drops you below the unjudged baseline); per-
component P8 gives a centred spread where the loser sits BELOW default —
consistent with the validated pure-gauge regime (0.4 < 1.0), but it changes
"lost one comparison" from neutral to negative relative to unjudged
entities.

Touch surface: src/comparison/project.rs gauge branch keyed per component;
re-pin s2_partial_order_gauge golden (pendant 0.8/0.4 → 1.333/0.667);
unscope the P12 locality property test; add a mixed-corpus-island golden;
design.md §3 P1/P8/P12 reworded back. Single-component corpora are
unaffected, so live-data migration risk is low. Slice-worthy (design
conversation required for the mixed-regime call), not a quick fix.
