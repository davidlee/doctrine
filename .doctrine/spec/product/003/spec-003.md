# PRD-003: Skills

## 1. Intent

Doctrine's working conventions only take hold when the agents in front of a
codebase actually carry them. An agent without doctrine's skills improvises:
it routes by instinct, skips the gates, and rediscovers the process every
session. The need is to put a curated, governed set of agent skills *into the
agent* — reliably, repeatably, and across the many agent tools a team might use
— so the doctrine way of working is present at the point of work rather than
left to memory or to a copy-paste ritual.

This is hard because the agent landscape is plural and fragmented: each tool
keeps skills in its own layout, and a team rarely standardises on one. The value
of treating skills as a first-class shipped capability is that the same curated
source reaches every agent through more than one channel — a consumer with
nothing but a plugin marketplace, and a consumer who already holds the doctrine
binary — without the skill set forking, drifting, or being maintained twice. The
desired end state: an operator can see what skills exist, install the ones they
want into whichever agent they use, and trust that re-running changes nothing
already in place.

## 2. Scope

In scope:

- Curating a set of agent skills as a single canonical source, grouped into
  coarse capability domains, with no skill duplicated across channels.
- Distributing that source through two independent channels: an open marketplace
  consumable with no doctrine binary, and the doctrine binary itself.
- Surveying skills — what exists, grouped by domain, and whether each is already
  installed for a named or detected agent.
- Installing a chosen subset of skills into an agent, selectable by skill or by
  domain, into either the project or the user scope.
- Distinguishing the agent doctrine installs directly from every other agent,
  for which it defers to a universal external installer, and making that routing
  visible before anything is written.

Out of scope:

- Removing or updating already-installed skills — installation is additive in
  this capability.
- Authoring or scaffolding new skills.
- Publishing or maintaining the marketplace manifests — they are consumed here,
  not produced.
- Reproducing per-agent install layouts that an external universal installer
  already understands.

Boundary: this capability owns *which skills ship and getting them in place*; it
does not own *what a skill says* (the skill bodies themselves) nor the agents'
own internal layout rules. Where doctrine does not own an agent's layout, it
delegates rather than reimplements.

## 3. Principles

- **One source, many channels.** Every channel reads the same canonical skill
  tree; a skill is never duplicated to serve a second consumer. Duplication is
  drift waiting to happen.
- **Install directly only where doctrine owns the layout; delegate everywhere
  else.** Reimplementing dozens of foreign agent layouts is a maintenance trap;
  defer to the universal installer that already knows them.
- **Additive and idempotent.** Installation never overwrites what is already
  present; re-running is always safe and changes nothing in place.
- **Show the plan before touching disk.** Routing and actions are inspectable up
  front, so an operator can see what will happen — and reproduce a delegated
  action by hand — before consenting.
- **Honest about reach.** Install status and routing are stated plainly:
  authoritative where doctrine controls the layout, best-effort and labelled as
  such where it does not.

## 4. Requirements

The functional and quality requirements this capability must satisfy are recorded
as requirement entities and appear under the synthesized Requirements section
below. This section carries only the constraints and invariants that bound every
valid implementation.

Constraints:

- The marketplace channel must remain consumable with no doctrine binary present.
- For agents whose layout doctrine does not own, the capability must defer to the
  universal external installer rather than encode that layout itself.
- Installation must offer both a project scope and a user scope, and must let the
  operator narrow the set by skill or by domain.
- Direct installation must require no external runtime; delegated installation may
  require one, and its absence must be reported, never silently worked around.

Invariants:

- The canonical skill source is single; no channel ever serves a duplicated or
  forked copy.
- An existing installed skill is never overwritten by an install run.
- No skill is written to disk before the planned actions have been surfaced for
  consent (unless consent was given up front).
- Install status reported as authoritative reflects only the agent whose layout
  doctrine owns; status for other agents is labelled best-effort.

## 5. Success Measures

- A consumer with only a plugin marketplace, and a consumer holding the doctrine
  binary, both obtain the same curated skill set from the one source.
- An operator can list every shipped skill, grouped by domain, and read at a
  glance whether each is installed for their agent.
- An operator can install a chosen subset into their agent in one step, and a
  second run of the same command reports nothing changed.
- For an unsupported or unspecified agent, the operator receives a clear request
  for an explicit target rather than a wrong-guess install.
- A delegated install is reproducible by hand: the exact external command appears
  in the surfaced plan.

## 6. Behaviour

Primary flow — survey skills: an operator asks what skills exist; the system
enumerates them grouped by domain, each with its description and its install
status for the detected or named agent, and can restrict the view to skills
already installed.

Primary flow — install skills: an operator asks to install; the system resolves
the target agent, builds a plan, surfaces it, and on consent executes it. The
operator may narrow the selection by skill or by domain and choose project or
user scope.

Routing flow: each target agent is classified — the agent whose layout doctrine
owns is installed directly by placing the skill trees into that agent's layout;
every other agent is delegated to the universal external installer with an
equivalent selection.

Guard — agent detection: when no agent is named, the system targets the owned
agent if its presence is detected; otherwise it stops and asks for an explicit
target rather than guessing a foreign agent.

Idempotency guard: a direct install skips any skill already present and leaves it
untouched; delegated idempotency follows the external installer's own behaviour.

Edge cases and failure modes: a missing external runtime on the delegated path is
reported with guidance and aborts that path without a silent fallback; a delegated
install that fails for one agent is reported and does not prevent other agents
from proceeding; install status for delegated agents may be unknowable and is
reported as such rather than asserted.

## 7. Verification

Verification confirms that the one canonical source reaches every channel
unduplicated, that routing chooses the right path per agent, that planning is
honest and consent-gated, and that installation is additive and idempotent —
without binding the spec to a particular implementation.

The single-source guarantee is proven by confirming both channels resolve to the
same skill tree with no duplicated copy. Routing is proven by confirming the owned
agent yields direct placement steps while any other agent yields a single
delegated step carrying the expected external command. Selection is proven by
confirming a narrowing by skill or by domain produces only the chosen skills, in
either scope. Idempotency is proven by confirming a direct install skips a skill
that already exists and leaves it untouched, so a re-run changes nothing.
Detection is proven by confirming the owned agent is targeted when present and
that an absent or unspecified agent yields a stop-and-ask rather than a guess.
Failure handling is proven by confirming a missing delegated runtime aborts that
path with guidance and no silent fallback. Planning honesty is proven by
confirming the surfaced plan names each agent, its path, and — for delegation —
the exact reproducible command before any disk is touched.

Where a check must reference a specific obligation, cite the durable requirement
entity (REQ-NNN), never a mobile membership label. Coverage of the functional and
quality requirements is tracked against those entities, not duplicated here.

## 8. Open Questions

- How much effort should the survey spend probing the directories of delegated
  agents to report install status, versus reporting it as untracked? This blocks
  the precision of the status column for every non-owned agent.
- The delegated channel tracks the source repository's head rather than the
  snapshot the running binary carries, so a delegated agent can receive a skill
  set that is newer or older than the binary's. What is the acceptable interim
  posture before the delegated source can be pinned to the binary's build, and
  what would close it?
