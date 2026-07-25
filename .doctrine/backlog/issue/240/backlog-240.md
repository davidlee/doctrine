# ISS-240: Worker-mode guard resolves root from CWD, ignoring -p

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

Inside a dispatch worker's worktree, `doctrine check gate` fails ~8 e2e targets
(`e2e_link_unlink`, `e2e_adr_cli_golden`, `e2e_dep_seq_verbs`, …) with:

```
worker fork (signal: marker): refusing authored write
```

The same targets pass when the identical suite runs inside the `worker_commit`
commit gate, so this is a worker-shell artifact rather than a real regression.

## Cause

`src/commands/guard.rs:461` resolves the project root from **CWD** rather than
from the `-p <path>` the test passes. Each affected e2e sets up its own temp
root and drives the CLI with `-p <temp root>`; the guard ignores that, walks up
from CWD instead, finds the *worktree's* `.doctrine/state/dispatch/worker`
marker, and refuses the authored write the test is exercising.

## Why it matters

It is not a correctness bug in shipped behaviour — the marker is doing its job
against the wrong root — but it makes **a worker's own `doctrine check gate` run
useless as a signal**. Combined with `just validate` short-circuiting to a no-op
inside a worker fork (`justfile:36-40`), a dispatch worker has no reliable local
green signal at all: it must burn a `worker_commit` round trip to discover
whether its delta passes the gate, and it must be told in its prompt to ignore a
block of failures that look exactly like real ones.

## Fix sketch

Thread the explicit `-p` root into the worker-marker lookup so the guard tests
the marker of the root it is actually operating on, not the one it happens to be
standing in. Check whether other CWD-rooted lookups in `guard.rs` have the same
skew.

## Provenance

Surfaced by the SL-228 PHASE-03 dispatch worker (deviation 7 of its hand-back);
`guard.rs` was last touched by SL-228 PHASE-01/02. Also logged as RFC-011
case-note item 2 under `[dispatch-agent / phase-plan; SL-228-P03-drive]`.
