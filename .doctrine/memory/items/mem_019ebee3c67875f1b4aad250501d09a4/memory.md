# Guard test must assert the property, not the proxy

A heuristic's guard test must assert the property it guarantees, not the arithmetic that approximates it — else it can't detect its own breakage

When a constant or formula approximates a desired property (a readability floor, a
fit threshold, a budget), a test that pins the *formula's value* gives false
confidence: the thing the formula approximates can drift while the assert stays
green.

Worked example (SL-054, RV-012 F-2). `grid_min_width(cols) = 4·cols-3` is reverse-
engineered from comfy-table 7.2.2's internal width accounting; its purpose is "at
this width every column seats ≥1 readable content char (no 1-char sliver)". Two
tests guarded it and both missed:

- `grid_min_width(6) == 21` pins the *formula*, not comfy's *agreement* with it. A
  comfy-table bump that changes the subtraction leaves the assert green while the
  real floor drifts.
- The boundary test asserted the at-floor render "wraps to >2 lines" — but a 1-char-
  per-column sliver IS >2 lines, so the exact pathology the floor exists to prevent
  PASSES.

The fix is to assert the *property* against the real dependency: at the floor, every
visible column has ≥1 content char and below it the render equals the unwrapped
output. Then a coupling break fails a test instead of silently shipping garbage.

Adjacent to [[mem.pattern.review.invariant-test-must-drive-the-write-seam]] (drive
the real seam, not a pure helper) and [[mem.pattern.parse.toml-error-classification-fragile]]
(pin shapes with canaries when coupled to an external version).

## Second worked example: the proxy that was only ever a coincidence (SL-241, F-P06-8)

The first example is a *formula* standing in for a property. This one is a
**container** standing in for its contents, and it adds the diagnostic rule.

`probe-c2.sh`'s `api-cred` row claimed *"the capsule cannot rewrite the
credential"*. It asserted that a write to `/agent/.claude/<some-other-file>`
failed — the credential's **directory**, not the credential. That passed for
months because the whole of `~/.claude` was ro-bound, so directory-unwritable and
file-unwritable were the same observation.

Then a legitimate change (a `--tmpfs` agent home, so the harness could create its
session directory) made the directory writable while the credential stayed
ro-bound inside it. The row went red.

**The proxy was never sound in the direction that mattered.** A read-only
directory with a secret bind-mounted *rw over it* passes the old leg while the
secret is writable. The row could never have detected the failure it existed to
detect.

### The diagnostic rule this buys

When a guard goes red after a change, the reflex is "the change broke the
property". Ask first: **was this observable ever tied to the claim, or did they
merely coincide under a condition the change just removed?** A red that moves an
assertion onto its actual subject is a strengthening, not a regression — and the
green it replaced was never evidence.

Corollary for absence/refusal assertions: pair them with a **positive control**
in the same medium. "The capsule cannot write the credential" means nothing
unless the capsule is shown writing successfully right beside it, or the row
passes just as well on a broken write mechanism —
[[mem.pattern.harness.grep-negative-needs-positive-control]].

Sibling in the same family, found one probe earlier in the same slice:
[[mem.pattern.tests.smoke-the-capability-not-the-dependency]] — a smoke test
that proves the *dependencies* of a capability and calls it proof of the
capability. Same error, one level up: assert the subject, not something adjacent
to it.
