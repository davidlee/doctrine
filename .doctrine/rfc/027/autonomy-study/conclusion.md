# Brief 05 (bounded autonomy / arbitration) — diff against landed RFC-027

**Date:** 2026-08-08. **Mode:** `/preflight` diff, not a study run. **Status:**
brief 05 should **not** be run as authored.

Brief 02's §6/7 summarised brief 05 into RFC-027 in three places — `H12`
(`rfc-027.md:762`), implication 8 (`:1137`), Stage 3's probe (`:1601`). Nobody
had read the brief itself. This is the diff of brief against landed text, plus
the incumbent-ownership check the pack's own thesis demands and the summary
skipped.

## Headline

Brief 05 was commissioned against RFC-025 as an *open* question. That question
**closed on 2026-08-03** — five days before the pack landed — and RFC-027 does
not know it.

| Governance | Status | Cited in RFC-027 |
|---|---|---|
| `ADR-020` — execution capsules as the dispatch authority boundary | accepted 2026-08-03 | **no** |
| `SPEC-030` — dispatch execution capsules | active | **no** |
| `REV-046` — capsule cutover | proposed | **no** |
| `DEC-133`–`DEC-137` — journal/exhibit, orchestration topology, ingestion, policy, recovery | accepted 2026-08-03 | **no** |
| `ADR-017` — actionability gating via inbound `needs` on unsettled records | accepted 2026-06-26 | **no** |

Verified by grep with a positive control (`DEC-` matches 9 lines in
`rfc-027.md`; `DEC-13[0-9]` matches zero. `ADR-0` matches only 003, 007, 014,
016). Every capsule passage in RFC-027 attributes the model to "RFC-025 and
SL-241", and the threat model at `:966` still reads *"SL-241 is testing that
claim; this RFC does not pre-judge its result."* SL-241 reported; ADR-020
selected the architecture.

## What the summary dropped

Brief 05 has nine required outputs, seven falsifiers, six constraints.

| # | Required output | Landed | Verdict |
|---|---|---|---|
| 1 | Authority/communication map (7-column actor table) | 4 prose actor bullets, `:979`–`:992` | **dropped** — no `may propose` / `persistence tier` columns, no seam inventory |
| 2 | Decision-case payload mapped to incumbent owners | 6-field case asserted, `:1002` | **partial + distorted** — 3 of 9 fields dropped, ownership test not run |
| 3 | Adaptation-envelope projection analysis | envelope shape, `:1139`–`:1148` | **shape landed, analysis dropped** — "can it be assembled from incumbents?" never asked; punted to RFC-026 `P9` |
| 4 | End-to-end trusted-side arbitration sequence | probe *question* only, `:1604` | **dropped as a trace** — but 4 of its 5 sub-determinations are already answered by governance (below) |
| 5 | Historical A/B/C case table + missing-context analysis | stated as `H12`'s confirm/kill, `:797` | **dropped** — the empirically load-bearing arm |
| 6 | Arbitration-strategy comparison | 5 strategies + 5 roles, `:777`–`:789` | **distorted** — see D1 |
| 7 | Delegable-by-policy vs human-reserved | open question 19, `:1725` | registered open, not answered — honest |
| 8 | Is a new persistent case/record earned? | open question 20 + non-goal `:468` | **pre-empted** — the non-goal forecloses the output before the test ran |
| 9 | Smallest next experiment / slice | probe question, `:1601` | partial |

**Falsifiers: one of seven landed.** Only #1 (tacit user preferences) survives,
as `H12`'s Kill. The six dropped include #5 — *"existing inquiry/review records
already solve the problem and a new case concept would duplicate them"* — which
is the pack's own governing thesis, and which **fires** (see below).

**Constraints: four of six landed.** Dropped: the explicit contrast "compact
evidence-bearing escalation **over transcript forwarding**", and "do not make
multi-agent arbitration mandatory where a cheaper path is adequate" (implied by
the ladder's order, never stated).

## What the summary distorted

**D1 — a comparison set was rendered as a monotonic ladder.** Brief E: *"Do not
select a quorum algorithm yet. **Compare** the information and authority needs
of"* five strategies. RFC-027 `:774`: *"route a decision case through
**progressively stronger** mechanisms."* The ordering — in particular recursive
research/design being *stronger* than differentiated multi-agent arbitration —
is asserted, never argued, and the brief did not claim it. Brief E's actual
question (*which roles need genuinely different context/capabilities versus
merely different prompts*) was dropped entirely; it is the falsifier against the
differentiated-arbitration proposal.

**D2 — a hypothesis-to-test was rendered as a settled shape.** Brief B offers a
*"candidate logical payload"* and says *"test whether this can be represented
using existing observed runtime records, review findings, inquiries, evidence
and control-plane dispositions"*, explicitly invoking the test RFC-027 used
against the rejected change row. RFC-027 `:1002` states *"The submission it
would receive **is** a decision case: …"* — declarative, three fields lighter,
ownership test never applied. The fact-ownership table (`:931`–`:942`) has no
decision-case row.

