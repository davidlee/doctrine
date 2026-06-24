# Doctrine skill and route map

Doctrine ships skills under `.doctrine/skills/` (glob
`.doctrine/skills/**`). Each governs a situation; `/route` chooses one
*before* you act. Route-before-you-act is the gate, not a suggestion.

The routing table is authoritative in boot.md — see
[[concept.doctrine.boot-snapshot]] for the `## Routing & Process` section
that carries the When → Skill mapping.

The lifecycle ordering these stages follow:
[[signpost.doctrine.lifecycle-start]]. Why routing is a hard gate:
[[concept.doctrine.routing-gate]]. The CLI verbs the skills wrap:
[[signpost.doctrine.overview]]. See [[concept.doctrine.reading-entities]]
for the read-via-show rule, and [[signpost.doctrine.recording-memories]] for
the capture-retrieve cycle.
