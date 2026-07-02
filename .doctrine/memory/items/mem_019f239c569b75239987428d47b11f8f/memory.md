# REV-only / governance-output slice reads as undeclared in slice conformance

slice conformance flags a REV-only slice's authored deliverable (.doctrine/revision/NNN) as undeclared; the code-delta algebra has no design-target selector for governance output — not drift.

## Why

`slice conformance` compares recorded source-deltas against the slice's
`design-target` selectors. A REV-only / governance-output slice (design declares a
**zero touch-set** — no `src/`) produces its deliverable as *authored governance
entity files* (`.doctrine/revision/NNN/…`), which land inside the phase boundary but
match no selector. So the deliverable **necessarily** reports as `undeclared`. This
is structural, not scope creep.

## How to apply

At audit of a REV-only / zero-code slice, expect `undeclared(N)` = the REV (or ADR /
spec) entity files. Verify the real code-impact gate instead: `git diff --stat --
src/` must be empty (the design's EX-N). Dispose the conformance finding `aligned`
("deliverable, not drift"), not `tolerated`. First confirmed on SL-181 / REV-018
(first REV in corpus).

Distinct root cause from [[mem_019f031a315c7803900fcf398092e674]] (undeclared noise
from boundary **start-oid pollution** — foreign commits inside `start..end` when
`edge` advances mid-phase). Rule them out in order: (1) are the undeclared paths the
slice's own authored deliverable? → this memory, `aligned`. (2) Are they foreign
`src/` commits from other slices? → pollution, `record-delta`.