The three dropped fields are not filler:

- **authority requested / why local authority is insufficient** — the field that
  makes a case *routable*. Without it the escalation ladder has no routing input.
- **attempted resolutions** — what stops an arbiter re-deriving what the worker
  already tried.
- **worker observation, distinct from evidence** — the `P5`-clean proposal slot.

**D3 — "delegated *LLM* semantic arbitration" became "delegated semantic
arbitration"** (`:770`). Mechanism-neutral, arguably an improvement, but it
silently widens the middle tier to include the mechanical rung.

**D4 — Stage 3's probe asks a question the same document already answered.**
The probe (`:1604`) asks whether a worker can *"block only the affected
frontier, continue unrelated work"* — an obligation-granular question. `H9`
(`:669`, same document, 900 lines earlier) tested obligation granularity and
found it *"moved the actionable frontier on none of SL-233 (16 phases), SL-057,
or SL-229."* And `ADR-020` answers it structurally: the **phase capsule is the
transaction unit**, so a partial halt has no transaction to express it in.
`DEC-134` is explicit — *"Escalation from a headless worker is a payload-minimal
notification followed by a halt."*

## The ownership test the summary skipped

Run now. It fires, for the fifth time in this pack.

### Four of brief 05's five D-determinations are already owned

| Brief 05 question D | Owner | Answer |
|---|---|---|
| capsule stopped on only the affected obligation | `ADR-020`, `DEC-134` | **no** — phase capsule is the transaction; escalation halts it |
| unrelated obligation frontiers continue | `ADR-020` | yes, but at *phase capsule* granularity, not obligation |
| what state survives capsule cleanup | `ADR-020`, `DEC-133` | frozen capsule non-evictable until integration/closure; admission journal endures, exhibits may expire |
| chronology anchor when capsule history is rewritable | `ADR-020` step 5, `DEC-133` | the durable admission journal — control-plane-owned, written before canonical mutation |
| disposition applied without trusting worker normative state | `ADR-020` | yes — *"local claims, process exit, branch names, and prose are evidence to inspect, never admission authority"* |

RFC-027 `:1598` says continued capsule work after detection *"needs an
externally ordered checkpoint or receipt outside rewritable capsule state"* —
and does not name the admission journal, which is exactly that.

### The decision case is three incumbent mechanisms, unevenly composed

**1. `QUE` record + inbound `needs` gating — `ADR-017`, accepted, shipped.**
An unsettled record on the dep overlay blocks the work that declares
`needs → <record>`; settling it unblocks, for free. Verified live in code:
`StatusClass::Gating` at `src/priority/partition.rs:38` with `VT-2`/`VT-5`
tests; `blocked_by` filters `class_of(...) != Terminal` at
`src/priority/channels.rs:62`; `is_admissible_dep_target` (work-like ∪ records)
at `src/commands/dep_seq.rs:34`.

**Live exhibit:** `.doctrine/revision/046/revision-046.toml:25` —
`needs = ["QUE-200", "QUE-201", "QUE-202"]`. `REV-046`, the capsule cutover
revision, was itself gated on three unsettled questions; all three are now
`answered` and `doctrine inspect REV-046` reports `actionable: true`. The
narrow-blocking mechanism brief 05 asks for is **running in production, on this
very subsystem**.

**2. Design-run inquiry map.** `InquiryLifecycle {Open, Resolved, Deferred,
Pruned}`; `is_blocked` derived-never-stored (`DEC-060`) at
`src/design_run/inquiry.rs`; the `blocking-inquiries-dispositioned` gate pauses
`inquiring → drafting`; disposal requires a semantic `Disposition`
(`Created` / `Adopted` / `RetainedUnresolved` / `NonDurable`). This is
Doctrine's existing *ask-the-question-and-pause* model.

**3. `/consult` + `/knowledge`.** The skill's elicitation shape is brief 05's
payload almost field-for-field (governing canon / obstacle / discoveries /
options / tradeoffs), and it already names the durable sink: *"a resolved
tradeoff → DEC; one still open → QUE, with the blocked work gating itself
(`doctrine needs <work> QUE-n`)."*

### What is genuinely missing is the composition, not the concept

Verified by scout with positive controls (`raw/escalation-seam-scout.md`):

- A confined worker has exactly **two** structured outcomes — `Committed` or
  `Refused{token}` (`src/mcp_server/worker_commit.rs`). No blocked / question /
  needs-judgement arm.
