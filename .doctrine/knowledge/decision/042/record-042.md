# DEC-042: Invalid observation controls are diagnostic and inert

An invalid observation-control component does not affect the resolved active
projection. Resolution emits deterministic diagnostics and treats every
control in that weakly connected component as inert; its parseable public
observations remain independently active.

Invalidity includes dangling targets, cycles, multiple successors,
kind-incompatible replacements, conflicting terminal controls, and malformed
controls whose target relationships can be recovered. A malformed control
whose relationships cannot be recovered is diagnostic and inert by itself.
Valid components elsewhere in the corpus continue to resolve normally.

This fail-open treatment preserves raw occurrence signals and keeps tolerant
reads useful. Quarantining the component would make valid raw observations
disappear because of a bad correction, while failing the whole query would let
one bad record deny access to the collection. History views always retain the
invalid controls and their diagnostics so the conflict cannot pass silently.
