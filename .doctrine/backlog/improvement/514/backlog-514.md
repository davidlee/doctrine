# IMP-514: Detect a missing or stale install and say how to fix it

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## The gap

Once framework-owned paths are gitignored (SL-281, DEC-375), a fresh checkout
is missing them until someone runs install. An agent in that checkout needs to
work out which of three states it's in:

1. **No binary.** doctrine isn't installed on this system; install it first.
2. **Binary present, checkout not installed.** Run `doctrine install` here.
3. **Install older than the binary.** The last install came from an older
   version; reinstall.

Today nothing tells it which.

## Direction

- **Install stamp.** Install records the binary version that last ran it, in a
  derived, gitignored file. Any doctrine invocation (boot, `status`, a session
  hook) compares the stamp with its own version and prints a one-line
  reinstall recommendation on mismatch or absence. That covers states 2 and 3
  automatically.
- **State 1 can't depend on untracked files or the binary.** The only surfaces
  available are tracked ones, e.g. one line in the tracked `AGENTS.md` /
  `CLAUDE.md` saying "if `.doctrine/state/boot.md` is absent, doctrine is not
  installed for this checkout; see <install docs>". Keep it short: it costs
  tokens in every session.
- A library entry (install / recovery) carries the detail; boot context gets
  one line at most.

## Related

- SL-281 — install artefact ownership; makes the gitignore contract wider, which
  makes this gap bite more often.
- ISS-216 — reseat of changed defs; the stamp gives a trigger for that too.
