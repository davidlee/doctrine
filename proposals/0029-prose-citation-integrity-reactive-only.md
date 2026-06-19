---
seq: 0029
scope: codebase
target: src/integrity.rs prose-citation scan vs corpus validation
confidence: med
reversible: yes (read-only analysis; nothing authored)
---
## What
Doctrine validates **structured** relation edges thoroughly (`spec validate`:
"corpus clean"; id-integrity via `validate`), but **prose citations of entity ids**
— the `SL-046`, `ADR-004` references that pepper `.doctrine/**/*.md` bodies — are
only checked **reactively, never corpus-wide.**

The machinery exists: `integrity.rs:518` scans authored `.doctrine/**/*.md` prose
for inbound citations of a given `needle`, and `:500` reports them — but this is
invoked **only by `reseat`**, for the *single* id being renumbered ("reports inbound
prose citations as danglers, never rewrites", `main.rs:641`). There is no proactive
verb that asks the inverse: *do all prose citations across the corpus resolve to a
real entity?* `validate` is id-integrity (dir==id, dup, alias); `spec validate` is
member/interaction FK + orphans; neither scans prose. So a prose ref to a
never-minted or deleted id (e.g. a stale "see SL-072" after SL-072 was reseated)
dangles **silently** — invisible to every integrity surface.

This matters because prose is where a lot of the graph's *human* meaning lives
(design rationale, ADR Context/Decision, spec overviews all cite ids in prose), and
those citations are exactly what rots when ids move (reseat) or entities are removed.
The reactive check proves the project cares about prose-citation integrity at *write*
time (reseat); the gap is that nothing audits it at *rest*. The scan primitive
(`:518`) is already the hard part — a corpus check is "run it for every id, or invert
it: extract every prose `KIND-NNN` and confirm it resolves."

## Options
1. **Add a corpus-wide prose-citation check** (`validate --prose` or fold into
   `doctrine doctor`, proposal 0011): extract every `KIND-NNN` citation from authored
   `.md`, report any that don't resolve to a minted entity. Reuses the `:518` scan /
   the `require_minted` oracle. Tradeoff: closes the silent-dangler class; advisory
   or gating per your call; cost is the inversion + a precision pass (avoid flagging
   code spans, sentinel ids like the SL-999 in RSK-007's title, doc-local refs).
2. **Leave reactive-only.** Tradeoff: zero work; relies on reseat to surface
   citations and on authors to fix them; stale prose refs accrete invisibly between
   reseats.
3. **Lint at authoring time** (a write-path hook that warns on a new prose citation
   of a nonexistent id). Tradeoff: catches new danglers early; doesn't find existing
   ones; more intrusive than a read-only audit.

## Recommendation
Option 1, as an **advisory** check folded into `doctrine doctor` (0011): reuse the
existing prose scan to report unresolved prose citations corpus-wide, never auto-
rewrite (consistent with reseat's "report, never rewrite" stance). Rationale: the
project already validates structured edges to "corpus clean" and already scans prose
reactively — the missing piece is the proactive inversion, and it's pure reuse of
shipped primitives. Keep it advisory and precision-tuned (exclude code fences,
doc-local enumerations, and known sentinels) so it doesn't cry wolf. This is the
prose half of the same "make integrity legible in one gate" story as 0011.

Decisions deferred to YOU:
- (a) **build the corpus prose check, or stay reactive** (is silent prose-dangler
  drift acceptable between reseats)?
- (b) **precision policy** — how to exclude false positives (code spans, sentinel
  ids like RSK-007's `SL-999`, doc-local `OQ-1`/`D1` forms which aren't entity ids)?
- (c) **advisory vs gating**, and where it lives (`validate --prose`, `doctor`,
  `spec validate`).

## Next doctrine move
```
# confirm reactive-only + the reusable scan (read-only):
sed -n '515,525p' src/integrity.rs      # the prose scan primitive (reseat-only caller)
doctrine validate ; doctrine spec validate   # neither scans prose

# capture (NOT executed — fence forbids backlog transition):
doctrine backlog new improvement "Corpus-wide prose-citation integrity: extract \
  every KIND-NNN cited in authored .md and report unresolved ones — reuse the \
  reseat prose scan (integrity.rs:518); advisory, fold into doctrine doctor (0011); \
  precision-exclude code spans / sentinels / doc-local refs" --tag area:governance \
  --tag area:cli
```
(Verbs described, NOT executed.)

## Illustration (optional)
None — reuse of an existing scan + an exclusion-precision policy (decision b), which
a speculative diff would prejudge.
