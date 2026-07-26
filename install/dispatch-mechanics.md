<!-- Shipped reference (ADR-005 PULL tier). Edit the source in
     `install/dispatch-mechanics.md`; the installed copy at
     `.doctrine/dispatch-mechanics.md` is inert. Explains the fork→land funnel and
     its sharp edges — it never reproduces `doctrine --help`; ask the CLI for exact
     flags. Distilled from project-local dispatch memories (CHR-036). -->

# Dispatch mechanics

How the `/dispatch` funnel actually moves code — the fork→verify→import→land
pipeline, the git-plumbing invariants it rests on, and the failure modes that
have cost real respawns. This is the *explain* tier: read it once to build the
mental model. For the sharp mid-operation traps you hit *while* driving a
dispatch, retrieve `mem.signpost.doctrine.dispatch`; for command shapes ask
`doctrine <command> --help`.

The orchestrator is the sole writer. Workers execute one phase inside an
isolated worktree and hand back a single source-delta; the orchestrator imports
that delta, verifies, and commits. Everything below protects that contract.

## The funnel is driven by an oracle, not by a remembered checklist

Each phase's position in the funnel (spawned → worker-committed → imported →
verified → concluded → reaped) is **durable committed state**, not something the
orchestrator holds in its head or reconstructs from `git log`. So the funnel is
not a sequence to memorise and it is not something to work out by hand: ask it.

`dispatch next` reads the committed funnel record and returns **exactly one**
prescription — a kind, the phase it applies to, and the runnable literal in the
surface that owns that verb (some verbs exist only as MCP tools, some only as CLI
lines; they are not interchangeable). Do that one thing, ask again, repeat. It is
strictly read-only, it never heals anything, and it always exits 0 — a
"triage this red evidence" answer is information, not a command failure.

Two properties matter when several file-disjoint phases are in flight at once:

- **Red evidence is global.** The verify suite covers the whole coordination
  tree, so red evidence anywhere is a halt for the whole batch. Triage outranks
  every mechanically-runnable phase, whatever the phase ids are.
- **Waiting never starves work.** A phase whose worker has not returned yet can
  never suppress a phase that has something runnable pending. The other in-flight
  phases are named alongside every prescription, so a single-prescription oracle
  never hides the batch.

**Two altitudes, and they do not overlap.** `dispatch next` answers the
*sub-funnel* question — where ONE phase stands between spawn and reap. `dispatch
status` answers the *slice-lifecycle* question — phases remaining, bundle
staleness against trunk, candidate admission, close-readiness. When `next` reports
that everything is reaped, it hands off to `status` by name; that is the boundary
between the two, and neither subsumes the other.

**A refusal is the recovery procedure.** Every funnel verb refuses in one shape —
the attempted transition, the position it was attempted at, why, and what was
expected instead. That text is not a diagnostic to be interpreted into a repair
plan; it *is* the plan. Read it, act on it, and never improvise around it: a
refusal is a defect or a halt, never a detour.

## The fork base is explicit, never the session HEAD

A worker's fork **must** be created from the explicit coordination base `B` —
the orchestrator's coordination-branch HEAD captured pre-spawn — never from the
implicit current/session HEAD.

The session repo may sit on a different branch (e.g. `main`) than the
coordination branch, so **session HEAD ≠ `B`**. A fork that inherits the
implicit HEAD lands on a divergent base: `S.parent != B`, and the net diff
`B..S` then smuggles the session↔coordination divergence into the import —
unrelated commits ride into the wrong slice's delta.

Two belts enforce it, and both are implemented by verbs rather than run by hand:
- **Worker baseline guard.** The fork's HEAD must be `B` (or a descendant of it,
  once the worker has self-committed) — checked at creation and again by
  `worktree verify-worker`.
- **Import guard (trusted side).** The delta's parent must be `B`, checked on the
  orchestrator's side before anything is applied — so a misbased fork is caught
  even if the worker's own guard never ran.

Harness trap: some spawn backends build the fork from the session HEAD and give
no reliable base control (the claude `Agent` tool at `isolation: worktree` is
one). Where the backend won't honour `B`, spawn a plain agent that self-forks
from `B` explicitly rather than trusting the backend's isolation.

## Verify scope: never run the project-wide gate in the funnel

