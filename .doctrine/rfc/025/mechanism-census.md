# RFC-025 mechanism census — what capsules delete, transform, or keep

Per RFC-025 § Investigation method: for every current dispatch mechanism,
record the property protected, the failure prevented, whether that failure
still exists under capsules, and the replacement. The purpose is to avoid
migrating complexity whose only purpose was defending shared-worktree hazards
— and, symmetrically, to avoid deleting a mechanism whose invariant survives.

Sources read in full: SPEC-012 (mechanism), SPEC-021 (orchestrator process),
SPEC-022 (git interaction model); boot governance (DEC-003, ISS-218 context);
the dispatch memory corpus as cited inline. Census date 2026-08-01, authored
against the capsule model as sharpened in discussion: workers are uniform
`claude -p` (or codex/pi) subprocesses under OS sandbox (bwrap/seatbelt) in a
fresh clone at a pinned base — never in-session subagents; the control plane
harvests, verifies, normalizes, and admits (QUE-200 pipeline).

**Verdict vocabulary.**
- **DELETE** — the failure is unrepresentable under capsules; no replacement.
- **TRANSFORM** — the invariant survives; the mechanism is replaced by a
  named capsule-model counterpart.
- **KEEP** — survives essentially unchanged.
- **SCOPED** — dies for dispatch but retained while another consumer (solo
  `/execute` worktrees) still rides it.

Flagged rows (†) are contentious — judgment calls needing operator review,
expanded in § Contentious rows.

## A. Workspace provisioning (SPEC-012)

| # | mechanism | property protected | failure prevented | still exists? | verdict / replacement |
|---|---|---|---|---|---|
| A1 | `worktree provision` + `.worktreeinclude` allowlist; `select_copies` two-layer withheld-tier exclusion (REQ-189/190) | fork carries no coordination/runtime tier; no shared-mutable file crosses isolation | worker corrupting shared phase sheets / handover / memory caches | Sharing dies (fresh clone shares nothing), but *selection* survives: something must still decide which non-git files enter the capsule | **TRANSFORM** → capsule provisioning manifest (RFC-025 names `.worktreeinclude` as the seed). Default-deny opt-in; the "guarantee vs smell-test" two-layer structure collapses — with nothing shared, a leak is a contamination bug, not a corruption vector |
| A2 | `check-allowlist` / `allowlist_violations` static smell test | early warning on a withheld-tier allowlist pattern | operator confusion, late provision failure | subsumed | **DELETE** — absorbed into manifest validation |
| A3 | `verify_sibling_worktree` (refuse provisioning source onto itself) | provision sanity in a linked-worktree web | self-provision clobber | worktree-specific | **DELETE** |

## B. Identity, confinement, mediation (SPEC-012, ADR-006 D2/D2a/D2b, ADR-011)

