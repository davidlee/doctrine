# IMP-162: Lint over-broad design-target selector globs (SL-147 F-7 follow-up)

## Problem

Design-target selectors use globs (e.g. `src/**`) that expand to files far
beyond the slice's actual touch surface. This:
- Inflates the review priming cache with irrelevant paths
- Makes `slice conformance` flag incidental foreign-slice files as `undeclared`
  when the glob catches them
- Blunts the adversarial review surface — a reviewer can't distinguish the
  slice's intended scope from accidental glob expansion

SL-147 F-7 identified the class; this item is the lint to catch it at authoring
time.

## Fix direction

- **Selector breadth lint**: flag `[[selector]]` globs whose resolved file
  count exceeds a threshold (e.g. >20 files) or whose expansion crosses module
  boundaries (touches files in `src/commands/` AND `src/engine/` when the slice
  targets only one subsystem).
- **`doctrine check design <ID>`** or `slice selector doctor` (see IMP-256):
  runs at `/design` lock time, warns on over-broad globs, suggests narrowing.
- Not a hard block — there are legitimate broad selectors (e.g. a
  cross-cutting refactor) — but the default should be narrow.
- Complements IMP-256 (selector completeness — the inverse check, ensuring
  selectors aren't too narrow).

## Context

SL-147 (domain-map) established the selector surface for review priming. This
item was deferred as a follow-up (F-7).

## Related

- SL-147 F-7 (originating finding)
- IMP-256 (selector completeness check — the inverse problem)
- IMP-025 (content-hashed-path-set — the priming primitive selectors feed)
