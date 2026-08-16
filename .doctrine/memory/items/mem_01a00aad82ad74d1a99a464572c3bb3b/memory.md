# Before pinning a before-state, grep for the pin

Characterisation phases exist to pin behaviour a later phase deliberately
changes, so the change reads as a change rather than as a test that was always
going to pass. The design that orders the pin usually also *states* that the
surface is uncovered. **Check that claim before acting on it.**

## The trap (SL-238 PHASE-02, 2026-08-16)

`design.md` said twice — §6 and §7 — that `doctrine after --prune` had "no test
coverage at all, in either copy". Five goldens existed in
`tests/e2e_dep_seq_verbs.rs`, landed three slices earlier by SL-105. The design
was locked and had survived a two-round external adversarial review; nobody
checked, because a claim about *absence* does not look like a claim a prose
reviewer needs to verify.

Both readings of that were wrong in opposite directions, and both cost:

- **Take the design at its word** → you re-pin what is already pinned. Two
  duplicate tests, and the reviewer later cannot tell which is authoritative.
- **Take the existing tests at their word** → you skip the phase as redundant.
  Worse, because the incumbents were *weak*: `assert!(out.contains("resolved"))`
  passes whether or not the reason renders its `/resolution` suffix, so the
  behaviour the phase existed to pin was invisible to the coverage that
  "already existed".

## The move

Two greps, both cheap, before writing a line:

1. **Does a pin exist?** Grep the test tree for the verb / surface, not just for
   the function name — a black-box golden invokes the CLI and never contains the
   production identifier, so a source-symbol grep reports a false absence.
2. **What does it actually assert?** Read them. A `contains()` on a substring
   that appears in both the before- and after-state pins nothing. That is the
   normal shape of an incumbent test, because it was written to prove the
   feature worked, not to pin it against a future change.

Then write only the delta, and say in the commit which incumbents you found and
why they were insufficient — otherwise the next reader re-runs this whole
analysis.

## Why it generalises

The three defects SL-238 found before this one all sat at the seam between a
design's prose rule and the criterion meant to enforce it, and were found by
being *obliged to write the assertion*. This one sits at the same seam and was
found from the other side: being obliged to *look for the assertion that already
exists*. Both are properties of the seam, not of the slice — a design's account
of what is untested is unverified prose like any other, and it decays faster,
because tests land continuously and designs are locked.

Adjacent: [[mem.pattern.testing.black-box-cli-golden]] (why a source-symbol grep
misses e2e coverage), [[mem.pattern.tests.mutate-the-data-not-just-delete-it]]
(the companion check — after green, mutate the fixture and confirm the intended
assertion fires by message).
