# IMP-354: Platform-agnostic worker confinement wrapper (seatbelt/bwrap)

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

Ship a platform-agnostic confinement wrapper for subprocess-arm workers — bwrap
on Linux, seatbelt (`sandbox-exec`) on macOS — and have the normative
`dispatch-subprocess` skill invoke it.

## Why

The shipped skill spawns unconfined:

```sh
env -C "$D" DOCTRINE_WORKER=1 codex exec "<pre-distilled prompt>"
```

`.agents/skills/dispatch-subprocess/SKILL.md:15` (codex arm) and `:31` (pi arm).
Both are annotated *"Confined (bwrap)"* / *"Same confinement as codex arm"* — but
neither command implements it, and nothing under `.agents/skills/` or `install/`
references `scripts/pi-spawn-confined.sh`, where the real bwrap confinement lives
(`:100-115`: `--ro-bind / /`, rw-bind only `$D`, plus a fail-closed empty-PREFIX
guard).

So the platform lags this project: doctrine's own dispatch is confined by the
project-local script (CLAUDE.md prescribes it), while a consuming project
following the shipped skill gets `DOCTRINE_WORKER=1` only. That env var gates
**doctrine CLI write-class verbs**; it does not stop a raw file write to a
reachable primary tree.

The blocker on closing the gap properly is portability — confinement has to be
expressed once and instantiated per-OS, rather than baking a Linux-only `bwrap`
argv into a shipped skill. Hence a wrapper, not a copy of the script.

**The misleading annotation is the cheap part and worth doing first**: a comment
asserting confinement the adjacent command does not implement is a trap for the
next reader.

## Not a security boundary either way

Worker confinement in doctrine is **actor-based and cooperative** — an
accident-fence, not a security boundary
(`mem.fact.dispatch.worker-confinement-is-actor-based`; `WriteClass::MarkerClear`
is deliberately unguarded, and a worker can stand itself down explicitly via
`--operator`). This item raises the floor for the subprocess arm; it does not
change that posture and should not be scoped as if it did.

## Provenance

Surfaced by RV-322 round 5 (F-1) during SL-237's design review, where REV-043
cited `pi-spawn-confined.sh` as evidence of a codex/pi-arm OS floor. Citing a
doctrine-repo script as *platform* evidence is also a POL-002 problem — removed
from REV-043's argument by the split.

Not caused by SL-237 and not a precondition for it: single-homing phase state
does not depend on this. Filed so the observation outlives the review.

The adjacent governance question — whether SPEC-012 REQ-304's "by construction
rather than a trusted check" is still accurate given cooperative confinement — is
a separate thread, deliberately not folded in here.
