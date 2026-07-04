# IMP-259: review prime — support non-slice targets (SPEC, ADR, design artefacts)

## Problem

`doctrine review new --target SPEC-003` succeeds — the target is accepted and an
RV is minted — but `review prime RV-NNN` refuses: "priming is slice-selector
only, needs a slice target." An inquisition against a spec or ADR target is
forced to proceed **unprimed**, gathering evidence via manual `show` calls
rather than riding the staleness-signal cache.

Witnessed twice:
- SPEC-003 inquisition (RV-236): prime refused → manual evidence gathering
- SPEC-023 inquisition (RV-237): same, prime refused

The mismatch is structural: `review new` accepts any entity reference as a
target, but `prime` exclusively resolves slice selectors. Either `new` should
refuse non-slice targets (fail early), or `prime` should walk non-slice targets'
authored files generically.

## Fix direction

- **Generic file walk**: for non-slice targets, `prime` resolves the target's
  authored files via `doctrine <kind> paths <ID>` (or the entity engine's
  file-set query), hashes them, and populates the cache. A spec's files are its
  TOML + MD + requirements; an ADR's are its TOML + MD. This is strictly
  simpler than the slice-selector path (no glob expansion needed).
- **Fail-early alternative**: `review new` rejects non-slice targets with a
  clear message if prime can't support them. Less useful (blocks spec/ADR
  inquisitions entirely) but consistent.
- Prefer the generic walk: it's low-risk (non-slice targets have small,
  well-defined file sets) and opens the RV surface to all entity kinds.

## Related

- RFC-011 case-notes: SPEC-003 inquisition, SPEC-023 inquisition
- ISS-059 (prime fails on directory/symlink selectors — adjacent, same verb)
- IMP-024 (review baton parent-tree residency)
