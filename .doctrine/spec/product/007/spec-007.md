# PRD-007: Boot & Governance

## 1. Intent

An agent working in a governed project must know the project's rules of the road
before it acts — its routing and process discipline, its accepted decisions, the
durable knowledge it has accumulated, and where the authoritative sources live.
Today that orientation is re-earned every session: the agent rediscovers the same
stable governance state turn by turn, reading the same files and running the same
listings, paying again for knowledge that did not change. The cost is paid in every
session, recurs forever, and scales with the size of the governance corpus — yet the
underlying state is almost always identical to the last time it was learned.

This capability removes that recurring tax. It projects the project's stable
governance state into the agent's context as a single, ready-made orientation
surface, so an agent arrives already oriented rather than rediscovering. The value
is threefold: orientation becomes effectively free per session and is paid for only
when governance actually changes; a project gains one place — owned by its
maintainers — to point agents at what matters without restating the sources of
truth; and the surface stays honest, so an agent (or its operator) can tell when what
context carries has fallen behind what the project actually governs. The desired end
state is that every session begins oriented, that orientation is current or visibly
flagged when it is not, and that maintainers can shape it without forking the
authoritative material it draws from.

## 2. Scope

In scope:

- Projecting a project's stable governance state — routing/process discipline,
  accepted decisions, durable memory pointers, and orientation cross-references —
  into a single agent-readable orientation surface.
- A user-owned governance pointer layer that maintainers author and the tool
  preserves, contributing project-specific orientation to the projected surface.
- Keeping the projection honest: detecting when the projected surface has drifted
  from current governance, or when parts of it are not yet populated, and surfacing
  that rather than presenting stale content as fresh.
- Carrying the tool's own resolved invocation handle in the surface, so an agent
  reaches the correct tool regardless of installation layout.
- Establishing the projected surface in each agent harness's session-start context,
  and an extensible seam by which new categories of governance join the surface
  without reshaping it.

Out of scope:

- The authoritative governance content itself — decision records, evergreen
  specifications, conventions, and durable memory are owned by their own surfaces;
  this capability points at and projects from them, it does not replace or restate
  them.
- Producing, editing, or curating decisions, memories, or specifications — those are
  governed elsewhere; this capability only assembles a read surface from them.
- Per-turn injection of governance content during a session, and any mechanism that
  re-pays orientation cost mid-session.
- Governance enforcement or gating — the surface orients; it does not police.

Boundary: this capability is a projection and a pointer layer, never a source of
truth. Every fact it carries is owned by an authoritative surface elsewhere; the
projection may be discarded and regenerated at any time without loss.

## 3. Principles

- **A projection, never a source of truth.** Everything the surface carries is owned
  elsewhere and reproduced here for convenience; the projection is disposable and
  regenerable, and is never the place a governance fact lives.
- **Pay for orientation on change, not on cadence.** The surface costs its full price
  only when the governance it reflects actually changes; an unchanged surface must
  not impose a recurring per-session cost.
- **The pointer layer points; it does not compete.** The user-owned layer carries
  short, stable cross-references and orientation, never a fourth copy of the
  decisions, specs, or conventions it points at.
- **Honest about its own freshness.** A surface that has fallen behind current
  governance, or that is not yet fully populated, must say so; presenting stale
  projection as current is worse than admitting drift.
- **Maintainer-authored content is preserved, never clobbered.** Material a project's
  maintainers author into the pointer layer survives regeneration; the tool seeds it
  once and thereafter only reads it.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded as
requirement entities and appear under the synthesized Requirements section below.
This section carries only the constraints and invariants that bound every valid
implementation.

Constraints:

- The projected surface must reach the agent's context at session start without a
  per-turn or mid-session injection that re-pays orientation cost.
- The surface must draw only from existing authoritative governance surfaces; it must
  not introduce a new place where a governance fact originates.
- The pointer layer the tool seeds for a project must never be overwritten once a
  maintainer has authored it.
- Honesty reporting must distinguish a drifted surface on disk from the orientation
  already resident in the running session, and must not claim the resident
  orientation is fresh.

