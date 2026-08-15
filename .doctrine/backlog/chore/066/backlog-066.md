# CHR-066: Move mem-surface receipts out of the state root

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What

`.doctrine/state/` accumulates `mem-surface-seen-*.txt` receipts directly in its
root — ~220 of them at the time of the sweep. They sort ahead of the real
subdirectories, so `ls .doctrine/state/` (the natural way to locate a design run's
`design.toml`) buries the answer under a screen of receipt filenames.

## Why it matters

Small, cheap, and paid repeatedly. `.doctrine/state/` is runtime state — disposable
by design — but it is also the directory an agent lands in when orienting inside a
run, and the receipts make that landing needlessly expensive in tokens and
attention. Nothing about the receipts requires them to be at the root.

## Fix

Nest them: `.doctrine/state/mem-surface/`. Same tier, same disposability, no
contract change — the receipts are read by key, not by directory listing.

Worth confirming there is no glob elsewhere that assumes the flat layout before
moving.

## Evidence

- `019ff6a6-b6e0` — listing `.doctrine/state` to find the design run dumps ~220 mem files

## References

- `CHR-050` — audit runtime-state scope resolution for `dispatch/` and `review/` subtrees (same tier, adjacent question)
