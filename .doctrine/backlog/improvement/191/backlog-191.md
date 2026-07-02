# IMP-191: slice status: no read-only query form (setter-only overload)

`doctrine slice status <ID>` is transition-only — it requires `<ID> <STATE>` and
errors on a missing positional. There is no cheap read-only "what lifecycle
state + phase rollup is this slice in?" affordance. Discovering current state +
legal transitions needs a `--help` round-trip or `slice list | grep`.

## Cost (RFC-011 instrumentation)

Recurred across 4 sessions (SL-166 orientation/audit/close, SL-163). Every
lifecycle-touching session re-pays the discovery. Reader/writer overload on a
single verb name is the root anti-pattern.

## Proposal

`slice status <ID>` with no `<STATE>` prints current lifecycle state + phase
rollup (read-only); the setter form keeps requiring `<STATE>`. Standard
read-when-bare / write-when-armed split.

## Sibling verbs (same pattern)

All share the reader/writer overload — `<STATE>` / `--status` is required, no
bare read-only form. `review status` (already read-only) is the model to follow.

| verb | arg pattern | notes |
|---|---|---|
| `slice status` | `<ID> <STATE>` (pos) | primary target here |
| `revision status` | `<REFERENCE> <STATE>` (pos) | same positional shape |
| `adr status` | `<ID> --status <STATUS>` (flag) | flag-based, same gap |
| `policy status` | `<ID> --status <STATUS>` (flag) | |
| `standard status` | `<ID> --status <STATUS>` (flag) | |
| `rfc status` | `<ID> --status <STATUS>` (flag) | |

Fix pattern: make the transition argument optional; when absent, print current
state (read-only). Implement for one, then replicate.

Surfaced by: RFC-011 case-notes. Sibling of IMP-189 (id-form). Platform issue —
affects every doctrine user, not a project-local convention.
