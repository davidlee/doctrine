# ISS-040: Sequencing verb arg order too easy to reverse

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## Problem

The soft-sequence verbs (`after`, `needs`) take bare positional
`<SOURCE> <TARGET>` with the verb implied:

```
after SL-129 SL-115     # encoding: SOURCE is-after TARGET → SL-129 waits on SL-115
```

The encoding is "SOURCE after TARGET", but natural English reads "after X, do Y"
— i.e. X first. The two parse in opposite directions, so the edge is trivially
authored back-to-front. `link` does not have this problem: it carries the verb
between the operands (`link SL-048 governed_by ADR-010`), reading subject-verb-object.
`after`/`needs` drop the verb, leaving two refs whose order is not self-evident.

## Evidence (real instance)

Found during SL-129 plan review. Intended order (SL-115 design R1): *SL-129 lands
first, SL-115 after it* — edge should be `after SL-115 SL-129` authored on SL-115.
What actually landed:

- `slice-129.toml` carried `after = [{ to = "SL-115" }]` — the reverse: SL-129
  waits on SL-115. Inverts the whole dependency.
- The same line's inline comment said "SL-115 ... must land after SL-129" —
  contradicting its own edge direction. Author clearly knew the intent; the verb
  encoding flipped it anyway.
- The edge also landed on the wrong entity (SL-129 not SL-115), but the direction
  reversal is the recurring trap.

## Possible fixes (not yet decided — design later)

- **Help-text disambiguation** (cheap): rewrite the usage blurb to state the
  reading unambiguously, e.g. "`after <DEPENDENT> <PREDECESSOR>` — DEPENDENT runs
  after PREDECESSOR", drop the bare `after SL-060 SL-047` example for a
  role-labelled one, and echo the resolved sentence back on success
  ("SL-129 will run after SL-115").
- **Confirmation echo** on write — print the human-readable sentence so a reversed
  edge is caught at author time.
- **Verb-in-the-middle form** to match `link` (`after SL-115 then SL-129` /
  `seq SL-129 before SL-115`) — larger surface, only if help text proves
  insufficient.

Scope is the sequencing-verb UX (`after`, `needs`, and their `--remove`/`--prune`
variants), not the relation engine. Likely a small slice or a quick-fix.
