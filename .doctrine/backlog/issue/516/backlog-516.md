# ISS-516: memory search matches no memory-body text

## What

`doctrine memory search` matches title, summary and tags, but not the memory
body. Probe (2026-10-01) against `mem_019fc33b` ("Invert an ordering by wrapping
the subject…"):

| query | where the term is | hit? |
|---|---|---|
| `compose` | summary | yes |
| `falsifiability` | tags | yes |
| `wrapper`, `drifts`, `unmutated` | body only | no |

Second symptom, cause unknown: `memory search mutation` returns two rows and
omits `mem_01a0ba88` (status active, verification unverified), whose title and
summary both contain "mutation". Possibly an unverified-status filter or a stale
derived index; disclose it if deliberate (STD-003).

## Why it matters

Agents search for the hazard they are about to hit, in the words of the body
("dead_code", "-D unused", "python3"), and get "no match" while the memory
exists. EVD-031 notes lint traps recurring despite memories describing them.
`memory search python3` returned no match while 10 memory files contained it.

## Fix sketch

Index body text (with lower weight than title/summary), and make the second
symptom's exclusion visible or remove it.
