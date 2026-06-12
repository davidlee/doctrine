# Dispatch worker verify gate: run with DOCTRINE_WORKER unset when tests mint entities

A dispatch/worktree **worker** self-arms with `export DOCTRINE_WORKER=1` (the
prompt contract that disarms its own doctrine-mediated repo writes via the D2a
guard). But the guard keys off the env var **globally**, so it also blocks the
authored-id minting that the worker's own TDD tests legitimately perform in
tempdirs (`spec req status`, `reconcile`, any `*_new`/materialise path). Leaving
`DOCTRINE_WORKER=1` set in the test shell makes those e2e/unit subprocesses fail
spuriously ("guard refused mint").

**Fix:** run the green gate with the var unset for the duration of the run:
`env -u DOCTRINE_WORKER just check`. The self-arm is a contract about the
worker's *repo* writes (of which a source-only worker makes none); it is not meant
to disarm tempdir test fixtures. The real protection against a worker minting
authored ids is the orchestrator's import-time **R-5 belt** (`git diff --name-only
B..S` rejects any `.doctrine/` touch), which fails-closed on the trusted side — so
running tests with the guard off costs no safety.

First hit in SL-044 B·P1 (worker discovered it); pre-warned for B·P2/B·P3 whose
whole point is minting RECs/requirements. See [[mem.pattern.dispatch.fork-rung3-base-not-session-head]]
and the dispatch skill's "DOCTRINE_WORKER fails OPEN (C-I)" note.