| # | mechanism | property protected | failure prevented | still exists? | verdict / replacement |
|---|---|---|---|---|---|
| B1 | disk worker marker `.doctrine/state/dispatch/worker` (presence-only, primary identity) | worker-sole-writer: CLI refuses privileged verbs in worker context | worker invoking Orchestrator/write verbs against the shared `.git` / authored state | No. A capsule holds no authority over canonical state; a capsule-local `doctrine` write touches only capsule state, which admission filters | **DELETE** for dispatch. † see B1 note |
| B2 | `DOCTRINE_WORKER` env leg (worker-on-main catch) | catches a harness dropping the worker on the coordination root | privileged writes from a mis-placed worker | "on main" is unrepresentable — the capsule is never the coord tree | **DELETE** |
| B3 | fail-closed marker-absent-linked-worktree rule (REQ-192) | contains the unstamped worker (fail-open SubagentStart) and the marker self-clear | privilege escalation by stamp failure or self-clear | no linked worktrees, no stamps | **DELETE** |
| B4 | `marker --clear --operator`, `worktree status --assert` / `describe_mode`, stale-marker remedies | operator recovery from marker states | self-brick; stale-marker confusion (incl. the coord-tree mis-stamp footgun, mem_019ef99b…) | no marker | **DELETE** |
| B5 | SubagentStart stamp hook (`marker --stamp-subagent`, matcher-scoped, sync-blocking, fail-open) | marker present before worker's first command (claude arm) | unstamped claude worker | no in-session workers; worker identity = OS process in sandbox | **DELETE** — the entire hook-choreography class |
| B6 | base-by-placement (cwd parked in coord tree, `baseRef='head'`) + post-spawn `verify-worker` ancestor check + opaque-Claude-chosen-base residual (IMP-043) | base==B on the claude arm | forking off `main`/wrong base (ISS-029 class); clean-applying-but-semantically-wrong import | No — base is explicit in the provisioning contract | **DELETE** → contracted base pinned at provision; admission ancestry check (QUE-200) is the stronger, exact replacement. IMP-043 is absorbed, not deferred |
| B7 | nested bwrap OS floor (ADR-008 D-B3), codex/pi-only | OS-level write/net confinement | raw-tree escape beyond the worktree | Yes — confinement is still the point | **KEEP, promoted**: becomes the *uniform* mechanism (every worker, every harness), no longer one arm's altitude. Seatbelt is the macOS leg. New design point: admit the Claude API credential, exclude git credentials |
| B8 | confined-orchestrator tier (FR-007/REQ-335 pending): mediated write-funnel, reads-raw/writes-mediated wall, jailed coord tree with ro object store | orchestration without direct `.git` write authority | confined orchestrator composing canonical commits | The *goal* (bounded authority) is the capsule thesis itself; the *mechanism* (mediating writes to a shared store) loses its object — there is no shared store to mediate | **DELETE** as confinement mechanism. † see B8 note — the narrow-boundary idea survives as doorbell + harvest verbs |
| B9 | `worker_commit` gated MCP tool + fork-gate `just validate` doctor/prompt-check skip (DEC-003, ISS-218) | gated worker self-commit; no stale-binary false-red in forks | ungated commits; fork false-reds | No — worker commits freely in-capsule and none of it is canonical; verification runs control-plane-side on the exact candidate | **DELETE** — both the gate and the fork-skip carve-out it needed |
| B10 | import `.doctrine/`/`.claude/` belt (`doctrine-touch`/`claude-touch`, REQ-249) | authored/config state cannot ride a worker delta | governance/config escape landing silently | Yes — a capsule can still *produce* such paths | **TRANSFORM** → forbidden-path admission policy in the harvest pipeline (QUE-200). Same predicate, relocated from patch-time belt to admission gate; force-add-injection subtlety dies with the tracked-diff transport |
| B11 | raw-tree confinement deferral (ADR-006 D2b: hand-edit / bare `git commit` not CLI-stoppable) | — (an honestly-confessed gap) | — | The gap *closes*: OS sandbox bounds the tree a worker can touch; admission bounds what is believed | **resolved by B7 + C3-pipeline** — the deferred residual is the thing capsules actually fix |

## C. Funnel verbs and cadence (SPEC-012 verbs, SPEC-021 process)

