## The trap

A probe that proves a property by observing an **absence** cannot, on its own,
tell *"the thing was not there"* from *"I never looked"*. Both produce the same
empty reading, and the empty reading is the passing one. So the failure mode is
a silent green: the payload reads nothing, reports nothing, and the row it feeds
declares the property enforced.

This bites hardest in shell payloads, where an unset variable is indistinguishable
from a variable set to the empty string:

```sh
# WRONG — an unread field passes
case "$value" in *[!0]*) echo HELD ;; *) echo EMPTY ;; esac

# RIGHT — an unread field convicts
case "${value:-unread}" in *[!0]*) echo HELD ;; *) echo EMPTY ;; esac
```

The substituted default is the whole of the defence: it routes "I read nothing"
onto the *failing* side, where a failure to demonstrate the property belongs.

## Measured, not reasoned

SL-248 PHASE-10 `T8` (table A row 14, capability confinement: `CapBnd`/`CapInh`
must both be empty). With the default removed and the `/proc/self/status` key
deliberately misspelled so `CapBnd` was never matched, **every** test of the row
passed — the probe-arm assertion, the single-axis assertion, the mutant, and the
row verdict itself. Restoring one `:-` expression turned all of it red with a
diagnostic naming the unread field.

## The general rule

For any probe whose passing observation is *nothing*:

1. Report the fields read, by name, and assert the list **whole** — that every
   declared field was read and none was lost. A count or a "my field appeared"
   check will not catch a payload that read half of them.
2. Route an unreadable / unparsed / absent surface to the **failing** side, never
   the passing one.
3. Test both — misspell the surface and watch the suite convict. A guard you have
   not seen fire is a guard you have not got.

The same shape appears outside shell: an empty `Vec` from a parse that silently
dropped everything, a `None` that means "not queried" reused as "not present", a
grep with no matches over a file that failed to open.


## The second half: a degraded reading, not an empty one

Rule 1 above is necessary and not sufficient. The empty reading is only the
easiest defeat; the harder one is a reading that is **shorter than the truth and
still plausible**, because it satisfies exactly the assertions an author writes
first — "the expected token is present".

Measured at SL-248 PHASE-10 `T9`, on the same file. The observation payload
reads the capsule's supplementary group list. Dropping one shell expansion
(`${rest:+ $rest}`) truncated the list to its **first token** — which still
contained the unmapped `65534` every assertion looked for, so the whole test
passed. Nothing was empty; the reading was simply *less*.

What convicted it was a property the truncation could not fake: the list's
**length**, compared against the trusted side's own reading of the same field.
The claim being tested was "the host list was *unmapped*, not *dropped*", and a
count is what distinguishes those; a `contains` never could.

So extend the rule:

4. Name the property the degraded reading is unable to satisfy, and assert that
   one. A cardinality, a total, a comparison against an independently-read
   trusted side — something that moves when the reading is merely shorter.
   `contains(expected)` is rarely it.

## When the probe is a *report* rather than a verdict

Where the absence is "no verdict attaches", there is no failing side to route an
unread surface onto — so the defence moves into the type. Make the reported
reading a sum that renders an unread surface as an unread *variant*, not as a
value that happens to be empty:

```rust
enum Reading { Read { value: String, caveat: Option<String> }, Unread(String) }
```

Then assert the report's shape is invariant: a payload that read *nothing at
all* must produce the same number of observations, all on the `Unread` side. A
report whose length depends on what the payload managed to read cannot support
an absence claim, because the missing entry and the absent property look alike.

## Verifying the guard: a lint is not a conviction

Rule 3 says watch the guard fire. Watch *which* thing fired. One mutation arm at
`T9` removed the unread path, which made its reason constant dead code — the
mutant failed to **compile** under `-D dead-code`. That is a lint conviction,
and accepting it credits the test with evidence the test never produced. Re-run
the arm in a shape that compiles (there, the unread reason moved into an
otherwise-empty `Read`), and only then read the result.
