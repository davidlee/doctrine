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
