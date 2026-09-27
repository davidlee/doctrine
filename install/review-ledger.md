<!-- Shipped reference. Published, not projected: there is no copy on disk in an
     installed project — read it with `doctrine library show
     reference/review-ledger.md`. Names verbs and states the invariant protocol — it never reproduces
     `doctrine review --help`; ask the CLI for exact flags. -->

# The review ledger

How to drive a review on the **RV kind** (`RV-NNN`) — the structured,
append-only audit substrate the hand-made `audit.md` lacked. This doc owns the
*invariant* protocol shared by every review skill (`/audit`, `/code-review`,
`/inquisition`): pick the subject, open + prime, raise, dispose + resolve,
conclude, synthesize + harvest, close. Each skill restates the trigger in its own
voice and keeps its own lens and harvest tail — but the mechanics live here, once.

For **exact command shapes and flags**, ask `doctrine review <command> --help` —
this doc names verbs, never their flag tables. For the work/knowledge/decision
boundary, see `using-doctrine.md`; for ids and the verification taxonomy, see
`glossary.md`.

## Acts and roles

A finding moves only through the ledger's **acts**. Each act belongs to one
role, and the ledger refuses an act from the wrong role or the wrong state:

| act | role | finding moves | required prose |
|---|---|---|---|
| `raise` | raiser | (new) → `open` | `--title`, `--detail` |
| `dispose` | responder | `open` / `contested` → `answered` | `--response` |
| `amend` | responder | `answered` → `answered` | `--response`, `--note` |
| `verify` | raiser | `answered` → `verified` | — (`--note` optional) |
| `contest` | raiser | `answered` → `contested` | `--note` |
| `reopen` | raiser | `verified` → `contested` | `--note` |
| `withdraw` | raiser | `open` / `answered` → `withdrawn` | — (`--note` optional) |
| `conclude` | raiser | the pass, not a finding | `--basis` |

`verified` and `withdrawn` are the **terminal** statuses — they count toward
done (§6). `withdrawn` has no exit; `verified` does: `review reopen` hands a
verified finding back to the responder.

Every `--note`, `--response` and `--basis` is **durable** — the ledger records
it as a turn on the finding (or the pass), and a later act never erases it.

**One rule: roles belong to acts, not agents.** `--as` names the role an act is
performed in: `raiser` or `responder`, or the labels the ledger declared at
`review new --raiser <L> --responder <L>` (accepted as aliases). A responder that
spots a new defect raises it `--as raiser`; one agent may drive both roles.
`--as` is **cooperative role assertion, not a security boundary**. `review new`
refuses two identical labels, and a label that names the other role.