- The orchestrator's `NextKind` oracle is closed at `spawn / await-worker /
  import / verify / triage-verify-failure / reverify-stale / conclude / reap /
  all-reaped` (`src/dispatch.rs:6502`). The only judgment halt is `Triage`, and
  it is triggered by *red verify evidence*, not by a worker.
- `ReceiptStatus::Blocked` exists but is derived from the runtime phase sheet,
  written by the **orchestrator** via `slice phase`. A worker cannot write it —
  `.doctrine/` is a hard `forbidden-zone` refusal in the commit belt.
- `design apply` is `Write`-classed and **refused under worker-mode**
  (`src/commands/guard.rs:475`), so the inquiry machinery is orchestrator-only.
- The **one** worker-reachable structured write is `observation_record`,
  narrowed to `friction` only and performed **server-side at the primary root**.
  That is the living template for a worker-reachable narrow tool.
- **No admission journal exists in code.** `ADR-020` specifies one; it is
  target-state. (Positive control given: `admission` hits `src/ledger.rs`;
  `"admission journal"` hits zero.)

So: Doctrine implements *unsettled predecessor blocks dependent* **twice
already**, in two subsystems, with two independent settled-ness vocabularies —
`InquiryLifecycle::{Open, Deferred}` vs `StatusClass::{Gating}` — neither aware
of the other. `H9`'s third finding (`:673`) calls obligation edges *"a second
dependency implementation"*; counting `design_run/inquiry.rs` it would be a
**third**. That strengthens `H9`'s rejection and is a free correction.

**This is a third shape for the pack's thesis.** The corpus so far names two:
*mechanism present in one subsystem, absent from its neighbour* (the has-it/
lacks-it table), and *one rule written twice inside one subsystem and drifting*
(`ISS-324`). This is neither: **the same rule solved twice in parallel, in two
subsystems, at two altitudes, both correct.** The defect is not drift — it is
that no third consumer can reach either one.

## Recommendation

**Do not run brief 05 as authored.** Arms A, C and D would re-derive accepted
governance. What remains genuinely un-run:

1. **Brief F — the historical A/B/C retrospective.** The only arm no governance
   touches, and it is `H12`'s stated confirm/kill test. RFC-027 currently
   asserts the middle-tier claim (`:791`) *as a premise*, un-run. That is the
   same failure mode as the withdrawn correction-rarity leg: a claim in the
   corpus with no instrument behind it. **Highest value.**
2. **Brief E's real question** — which arbitration roles need different
   *context/capabilities* versus different *prompts*. Cheap; it is the falsifier
   against differentiated arbitration.
3. **The `DEC-134` reconciliation** — is a structured "decision case" compatible
   with *"payload-minimal notification followed by a halt"*? Probably yes, via
   evidence-in-the-bundle harvested trusted-side, but the RFC does not say so
   and someone will otherwise build the wrong thing.

### Repairs earned against RFC-027

Applied 2026-08-08: `R-a`, `R-b`, `R-e`, `R-g` — the four that fix statements
which are *wrong in the corpus as it stands*. Held: `R-c`, `R-d`, `R-f` — each
is substantive re-authoring of `H12` or implication 8 and deserves its own pass.

| id | repair | state |
|---|---|---|
| R-a | Cite `ADR-020` / `SPEC-030` / `REV-046` / `DEC-133`–`137` / `ADR-017`; amend the stale *"SL-241 is testing that claim"* at `:966` and the non-goal at `:464` | **applied** |
| R-b | Re-frame Stage 3's probe: withdraw the *"block only the affected frontier"* and *"without an interactive channel"* legs as already decided; name the admission journal as the chronology anchor; land the reconciled escalation shape | **applied** |
| R-c | Restore the three dropped payload fields at `:1002` or state why dropped; add a decision-case row to the fact-ownership table, or record that the test has not been applied | held |
| R-d | Re-label the arbitration ladder (`:774`) — argue the ordering or present it unordered, as brief E asked | held |
| R-e | Mark `H12`'s confirm/kill as **un-run**, so the middle-tier claim stops reading as established | **applied** |
| R-f | Add `ADR-017` + `QUE`-gating and the design-run inquiry map as named incumbents to `H12` / implication 8 | held (Stage 3 names them; `H12` and implication 8 do not) |
| R-g | Correct `H9`'s *"second dependency implementation"* (`:673`) to third — `design_run/inquiry.rs` is the missed one | **applied** |

### The reconciled escalation shape — confirmed

Maintainer-confirmed 2026-08-08 and landed in Stage 3: `DEC-134`'s
*payload-minimal notification* and this RFC's *structured decision case* are not
in tension once the **assembly point** is fixed. The worker emits a minimal
notification; the case is assembled **trusted-side, from evidence harvested out
of the frozen capsule**. The worker sends a doorbell, not a dossier — which is
also how brief 05's constraint *"prefer compact evidence-bearing escalation over
transcript forwarding"* is satisfied. `ADR-020` already supplies both halves.

What remains probe-able is the **lift**: how a minimal notification becomes a
disposed case, and which of the two incumbent blocking-derivations carries it.

## Method notes

Per the pack's research discipline: every load-bearing claim above is cited to
primary source and re-read at the cited line; every absence claim carries a
positive control naming the exact command. The scout thread's four findings were
spot-verified independently against `worker_commit.rs`, `dispatch.rs`,
`inquiry.rs`, `channels.rs` and `guard.rs` before being carried here.

Not established, and flagged: whether harness-level parsing consumes a worker's
unstructured hand-back prose (outside the Rust corpus); the precise CLI branch
that surfaces the `blocking-inquiries-dispositioned` refusal;
`design_run/change_log.rs`'s storage shape.