| # | mechanism | property protected | failure prevented | still exists? | verdict / replacement |
|---|---|---|---|---|---|
| C1 | `fork` (create+provision+stamp+env-emit, compensating rollback, REQ-248) | one-act correct worker workspace | half-created worktrees; spawn-window marker races | Creation survives; the hazard class shrinks (a broken capsule is deleted, not rolled back inside a shared repo) | **TRANSFORM** → `provision(contract)`; compensation = `rm -rf` the capsule (disposable by construction) |
| C2 | `import` (precond HEAD==B + clean tree; `S^==B` single-non-merge; belt; `git apply --3way --index` non-committing; no runtime receipt) (REQ-249) | exact, belted, crash-honest delta transfer onto the coordination branch | wrong-base import; multi/merge commits; belt escapes; receipt-lies-after-crash | The *invariants* survive; the patch-application mechanism does not | **TRANSFORM** → harvest+admission pipeline: ancestry validation replaces `S^==B`; belt → admission policy (B10); control-plane-authored normalized commit replaces apply-3way; "no runtime receipt" survives as journal-is-the-oracle |
| C3 | `land` (solo non-squash `merge --no-ff`; refuses marker-bearing / worktree-gone forks) (REQ-250) | solo isolated-branch landing with certifiable ancestry | squash-uncertifiable landings; unverifiable provenance | solo `/execute` worktrees are outside RFC-025 scope | **SCOPED** — untouched while solo isolation rides worktrees. † see C3 note |
| C4 | `gc` + `git cherry` landed-oracle (ancestry OR every-commit patch-id; `--superseded-head` TOCTOU guard) (REQ-251) | reap only what provably landed, from durable git state | reaping unlanded work; trusting a crash-surviving runtime flag | The *durable-oracle* principle survives; the patch-id forensics exist only because import severs ancestry | **TRANSFORM** → capsule GC keyed on the admission journal (admitted/abandoned recorded control-plane-side). Squash-indistinguishability machinery dies. SCOPED residue for solo gc |
| C5 | branch-point guard / stationary-head precondition / one-worker-per-base landing / self-base serial advance (REQ-191, SPEC-021 FR-003) | landing exactness on a shared moving HEAD | foreign HEAD movement silently absorbed; parallel-landing races | The shared-HEAD hazard dies; *ordering parallel results from one base* survives (RFC-025 Q8) | **TRANSFORM** → expected-base CAS at admission; second-result-from-same-base becomes an explicit rebase/supersede decision. † see C5 note |
| C6 | eight-step funnel cadence, report-and-halt, knowledge-trails-code (SPEC-021 FR-001) | ordered, crash-equals-resume batch discipline | auto-resolve disasters; knowledge recorded ahead of unconfirmed code | Yes — a capsule phase transaction needs the same ordered discipline | **TRANSFORM** → capsule lifecycle contract (provision → execute → harvest → verify → admit → integrate → gc), same halt discipline, journal-backed |
| C7 | funnel run-state + verb legality gating + `dispatch next` oracle + one-state-machine-per-transport (REQ-384–387, zero-rescue) | drivable-blind funnel; refuse-out-of-order with positional naming | stuck orchestrators; skipped verification | Yes — transport-independent by design (REQ-387 anticipated exactly this) | **KEEP** — transitions renamed to capsule lifecycle states. The largest recent investment that carries over whole |
| C8 | per-batch pre-spawn `B` capture | landing target pinned before spawn | drifting base mid-batch | Yes | **TRANSFORM** → contracted base recorded in `contract.toml` at provision — same act, earlier, durable |
| C9 | pre-dispatch baseline-verify (codex/pi only; claude gap confessed at D6) | worker never starts from a broken base | wasted worker run on unbuildable fork | Yes | **KEEP, promoted** → capsule cold baseline build+verify (RFC-025 v0 step 3), now uniform — closes the claude-arm gap. Inheritance across same-generation capsules is RFC-025 Q5 |
| C10 | arm routing: `claude-force-subprocess-dispatch` → env-marker → mismatch-refusal (SPEC-021 FR-002, ADR-011 D3) | deterministic arm choice; no blind spawn | wrong-arm spawns | No — one arm: uniform subprocess under sandbox; harness choice is which binary the launcher execs | **DELETE** — with the whole per-harness altitude table (SPEC-021 D5, SPEC-012 D6). The largest single complexity deletion |
| C11 | coordination worktree (`worktree coordinate`, in-root placement, cwd parked, jail cwd-revert hazard, worktree-life<branch-life) (SPEC-021 FR-004) | placement-based base control; isolation from dirty primary; refuse concurrent same-slice | ISS-029 class; shared-main contention | Placement-as-base dies with B6; landing is ref/object-db-level and needs no checkout | **DELETE** † see C11 note — concurrent-same-slice refusal relocates to the work-contract registry |
| C12 | claude `Agent`-tool hazards: collapse-onto-parent, forks-bash-cwd-head, seatbelt write-floor in-situ confinement (RSK-014), `dispatch/` prefix ambiguity | — (hazard containments, not chosen mechanisms) | various | No — no in-session workers | **DELETE** — the entire memory-corpus containment class rides out with the subagent worker |

## D. Git substrate (SPEC-022)

