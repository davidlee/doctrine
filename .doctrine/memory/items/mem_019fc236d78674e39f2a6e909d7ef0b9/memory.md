When the hostile input under test is **that something happened again** — a
duplicate signal, a retried request, a second harvest, a re-run of an idempotent
step — the observable must assert the **occurrence count**, not merely that the
outcome was unchanged.

Sameness is exactly what a *single* occurrence also produces:

- a waiter that answers the same way twice answers the same way when asked twice
  about one event;
- a ref that never moves never moves;
- a content-addressed store returns the same id whether it was written once or
  twice.

So every "unchanged" clause passes vacuously when the repetition silently never
happened — a mutation that no-ops the repeat **survives**, and the test reports
idempotency surviving an experiment in which nothing was repeated.

**Do this.** Record the repetition itself as data at the moment it is performed,
and assert it before the sameness clauses:

```sh
# after the second ring: the bell was rung TWICE with ONE distinct line
printf 'rings=%s\n'          "$(wc -l <"${bell}")"        >>"${evidence}"
printf 'rings-distinct=%s\n' "$(sort -u -- "${bell}" | wc -l)" >>"${evidence}"
```

The distinct-count matters as much as the total: it is what says the repeat was
**verbatim** rather than a second, different event wearing the repeat's name.

Give the repeating act its own named function so a falsifiability round can
no-op it (`c3_h14_rering`, beside `bundle_symlink`); an act with a name is an
act a mutant can remove.

Found by mutation testing before the row was scored — SL-241 PHASE-05 F-P05-30,
`scripts/spike-capsule/lib/instantiations.sh` (H14, the duplicated doorbell).

Same family as [[mem_019fbd70c9247f02a4c25c76fa3ca57e]] (an absence assertion is
evidence only if its subject is reachable) and
[[mem_019fa18161f47651af7687d8dccbbc67]] (a negative grep needs a positive
control): the observable that passes when its subject was never there.
