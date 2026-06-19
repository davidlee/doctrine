---
seq: 0020
scope: codebase
target: boot routing snapshot vs plugins/doctrine/skills/ (IMP-042 generalised)
confidence: med
reversible: yes (proposal only; read-only analysis — nothing authored)
---
## What
Of the 30 shipped skills (`plugins/doctrine/skills/`), **four are referenced zero
times in the boot routing snapshot** (`.doctrine/state/boot.md`, the discovery +
routing surface an agent loads at session start): **`code-review`, `handover`,
`dreaming`, `reviewing-memory`**. Controls confirm the check is sound (boot.md
mentions `design`=6, `canon`=4, `walkthrough`=3, `backlog`=4, `pair`=1).

IMP-042 already names **one** of these — "code-review skill is structurally
orphaned — integrate it into the corpus." But the orphaning is a **class of four**,
not one: an agent that boots from the snapshot (the AGENTS.md/CLAUDE.md contract:
"the routing table … ride the boot snapshot") is never told these four skills exist
or when to reach for them. IMP-042 captures 25% of the pattern.

The four split into two plausibly-different cases:
- **`code-review`** (quality gate) and **`handover`** (context packet; the heavier
  sibling of `/next`, which *is* routed) read like skills that *should* be
  discoverable from the routing surface but aren't — `handover` especially, since
  the snapshot routes its sibling `/next` for "handing off to fresh context" yet
  never mentions `handover` as the richer option.
- **`dreaming`** and **`reviewing-memory`** (memory-corpus maintenance, trigger/
  periodic) are plausibly **deliberately out-of-band** — not per-task-stage skills,
  so absent from the *stage* routing table by design. But "by design" is not
  recorded anywhere; their absence is indistinguishable from an oversight.

(Caveat: these may be reachable via the `mem.signpost.doctrine.skill-map` memory
signpost rather than inline boot prose — so "undiscoverable" is too strong. The
precise gap is *inline routing-surface* presence, which is what an agent reads first
and what IMP-042 treats as the bar for code-review.)

## Options
1. **Generalise IMP-042 to the four-skill set; decide each: route or
   declare-out-of-band.** For `code-review`/`handover`, add a routing-table or
   core-process line (when to reach for them). For `dreaming`/`reviewing-memory`,
   add a one-line "out-of-band, trigger-invoked — not stage-routed" note so the
   omission is intentional-of-record. Tradeoff: closes the discovery gap and removes
   the oversight-vs-intent ambiguity; small authored edit to the snapshot generator.
2. **Fix only `handover` (the clearest case); leave IMP-042 to handle code-review;
   accept dreaming/reviewing-memory as out-of-band.** Tradeoff: smallest; `handover`
   is the sharpest gap (sibling routed, it isn't). Leaves the "is this intentional?"
   ambiguity on the two memory skills.
3. **Leave as-is (let IMP-042 cover code-review only).** Tradeoff: zero effort; three
   more skills stay invisible to the routing surface, and IMP-042 looks like a
   one-off when it's an instance of a pattern.

## Recommendation
Option 1, but as a *cheap classification pass*, not a build: for each of the four,
the human decides "route it" or "out-of-band, recorded as such," and the snapshot
generator gains one line per skill accordingly. Rationale: the value is removing the
ambiguity — right now an agent (and a maintainer) can't tell a deliberately-unrouted
skill from a forgotten one. Folding this into IMP-042 (widen its scope from
code-review to "skill routing-surface presence audit") is cleaner than four separate
items. `handover` is the strongest single case — its routed sibling `/next` makes
its omission look accidental.

Decisions deferred to YOU:
- (a) **per skill: route vs deliberately-out-of-band** — especially is `handover`
  meant to be discoverable alongside `/next`, and are `dreaming`/`reviewing-memory`
  intentionally stage-unrouted?
- (b) **widen IMP-042** to the four-skill set, or keep it code-review-only and file
  the rest separately?
- (c) where "routed" lands — the routing table, core-process prose, or the
  skill-map signpost (and is the signpost itself the intended discovery path,
  making inline absence fine)?

## Next doctrine move
```
# confirm the set against the real (gitignored) boot snapshot (read-only):
for s in code-review handover dreaming reviewing-memory; do \
  echo "$s: $(grep -ic "$s" /workspace/doctrine/.doctrine/state/boot.md)"; done
doctrine backlog show IMP-042        # the existing single-skill instance

# widen the existing item rather than file anew (NOT executed — fence):
doctrine backlog edit IMP-042 ...    # broaden to "skill routing-surface audit:
                                     # code-review, handover, dreaming, reviewing-memory"
# (verb shape per `doctrine backlog --help`)
```
(Verbs described, NOT executed — fence forbids backlog transition / authored edits.)

## Illustration (optional)
None — the move is a per-skill route-or-declare classification + a snapshot-generator
line each, gated on decision (a); a diff would prejudge the classification.
