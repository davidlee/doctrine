## The situation

A test measures a value correctly but cannot establish *what produced it*,
because the environment it runs in already exhibits the same value. The reading
is right; the attribution is not available from in here.

Concretely (SL-248 PHASE-10 `T9`): the capsule reads `NoNewPrivs: 1`. The jail's
own parent process also reads `NoNewPrivs: 1`. So an in-jail run cannot tell
whether the sandbox backend set the bit or the capsule inherited it. On a host
whose parent reads `0`, the same reading *is* attributable.

## The tempting wrong move

Write the caveat as a constant beside the observation — a sentence saying
provenance is not establishable. It reads as diligence. It is **wrong on every
host that can attribute**, permanently, and nothing will ever tell you so,
because a constant has no failure mode.

## The move

Make the caveat a **pure function of the environment's own reading**, taken from
the trusted side at run time:

- host already exhibits the value → attach the caveat (cannot attribute);
- host does not exhibit it → attach nothing (the reading *is* the backend's);
- host could not be read → attach a **third, distinct** value.

That third case matters as much as the first two. Collapsing "could not measure"
into "measured, and it was unset" is the same class of lie as a vacuous pass: it
converts an absence of evidence into evidence.

The caveat is then a measurement that appears and disappears with the facts,
rather than an apology that outlives the condition it describes.

## The test obligation this creates

The environment read feeding the caveat must be asserted **positively**, against
the real source, in the same test. If that parse silently fails, every host
looks unreadable and every reading carries the "could not establish" caveat
forever — an apology wearing a measurement's clothes, and green. Assert a
successful parse of a key that exists, and a `None` for a key that does not, so
that finding one establishes something.

## Generalises to

Any "not available in CI", "not measurable under this harness", "skipped on
platform X" note. If the condition is readable at run time, read it. A caveat
that cannot become false is documentation of a belief, not of a system.
