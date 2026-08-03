# DEC-131: Writable agent home, unwritable credential

The capsule's `$HOME` used to be read-only. It is now a tmpfs, with the
credential read-only *inside* it. Written down carelessly that is a story about
a sandbox getting looser, so here is the part that matters:

> **"The agent home is read-only" was never the property. It was a proxy for
> the property.**

The property is: **a capsule cannot modify the trusted-side credential store.**

## What happened

PHASE-06's first real-agent run failed. The harness could not create its
per-session working directory, because `$HOME` was read-only. That tripped
STOP-5 — a change touching the confinement profile is a finding and a consult,
never a quiet rig edit — and it went to the operator instead of being patched.

The ruling: **setup oversight, not a weakening.** Fix the profile, re-run,
disclose the first attempt.

The new shape is `--tmpfs /agent/.claude` with `~/.claude/.credentials.json`
ro-bound **inside** the tmpfs. Canonical, other capsules, `~/.ssh` and
`~/.gitconfig` all remain absent from the mount profile entirely.

## Why a writable directory does not weaken an unwritable file

Only because the refusal is a **mount** property. A read-only bind mount refuses
writes with `EROFS` regardless of who owns the file or what its mode bits say —
and the capsule runs as `uid=1000` and *owns* that file.

That distinction is load-bearing enough to have its own record: [[DEC-132]].
Read it before touching this profile. A narrowing that keeps the directory
tight but puts the secret back on a **writable** mount is a real weakening no
matter how restrictive its permissions look, because the capsule can chmod its
own file.

## The rejected narrowing, and why

An obvious-looking alternative was a tmpfs at `/agent/.claude/session-env` —
tighter, and it would have left the original probe row passing unchanged. It was
rejected because it **pins the rig to an undocumented harness internal**. That
path is not API; it moves when the harness moves, and the confinement story
would silently degrade the day it did. It also buys nothing the capsule-scoped
tmpfs does not already give.

## The consequence for the probe

With a writable home, the old credential probe row became unsound — a refusal
against a broken write mechanism proves nothing. That is what forced the
realignment in [[DEC-132]]: assert on the **file**, and put a positive control
writing successfully beside it.

## Reading the evidence

`evidence/results-c1b.tsv` holds **both** agent runs. The superseded first
(`03:24:26Z`) ends `agent-committed=no tree-dirty=yes`; the scored second
(`04:38:58Z`) ends `agent-committed=yes tree-dirty=no`. It is kept because a
correction is only legible against what it replaced — the same reason
`results-c2.tsv` keeps its superseded `api-cred` rows.
