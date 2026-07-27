# Fix the defect class, not the instance

Evidence: SL-231 PHASE-01..03. Three orchestrator cleanup turns each diagnosed a
real defect and each fix was correct and verified. A later independent review
(RV-317) found that **every one of those classes had a surviving sibling** the
cleanup never swept for.

| Cleanup fixed | Surviving sibling found later |
|---|---|
| `escape_hostile` iterated bytes through `char::from(u8)` (Latin-1) | `store::shard_dir` byte-slices a UTF-8 uid → **reachable panic** on 3 of 6 public verbs |
| Adapter's hand-rolled `filter_and_order` duplicated `query::query` | dead `Service::load_all_resolved` re-implements `query(Projection::History)` with a byte-identical comparator; `run_list`/`run_search` still 43 duplicated lines |
| EX-5 row injection: gave one escaper an `EscapeContext` | applied to payload summary/detail only — envelope metadata (`uid`, `recorded_at`, control uids) still rendered raw at 9 loci, re-opening the injection |

Each individual repair was right. None was followed by "where else does this
shape appear?" — so the review that should have been a formality found a blocker.

## The move

When a cleanup turn identifies a defect **class** (not just a defect), before
closing the turn:

1. Name the class in one line — "byte-index reasoning over `str`", "hand-rolled
   ordering duplicating the service", "untrusted field reaching a terminal
   unescaped".
2. Grep the whole delta for that shape, not just the reported file. For the three
   above: `\[[a-z_]*len\(\) *- `, `sort_by|cmp\(`, and every call site of the
   escaper rather than its body.
3. Fix or explicitly record each sibling.

## Why it is cheap and why it is skipped

The sweep costs one grep per class and the fix is usually mechanical — the
diagnosis, which is the expensive part, is already done. It gets skipped because
a correct verified fix *feels* complete, and because the reporter framed one
location. The framing is the trap: a reviewer reports where they looked, not
where the class lives.

## Corollary for reviewers

Ask for the class, not just the instance. A finding phrased "X is wrong at
file:line" invites a point fix; "X is wrong at file:line — check every other
place this shape appears" invites the sweep. RV-317's raises were written in the
second form for exactly this reason.

Related: [[mem.fact.dispatch.deepseek-review-capability]] — a separate review turn
is what closes the self-review gap; this pattern is what stops that turn from
having to.