| # | mechanism | property protected | failure prevented | still exists? | verdict / replacement |
|---|---|---|---|---|---|
| D1 | ref taxonomy, two mutability classes (REQ-311) | class-per-ref lifecycle clarity | treating evidence as mutable (SL-067 trap) | Yes — the model keeps refs | **KEEP** — population changes (see D2, C11); classes survive |
| D2 | evidence refs `review/<N>`, `phase/<N>-NN`: zero-oid CAS, report-not-clobber, reconstructability (REQ-312/321) | immutable, re-derivable audit inputs | clobbered evidence; unauditable history | Partly — candidate/admitted OIDs + archived capsule bundles become the primary evidence | **KEEP, reduced** † — RFC-025 Q6 (which refs remain as human-facing views) is exactly this row; forensic bundle archive may absorb some of it |
| D3 | pinned fork-point `trunk_base_B` + `refresh-base` sole-explicit-advance (REQ-313) | projections never parent on live trunk tip | silent reparenting; distorted diffs | Yes — verbatim RFC-025 invariant ("no integration from a drifting live branch tip") | **KEEP** as principle; mechanism simplifies — the contracted base *is* the pin; "refresh-base" becomes "provision a new capsule at the new accepted base" |
| D4 | two-stage audit-gated projection: trunk isolated to opt-in stage-2 (REQ-314) | audit interposable between preparation and integration | trunk-by-default | Yes | **KEEP** |
| D5 | CAS journal: journal-before-mutation, 3-way idempotent replay, worktree-aware advance, dirty pre-gate (REQ-315) | crash ≡ re-run; no force, no auto-resolve | ambiguous partial syncs; phantom reverse-diff (ISS-022/030) | Yes — RFC-025 names journal+CAS as principles to preserve | **KEEP** — the worktree-aware advance leg shrinks to the surviving checkouts (primary tree) |
| D6 | candidate layer: roles, admit-by-OID, provenance chain gate, no close-time merge (REQ-316) | what-lands-on-trunk is an explicit immutable choice from verified evidence | integrating a drifted tip; laundering unverified history | Yes | **KEEP** — capsule harvest feeds `candidate create`; RFC-025 Q7 (audit repair) decides whether repair mutates a retained capsule or mints a repair-capsule, but admission is unchanged |
| D7 | repair→integrate propagation contract (REQ-317) | candidate repairs not silently dropped at close | close from unrepaired default source | Yes | **KEEP** |
| D8 | run-ledger object-db sourcing (`read_path_at` from branch tip, never filesystem) (REQ-318) | checkout-independent, crash-safe ledger reads | uncommitted-state lies (ISS-039 zero-phase-cuts) | The principle survives; the specific failure dies — phase records become control-plane-authored at harvest, not worker-committed ledger files | **TRANSFORM** — committed-state-not-working-tree sourcing survives wherever the control-plane ledger lands; ISS-039's failure mode is unrepresentable |
| D9 | trunk resolution: `DOCTRINE_TRUNK_REF` wins / `freshest_descendant` ladder / close-time `is_ancestor` backstop (REQ-319) | correct trunk target; no terminal-but-unintegrated slice | stale-trunk forks (SL-127); projected-never-integrated (SL-126) | Yes — unrelated to worker topology | **KEEP** |
| D10 | crash-safety envelope: no force-push, no auto-resolve (REQ-320) | data loss structurally unreachable in projection | forced/auto-merged refs | Yes | **KEEP** |
| D11 | reservation refs (remote, permanent, zero-oid create-CAS) (SL-148) | cross-clone id reservation | id collisions | Yes — orthogonal to dispatch | **KEEP** untouched |
| D12 | first-class git read verbs, no raw-git funnel reads (REQ-388) | funnel reads immune to shelling/proxy hazards | rtk output-rewriting misreads | Yes — and it *absorbs* the whole SPEC-021 rtk-gotcha catalogue as harvest/admission reads move behind verbs | **KEEP, promoted** |
| D13 | working-tree-free funnel + no-pathless-commit guard (REQ-389, ISS-234) | coord-tree reverse-diff cannot commit mass reversions | ISS-234 | The coord-tree hazard dies with C11; pathless-commit discipline survives at remaining checkouts | **TRANSFORM, mostly delete** |

## E. Operational gotchas (SPEC-021 D7 catalogue)

| # | mechanism / constraint | still exists? | verdict |
|---|---|---|---|
| E1 | rtk proxy-safe git idioms (printed-output decisions, `rtk proxy` for blobs) | Only where an agent still shells git; control-plane reads move behind verbs (D12) | **TRANSFORM** — absorbed by REQ-388; residual advice for interactive use |
| E2 | checkout-import idiom (git-apply corruption fallback) | No — no patch application in the transport (fetch/bundle, OID-computed deltas) | **DELETE** |
| E3 | never-widen-the-delta (`git add -A` ban at integration) | No — normalization constructs the commit from harvested trees, no index staging of a shared tree | **DELETE** (general git hygiene remains, but the mechanism is gone) |
| E4 | re-anchor-on-proven-disjoint-HEAD-move (byte-identical per-path proof) | Transformed — same-base second result at admission (C5/Q8) | **TRANSFORM** → admission rebase/supersede policy |
| E5 | landed-oracle-is-durable-git (never a runtime receipt) | Yes | **KEEP** as principle → admission journal is the oracle (C4) |
| E6 | memory-on-trunk-never-in-fork (orphaned anchors) | Transformed — capsule-local writes are non-canonical by construction; the loss mode becomes "not in the harvest manifest" | **TRANSFORM** → harvest-manifest coverage; same for observation capture (fork-local capture refusal → nominated coordination artifacts, RFC-025 already specifies) |
| E7 | combined-tree-verify-is-the-real-gate | Yes — strengthened | **KEEP** → verify the exact candidate (RFC-025 invariant, verbatim) |
| E8 | DOCTRINE_BIN close-time fresh-build ritual (project rule, DEC-003) | Fork-side stale-binary questions die with fork gates; close-time freshness is untouched | **KEEP** (control-plane-only concern now) |

