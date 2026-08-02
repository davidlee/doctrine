# The baseline artefact — T-base

`VA-4` requires the comparison to name a **concrete baseline artefact**, not an
unspecified "before". This file names it precisely enough that CHR-049 can
recover the exact bytes, and records why the other two treatments are what they
are.

## T-base — the pre-SL-233 design skill

| | |
|---|---|
| artefact | `plugins/doctrine/skills/design/SKILL.md` |
| revision | the parent of `59c5bdab` (`59c5bdab^`) |
| blob | `65b6c45be3f8732adb1d59ec15bdc85a1a9a17a7` |
| size | **214 lines / 10,178 bytes** |
| availability | **always** — recoverable from git |

Recover it with:

```bash
git show 59c5bdab^:plugins/doctrine/skills/design/SKILL.md
```

The blob oid is recorded because it is the identity that survives a rebase of the
commit that carries it. If `59c5bdab` ever moves, `git cat-file -p
65b6c45be3f8732adb1d59ec15bdc85a1a9a17a7` still yields the treatment.

**What makes it the right baseline.** In T-base the same craft is *skill prose an
agent may scroll past* — one 10KB file read once at activation. `59c5bdab`
replaced it with a 57-line / 2,568-byte adapter and moved the process detail into
runbooks and per-stage fragments. That is precisely the move DEC-104 justified on
the ground that fragment delivery puts content in front of an agent more reliably
than a document it has already scrolled past. The baseline is therefore the
artefact the claim was made *against*, not merely an earlier version.

## T-step — the rejected step shape

| | |
|---|---|
| artefact | the **nineteen-step edge-3 shape** D14 rejected |
| availability | **where a fixture permits** — disclosed as opportunistic |

The trade DEC-104 made was against this shape specifically: nineteen discharged
2a steps producing nineteen attestations, versus fragment delivery producing few.
It is the treatment the *receipt* half of the claim is about, so S2 and S3 mean
most when it is available.

It is opportunistic rather than always-available because it was never shipped —
reconstructing it is fixture work CHR-049 may or may not have room for. Stated,
not assumed.

## T-frag — as shipped

Every-turn 2b fragment delivery, as SL-233 ships it. Always available; it is the
running system.

## Why an observation against neither is uncollected

An observation offered against no baseline says only *"the fragment arrived"*,
which nobody disputes and which no result could contradict. The kit records such
an observation as **uncollected** rather than weighing it — the same discipline
the S4 firing condition applies, and for the same reason: a measurement whose
comparison is unstated can be narrated into any conclusion after the fact.
