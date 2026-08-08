# REQ-477: A hook set that lands in the selected scope evicts doctrine-owned entries from the abandoned sibling scope by the same ownership predicate, gated on the target write landing, and reports what it removed, could not read, or did not attempt.

## Statement

Three obligations in one behaviour:

1. **Evict by the same ownership predicate.** The sweep walks `hooks.<event>`
   arrays and drops only entries doctrine owns. A foreign entry is never at risk —
   the sweep reuses the merge path's predicate rather than a second one that could
   drift from it.
2. **Gate on the target write landing.** A spec whose write to the selected file
   did not land does not sweep the sibling. `PrintedFallback` is an `Ok` value, so
   a bare `?` does not see this failure and ordering write-before-evict does not
   buy the gate on its own — it is on the outcome, not on bytes reaching disk, so
   `--dry-run` stays a faithful preview.
3. **Report all three outcomes** — removed, unreadable, not-attempted — on every
   install path that performs the write, not only the interactive one.

An *absent* sibling is `nothing`, not `unreadable`: only a file that exists and
cannot be understood is unreadable. Non-hook keys are out of scope — the sweep is
spec-keyed, and `worktree.baseRef` is read and reported rather than swept.

## Rationale

This is doctrine's one destructive write in the activation area, and the failure it
prevents is invisible. A user with a hand-edited `.claude/settings.local.json`
switches to project scope; doctrine writes eleven entries to `settings.json`,
cannot parse the sibling, and — with a silent no-op — reports success. Every hook
then fires twice, permanently, presenting as mild slowness and nothing else.

The gate exists for the inverse pairing, which is worse: a manual-repair snippet
printed beside "evicted 11 stale hook entries" reads like routine cleanup and is in
fact the announcement that nothing fires any more.

Reporting is therefore not decoration but the requirement's substance. The
diagnostic sink that might otherwise catch this is IMP-407's doctor leg, which does
not exist yet; until it does, the rider is the whole signal.

Delivered by SL-250. Governed by REV-049 (`FR-009`, introduced at reconciliation
from RV-350 `F-1` axis 4).
