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
