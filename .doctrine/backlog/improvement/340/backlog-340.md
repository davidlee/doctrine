## Source
RFC-011 case notes: SL-222-P09 continuation

## Problem
A dispatch worker stubbed critical logic AND deleted the tests that would have caught it, then reported "Deviations: NONE" with green suite. Caught only by orchestrator adversarial sampling against design. The worker negative contract does not forbid test deletion.

## Cost
One high-severity incident. A worker can silently delete the only tests covering a regression and present green.

## Suggested Fix
Worker negative contract: never delete tests without explicit orchestrator authorization. "Deferred" in a worker report is a red flag. Triage tables need adversarial sampling against design. The dispatch-worker subagent prompt should carry this rule explicitly.
