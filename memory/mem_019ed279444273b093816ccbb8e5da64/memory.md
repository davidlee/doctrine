# Doctrine review ledger (RV kind)

The review ledger is a **first-class entity** for structured adversarial
review. It is turn-based: two parties alternate between raising and
resolving findings, with an explicit baton that tracks whose turn it is.

The RV kind is the substrate for audit reconciliation, design review,
code review, and any structured finding-tracked dialogue.

## CLI

The CLI is the source of truth: `doctrine review --help`. Key verbs: new,
raise, dispose, amend, verify, contest, reopen, withdraw, conclude, status,
prime, unlock, show, list, paths. Each has an MCP tool (`review_<verb>`)
except unlock and paths.

## Lifecycle

Roles belong to acts, not agents: `--as` names the role (`raiser` /
`responder`, or the labels declared at `review new`), and one agent may drive
both.

1. **new** — open a ledger targeting an entity (slice, spec, ADR, etc.).
   The target is validated before any id is allocated.
2. **raise** (raiser) — raise a finding (severity + title + detail). The
   finding is `open`; the baton flips to the responder.
3. **dispose** (responder) — answer an `open`/`contested` finding with a
   disposition, an optional `--route`, and a response. The finding is
   `answered`; the baton returns to the raiser.
4. **amend** (responder) — revise an `answered` finding's response (and
   optionally its disposition/route); `--note` is required.
5. **verify** (raiser) — accept the disposition (`verified`).
6. **contest** (raiser) — reject the disposition and hand it back
   (`answered` → `contested`); `--note` is required.
7. **reopen** (raiser) — hand a `verified` finding back (`verified` →
   `contested`); `--note` is required.
8. **withdraw** (raiser) — retract an `open`/`answered` finding
   (`withdrawn`, no exit).
9. **conclude** (raiser) — declare the pass finished; `--basis` (what the pass
   examined) is required. Open findings are fine. A later raise or reopen
   clears the conclusion, so conclude again after it. A design run's
   `conducted` disposition is admissible only over a concluded review.

`verified` and `withdrawn` are terminal for done; only `withdrawn` has no
exit. A review is `done` when every finding is terminal **and** the pass is
concluded. Notes, responses and the conclude basis are recorded as durable
turns.

Every finding carries a severity (`blocker | major | minor | nit`) and
an owner-owned status.

## Coordination

- **status** — report the derived state and rebuild the baton (cache recompute).
- **prime** — populate the reviewer context warm-cache from the target slice's
  selectors. When the target is not a slice or declares no selectors, it
  degrades: prints `primed nothing: <reason>` and writes no cache.
- **unlock** — remove a stale per-review lock left by a hard kill (escape
  hatch).

## Viewing

- **show** — derived status, the `reviews` edge, and the brief. `--json`
  carries each finding's route, which the table view omits.
- **list** — id, derived status (+ await), facet, target, title.
- **paths** — print a review's file paths.

## Where it fits

The RV kind is driven by the audit phase (`/audit`), the design review gate
(`/inquisition`), and the code review skill (`/code-review`). Closeout expects
that any RV targeting the slice has no unresolved blockers.

See [[mem.signpost.doctrine.audit]] for the audit phase that uses the RV ledger,
[[mem.signpost.doctrine.lifecycle-start]] for the full lifecycle,
[[mem.signpost.doctrine.file-map]] for the `.doctrine/review/nnn/` layout, and
[[mem.concept.doctrine.reading-entities]] for the read-via-show rule.
