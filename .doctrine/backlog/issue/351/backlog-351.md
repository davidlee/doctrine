# ISS-351: pi-spawn.sh spawns a dispatch worker with no confinement

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Raised at `SL-254`'s reconcile from `RV-356` `F-4` (an implementation-audit
finding, verified). The audit's brief routes it here explicitly: *delete-or-keep
is a decision, not a phase criterion*.

## The finding

`SL-254` collapsed dispatch onto one worker-spawn shape — a confined subprocess,
on every harness. `scripts/pi-spawn.sh` (66 lines, still present at the slice
tip) falsifies that as a statement about the **repository**, though not as a
statement about `/dispatch`'s own path. It is not a stub and not a doc:

- `:27` runs `doctrine worktree fork --base --branch --dir --worker` — a real
  worker fork;
- `:53` runs `timeout "$BACKSTOP" env -C "$D" DOCTRINE_WORKER=1 …` — worker
  identity asserted;
- `grep -c bwrap` = **0** — no confinement at all.

It was maintained after the confinement work landed and survived the collapse
untouched.

## What it is not

It is not a *fallback rung*. Nothing degrades into it, so `DEC-208`'s
"no unconfined rung" holds for `/dispatch`. The contrast controls run at audit:
`scripts/spawn-confined.sh` and `scripts/pi-respawn-nofork.sh` both carry
`bwrap`; `scripts/pi-agent`, `pi-scout` and `pi-research` carry neither `bwrap`
nor worker identity — they are research helpers, not dispatch spawns.
`pi-spawn.sh` is the only script in the repo that asserts worker identity
**without** confinement.

## The decision to take

Delete it, or bring it under the confinement prefix. Deletion is the cheaper
reading and looks close to free — no script invokes it; the three references
(`scripts/pi-agent:17`, `scripts/lib/pi-reap.sh:5`, `scripts/spawn-confined.sh:76`)
are comments naming it as a sibling, and they move with it. Confirm that before
acting: it also hard-codes `ROOT=/workspace/doctrine`, which is host-specific and
would have to go either way.

Whichever way it goes, the slice headline — *"dispatch has one worker-spawn
shape, a confined subprocess, on every harness"* — becomes true of the
repository, not only of the skill.