Invariants:

- The projected surface is always derived and disposable: it can be deleted and
  regenerated from authoritative state with no loss.
- Regenerating from unchanged governance yields a surface that imposes no fresh cost
  — identical inputs produce an identical surface.
- The surface always carries a resolved handle for invoking the tool, even when every
  other section is empty.
- A governance category with nothing to contribute renders as a benign placeholder,
  never as an absence that breaks the surface.

## 5. Success Measures

- An agent beginning a session is already oriented to routing, accepted decisions,
  durable memory pointers, and where authoritative sources live, without spending
  turns rediscovering them.
- The orientation cost recurs only when governance changes: across sessions with no
  governance change, the surface imposes no fresh cost.
- A maintainer can add project-specific orientation to the pointer layer and see it
  carried into the projected surface, with that authored content surviving every
  subsequent regeneration.
- When the projected surface has fallen behind current governance, an operator is
  told — distinctly from being told the surface is current — and pointed at how to
  refresh it.
- A new category of governance can be added to the surface without reshaping the
  existing surface or disturbing the categories already present.

## 6. Behaviour

Primary flow — regenerate the surface: the tool gathers the project's current stable
governance state and assembles it into the orientation surface, recording it only
when its content differs from what is already recorded; an unchanged surface is left
untouched so its already-paid cost is not re-incurred.

Primary flow — establish the surface in a harness: the tool wires the projected
surface into a given agent harness's session-start context and arranges for it to be
regenerated at session start, so future sessions arrive oriented without manual
action. The wiring is idempotent and preserves any maintainer-owned configuration it
finds.

Pointer-layer flow: on first establishment the tool seeds a maintainer-owned pointer
file for the project; thereafter it only reads that file, folding its content into
the surface. Maintainer edits persist across every regeneration.

Honesty flow: on request the tool reports whether the recorded surface has drifted
from current governance and whether any category is unpopulated, scoping its claims
to the recorded surface on disk and never asserting that the orientation already
resident in the running session is fresh.

Guards and edge cases: a governance category with nothing to contribute renders a
benign placeholder rather than failing; a missing or unreadable recorded surface is
treated as drifted, since it differs from what current governance would produce; and
the surface always carries a usable handle for invoking the tool even when no other
governance is present. The orientation resident in a running session can lag the
on-disk surface until the session refreshes, which the honesty flow is careful not to
mask.

## 7. Verification

Verification confirms that the surface faithfully projects current governance, that
it imposes cost only on change, that the maintainer-owned layer is preserved, and that
the surface is honest about its own freshness — without binding the spec to a
particular projection mechanism.

The projection's faithfulness is proven by assembling a surface from known governance
state and confirming it carries the expected routing/process orientation, accepted
decisions, memory pointers, and the resolved invocation handle, with empty categories
rendering as benign placeholders rather than failures. The pay-on-change property is
proven by regenerating from unchanged governance and confirming the recorded surface
is left untouched, and by regenerating after a change and confirming the surface
updates. The pointer layer's preservation is proven by authoring maintainer content,
regenerating, and confirming the content survives unclobbered. Honesty is proven by
driving the report against a fresh surface, a drifted surface, an absent surface, and
an unpopulated category, and confirming each is reported distinctly and that the
report never claims the running session's resident orientation is fresh. Harness
establishment is proven by confirming the wiring is idempotent and preserves
maintainer-owned configuration.

Where a check must reference a specific obligation, cite the durable requirement
entity (REQ-NNN), never a mobile membership label. Coverage of the functional and
quality requirements is tracked against those entities, not duplicated here.

## 8. Open Questions

- Harnesses differ in whether they can regenerate the surface at session start or
  only carry a statically imported copy; for the latter, the surface can lag until the
  tool is next run. What is the acceptable freshness posture for a harness that cannot
  self-refresh, and should the honesty report account for it differently?
- The pointer layer is preserved once authored, so the tool cannot later evolve its
  seeded scaffolding in place. How should an improved scaffold reach projects whose
  maintainers have already authored over the original, without clobbering their work?
