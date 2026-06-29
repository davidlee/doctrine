# Review RV-191 — design of SL-177

Adversarial-review ledger (ADR-007). Structured findings live in the sister
ledger toml; this prose companion carries the reviewer's framing.

## Brief

Lines of attack pressed against SL-177's design (default value 1.0 for valueless
value-bearing kinds, feeding SL-176 burndown):
1. Is `base_score` the only value read-site, or does burndown read value via a
   different seam where the default would be missing?
2. Does the reused `WORK_PREFIXES` set actually equal the value-bearing set, or
   is it a false-equivalence trap?
3. ADR-001 cohesion: is `value.rs` the right home for `DEFAULT_VALUE`?
4. Full test/golden blast radius of changing valueless-work==0.
5. `kind_weight` × `est_cost` interaction — surprising/unbounded `value_dim`?
6. STD-001 / DRY — duplicate kind-prefix literal sets.

Tribunal: internal pass (AR-1..4, design §10) + external adversary (codex/GPT-5.5,
read-only). Internal pass caught the false preservation claim (AR-1); external
pass caught the burndown-seam blocker (F-1) the internal pass missed.

## Synthesis

**Verdict: heretical as first drafted — one blocker, two majors, two minors; all
reconciled. The design is corrected, not condemned.**

The cardinal heresy (**F-1**) was a seam error masquerading as completeness: the
default was nailed to `base_score`'s `value_dim`, while SL-176's burndown
(`.doctrine/slice/176/design.md:297-311`) draws on the **raw authored `value`
facet** through a wholly separate channel. The slice would have shipped, compiled,
passed its own tests — and silently failed its only purpose, valueless work
contributing nothing to burndown. Confessed under cross-examination against the
SL-176 spec. Penance: a single shared `effective_raw_value(kind, &facets)` accessor
in the priority tier, consumed by both `base_score` and the burndown post-pass;
the default homed beside it (**F-4** absolved into the same act of contrition).

The second heresy of names (**F-3**): `kinds::WORK` would have stood a hand's
breadth from the established `is_work_like` ({slice,backlog}∪**REV**) — same word,
different flock. A later refactor would have merged them and granted a Revision a
value default it must never hold. Penance: the set is named for what it *is* —
`VALUE_BEARING` / `is_value_bearing` — and `is_work_like` left untouched;
value-bearing ⊂ work-like, parted by REV.

The third (**F-2**): the draft confessed only two tests changed; the rack revealed
a broader congregation — graph.rs score-consequence tests and both e2e goldens
bake valueless-work scores. Penance: §9.1 names the full radius; the plan gr*eps*
line-by-line and re-baselines the goldens. The behaviour-preservation gate now
guards only genuinely-unrelated behaviour.

The minor taint (**F-5**), legibility — score moves on an implicit default the
`value` column does not show — is consciously **tolerated for this slice** and
exiled to **IMP-211**. The User's revelation stays its danger: unvalued *and*
unestimated items inherit the absent-estimate anchor (greater than any real
estimate) as cost, so their `value_dim ≈ 1.0/large` is small. Only the
unvalued-but-cheaply-estimated may float, and rarely.

**Standing risks.** (R1) `VALUE_BEARING` and the actionability node set coincide
today; if they diverge, split then. (R2) Sequencing is now load-bearing: SL-177
**needs SL-176** — the `raw_value` site must exist to be retrofitted; SL-177 lands
second. (R4, AR-3) Standalone ordering shift, bounded as above.

**Tolerated, consciously:** F-5 render legibility (→ IMP-211).

> **HERESIS URITOR; DOCTRINA MANET**
