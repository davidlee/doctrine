# Doctrine skill and route map

Doctrine ships skills under `plugins/doctrine/skills/` (glob
`plugins/doctrine/skills/**`). Each governs a situation; `/route` chooses one
*before* you act. Route-before-you-act is the gate, not a suggestion.

The route table's When → Skill mapping (authority: `.doctrine/state/boot.md`):

- Correctness depends on governance / unfamiliar subsystem → **/canon** +
  **/retrieve-memory**
- Substantive work, path not yet clear → **/preflight**
- Code-changing intent, no governing slice → **/slice**
- Slice exists, design missing or stale → **/design** → **/inquisition**
- Design locked, no plan → **/plan**
- Expanding the next phase before executing → **/phase-plan**
- Plan approved, phase active → **/execute**
- Implementation done — evidence / reconciliation → **/audit** → **/close**

Mid-flight, any stage:

- Unanticipated obstacle / tradeoff / emergent complexity → **/consult** (don't
  improvise past it).
- Durable gotcha or pattern discovered → **/record-memory**.
- Authoring durable product / technical intent (the evergreen specs, upstream of
  per-slice design) → **/spec-product**, **/spec-tech**.
- Finished a coherent unit of work → **/notes**.
- Handing off to a fresh context → **/next** (or **/handover**).

The lifecycle ordering these stages follow:
[[signpost.doctrine.lifecycle-start]]. Why routing is a hard gate:
[[concept.doctrine.routing-gate]]. The CLI verbs the skills wrap:
[[signpost.doctrine.cli-command-map]].
