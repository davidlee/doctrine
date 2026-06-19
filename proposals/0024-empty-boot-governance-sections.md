---
seq: 0024
scope: capture
target: boot snapshot §"Where things live" / §"Project-local rules of the road"
confidence: low
reversible: yes (read-only analysis; nothing authored)
---
## What
The boot governance snapshot — the orientation surface every agent loads at session
start (`.doctrine/state/boot.md`) — renders **two empty section headers**:
`## Where things live` and `## Project-local rules of the road` (lines 64–68:
header, blank, header, blank). The section *titles* are sourced from the governance
template (`install/governance.md`), but doctrine's own project governance supplies
**no content** for them, so they project as bare headers in every boot.

These two sections are precisely the **project-local orientation** a fresh agent
would want — "where things live" (the file/dir map) and "rules of the road"
(project-specific conventions). Their content does exist, but in `AGENTS.md` (the
storage-model + conventions sections) rather than in the governance snapshot. So the
boot surface points at empty headers while the real orientation sits in a different
file the routing snapshot doesn't inline.

Two readings (hence low confidence — this may be deliberate):
- **Deliberate de-duplication** — orientation lives in `AGENTS.md` (which the boot
  contract `@`-includes), so the governance-snapshot sections are intentionally
  empty to avoid two copies. If so, the empty *headers* are the wart: they read as
  "unfilled," not "see AGENTS.md."
- **Unfilled gap** — doctrine never populated its own governance sections (a
  dogfooding miss, same family as proposal 0023's unused STD kind): the project
  ships the template sections but doesn't use them on itself.

Either way the symptom is the same: an agent reading the snapshot top-to-bottom hits
two empty governance headers, which is mildly disorienting and indistinguishable
from incomplete setup.

## Options
1. **Fill the sections** with a terse pointer (e.g. "Where things live → see
   AGENTS.md storage model; key dirs: `.doctrine/{slice,spec,adr,...}`, `src/`,
   `crates/cordage`") — even one line each. Tradeoff: the snapshot becomes
   self-orienting; tiny authored edit; risks light duplication with AGENTS.md.
2. **Suppress empty governance sections in the snapshot generator** — don't render a
   header with no content. Tradeoff: removes the "unfilled?" ambiguity with zero
   authoring; a code change to the boot assembler; loses the prompt-to-fill the empty
   header provides.
3. **Leave as-is.** Tradeoff: zero effort; every boot keeps showing two empty
   governance headers.

## Recommendation
Option 2 (suppress empty sections in the generator) as the general fix —
header-with-no-body is noise on the most-read surface, and the generator should
elide it — *plus* a one-line fill (Option 1) for "Where things live" since a key-dir
map is genuinely useful at boot and barely duplicates AGENTS.md's prose. Rationale:
the snapshot is the highest-traffic orientation artifact; it should either say
something or say nothing, never show an empty promise. Confidence is low because
this may be a conscious de-dup — confirm intent first.

Decisions deferred to YOU:
- (a) **deliberate (content lives in AGENTS.md) or unfilled?** — sets fill vs suppress.
- (b) if suppress: should the generator elide *any* empty governance section
  generally (a small assembler rule), benefiting downstream projects too?
- (c) is this the same dogfooding pattern as 0023 (STD unused) — worth one decision
  about "does doctrine populate its own governance surfaces, or rely on AGENTS.md"?

## Next doctrine move
```
# confirm empty + where the content actually lives (read-only):
sed -n '62,69p' /workspace/doctrine/.doctrine/state/boot.md   # the empty headers
sed -n '1,60p' AGENTS.md                                      # storage model + conventions

# fill (authored governance) or fix the generator — route it (NOT executed; fence):
/route   # → governance content edit, or a chore for the boot-snapshot assembler to
         #   elide empty sections.
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — a one-line content fill or a small generator elision, gated on the
deliberate-vs-unfilled question (a).