Funnel and worker verification must be **scoped to the touched files**, not the
whole tree. A project-wide format gate (e.g. `cargo fmt` inside `just gate`)
reformats in place across the crate and pulls unrelated pre-existing drift into
the worker's delta — a base commit that isn't format-clean under the pinned
toolchain will smuggle format churn into every slice.

Scope it: lint + test + a `--check`-only formatter over the touched files. Prove
the phase, don't reformat the world.

## Two ways a worker returns its delta: gated self-commit vs working-tree diff

The worker cannot run raw `git commit` — the linked worktree's `.git` is
read-only (jail wall). Two arms clear that wall differently:

- **claude arm — gated server-side self-commit.** The worker calls the
  `worker_commit` MCP tool, passing only its own opaque `agent` id (its worktree
  name — **never a path**). The *unconfined* server resolves that id to the
  worker's worktree and lands the commit on its behalf, so the jailed worker never
  touches `.git` directly. This is a deliberate, single-purpose bypass of the jail
  wall — therefore the tool's **belts are the security boundary**, not the wall:
  non-empty pre-fmt delta → two-tier scope (a HARD forbidden-zone that hard-refuses
  any write under `.doctrine/`, `.claude/`, or the configured
  `[dispatch].worker-forbidden-writes`, plus a SOFT undeclared-path report) →
  `HEAD == B` → the `check commit` gate → exactly one non-merge commit `C`
  (`C^ == B`) on the worker's own `dispatch/<agent>` branch. A spoofed sibling id
  commits to the *sibling's* branch and leaves its own at `B`.
- **subprocess (pi) arm — working-tree diff.** The worker cannot self-commit at
  all; it hands the tree back and the orchestrator captures the working-tree diff
  (`import --from-worktree`). This is also the fallback when the MCP server is down.

The orchestrator then imports. On the claude arm it imports the **commit**
(`import --fork <C> --branch dispatch/<agent>`); the `--branch` coherence belt
binds the import to the branch the orchestrator *armed*, so it promotes nothing of
a poisoner who committed to a sibling's branch. `verify-worker` accepts the
post-commit `HEAD` because it tests `merge-base --is-ancestor B HEAD` (a
descendant), not `HEAD == B`. Either arm's import is **non-committing** (next
section) — the delta is diff-applied, and the orchestrator commits separately.

Ask `doctrine worktree --help` / `doctrine mcp` for exact flags and the tool's
refusal tokens.

## The orchestrator's authored writes go through `dispatch commit`

The worker's delta lands through the import/`worker_commit` path above. The
*orchestrator's* own authored coord-tree writes — slice status, memory, audit
notes, the boundaries/funnel ledgers — are committed with **`doctrine dispatch
commit --slice N -m <msg> -- <path>…`**, never a raw `git commit`. The verb:

- makes the **pathspec structurally mandatory** (at least one path after `--`;
  a pathless commit is unrepresentable, so another agent's already-staged file
  never rides along — the AGENTS.md path-limit rule, enforced);
- **refuses before touching the index** if the to-be-committed set escapes the
  declared paths (`commit-undeclared-path`) or deletes a path that was not
  *explicitly* named (`commit-undeclared-deletion`) — the ISS-234 mass-delete shape;
- hands the child `git commit` exactly the *validated* deletion set via a
  child-scoped `DOCTRINE_ALLOWED_DELETIONS`, so a **declared** deletion passes the
  coord pre-commit hook's deletion arm while its **reversion arm still runs**.

A **coord-worktree pre-commit hook** (installed by `dispatch setup`,
worktree-scoped `core.hooksPath`) is the backstop under *any* `git commit` in a
coordination worktree: it refuses the funnel-reversion signature — a staged
deletion, or a modification that reverts a funnel-advanced file back to its
pre-advance blob (`doctrine dispatch hook-check`) — then chains to the operator's
effective global/local hook. A deliberate act bypasses **both** arms with
`DOCTRINE_ALLOW_DELETE=1`; the verb never sets it. `doctor` re-checks that every
live coord worktree carries the hook.

## Mode B — the confined-orchestrator arm drives the funnel through MCP

