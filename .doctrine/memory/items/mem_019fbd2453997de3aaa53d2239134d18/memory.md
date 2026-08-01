# A guard that refuses by `exit` is swallowed by command substitution

Found live in SL-241 PHASE-01 (F-P01-1), building the rig's I6 real-repo guard.

## The footgun

```bash
guard_not_real_repo() { ...; exit 3; }        # refuses by exit
rig_enter() { local r; r=$(resolve_root); guard_not_real_repo "$r"; printf '%s' "$r"; }

dispatch "$(rig_enter)"                        # ← the bug
```

`$( … )` runs in a **subshell**. The `exit 3` ends the subshell, not the script.
So on a mis-resolved root the guard printed its refusal to stderr, the
substitution evaluated to the **empty string**, and `dispatch` ran anyway — with
no root at all. Observed exiting **0** on a root inside the protected repo.

`set -euo pipefail` does **not** save you. A failed command substitution
propagates errexit in **assignment** position (`x=$(false)` exits) but **not in
argument position** (`f "$(false)"` does not) — the status is discarded once it
becomes a word.

## Why it survives testing

The unit probe was **green the whole time**. A guard probe has to run the guard
in a subshell in order to read its exit status without dying:

```bash
if ( guard_not_real_repo "$case" ); then echo FAIL; else echo ok; fi
```

So the probe and the bug use the same construct, and the probe is blind to it by
construction. "A guard that runs late is not a guard" has a sibling:
**a guard whose refusal cannot reach the entry point is not a guard**, and only
an END-TO-END observation at the real entry point catches it. shellcheck does
not flag it either.

## The fix

Have the guard wrapper **publish a variable** rather than print, and call it as
a **statement**:

```bash
rig_enter() {
  [ "$BASHPID" = "$$" ] || { warn "called from a subshell — refusal cannot propagate"; exit 3; }
  RIG_ROOT=$(resolve_root)      # this substitution is harmless: resolve never exits
  guard_not_real_repo "$RIG_ROOT"
}

rig_enter && dispatch "$RIG_ROOT"
```

The `BASHPID != $$` tripwire is the durable part: it turns a silent recurrence
into a loud one, and it costs one line. Put it on the entry *wrapper*, not on the
guard itself — a probe legitimately subshells the guard.

## How to apply

1. A guard that refuses by `exit` MUST run in the entry shell. Never
   `f "$(guard_wrapper)"`.
2. Prefer publishing a global to printing, when the value and the refusal travel
   together.
3. Test the guard at the **real entry point**, not only as a unit. See
   [[mem.pattern.harness.grep-negative-needs-positive-control]] — same shape: the
   check that proves the mechanism must not share the mechanism's blind spot.
