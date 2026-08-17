When you pin a before-state ahead of deliberately changing a refusal, the
assertion must discriminate **why** the command refused — by message equality,
not by exit status and not by `contains`.

The trap: after the change, the command very often *still refuses*, just from a
later layer. So the loose assertions stay green across the exact change the pin
exists to catch, and the pin silently fails to supersede.

## The instance

SL-238 QUE-221. `backlog after ISS-001 SL-154` is refused today by an author-time
kind gate (`require_item` → `parse_ref`, backlog prefixes only):

    Error: unknown backlog prefix `SL` in `SL-154` (expected ISS/IMP/CHR/RSK/IDE)

SL-238 §6 removes that gate on purpose. Mutation-tested by deleting both
`require_item(&root, to)?` call sites: the `--remove` leg then falls through to
the zero-count bail and emits

    Error: ISS-001 has no after edge to SL-154

Still exit 1. Still names `SL-154`. Both `assert!(!out.status.success())` and
`assert!(stderr.contains("SL-154"))` pass **after the behaviour changed**. Only
equality on the message goes red.

## How to apply

- Pin the refusal **message** by `assert_eq!`, and name the discriminated reason
  in the failure label ("refused on the KIND", not "refused").
- **Mutation-test the pin before trusting it.** Remove the gate you expect the
  future phase to remove, run the pin, confirm red, revert. A characterisation
  test that is green on first write has proven nothing until you have seen it
  red — that is the whole point of red/green, and a before-state pin is the one
  case where the red arrives in a *later* phase, so you must manufacture it now.
- Add a positive control (same verb, admissible input, succeeds) to guard the
  other vacuity direction — a fixture broken in some unrelated way also refuses.
- Make the target **exist on disk**, so the refusal is provably about
  admissibility rather than absence.

## Related

[[mem.pattern.testing.grep-for-the-pin-before-characterising]] — the prior step:
before writing a pin, look for the one that already exists. Together: find the
pin, then make sure the pin can actually fail.
