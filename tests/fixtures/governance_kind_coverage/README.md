# `governance_kind_coverage` fixtures

Read this before "fixing" anything in `pre_amendment/`.

## What `pre_amendment/` is

A frozen snapshot of `SPEC-019` and `PRD-010` **before** `REV-050` amended them
from four record kinds to seven (SL-249 PHASE-07). It is deliberately, correctly
**stale**: it says "four kinds", it names only `assumption` / `decision` /
`question` / `constraint`, and it carries none of `EVD` / `HYP` / `CPT`.

Recovered from `59f77ce104dc26165a0cf167620b5d80ae2aef59` (the tip immediately
before the amendment) with `git show <sha>:<path>` — no branch switch.

## Why it exists

`tests/governance_kind_coverage.rs` has a live arm that reads the four authored
tier files and asserts they enumerate every kind in `kinds::RECORD`. That arm was
written *after* the corpus was corrected, so it passed on its first run and
proves nothing on its own — a checker that returns "no failures" unconditionally
would pass it too.

This fixture is that arm's compensating control. The **same** `check` function
runs over these bytes and must report the **exact** failure set the live arm
would have reported on the day before the amendment: 20 missing paired forms
(7 + 7 in the two TOML tiers, 3 each in the two prose excerpts) and 3 identity
failures. It is permanently green and permanently red-in-effect: it is the only
evidence in the tree that the checker can see the defect it exists to catch.

## Contents

| file | what |
|---|---|
| `spec-019.toml` | verbatim, all 28 lines — carries 3 of the stale `four` and **zero** paired forms |
| `spec-010.toml` | verbatim, all 18 lines — zero `four`, **zero** paired forms |
| `spec-019.md.excerpt` | line-accurate excerpt of the 453-line prose tier |
| `spec-010.md.excerpt` | line-accurate excerpt of the 430-line prose tier |

The two `.md` tiers are ~60 KB combined, too much to duplicate wholesale, so
each is an **excerpt**: every line carrying `four` or a paired form, plus the
line after it, with the original line breaks intact. Discontinuities are
expected — this is not readable prose and is not meant to be.

The excerpts are self-verifying. `pre_amendment_excerpts_are_complete` asserts
`spec-019.md.excerpt` carries all 25 `four` occurrences and `spec-010.md.excerpt`
all 4; a short excerpt would make the control a lie, so the count is asserted
rather than trusted.

## The one thing you must not normalise

`spec-019.md.excerpt` carries the **wrapped** exempt phrase, verbatim:

```
… Admitting the record kinds touches four
  coupled sites — `integrity::KINDS`, …
```

`four coupled sites` counts *integration sites*, not record kinds — it is the
allowlist's single entry, and it stays correct at seven kinds. Because the
phrase is split across a line break it matches **zero** times on the raw bytes
and **once** after whitespace collapse. That asymmetry is what makes the collapse
load-bearing: without it the identity reads `32 == 0` and the only green path is
rewording an accurate sentence to satisfy a test (RV-349 `F-9`, `F-14`).

Do not re-wrap, re-indent or reflow those two lines. A test asserts both
directions of the collapse against them.
