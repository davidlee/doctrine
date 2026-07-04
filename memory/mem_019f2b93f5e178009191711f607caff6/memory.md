# Dispatch mechanics — funnel model and mid-operation traps

Orientation signpost for the `/dispatch` fork→verify→import→land funnel and its
sharp edges. Two tiers, by access pattern:

- **The model** (read once, up front): the shipped reference doc
  `dispatch-mechanics.md` explains the funnel — explicit fork base `B`, scoped
  verify, the patch-id landed-oracle and its squash blind spot, shared-trunk
  landing races, worker-identity fencing, and worker self-discard traps.
- **The traps** (retrieve mid-operation): the sharp edges you hit *while*
  driving a dispatch. Retrieve with `doctrine memory retrieve dispatch`.

## Trap territory (what to retrieve when)

| When you are... | Look for traps tagged... |
|---|---|
| Spawning a worker / choosing its fork base | base-control, session-HEAD, worker isolation |
| Importing a delta / verifying the funnel | 3-way import, scoped verify, build-artifact provisioning |
| Deciding a fork is spent (gc/cleanup) | landed-oracle, patch-id, squash, gc |
| Landing / integrating on a shared trunk | close-integrate, trunk-race, dirty-worktree |
| Auditing or closing a dispatched slice | audit fork-ban, distrust-green-claim, candidate-detach |
| On a specific arm (claude / codex / pi) | arm-routing, subagent-identity, RPC hygiene |

## Notes

Shipped orientation master (ADR-002 global class, ADR-005 tiering: signposts
route, reference docs explain). The detailed trap memories are being promoted
from project-local to shipped reference tier incrementally (CHR-036) — until
then, some live only in the originating repo's local corpus.

Related shipped decisions: ADR-006 (worktree posture), ADR-008 (jail
isolation), ADR-011 (harness-agnostic spawn), ADR-012 (integration topology).
