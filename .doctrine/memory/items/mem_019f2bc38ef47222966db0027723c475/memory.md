# Close integrate: ISS-030 tree-true (a) false-positives under concurrent agents; scope phantom-reverse-diff to projected paths

During `/close` of a dispatched slice, after `dispatch sync --integrate --trunk`,
the ISS-030 tree-true verify has two checks:

- **(a)** `git diff --quiet HEAD` — no phantom reverse-diff (whole tracked tree
  must match HEAD).
- **(b)** journal `planned_new_oid` == trunk ref — the projection landed.

**Gotcha.** In this repo multiple agents commit to the shared `edge` primary tree
concurrently. Check (a) is deliberately whole-tree (a phantom reverse-diff can span
any projected file), so it **false-positives** whenever a sibling agent is mid-write
on files that have nothing to do with your slice — typically `.doctrine/backlog/*`
or another slice's authored artefacts. The dirty file flickers in and out across
successive `git diff` calls as the other agent commits/reverts.

**How to tell a false-positive from a real desync.** The ISS-030 detector's actual
concern is "did *my* integrate advance the trunk ref but desync *my* projected
checkout." So scope the judgment to the **projected surface**, not the whole tree:

```bash
# projected paths only — src/ + the slice's authored dir. MUST be clean:
git diff --name-only HEAD -- src/ .doctrine/slice/<N>/
# and confirm trunk holds the exact audited close_target tree:
git diff --quiet <trunk> <admitted-close_target-oid>   # must be identical
```

If no projected path is ever dirty across repeated checks, and trunk == the audited
close_target OID, and (b) passes, the integrate is clean — proceed to `done`. The
whole-tree (a) failure is a sibling agent's unrelated WIP, not your projection.

A **real** desync would show *projected* paths (your `src/` files, your slice dir)
in the reverse-diff and would be stable, not flickering. That is the STOP condition.

See [[signpost.doctrine.audit]] for the audit→reconcile→close seam and ADR-012 for
the dispatch integration topology.
