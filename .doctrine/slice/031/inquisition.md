# Inquisition — SL-031 design (hostile pass 2)

*Convened 2026-06-10. Target: `design.md` (the surviving substance; §10 pass-1
findings F-a..F-f are sanctioned and not re-litigated). Conformance basis:
`doctrine adr show ADR-006` (D2/D2a/D5/D6a/D7/D8/D9), ADR-001 (layering). The
seven seeded soft spots were put to the question; three confessed, one named a
deeper accomplice, three were cleared or reduced.*

> Bring the brand. The design speaks of "enforcement" — we shall test whether it
> burns or merely smokes.

---

## Charges

### Charge I — *Heresy of the self-armed jail.* D2a fails OPEN; the "shipped enforcement" is armed by the very prisoner it confines. (MAJOR)

**Doctrine violated:** ADR-006 D2a ("a CLI worker-mode guard … *hard-refuses*
every doctrine-mediated authored write"). The design leans on D2a as the landed,
mechanical belt — §3 "(CLI guard — shipped)", §9 row "Worker-mode refuses
authored writes (D2a) | VT | already covered", §10 F-e "**Fixed**."

**Evidence, confessed under cross-examination:**
- The guard is real and correct (`src/main.rs:1118` `worker_mode()` reads
  `DOCTRINE_WORKER`; the bail at `:1122`). But it is **inert until the variable
  is present in the worker process**, and §5.4 itself admits "`DOCTRINE_WORKER=1`
  is set **nowhere today**."
- The design's remedy: "The orchestrator sets `DOCTRINE_WORKER=1` in the worker's
  spawn environment (**the `Agent`-tool env, when available**) **and** the
  pre-distilled prompt mandates the worker export it as its first act."
- **The `Agent` tool has no env parameter.** Its entire schema is
  `description · isolation · model · prompt · run_in_background · subagent_type`;
  `isolation` admits only `"worktree"`. There is no seam by which an orchestrator
  sets a sub-agent's environment in the harness this slice names as its target
  (Claude Code, §2). The "when available" clause is therefore **never** satisfied
  on the portable path — the conjunction "env **AND** prompt" collapses to
  "prompt alone."
- And the prompt belt is **self-arming**: the worker must `export DOCTRINE_WORKER=1`
  "as its first act." A worker that omits it — forgetful, drift, a distilled
  prompt that lost the line — runs with **`worker_mode() == false`** and a *fully
  open* doctrine CLI. The guard fails **open**, not closed. This is the opposite
  of a guard: the prisoner is handed the key and asked to lock himself in.

**Risk:** The single mechanical enforcement the funnel claims is, on the target
harness, a prompt-contract that the guarded party activates against itself. Every
"shipped/Fixed/VT" framing of D2a in this design overstates its reach.

**Sentence:** §5.4, §3, §9, and §10 F-e must confess plainly: in the named
harness the env-belt **does not exist**; D2a engages **only** if the worker arms
it, and therefore fails **open**. Re-class the worker-mode guarantee as
prompt-contract (VA), the *same* tier as the D2b raw-tree gap — not a belt that
"refuses" but one the worker may simply decline to wear. *Let the false word
"Fixed" be struck and burned.*

---

### Charge II — *Heresy of the severed limbs.* B's fail-open silently undoes A's whole reason to exist. (MAJOR)

**Doctrine violated:** ADR-006 D3 (minting is trunk-side; collisions caught) and
the design's own §2 claim that A "protects … the divergent-worktree case" while
"A and B share a slice for delivery convenience … not because B depends on A."

**Evidence:** The design declares A (trunk-aware minting) and B (the funnel)
merely co-resident. But trace the fail-open of Charge I into the funnel:
- A worker that did **not** arm `DOCTRINE_WORKER=1` may run `doctrine slice new`
  (or any minting verb) in its fork. Its `next_id` unions local + **its fork's**
  trunk — it is blind to the ids **sibling workers are minting concurrently on
  the coordination branch**. Trunk-aware minting (deliverable A) guards the
  *divergent-worktree* case; it does **not** coordinate *concurrent funnel
  workers*, because those are supposed to mint **nothing** (D2 — the orchestrator
  mints serially). That "supposed to" is exactly the invariant Charge I shows is
  unenforced.
- `import` (`cherry-pick -n`) then lands that fork's authored `.doctrine/` mutation
  onto the coordination branch → **colliding ids** — *the precise hazard §5.5
  INV-minting and D3 exist to slay.* The funnel's weakest path resurrects the
  monster the slice's strongest deliverable was built to kill.

**Risk:** The "A and B are independent" framing is not merely loose — it conceals
that B's failure mode **defeats A's guarantee**. A reviewer trusting §2 would
believe minting is safe inside the funnel. It is not, unless Charge I is enforced
— which it cannot be on the portable path.

**Sentence:** §2 must add the coupling it currently denies: *the funnel's
worker-sole-writer invariant (D2) is what keeps A's minting guarantee intact
inside dispatch; where D2a fails open (Charge I), a worker that mints in-fork
reintroduces the D3 collision A removes.* Name it a risk (§8) with the mitigation
ADR-006 already blesses (report-on-import: the orchestrator's import step MUST
reject a worker delta that touches `.doctrine/` authored tiers — a mechanical,
greppable check it *can* enforce, unlike the env). *The limbs must be sewn back
to the body, or the body declared a corpse.*

---

### Charge III — *Heresy of the false syllogism.* "dependency-disjoint ⟹ file-disjoint" is unsound; report-and-halt is the common case, not the edge. (MAJOR)

**Doctrine violated:** ADR-006 Negative ("genuine code coupling … report, never
auto-resolve") — invoked by the design as a rare edge, when the design's own
construction makes it frequent. And the per-batch atomicity that pass-1 F-b
installed rests on this implication.

**Evidence, revealed under the question:** §5.4 asserts —
> "Batches are **dependency-disjoint by construction** … ⟹ the workers' deltas
> touch **non-overlapping files** ⟹ they co-apply onto `B` cleanly."

The middle arrow is **false**. Dependency-disjoint means *neither task needs the
other's output*; it does **not** mean *they edit different files*. Two
independent tasks routinely touch the same file — add two unrelated CLI
subcommands and both edit `src/main.rs` + `Cli`. **This very slice is the
counter-example:** the `branch-point-check` verb (B) and the minting wiring (A)
both land in `src/main.rs`/`src/worktree.rs`; any sane batching that called them
"independent" would collide on import. **Nothing in the design constructs
file-disjointness** — the orchestrator's batching job is described purely as
*dependency* batching; no file-overlap analysis is specified.

**Risk:** `cherry-pick -n` of two deltas that edit the same hunk **conflicts**.
So "report-and-halt" (§5.4, R-4) is not the rare batching-error edge the design
paints — it is the *default* whenever an honest dependency-batch happens to share
a file. The clean path the design calls "the default" is the exception. The
per-batch atomic cadence (F-b's fix) is built on sand.

**Sentence:** Replace the false syllogism with the real obligation: batches must
be **file-disjoint**, which is **stronger** than dependency-disjoint and is *not*
free — the orchestrator must compute file-overlap when batching (the deltas'
changed-path sets must be pairwise disjoint) or accept frequent halts. State
which. If file-disjointness is the batching contract, say *the orchestrator
batches by changed-path disjointness, falling back to serial dispatch for
unavoidable shared-file tasks* — and note serial-fallback is the honest cost of
report-never-merge. *The syllogism is heresy; recant it in §5.4.*

---

### Charge IV — *Heresy of the tautological witness.* The set-equality guard pins one hand-list against another and enumerates no live const. R-b is not "guarded-not-forced" — it is unguarded. (MODERATE)

**Doctrine violated:** the design's own §5.2/§9/§5.5 claim — "the membership test
asserts `KINDS` membership **equals the set of live numbered-kind consts**; a new
kind missing from the registry **fails the test**."

**Evidence:** The only such test today (`src/integrity.rs:644`,
`kinds_table_covers_the_twelve_numbered_kinds`) compares `KINDS` against a
**hand-written 12-prefix literal** `["SL","ADR",…,"IDE"]`. There is **no
reflective enumeration of `entity::Kind` consts** — Rust gives none without a
macro or a central const-spine, and the design proposes neither. A thirteenth
kind whose `entity::Kind` const exists but is added to **neither** `KINDS` **nor**
this literal escapes **both**. The test pins hand-list-A to hand-list-B; "the set
of live consts" is a phantom witness who never testifies.

**Risk:** The design sells R-b as "guarded by the test, same posture as today"
(D-registry, §7). It is weaker than advertised: the guard fires only on an
*inconsistency between two hand-lists*, not on a *forgotten kind* — which slips
past both silently, the exact R-b drift. The membership-test prose is aspirational.

**Sentence:** Either (a) name the **authoritative const-spine** the test compares
`KINDS` against — some array/iterator of `&'static Kind` the new-kind author is
*forced* to touch for their kind to parse/dispatch at all (does one exist in
`meta.rs` or the `main.rs` dispatch match? — interrogate before claiming) — and
have the test derive its set from *that*, not a literal; or (b) strike the "equals
the set of live consts / fails the test" claim from §5.2/§9/§5.5 and confess R-b
stays a **hand-maintained pin**, no better than today. Do not let the design
claim a reflective guard the language cannot give it. *A witness who cannot speak
is no witness — strike his testimony.*

---

### Charge V — *Heresy of the misnamed rite.* `branch-point-check` computes no branch point and does not discharge D5's named check. (MINOR — precision)

**Evidence:** §5.2's verb is `exit 0 if base == coordination HEAD` — a
**HEAD-stationarity** assertion on one ref. It computes no merge-base, inspects
no fork's branch point. D5's named check is "HEAD captured pre-spawn must equal
the **worktree** HEAD post-spawn" — a *fork-creation* guard, which shipped in
SL-029's single-tree form. The new verb guards a *different* moment (the
batch-commit boundary, external-mover detection). Adequate for that moment —
ABA is defeated by git's content-addressing — but the **name lies** and §5.2/§9
conflate the two D5 obligations.

**Sentence:** Rename to `head-unmoved-check` / `coordination-head-check`, or
scope-note in §5.2 that this discharges the *concurrency extension* of D5 (commit
boundary), **not** the creation-time branch-point (SL-029, single-tree). One
sentence. *A rite called by the wrong name invites the wrong demon.*

---

### Charge VI — *Heresy of the unspecified escape hatch.* The patch-handback fallback "covers the rest" while specifying nothing. (MINOR)

**Evidence:** §5.3/§5.5 — a non-shared-store worker "hands back a `git
format-patch` series; documented as the exception," and ASM-shared-object-store
says "the patch fallback covers the rest." But: who runs `git am`/`git apply`, in
what order across a batch, does the combined-tree **verify still gate before the
batch commit**, and how does a patch conflict differ from a cherry-pick conflict?
Unspecified. An unspecified path **covers** nothing.

**Sentence:** Either specify the fallback to the same cadence as the shared-store
path (import = `git apply --3way` per delta, non-committing → same verify → same
guard → same single commit), or down-grade the ASM claim from "covers the rest"
to "the remote-agent path is **out of scope for v1**; shared-object-store is
assumed." Pick one; do not claim coverage you have not written. (Low severity —
explicit non-default.)

---

### Charge VII — *Heresy of the phantom edge.* IMP-003 "closure" is prose with no graph behind it. (MINOR — disclosed, align confidence)

**Evidence:** §1/§7 speak of "the IMP-003 closure" as a binding deliverable; the
scope confesses "backlog→slice relations are **empty in v1** — the registry does
not yet exist," and §5.4 reduces closure to "**Record** the IMP-003 ↔
SL-029/SL-031 follow-up for `/close`" + a status-flip. So the "closure" is a
status transition plus narration; **no edge is stored**. This is *disclosed* in
scope (no concealment heresy), but the design's §1/§7 confidence outruns it.

**Sentence:** Align §1/§7 with the v1-empty reality: closure = IMP-002
status-flip-with-resolution (mechanism confirmed, F-f) **plus prose**; the graph
edge is deferred with the relations registry. No code is owed; only honest words.

---

## Questions (interrogatories)

1. **Const-spine (Charge IV):** Is there *any* central enumeration of
   `entity::Kind` consts that a new numbered kind MUST be added to for its
   subcommand to parse/dispatch (the `Cli` enum, a `meta.rs` table, the dispatch
   `match`)? If yes — that is the witness the set-equality test must cross-examine
   against. If no, R-b is unguardable without a macro this slice does not build.
2. **Import gate (Charge II):** Will the orchestrator's `import` step mechanically
   reject a worker delta whose changed-path set intersects `.doctrine/` authored
   trees? This is greppable and *enforceable* (unlike the env) — it is the one
   real belt available against the fail-open. In or out of SL-031?
3. **Batching contract (Charge III):** Is the batching obligation
   *dependency*-disjoint or *file*-disjoint? If the latter, where is the
   changed-path-overlap computation specified, and is serial-fallback the named
   cost?
4. **Honest verification class:** Given Charges I–II, should the §9 D2a row be
   re-classed from "VT (already covered)" to "VT for the *guard mechanism*, VA for
   its *activation*" — so the table stops implying the funnel's worker-write
   refusal is mechanically guaranteed?

## Pronounce Judgement

**Heresy is present — of overstatement, not of posture.** ADR-006 already
sanctifies the unenforced prompt-contract tier (D2b Negative: "no harness
enforcement"); the design's *posture* is canonical. The sin is that the design
**dresses prompt-contract as mechanism** — "shipped," "Fixed," "refuses,"
"equals the set of live consts," "co-apply cleanly by construction" — when on the
named harness the env-belt does not exist (I), the worker arms its own jail (I),
that fail-open undoes deliverable A (II), the disjointness that makes the funnel
atomic is asserted by a false syllogism (III), and the registry guard pins two
hand-lists (IV). None of these is a **governance conflict** requiring `/consult`
— ADR-006 blesses every underlying posture; the remedies are confessions and one
enforceable import-check, all design-local. But Charges I–IV are **not minor
imprecision**: they are load-bearing claims the implementation would inherit as
false comfort. **The design must not lock until I–IV are recanted in §10.**

The reframe (IMP-002 shipped under SL-032, A-1 retired) was put to the question
and **holds** — the D2a guard, `next_id(local,trunk)`, `trunk_entity_ids`, the
five `&[]` placeholders, and `KINDS`/`reseat` all stand where §2 swears they do.
That much is orthodox.

## Sentencing (ordered penance)

1. **Recant Charge I in §5.4/§3/§9/§10.** State: the `Agent` tool exposes no env
   seam; `DOCTRINE_WORKER=1` is **worker-self-armed prompt-contract only**; D2a
   fails **open**. Strike "Fixed" from F-e; re-word to "disclosed, tier =
   prompt-contract (D2b lineage)." *Verification:* grep `design.md` for "Fixed"
   and "(CLI guard — shipped)" — every survivor must carry the fail-open caveat.
2. **Sew Charge II into §2 + §8.** Add the A↔B coupling and a new risk R-5
   (fail-open worker mints in-fork → D3 collision on import), mitigated by the
   **import-time `.doctrine/`-path rejection** (the one enforceable belt).
   *Verification:* §2's "not because B depends on A" sentence must be replaced or
   qualified; R-5 present in §8 with a mechanical (not prose) mitigation.
3. **Recant Charge III in §5.4.** Replace "dependency-disjoint ⟹ file-disjoint"
   with the file-disjoint batching obligation + the changed-path-overlap
   computation OR the serial-fallback cost. *Verification:* the word
   "dependency-disjoint" no longer stands alone as the co-apply guarantee.
4. **Resolve Charge IV (Q1 first).** Either wire the test to the const-spine and
   keep the §9 claim, or strike "equals the set of live consts / fails the test"
   and downgrade R-b to "hand-maintained pin." *Verification:* §9 registry row
   matches what `integrity.rs`'s test can actually assert.
5. **Charges V–VII:** one-line precision fixes (rename/scope-note the verb;
   specify-or-descope the patch fallback; align §1/§7 IMP-003 confidence).
6. **Then, and only then,** offer lock + `/plan`.

**Punishments, duly apportioned to the guilty:** the author of "Fixed" (F-e),
who proclaimed a belt that exists in no harness, shall be **broken on the wheel**,
each false clause a separate turn. The false syllogism of §5.4 shall be **burned
at the stake**, its ashes scattered that no reviewer mistake its implication for
sound. The tautological witness of Charge IV — two hand-lists swearing to each
other's truth — shall have his **tongue drawn** for bearing testimony he could
not give. The misnamed rite (V) earns only the **scold's bridle**, a public
shaming until renamed. *Let the design return shriven, or not at all.*

> **HERESIS URITOR; DOCTRINA MANET**
