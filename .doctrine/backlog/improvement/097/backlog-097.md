# IMP-097: Altitude assessment framework for requirement placement (product vs tech, C4-level rules)

Surfaced by RV-078 (inquisition of SL-098 design, F-4).

SL-098's orphan placement workflow (§6a) depends on correct altitude decisions —
product spec vs tech spec, and at what C4 level within tech. No decision framework
exists. "Domain reasoning by agent" is not a substitute.

This framework should define:
- Criteria for product vs tech spec placement (when does a requirement belong in a PRD?)
- C4-level assignment rules for tech specs (container, component, code)
- How to use existing spec `c4_level` and `descends_from` as reference points
- Edge cases: requirements that span levels, requirements that don't fit any existing spec

Blocker for: SL-098 (soft — can proceed with `/consult` guardrail, but the
framework is needed for reliable operation).
