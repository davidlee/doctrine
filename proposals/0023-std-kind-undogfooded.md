---
seq: 0023
scope: codebase
target: STD (standard) kind vs project conventions in CLAUDE.md/AGENTS.md
confidence: med
reversible: yes (proposal only; read-only analysis — nothing authored)
---
## What
Doctrine ships a fully-built **`standard` (STD) governance kind** — "standing
conventions of practice" (`doctrine standard new|list|show`, `STANDARD_KIND` in
`src/standard.rs`, specced as SPEC-016, with IMP-063 proposing to grow its
supersession vocab) — yet it has **zero instances** (`.doctrine/standard/` holds no
numbered entity; POL has 1, STD has 0). Boot.md's "Active Standards" section is
empty.

Meanwhile, **the project has standing conventions of practice — just not as STD
entities.** They live as prose in `CLAUDE.md` / `AGENTS.md`: 2-space indentation,
"no parallel implementation," "lint as you go / `just gate` before commit,"
conventional commits scoped by slice id, the pure/imperative split, "ask, don't
infer." These are textbook "standing conventions of practice" — the exact thing the
STD kind exists to capture as first-class, citable, supersedable governance
entities.

So doctrine **does not dogfood its own STD kind.** That is a notable signal for a
tool whose pitch is "capture governance as graph entities, not scattered prose":
the project's own standards are scattered prose. Two readings, and which one is true
matters:
- **Friction/gap reading** — the authors author everything else (15 PRDs, 21 SPECs,
  13 ADRs, 600+ requirements) but not a single STD; maybe the STD authoring/citation
  experience has friction that even its builders route around (worth finding out
  *before* investing further via IMP-063).
- **Out-of-scope reading** — conventions are deliberately kept in CLAUDE.md/AGENTS.md
  (the agent-instruction surface) because that's where agents read them, and STD is
  intended for *downstream projects*, not doctrine itself. Legitimate — but then it
  should be stated, and continued STD investment (IMP-063) judged on downstream
  value, not self-use.

The graph-topology angle (the standing focus): standards in prose can't be
`governed_by`-linked, can't be cited by slices, can't be superseded, don't appear
in `inspect`/the graph. Authoring even a few as STD entities would both exercise the
kind and connect the project's conventions to the entities they govern — a concrete
dogfooding + topology win.

## Options
1. **Dogfood: author the project's core conventions as STD entities.** Convert the
   highest-value CLAUDE.md/AGENTS.md conventions (no-parallel-implementation,
   pure/imperative split, lint-gate, 2-space) into STD-NNN, then `governed_by`-link
   the slices/specs they constrain. Tradeoff: exercises the kind, connects
   conventions to the graph, validates (or exposes) the STD authoring experience;
   cost is authoring + deciding which prose conventions rise to STD.
2. **Author one STD as a smoke test**, keep the rest in prose for now. Tradeoff:
   cheapest way to learn whether STD authoring/citation has friction (the
   friction-reading test) before IMP-063 invests further; minimal commitment.
3. **Declare STD downstream-only; document the decision.** Record that doctrine
   keeps its own conventions in the agent-instruction surface by design and the STD
   kind serves consuming projects. Tradeoff: zero authoring; removes the "why is it
   unused?" ambiguity; reframes IMP-063's ROI as downstream-only.

## Recommendation
Option 2 first (author one real STD — e.g. "no parallel implementation" — and
`governed_by`-link a slice to it), as a deliberate dogfooding probe. Rationale: a
zero-instance kind that its own authors don't use is either friction worth finding
or a scope decision worth recording — and one real authoring pass answers which,
cheaply, *before* IMP-063 grows the STD supersession vocab. If authoring is clean
and the links add value, expand (Option 1); if STD is genuinely downstream-only,
record that (Option 3) and judge IMP-063 on downstream merit. Don't grow an unused
kind's vocabulary without first confirming the kind earns its keep.

Decisions deferred to YOU:
- (a) **is STD meant for doctrine-self or downstream-only?** (the load-bearing call).
- (b) if self: **which prose conventions rise to STD** vs stay agent-instruction
  prose (not everything in CLAUDE.md is a "standard")?
- (c) sequencing vs **IMP-063** — confirm STD is used/useful before growing its
  supersession vocab.

## Next doctrine move
```
# confirm zero instances + the prose conventions (read-only):
doctrine standard list                          # empty
sed -n '1,40p' CLAUDE.md AGENTS.md              # the un-captured conventions

# probe (NOT executed — authoring an STD is an authored-tier write; route it):
/route        # → governance authoring: doctrine standard new "No parallel \
              #   implementation — ride existing seams" ... then link a slice
              #   governed_by STD-001. (one smoke-test entity)
```
(Verbs described, NOT executed — fence forbids authoring governance entities.)

## Illustration (optional)
None — the move is an authoring probe + a scope decision, not a code diff.