The remaining verbs read or maintain the ledger: `new`, `list`, `show`,
`status`, `prime`, `unlock` (removes a stale per-review lock left by a hard
kill), and `paths` (prints a review's file paths).

### Passing prose

Prefer the **MCP tools** (`review_raise`, `review_dispose`, …) when your harness
has them: the prose travels as structured fields and never touches a shell. On
the CLI, `--title`, `--detail`, `--response`, `--note` and `--basis` each accept
`-` (read stdin) or `@path` (read a file). Pass prose through a quoted heredoc,
so `$`, backticks and quotes reach the ledger unexpanded:

```
doctrine review raise RV-NNN --severity major --title @/tmp/title.txt \
  --detail - <<'EOF'
Expected: …
Observed: …
EOF
```

At most one flag per invocation may read `-`. A literal value that starts with
`@`, or is exactly `-`, must itself go through stdin or a file. A `--title` read
through `-` or `@path` drops its trailing newline, since a title is one line;
every other prose flag is stored exactly as read. The `@` in a
`--target SL-NNN@PHASE-NN` is a phase scope, not a file read.

## §1 — Pick the subject

A review needs a **subject**: the thing findings attach to and outlive the
conversation against. Steer toward a *proximate, typed* subject — the closer the
RV's `--target` sits to a real doctrine entity, the more the findings can be
queried, gated, and handed off later.

**Target ladder** (descend only when the rung above genuinely does not fit):

1. **A slice or phase** — the implementation, design, or plan artifact under
   review. The strongest subject; an audit lives here always.
2. **A backlog item** — `issue` / `improvement` / `chore` / `risk` / `idea`
   (`ISS-` / `IMP-` / `CHR-` / `RSK-` / `IDE-`). A durable diff or
   investigation with no slice yet still has a typed home here.
3. **Create one** — if no proximate subject exists but the work is durable,
   `doctrine backlog new <kind>` mints one, *then* target it. A review worth the
   ledger is worth a subject; do not skip to prose to dodge the mint.
4. **Prose, last resort** — only an explicitly throwaway one-shot with no durable
   subject, no lifecycle gate, no handoff, and no finding that should survive.

`--target` is a **validated canonical ref**: `doctrine review new` refuses a
target that does not resolve, so a backlog kind is only a legal target once it
exists (rung 3 before rung 2's degenerate case). The full ladder is presented
here, but **each consuming skill pins which rungs apply** — an audit always
targets its slice (rung 1) and never degrades to prose.

### The ledger-vs-prose trigger

Drive the ledger when the review is **closure-grade** — it:

- gates a lifecycle move (a slice's `audit→reconcile→done`), **or**
- runs adversarially across more than one round, **or**
- hands off between agents, **or**
- raises findings that must outlive the conversation.

An existing doctrine subject — a slice, a phase, a backlog item, a slice-tied
implementation diff, a design or plan artifact — makes that presumption **strong**:
open an RV. Durable diff-only work with no subject yet → create/use a backlog
target (rung 3). Reserve **prose** for the genuinely throwaway one-shot above.

The **cost asymmetry is the test**: an RV that turns out trivial cost a few verbs;
a prose review whose findings mattered is lost the moment the context clears. When
the two are close, open the ledger.

## §2 — Open + prime

### Facet

Pick the **facet** by *what aspect you interrogate* — the subject's lifecycle
aspect (e.g. `reconciliation` for a post-implementation audit). The facet always
names a **lifecycle aspect, never a posture**. An adversarial *posture*
(inquisitor, devil's advocate, …) rides `--raiser <label>` — **never** a bespoke
facet. Same subject, same facet, different raiser label: that is how a posture is
expressed.

### Open

A slice design under a managed design run already has its ledger: the run
opened it on entering `reviewing`, and `doctrine design show <slice> --format
prompt` names it as `review_pass RV-NNN`. Prime and raise on that one; a second
RV cannot be disposed as the run's pass.

Otherwise:

```
doctrine review new --facet <F> --target <REF> [--phase <P>] [--raiser <L>]
```

`<REF>` is the canonical ref from §1's ladder; `--phase` narrows to one phase;
`--raiser` stamps the posture label.

### Prime

Warm the reviewer context so the staleness signal has a path-set to hash:

1. `doctrine review prime RV-NNN` — populates the warm-cache from the **target
   slice's selectors** (`scope-relevant` + `design-target`; the path-set the
   staleness signal hashes). One call, no curation step; selectors, seeded at
   `/slice` and `/design`, are the path-set. When the target is not a slice, or
   the slice declares no selectors, prime **degrades** instead of failing: it
   prints `primed nothing: <reason>` and writes no cache (removing any earlier
   one).
2. Seed the ledger's `## Brief` (in `review-NNN.md`) with the **lines of attack**:
   what this review is probing and the invariants it pins the subject to — this is
   where the reviewer's intent lives, not in a persisted map.

`doctrine review status RV-NNN` reports `cache: current` / `stale` as an
**optimization signal, never a gate** — a stale cache costs a re-prime, not a
refusal.

## §3 — Raise findings

```
doctrine review raise RV-NNN --severity <S> --title <expected vs observed> \
  --detail - <<'EOF'
<evidence>
EOF
```

The **raiser owns `severity` / `title` / `detail`**, fixed at raise — the ledger is
append-only, so frame each finding as *expected vs observed* with its evidence the
first time. A raise clears the pass's `concluded` marker: conclude again after it
(§4, "Conclude the pass").

**Severity vocab** — `blocker | major | minor | nit`:

- **`blocker`** is the only severity that gates the *target's* close. An unresolved
  blocker on an active RV refuses the `audit→reconcile` and `reconcile→done`
  transitions (the close-gate teeth, enforced in the binary). Reserve it for
  findings that must not ship unreconciled.
- **`major` / `minor` / `nit`** record the finding but never block close.

## §4 — Dispose + resolve

Every finding gets an explicit disposition, then a terminal close:

```
doctrine review dispose RV-NNN --finding F-n --disposition <vocab> \
  [--route <route>] --as responder --response - <<'EOF'
<rationale>
EOF
```

**Disposition vocab** (use consistently):

- **aligned** — observed behaviour is already correct; no follow-up.
- **fix-now** — reconcile inside the current unit of work before closing.
- **design-wrong** — the design, not the code, is the defect; reconcile the design
  artifact (and scope) so canon tells the truth.
- **follow-up** — owned future work is the right route; capture it (`backlog new`).
- **tolerated** — explicit unresolved drift, with rationale, only when the tradeoff
  is consciously accepted.

To change an answer the raiser has not yet acted on, the responder uses
`doctrine review amend RV-NNN --finding F-n --response … --note …` (answered →
answered; `--note` says why, and `--disposition` / `--route` may be replaced).

Then close each finding **terminal**:

- `doctrine review verify RV-NNN --finding F-n --as raiser` — accept (terminal).
- `doctrine review contest RV-NNN --finding F-n --as raiser --note …` — disagree;
  hand back (answered → contested) for re-disposition. The note is the argument.
- `doctrine review withdraw RV-NNN --finding F-n --as raiser` — a finding **raised
  in error** is retracted (terminal), *not* disposed.

A verified finding can still be reopened when later evidence undoes it:
`doctrine review reopen RV-NNN --finding F-n --as raiser --note …` (verified →
contested). The responder then re-disposes, and the raiser verifies again.
Reopen clears the pass's `concluded` marker, just as raise does.

**Caveats:**

- **Self-review** drives both roles via `--as` (raiser raises / verifies /
  contests / reopens / withdraws / concludes; responder disposes / amends). The
  per-review lock and the per-finding act table keep a one- or two-party review
  correct.
- Loose conversation notes are **insufficient** for closure-grade work — findings
  live in the ledger, not the conversation.

**Route axis** (provisional — applies to design-review ledgers, not to
`/audit` or `/code-review` passes):

Severe findings on a design-review ledger additionally carry a **route** —
`--route <route>` on `dispose` or `amend`, one of `review | demonstrate | probe |
control | dedupe | refresh`. The disposition vocab above records what the responder
did; the route records what instrument can settle the finding. The CLI refuses a
`route:` prefix inside `--disposition`, and omitting `--route` keeps the
finding's current route.

The table view of `review show` renders the disposition, not the route: read it
from `doctrine review show RV-NNN --json` (`.review.finding[].route`), or from
the MCP `review_show` output (`Showed.findings[].route`, absent while unset). An older
ledger carries the route as a `route:` prefix inside its disposition string,
which still reads verbatim.

An instrument-routed finding's terminal close is **deferred**: it is verified
after `slice phases`, against the criterion its obligation became — not in the
pass that raised it. The immediate terminal close above is the rule for every
other finding, routed or not.

The operative rule — what each route owes, and the form `--response` must take
— is delivered on every reviewing turn by `design-prompts/reviewing.md`, which
owns it. This entry exists so the axis is discoverable beside the vocab, not to
restate it. The route value is validated on write; whether the obligation it
names is met is not checked.

### Anti-escape guardrails

- Do **not** pick **follow-up** because the fix feels large.
- Do **not** normalise **tolerated** without a real rationale.
- Do **not** downgrade a true **blocker** to dodge the close-gate.
- Unresolved ambiguity after reading the design and governance → stop and
  `/consult`. Do not improvise a disposition.

### Conclude the pass

The raiser's **closing move of every pass** is:

```
doctrine review conclude RV-NNN --as raiser --basis - <<'EOF'
<what this pass examined, and against what>
EOF
```

`--basis` is required: it records what the pass covered. Conclude **after the
last raise or reopen**. Either act clears the `concluded` marker, so conclude
again after it. Open findings do not block conclude — disposing them is the
responder's work — but the ledger is not done until they are terminal (§6). A
design run's `conducted` disposition is refused over an unconcluded ledger.

## §5 — Synthesis + harvest

When the findings are resolved, append a `## Synthesis` section to
`review-NNN.md` — the narrative the old `audit.md` carried: the **closure story**,
the **standing risks**, the **tradeoffs consciously accepted**. The ledger holds
the structured findings; the synthesis holds the prose that ties them together.

Then **harvest** — the shared harvest procedure (the moment, the three legs
and their sinks, the canonical `## Harvest` output, and the consumer contract)
is owned once by `harvest.md`; drive it from there. A clean review harvests
nothing — a valid outcome, not a skipped step.

Generic review-harvest is thin by design; **skill-specific harvest tails stay in
the owning skill** (e.g. an audit's phase-sheet harvest).

## §6 — Done + close-gate

A review is **done** when **every finding is terminal** (verified or withdrawn)
**and the pass is concluded**. `doctrine review status RV-NNN` then reports
`done · await=none`. Until the raiser concludes, a ledger whose findings are all
terminal — including one with no findings at all — reads `active ·
await=raiser`, so "no findings yet" is never mistaken for completion. Done is
about the *ledger*; closing the *subject* is the next, separate move.

The **close-gate**: an unresolved `blocker` on an active RV refuses the
target's closure transitions — resolve it (`verify` or `withdraw`) before the
subject can advance. `major` / `minor` / `nit` never gate.

**Parent-tree caveat.** The turn verbs (`raise`, `dispose`, `amend`, `verify`,
`contest`, `reopen`, `withdraw`, `conclude`) and `status`, `prime` and `unlock`
refuse a root inside a worktree **fork**: any linked worktree that is not a
dispatch coordination worktree. `review new`, `show` and `list` do **not**
refuse. A `review new` run in a fork succeeds and leaves a stray ledger there,
which the next turn verb then refuses, so a successful `new` is no proof you are
outside a fork. Open and drive reviews from the primary tree or a coordination
worktree (or land the fork first), never from inside an isolated worker fork.
