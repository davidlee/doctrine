## The shape

Some invariants are not about a data structure but about **a process-wide
resource**: file-descriptor flags, the working directory, `umask`, signal
dispositions, an environment variable. A mutex around every *production* mutation
of one looks complete, and is not — a `#[cfg(test)]` caller that pokes the same
resource is another writer, and the test harness runs its tests on threads of the
**same process**.

The failure is characteristically nasty:

- both tests pass **alone**;
- only the whole suite reds, and only sometimes;
- the red lands on the *victim*, which is correct code, while the culprit is a
  test that looks self-contained.

## The instance (SL-248 PHASE-09, `RV-346` `F-31`)

`doctrine-control`'s bubblewrap backend clears `CLOEXEC` on a status descriptor,
forks, and restores it, all under a `DESCRIPTOR_WINDOW` mutex — because the flags
are read by `fork` and are process-wide. A separate test asserting the sweep
marks every descriptor called `mark_inherited_descriptors_close_on_exec()`
*without* the guard. Fired from one test thread it re-marked another thread's
just-cleared status descriptor inside that thread's fork window; the child
exec'd without the descriptor and the handover file came back empty.

## What to do

- When you write a guard, ask what else in the **binary** touches the resource —
  `#[cfg(test)]` code included — not just what else in the production paths does.
- Say so on the guard: name the resource, not the call sites.
- If a test must mutate the resource, it takes the guard. Serialising a handful
  of tests costs milliseconds; this class of flake costs hours and is usually
  misdiagnosed as "the CI box is loaded".
- A doc comment that says "this is benign because nothing else does X yet" is a
  dated liability. It was written true and will be read false.


## It bites in both directions, and it recurs

Within one phase the same corpus paid for it three times, so treat one sighting
as a class, not an incident:

1. a **sweep in a test** re-marked another thread's cleared descriptor — the
   victim's child lost the handover;
2. a **capsule run's production sweep** re-marked descriptors a *test* had
   deliberately opened inheritable, between the open and the assertion — twice,
   in two different tests.

Direction 2 is the one people miss: the test is not mutating anything, it is
only *reading*, and a reader of process-wide state is as much a party to the
guard as a writer.

The repair that stuck was a named `pub(crate)` seam — `hold_descriptor_window()`
— rather than a private static plus a comment. A guard nobody outside the module
can take is a guard that will be skipped by everyone outside the module.
