# ISS-249: Coord pre-commit hook refuses every commit when no chained hook exists

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Symptom

In a coordination worktree carrying the SL-228 PHASE-02 hook
(`install/git-hooks/pre-commit`, installed per-worktree by `dispatch setup`),
**every** `git commit` is refused with no diagnostic when the resolved chain
target does not exist — including the orchestrator's own `dispatch commit`.

## Cause

The hook runs under `set -eu` (line 13). The chain tail is:

```sh
resolve() { [ -e "$1" ] && printf '%s' "$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"; }
self="$(resolve "$0")"
next_real="$(resolve "$next")"      # <-- aborts here
```

When `$next` (the resolved non-worktree `pre-commit`) is absent, `[ -e "$1" ]`
is false, so `resolve` exits non-zero; the exit status of an assignment from a
command substitution *is* that of the substitution, so `set -e` terminates the
hook with a non-zero status and git refuses the commit. The `[ -x "$next" ]`
guard on the next line — which correctly handles the no-chained-hook case — is
never reached.

Nothing is printed, so the operator sees a bare refusal with no cause.

## Evidence

- Reproduction of the shell semantics in isolation:
  `sh -c 'set -eu; resolve() { [ -e "$1" ] && printf "%s" "found"; };
  next_real="$(resolve /nonexistent)"; echo REACHED'` → exits 1, `REACHED`
  never prints.
- The condition holds in this very repo: `core.hooksPath` (local) is
  `/workspace/doctrine/.git/hooks`, and `.git/hooks/pre-commit` does not exist.
- Observed live by the SL-228 PHASE-07 memory-blind benchmark subject in a
  fresh coordination worktree — it diagnosed the cause unaided and patched its
  installed instance to proceed.

## Why it survived SL-228's own drive

`dispatch setup` installs the hook, and SL-228's coordination worktree was
created *before* PHASE-02 shipped it — `git config --worktree core.hooksPath`
is unset there and no `doctrine-hooks/` dir exists. So the slice that built the
hook never ran it. Any **fresh** coordination worktree hits it, which is the
first-install case for every client project without a global or local
`pre-commit`.

## Fix direction

Make the chain resolution non-fatal — the intent is already expressed by the
`[ -x "$next" ]` guard:

```sh
next_real="$(resolve "$next" || true)"
```

or resolve inside the `if`. Add a VT for the **no-chained-hook** cell: the
existing hook tests cover the chained-global case (design §11 REQ-389, R4) but
not its absence — the degenerate case that ships by default.

Relates to SL-228 PHASE-02 (REQ-389) and ADR-011 (mechanism over prose).
