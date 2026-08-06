# EVD-013: Capsule teardown needs both the pid namespace and --die-with-parent

Measured while drafting `SL-248` `sec-7` (the conformance suite), to settle
whether row 7's one-property-removed control can fire. `bwrap` 0.11.2, inside
this project's bubblewrap jail.

## Method

Payload `/bin/sh -c 'sleep 137 >/dev/null 2>&1 </dev/null & exit 0'` — the
shell orphans a descendant and exits immediately. Survivors counted
trusted-side with `pgrep -x sleep`.

Two controls on the method itself, both needed:

- An earlier attempt used `setsid`, which is **absent from the jail's PATH**, so
  the payload never started and all arms returned the same false negative.
- A no-sandbox arm confirms the orphan survives when nothing confines it, and
  that the counter sees it. Without that positive control the negative results
  are unreadable.

## Result

| pid namespace | `--die-with-parent` | descendant survives |
|---|---|---|
| present | absent | **yes** |
| absent | absent | **yes** |
| present | present | no |
| absent | present | **yes** |

Teardown requires **both** mechanisms. Neither is redundant with the other, and
neither alone is sufficient.

## Why the pid namespace does not reap on its own

The same measurement shows the reason. Under `--unshare-all` the payload reports
`pid=2`, not `pid=1`, and `/proc/self/ns/pid` differs from the host's. Bubblewrap
runs its **own init as pid 1** inside the new namespace, so that namespace does
not collapse when the command exits and the kernel's kill-the-namespace-on-init-
exit behaviour never fires. The reasonable-sounding inference — *a pid namespace
tears its members down, so `--die-with-parent` is belt-and-braces* — is false.

## What it settles

1. **Row 7's control is a clean single-flag removal.** Dropping
   `--die-with-parent` alone makes the descendant survive, so the control fires
   and the row is provable. Had the pid namespace been redundant, the control
   would have shown the property still holding and reported a correct backend
   `Unproven` — a false red baked into the design.
2. **Row 7 must remove `--die-with-parent`, not the pid namespace.** Both
   removals work, but the pid namespace is also what `sec-3`'s process-visibility
   row depends on. A control removing a mechanism two rows share cannot establish
   which guard produced the result — `RV-346` `F-2`'s objection one level down.
   Hence the general rule `sec-7` adopts: *a row's control removes the mechanism
   unique to that row's property.*
3. **`sec-3`'s process control leaks a process.** Removing the pid namespace
   disables teardown as a side effect, so that control's arm orphans a
   descendant. `DEC-156` already provides the containment (the outer reaper);
   this measurement is why it is needed on that row too, not only on row 7's.

## Adjacent fact, same session

Bubblewrap has **no `--share-pid`**. `--share-net` is its only re-share flag and
its help states it "can only combine with `--unshare-all`". Removing the pid
namespace therefore means assembling the explicit `--unshare-user --unshare-ipc
--unshare-uts --unshare-cgroup --unshare-net` set rather than subtracting a flag.

Relates to [[DEC-156]] (one-property-removed controls), [[SL-248]] `sec-7`.
