# IMP-428: Harden the worker confinement prefix

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Deferred out of `SL-254` by `DEC-209` ("Generalise the incumbent prefix, take no
new dependency"). The slice generalises `scripts/pi-spawn-confined.sh`'s bwrap
`PREFIX` past `pi` and stops there, deliberately.

## What the incumbent prefix already delivers

Verified at `scripts/pi-spawn-confined.sh:113-131`:

```
bwrap --ro-bind / / --dev /dev --proc /proc --tmpfs /tmp
      --bind $HOME/.pi $HOME/.pi --bind $D $D
      --chdir $D --die-with-parent --setenv DOCTRINE_WORKER 1
```

wrapped in `timeout $BACKSTOP`, with a fail-closed guard refusing an empty
`PREFIX`. So the **write floor** — everything read-only but the worker's own
directory — is delivered, as is the `DOCTRINE_WORKER` env seam and a wall-clock
bound. That is the control `SL-254`'s verification intent names, which is why
the hardening below is not on that slice's critical path.

## The gap

Research thread 5 of `SL-254` compared the prefix against
`crates/doctrine-control`'s `confinement_argv` (`backend/bubblewrap.rs:1110`)
and `wall_bounded_argv` (`:1234`), which are the reference implementation:

- no uid/gid mapping
- no `--clearenv` (so the worker inherits the orchestrator's environment)
- no `--new-session`
- no placement validation against forbidden scopes
  (`CapsulePlacement::try_new`, `backend.rs:491`)
- no `RLIMIT_FSIZE`
- no descriptor sweep

These are defence in depth — env leakage, session isolation, resource bounds —
rather than the write floor.

## Why it was deferred rather than ported

`DEC-203` schedules dispatch for replacement by capsules, and
`crates/doctrine-control` — which will carry that replacement — already
implements every item above. Porting them into `jail.rs`'s builder would build
the same property twice on a surface with a scheduled deletion. `DEC-209`
records that tradeoff explicitly, including the cost it accepts: two bwrap argv
builders coexist in the repo until the capsule cutover deletes dispatch's.

## Decide before doing

Whether this item is worth doing at all depends on when the capsule cutover
lands. If the cutover is near, close this `obsolete` rather than fixing it — the
convergence happens by deletion, not by merge. Reassess against `RFC-025`'s
roadmap rather than picking it up on age.
