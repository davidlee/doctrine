A design that prices a change asserts something a reviewer is expected to
check. If the figure names no population, it cannot be checked — and the cost
of that falls entirely on the reviewer.

## The asymmetry

Writing "14 references in tree" costs one clause. Verifying it means guessing
the predicate: `grep -rn "design show"` over the repo returns ~100 files
spanning observation records, review ledgers and slice notes, and deciding
which subset is migration-relevant is most of the work. The author knew the
denominator and did not write it down.

Observed on `SL-246` (`RV-370` `F-16`): the count was also **wrong** — it
omitted five committed memories that documented the verb's old meaning — and
the omission was invisible precisely because no population was stated. A
reviewer cannot audit the edges of a set whose boundary is unstated.

## The rule

A count asserted as *measured* or *counted rather than estimated* must state
the population it counted over. A table of populations with a count each is
usually the right shape, because it also makes the gaps visible to the author
while writing it:

| population | count |
|---|---|
| shipped prose | 1 |
| emitted strings | 4 |
| tests | 5 files |
| memory corpus | 5 items |

If the population is genuinely unbounded, say *estimate* and drop the word
measured. An honest estimate costs a reviewer nothing; a false measurement
costs them the whole search.

## The population that gets forgotten

Code sweeps and goldens reach code. They do not reach **shipped prose in
skills**, or the **memory corpus** — and a stale memory is the worse of the
two, because `memory retrieve` and the `memory surface` hook *inject* it into
agent context rather than waiting to be consulted. When pricing a rename or a
default change, enumerate those two explicitly; neither turns a build red.