## Contentious rows (†)

**B1 — does *anything* inside the capsule still need role discrimination?**
The census says no: uniform workers, no marker. The one residual candidate is
the RFC's own "capsule orchestrator vs subordinate subagents" section — if a
capsule internally spawns subagents, is their subordination worth mechanism?
Position taken here (per the sharpened thesis): no — the capsule boundary is
the unit of trust; internal structure is the worker's business. If a future
use case wants intra-capsule tiers, that is a new decision, not a migration.

**B8 — FR-007 (confined orchestrator) is pending investment that this
deletes.** The mediated write-funnel exists to let an orchestrator work
without direct `.git` authority *inside a shared store*. Capsules achieve the
same bound by removing the shared store. What genuinely survives is the
*narrow cross-boundary interface* idea — reborn as doorbell + harvest, not as
mediated writes. REQ-335/REQ-387's transport-abstraction framing ("the tier is
a property of the mediated-write contract, not of any harness") is the right
instinct pointed at the wrong boundary; the capsule contract
(`provision/launch/notify/inspect/harvest/verify/normalize/gc`) is its
successor. Recommend: hold FR-007 pending, do not build further on it while
the spike runs.

**C3 — solo `/execute` worktrees are the surviving worktree consumer.** The
census scopes them out, but they keep alive: linked worktrees, `land`, `gc`'s
ancestry leg, the marker-refusal checks in `land`, and worktree provisioning.
If solo isolation later moves to capsules too (nothing in the model prevents
it — a solo phase is just a capsule whose operator is interactive), the
SCOPED rows convert to DELETE and the worktree machinery retires entirely.
That is a follow-on RFC question, not assumed here.

**C5 — Q8 is the one place shared-worktree complexity could re-enter.**
Parallel phases from one accepted base produce N results, and only one can be
first. Stationary-head handled this by refusal; admission-CAS handles it by
making result #2 explicitly rebase/supersede. The danger is rebuilding
auto-merge (RFC-006 territory) at the admission layer under a new name. The
probe should deliberately produce a conflicting pair and confirm the model
report-and-halts rather than resolves.

**C11 — does the control plane want *any* dedicated worktree?** Deleting the
coordination worktree assumes landing is pure ref/object-db work (true under
D5's not-checked-out CAS leg) and that control-plane ledger commits can target
a ref without a checkout (true via the same plumbing that writes journals
today). The residual reasons to keep one: (a) a human-inspectable staging
checkout during audit — but audit reads prepared refs, and a temporary
checkout is cheap; (b) somewhere for the *interactive* control-plane session
to live that is not the primary edge tree — a real UX question flagged for
the workflow/UX affordance discussion, not a correctness need.

**D2 — evidence-ref population under Q6.** Admitted OIDs + archived capsule
bundles are strictly better *forensic* evidence than derived refs; the refs'
surviving value is human ergonomics (a fetchable, diffable name). Recommend
keeping `review/<N>` as a view, and letting `phase/<N>-NN` be superseded by
per-phase admission journal rows + bundles unless auditors demonstrably miss
them.

## Summary

Counts (dispatch scope): **13 DELETE**, **12 TRANSFORM**, **14 KEEP** (3
promoted to stronger positions), **2 SCOPED** to solo worktrees.

The deletions cluster exactly where the thesis predicted: identity and
role-discrimination choreography (B1–B6, C10, C12), the mediation-for-shared-
store apparatus (B8, B9), and patch-transport pathology (E2, E3, C2's apply
leg). The keeps cluster in SPEC-022: the object/ref substrate — journal, CAS,
admit-by-OID, candidate layer, trunk resolution — survives nearly whole, which
confirms RFC-025's "Git boundary" section: the topology shrinks; the
principles ride. The zero-rescue funnel state machine (C7) is the largest
recent investment that transfers intact.

What this feeds:
- **Probe specs (C1–C3 claims):** every DELETE row is a probe obligation —
  the probe must witness the failure being unrepresentable, not just absent.
  The C5/Q8 conflicting-pair probe and the B7 credential-boundary probe are
  the two the table adds beyond RFC-025's existing list.
- **The eventual REV:** the DELETE column enumerates what it retires
  (touching ADR-006, ADR-008, ADR-011, ADR-012, SPEC-012, SPEC-021; SPEC-022
  survives with edits); FR-007/REQ-335 and REQ-387 should be held pending
  meanwhile.
- **RFC-025 #1 cleanup:** the "capsule subagents" section should be rewritten
  to the uniform-subprocess model this census assumes (B1 note).
