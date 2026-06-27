# IMP-189: Lifecycle verbs reject prefixed ids with opaque parse error

`doctrine slice status|conformance|record-delta <ID>` accept ONLY the bare
slice number (`163`); passing the prefixed canonical id (`SL-163`) — the form
boot/AGENTS mandate everywhere in prose, commits, and comments — fails with
`error: invalid value 'SL-163' for '<ID>': invalid digit found in string`. The
error is a raw parse-int failure that names neither the expected form nor the
prefix as the culprit. Meanwhile `review new --target` wants the prefixed form.
The inconsistency + opaque error cost an agent ~2 failed invocations on first
reach of any lifecycle/slice verb (surfaced during /audit SL-163).

**Fix options:** (a) accept both forms — strip a leading `SL-`/kind prefix before
the int parse on the slice-scoped verbs; or (b) at minimum, replace the parse
error with a keyed message ("expected a bare slice number, e.g. `163` — drop the
`SL-` prefix"). (a) is preferred: uniform id acceptance across the surface.

Cross-ref: RFC-011 case-notes (token-inefficiency log); sibling to IMP-133 (CLI
usability) and SL-151 (opaque parse failures).
