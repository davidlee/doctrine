# ISS-298: design show --full widens nothing

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

## What was observed

In the CHR-049 subject run `dr-019fc13a`, at 06:47:25Z and 06:47:29Z:

```
doctrine design show 243
doctrine design show 243 --full
```

returned **byte-identical** output — sha256 `4a3a9e273d97204e…`, 819 B each.

That output contains:

```
changes: UNAVAILABLE — the change log covers revisions from 1 onward, and
revision 0 is below that floor; see `design show --full`
```

So `--full` rendered a pointer to `--full`. The reader is sent in a circle.

## Why it matters

`--full` is the documented widening of the turn view (`design show` help:
*"`--full` widens it"*). On a cold run it widens nothing, and the one line that
explains a gap advertises the flag the reader already passed.

Two candidate readings, not separated here:

1. `--full` is not wired to the changes projection at all;
2. it is wired, and the revision-0-below-floor case short-circuits before the
   flag is consulted — in which case the message, not the flag, is the defect,
   and it should say the floor makes those revisions unrecoverable rather than
   naming a flag that cannot help.

The second is the more likely and the cheaper fix, but the byte-identical output
is consistent with both and this item does not pick one.

## Provenance

Found while moderating CHR-049's measurement exercise. It is a defect in the
subject surface, not in the exercise instrument. Filed mid-run rather than held,
so it is not lost; the observation is recorded independently under
`.doctrine/observations/records/`.