Everything above assumes the **main thread** orchestrates: an unconfined driver
running raw git by hand. **Mode B** is the other arm — a *confined subagent
orchestrator* whose cwd is jailed to the coordination tree and whose raw `.git`
is read-only. It cannot run the funnel by hand; it drives the same fork→land
pipeline entirely through the dispatch MCP tools. Mode B is the capstone of the
confined-drive posture; the main-thread arm (this doc's default) stays the
fallback for when the MCP server is down.

**The fork override.** A confined nested `isolation:worktree` spawn would (per
"The fork base is explicit", above) fork off the session HEAD — no base control.
Mode B arms the create-fork one-shot with `arm-spawn` *first*, so the worktree
hook Forks the worker's `dispatch/<name>` branch at the explicit base `B` with
its jail record provisioned. That provisioning is what makes the worker's
`worker_commit` resolvable server-side — without the armed base there is no
branch for the gated self-commit to land on.

**The tools fold the beats.** Where the main-thread arm applies a delta, commits
it, flips the phase, and records the boundary as separate acts, Mode B's MCP tools
fold each pair into one server-side act:

- `dispatch_import` **applies AND commits** the delta server-side — the import
  folds the commit (no separate orchestrator commit step).
- `dispatch_conclude_phase` is a **two-tier, retry-safe conclude**. The
  `completed` flip is a **disposable runtime write** to the gitignored phase sheet
  (`.doctrine/state/…`) — it never enters committed history, so it can never be a
  "completed-without-boundary" hazard. The real completion signal is the
  **committed `(B, coord_tip)` boundary row**, landed by ONE working-tree-free
  `commit_on_behalf`. The only fault outcome is a flipped (disposable) sheet with
  **no committed boundary**; because that sheet is disposable, a retry simply
  re-composes the boundary — `completed`-WITH-committed-boundary is the only
  durable success state.
- an **undeclared-scope delta is hard-refused before anything lands** — the scope
  belt is server-side, not an orchestrator judgement call.

**reads-verb / writes-MCP split.** Funnel state comes from the funnel's own read
verbs (`next` for the prescription, plus the tree-state / delta / history reads),
never from a hand-assembled `rev-parse`+`diff` reconstruction. Raw git reads
remain fine for what the verbs do not cover — inspecting a worker's commit, for
instance — but every *write* goes through an MCP tool. Mode B never mutates
`.git` by hand.

**The boundary — report-and-halt, never auto-merge.** Trunk-facing ops
(`integrate`, `refresh-base`, candidate) write OUTSIDE the coord jail, which is
read-only to Mode B — so it **report-and-halt**s them to the main thread rather
than attempting them. The same boundary catches a red worker verify and a hard
scope refusal: Mode B returns a structured summary and stops. It never
auto-merges and never self-unblocks a refusal.

## The import severs ancestry — so "did it land?" needs a patch-id oracle

The funnel imports a worker's delta with a non-committing 3-way apply onto `B`,
then the orchestrator commits separately. This **severs git ancestry**: the fork
branch `S` is never an ancestor of the coordination commit. Every naive
landed-oracle is therefore unsound:

- `git branch --merged` — the apply-funnel branch is never merged, always
  reported unmerged (and `git branch -d` always refuses it; deletion needs `-D`).
- **Delta-emptiness** (`git diff B..fork` empty ⇒ landed) — `B..fork` is the
  whole worker delta, never empty for real work, so it refuses every fork. And
  the moment a sibling moves the coordination HEAD, `HEAD..fork` legitimately
  diverges ⇒ non-empty ⇒ also refuses a spent fork. Either way the operator
  learns a `--force` reflex and the safety gate collapses.
- **A runtime-tier "import receipt"** stamped on apply-success — unsound too. It
  certifies the *apply*, not the *commit*: it is born before the separate commit,
  lives in disposable state, and survives a crash-before-commit — reading
  "landed" when no commit ever reached the branch, so a recovery-time cleanup
  reaps the only surviving copy of unmerged work. A flag in disposable state must
  never gate an irreversible `branch -D`.

**Sound oracle: a durable patch-id check.** Run `git cherry <coordination-HEAD>
<fork-branch>` and treat the fork as landed **only when every commit in its
`B..fork` range is marked `-`** (its patch is already present in coordination's
history by patch-id). Any `+` ⇒ not fully landed ⇒ refuse. This is keyed on
durable git state *after* the commit, so it is crash-proof (a crash before the
commit leaves no landed patch ⇒ `+` ⇒ refuse), robust to a sibling moving HEAD
(patch-id matches the commit's patch, not a whole-tree diff), and robust to the
apply severing ancestry (patch-id ≠ ancestry). Ranging over *all* commits (not a
single tip) lets one oracle serve both the single-commit dispatch fork and the
multi-commit solo fork.

### …and where the patch-id oracle does *not* reach: a funnel-managed fork

The patch-id oracle is **not replaced**. It is **narrowed** to the case it can
still decide: a fork with **no funnel row** (solo / pre-funnel / legacy).

Once import became **atomic** — the worker delta ⊕ the funnel row landing in
*one* commit — the landing commit's patch became a strict **superset** of the
fork's, so no patch-id matches and `git cherry` reports **every** funnel-managed
fork unlanded. That is the delta-emptiness failure mode all over again: reap
becomes unreachable on its own prescribed path and the operator learns a
`--force` reflex, which is exactly the collapse the gate exists to prevent.

**For a funnel-managed fork the landing proof is the funnel record** — a
conjunction of three checks, evaluated in order, any one failing falling back to
`git cherry` unchanged (fail-closed):

1. **exactly one** funnel row's `spawn.fork` names the branch (two rows ⇒ refuse
   `ambiguous-fork-row`: no single import speaks for it, and binding the first
   would be a guess);
2. that row stands at **`concluded`** or beyond (`reaped` is the replay case);
3. the **live branch oid** still equals that row's **`import.fork_tip`** — the
   commit the funnel actually imported. A branch advanced past it carries work
   nothing certified, so it is never deleted.

**Why this is not the "import receipt" wearing a hat.** The receipt bullet above
is still correct, and it is the paragraph to check this design against. It fails
on two grounds, and the funnel record inverts *both*:

- *Disposable state.* The receipt lives in gitignored runtime state, which
  `rm -rf` (or a fresh clone) removes and nothing reviews. The funnel record is
  **durable committed state** on the coordination ref, and that ref is
  append-only under compare-and-swap — every advance is constructed with the
  expected old tip as its parent, so a recorded row cannot be quietly rewritten.
- *Born before the commit.* The receipt certifies the **apply**, not the
  **commit**, so it survives a crash-before-commit reading "landed" when nothing
  landed. The `Import` row lands in the **same commit** as the delta. There is no
  window between them: a crash mid-import lands **nothing**, so `imported` ⇒ the
  delta is committed. Positions only advance, so `concluded` ⇒ the delta is
  committed. Check 3 then pins the certificate to the exact commit that was
  imported, so it can never stretch over work pushed to the branch afterwards.

A flag in disposable state must never gate an irreversible `branch -D`. A
committed row that cannot exist unless the commit exists is a different animal.

**One consequence, deliberately accepted.** `doctrine worktree gc` alone takes no
slice and derives none, so it cannot read the record and keeps the patch-id
oracle as its only authority. On a funnel-managed fork it can therefore *disagree*
with `dispatch_reap` and refuse `not-landed`. That refusal's message signposts
`dispatch_reap` as the right route; do not answer it with `--force`.

### The squash blind spot

The patch-id oracle **cannot** distinguish a *multi-commit* squash-merge from a
fork that never landed. A multi-commit `git merge --squash` produces one squash
commit whose patch-id matches none of the fork's individual commits, so `git
cherry` lists every commit `+` and the tip is not an ancestor — byte-for-byte the
signal of a fork that never landed. (A *single*-commit squash is fine: its
patch-id equals the squash commit's, so `git cherry` marks it `-` and certifies
it landed — the blind spot is strictly the multi-commit case.) Do not build a
squash detector; it cannot exist.
Collapse squash + never-landed into one `not-landed` refusal whose message names
both remedies. This is the load-bearing reason a solo fork must land via a
non-squash (`--no-ff`) verb: squash destroys the oracle.

## Landing on a shared trunk races — report and halt, never auto-merge

Integrating onto a live shared trunk is fast-forward-only plus an expected-tip
compare-and-swap. On a trunk where other agents commit concurrently, two
failures bite — both report-and-halt by design:

1. **Trunk moved mid-command.** The admitted target's base went stale between
   "create candidate" and "integrate". Fix: re-create the candidate superseding
   the prior on the new base, re-admit, re-integrate. To shrink the race window,
   chain create→admit→integrate in one shell invocation (the candidate ref name
   is deterministic, so nothing needs threading). Expect retries under churn.
2. **Dirty-worktree refusal.** Integrate resyncs the live checkout and refuses a
   blanket-dirty tree — even when the dirty file is another slice's authored WIP
   that cannot conflict with the projected code. You may not stash or discard
   another agent's uncommitted work. Resolution is the work's owner committing
   it, then re-superseding (their commit advanced trunk) and integrating. The
   driver cannot self-unblock — surface it and wait.

The trunk ladder defaults to the remote's `origin/HEAD`, which lags a local
trunk; point dispatch verbs at the intended local trunk ref explicitly.

## Split lineage — record an already-landed payload, don't advance trunk

`--integrate` advances trunk fast-forward-only. When the reviewed code reached
trunk by a sanctioned route the dispatch journal never recorded — an operator
direct-land, a manual `--no-ff` merge of the payload, an external integration —
the landed payload is an **ancestor** of the trunk tip, not a descendant, so
`--integrate` can only refuse (non-ff) and the close gate refuses `done`
("dispatched but no trunk row"). The reviewed code *is* on trunk; only the ledger
row is missing.

`dispatch sync --slice N --record-integration --trunk <ref>` fills exactly that
gap. It resolves the earned trunk payload from the ledger — identical to
`--integrate`'s planning: the phase-chain tip, or the admitted `close_target` when
a candidate workflow is active (**never** `review/<N>`, a review surface that is
never a trunk payload) — asserts that payload is **already an ancestor** of trunk
(the same `is_ancestor` standard the close gate holds), and commits one
**Verified** trunk row to `dispatch/<N>`, mutating no external ref. It is the
sanctioned replacement for a hand-edited `journal.toml` trunk row (SL-211). It
*records*; it never *advances* — landing the payload is a separate, out-of-band
step (`git merge --no-ff phase/<N>-NN`, or the admitted candidate). Retrieve
`mem.pattern.dispatch.split-lineage-close-conflict-direct-land` for the full
recovery.

## Worker-spawn identity is accident-fenced, not fail-closed

A `SubagentStart`-style spawn hook that stamps a worker-identity marker runs
synchronously (the marker is present before the worker's first command *when the
hook succeeds*) but is **read-only** — it cannot abort the subagent on failure.
On a stamp failure the worker proceeds unstamped and un-gateable by the hook. So
worker identity must be fenced by the **import belt + a worker-mode env guard +
the pre-distilled prompt**, never by the hook's exit status. The only
fail-closed-capable creation seam is a worktree-creation hook (non-zero exit
aborts creation), preferable *where the harness exposes it with enough payload to
act on* — often it does not: a creation hook whose payload lacks the worker's
type/path can neither scope its check nor identify what to abort, so it stays
deferred and the belt-plus-guard fence remains the default.

## Workers can silently discard their own work

A worker may build a phase correctly (tests green) and then `git reset` /
`checkout -- ` / `stash` / `clean` the entire delta away — hallucinating a
"pre-existing WIP", reverting its own edits, and never committing. The fork comes
back clean (HEAD == B, empty diff). Fence it at the prompt:
- State the fork is a **clean** checkout with **no** WIP and that the target
  files do not yet exist.
- Forbid every work-discarding git verb; the only git the worker runs is the
  final `git add <paths>` + `commit`.
- For a red-proof reversion (TDD), instruct it to *edit* the scratch out, never
  to git-discard it.

And never trust the worker's self-reported success — the fork's committed git
state is the ground truth. Re-read the commit, re-run the suite; distrust the
handover's own green/failure labels.

## Subprocess-arm RPC hygiene (codex/pi)

When the worker is a subprocess speaking a line-oriented RPC:
- **One compact JSON object per line.** A pretty-printed prompt message emits
  multi-line JSON and every line fails to parse — the prompt never lands and the
  worker sits idle. Build RPC lines compact (single-line).
- **The process may park on success, not self-exit.** It emits a completion
  event but blocks on stdin, so a timeout-bound spawn burns the full timeout even
  though work finished in minutes. Run a watcher that polls the log for the
  completion event and kills the process on match, so the spawn returns at
  completion rather than at timeout.

## See also

- `mem.signpost.doctrine.dispatch` — the retrieval index for the sharp
  mid-operation traps (retrieve *during* a dispatch, not up front).
- ADR-006 (worktree posture), ADR-008 (jail isolation), ADR-011 (harness-agnostic
  spawn), ADR-012 (integration topology) — the decisions behind this machinery.
- `doctrine dispatch --help`, `doctrine worktree --help` — exact command shapes.
