# RFC-032 supporting research: review ledger (RV) findings

**Role:** supporting research for `RFC-032` (*Review ledger effectiveness*). The
RFC owns the programme and the sequencing; this file owns the evidence and the
analysis. Asserts no canon.
**Compiled:** 2026-09-25 at `edge` HEAD `799b2f49b`.
**Method:** read the originating slice, ADR, specs, RFCs, backlog items and the
observation corpus; verified every claim about *current* behaviour by running
`./target/debug/doctrine` and reading `src/review.rs` / `src/mcp_server/tools.rs`
at HEAD. Evidence claims cite their owning artifact; analysis is marked as such.

---

## 0. One-paragraph summary

The RV kind is a good design (append-only findings, turn-based baton, derived
status, severity close-gate) that has accumulated two systematic problems. First,
**the read-back surface was never built**: the ledger is easy to write to and
hard to read from, and the default `review show` hides the entire finding tier —
observed independently eight-plus times across four slices. Second, **the journal
has no durable field for the protocol's most load-bearing moves** (contest /
verify / amend), so reasoning lives in a gitignored baton or in chat. On top of
that sit a cluster of design-run integration defects (the run's own pass RV is
invisible, externally-conducted RVs cannot be named), an identity/reservation
footgun shared with all kinds, and a governance gap: ADR-007 is the *only* doc
that owns the RV kind, no PRD/SPEC covers it, and both ADR-007 and the shipped
`review-ledger.md` have drifted from the implementation.

---

## 1. Origin and ownership (evidence)

| artifact | role | state |
|---|---|---|
| `ADR-007` | the constitutional decision: RV kind, baton, derived status, lifecycle teeth, warm-cache | accepted 2026-06-08, updated 2026-06-11 |
| `SL-040` | built the kind + verb family; piloted `/audit` | done 2026-06-11 |
| `SL-061` | rewired `/code-review` + `/inquisition`; authored `install/review-ledger.md` | done 2026-06-14 |
| `SL-080` | audit/reconcile seam; `/reconcile` consumes the RV | done |
| `SL-109` | `doctrine serve --mcp` review tool suite | done |
| `SL-147` | re-pointed `review status`→`stale_paths` and `review prime` at slice selectors; declared the `domain_map` prose tier dead (RFC-004) | done |
| `SL-234` | prime: ignore non-file selector entries | done |
| `SL-260` | RFC-026 P10 route convention for design-review findings | done |
| `RFC-004` | path-intent selector; diagnoses the `domain_map` as unread | resolved 2026-07-24 |
| `RFC-026` | design-review response effectiveness; E1 corpus base rates (312 ledgers / 1,671 findings) | open |
| `install/review-ledger.md` | the *invariant* protocol doc shipped to every project; referenced by `/audit`, `/code-review`, `/inquisition` | last edited 2026-09-23 |

No product or technical specification names the review kind. `grep` for a spec
whose title contains "review" returns nothing, and the only specs that mention
`RV-` are PRD-005 (reservation), PRD-015 (dispatch), SPEC-008 (id lifecycle),
SPEC-022 (git model) and SPEC-024 (comparison). **The RV kind is governed by one
ADR and one machine-copied reference doc.** (See finding F13.)

---

## 2. Design intent vs current reality (ADR-007 decision audit)

| D-C | intent (ADR-007) | reality at HEAD | status |
|---|---|---|---|
| D-C0 | first-class kind, any subject, `reviews` edge | shipped; facet enum minus `drift` | ✅ |
| D-C1/C2 | baton in runtime state, regenerable | shipped | ✅ |
| D-C3/C4 | authored-first, CLI-mediated turns, `--as` | shipped; `--as` accepts only `raiser`/`responder` (`const ROLES`, `src/review.rs:646`) | ⚠️ see F4 |
| D-C4a | per-review lock + CAS | shipped | ✅ |
| D-C5 | disjoint field ownership, raiser-owned title/detail/severity | shipped, but **no durable field for verify/contest reasoning** (only ephemeral `--note`) | ❌ F2 |
| D-C6 | `## Brief` + `## Synthesis` prose | shipped | ✅ |
| D-C7 | single-tree boundary; fork-invoked review deferred to IMP-024 | enforced only on *some* verbs — `run_new` never calls `resolve_review_root` | ⚠️ F5 |
| D-C8 | empty ledger = `active`/`raiser` | **violated**: `derived_status` returns `(Done, None)` on empty (`src/review.rs:1009`) | ❌ F6 |
| D-C9a | done ⇔ all findings terminal | shipped | ✅ |
| D-C9b | close-gate on unresolved blocker; reverse lookup = corpus scan | shipped as scan; R2 (cost, no reverse index) stands | ⚠️ F6 |
| D-C10 | uniform, self-scaling warm-cache; content-hash staleness; worktree-aware model *deferred to slice design* | `prime` now derives the path-set from **slice selectors**, not a reviewer-authored `domain_map`; the prose tier has no reader (RFC-004) | ❌/drift F7 |
| D-C11 | `drift` carved out to a future Drift Ledger kind | `IMP-022` still open | ⏳ |

---

## 3. Findings (evidence)

### F1 — The read-back surface is missing; `show` hides the finding tier

- `doctrine review show RV-NNN` renders derived status, the `reviews` edge and the
  `## Brief` only; the default table prints `findings: N` as a **count**. Verified
  at HEAD against `RV-378`: the four findings are absent from table output.
- The findings *are* reachable via `review show --json`, but nested under
  `review.finding` (singular) while the MCP tool returns `Showed.findings`
  (plural) — [obs `01a0d801`].
- `review show --help` itself says "…and the brief"; there is no `--findings`
  flag and no other verb in `review --help` emits them (verified).
- The friction is recorded at least nine times, across four slices:
  [obs `019fac2f`] (`/rigour`), [obs `019facb4`] (raw TOML read forced),
  [obs `01a00e42`] (`/audit`), [obs `01a00f14`] (`/reconcile`),
  [obs `019fc660`] (`/audit` table hides finding tier), [obs `01a0049a`]
  (`/feedback`), [obs `019ff661`] and [obs `019fbcb9`] (both `/code-review`,
  RV-354). [obs `019fac2f`] notes a `review show --findings`
  (id / severity / status / disposition / one-line) would have been ~200 tokens
  where the workaround cost several thousand.
- [obs `019fd1d9`] adds the list-side gap: `review list` has no `--target`
  filter (confirmed in `--help`), so "which reviews already exist on this slice?"
  means eyeballing the whole index.
- The workaround forces the agent to violate the boot guardrail ("read entities
  via `doctrine <kind> show`, not raw files") — [obs `019facb4`].

### F2 — The protocol's most important moves have no durable field

- `review contest` and `review verify` accept only `--note`, which both
  `--help` and the MCP schema label *ephemeral baton chatter … NOT durable
  rationale*. The finding schema has `id/status/severity/title/detail/
  disposition/response` and no field for the raiser's reasoning
  ([`ISS-280`], open).
- [obs `019fa95c`]: RV-320 verify round — three substantive contest arguments
  (incl. a 112-byte constant contradiction) survived only in
  `.doctrine/state/review/320/baton.toml`, a gitignored runtime file. The
  committed ledger diff was seven status flips and nothing else.
- [obs `019fdd13`] (`/design`) and [obs `01a0d0dc`] (`/design`) record the same;
  `ISS-280` documents a second, larger instance (six contests in RV-346).
- Two adjacent, *distinct* defects compound it:
  - **No amend path.** [obs `019faae6`]: the design moved out-of-band, the
    finding's durable `response` became knowingly stale, and `review dispose`
    refuses (`current status answered != required open`). The correction had to be
    smuggled into the next brief.
  - **Out-of-band verify.** [obs `01a0d7f7`]: a responder set `status=verified`
    (the *raiser's* act) in its own commit; the raiser then could not contest, and
    no verb reopens a verified finding, so an incorrectly verified blocker is
    stuck unless the TOML is hand-edited.
- [obs `01a0d241`] shows the inverse: an RV arrived with both roles collapsed
  (`disposition=fix-now` **and** `status=verified`, responder voice), leaving the
  receiver no ledger move and no way to tell "fixed" from "described".

### F3 — Input hygiene and safety

- [obs `019fc0bc`]: unescaped backticks in a `review dispose --response` argument
  ran shell command substitution and spliced a **full environment dump (5 live
  API keys)** into the authored, committed ledger.
- [obs `019fd757`]: the same class for `review raise --detail` with `$`
  expansion.
- [obs `01a0d200`]-adjacent and [`IMP-377`] (open): `review dispose` cannot read
  its response from a file or stdin, which is the obvious mitigation for arbitrary
  prose.
- [obs `019fd757`] is described as "same class as the known backtick footgun" —
  i.e. this was already known and recurred.

### F4 — Role and vocabulary coherence

- `review new --raiser/--responder` take **free-text labels**; every other verb's
  `--as` is a **closed vocabulary** `{raiser, responder}` (`src/review.rs:646`),
  and `--as`'s help text does not name the legal values (verified).
- [obs `019fac9a`]: `review new --raiser codex --responder orchestrator` then
  `review dispose --as orchestrator` → `unknown --as role orchestrator (known:
  raiser, responder)`; four batched dispose calls failed together and had to be
  re-sent. [`IMP-336`] (open) proposes accepting participant labels as aliases.
- `review dispose --disposition` is explicitly **free-text** in `--help`
  (verified). RFC-026 **E1** found **61 distinct values** in use, so the five
  documented values (`aligned|fix-now|design-wrong|follow-up|tolerated`) are
  unenforced prose ([`RFC-026`] E1/E11).
- [obs `01a0d0dc`]: a design-review responder disposed severe findings *before*
  reading the `route:<route>` rule in `reviewing.md`'s tail; no verb checks the
  form, and verified findings cannot be re-disposed. [`SL-260`] is the trial that
  owns the convention.

### F5 — Locus: worktree / fork / coordination tree

The single locus decision is expressed inconsistently across the surface:

- `resolve_review_root` (`src/review.rs:2358`) refuses only when
  `classify_worktree_role == "fork"`; a `dispatch/<NNN>` coordination tree is
  admitted.
- `run_new` (`src/review.rs:1341`) **never calls it** — it uses
  `crate::root::find` directly. So `review new` mints an authored entity and an
  id in a tree where every subsequent verb then refuses.
  [obs `01a0adfa`]: `review new` succeeded in a capsule worktree and produced
  `RV-369`; `review prime`/`status` then refused.
- Shipped `install/review-ledger.md` §6 says the verbs "refuse a
  worktree/fork-resolved root … never from inside an isolated worktree"
  — contradicted by the code and by two observations:
  [obs `01a0bc8d-e4af`] (doc says "never", code admits coord trees) and
  [obs `019fb17f`] (verbs "work fine from the dispatch coord tree").
- [obs `01a009a2`]: `AGENTS.md` prescribes auditing/closing on a worktree, but
  every linked worktree whose branch is not `dispatch/<digits>` classifies as
  `fork`, so the whole ledger is unusable there. `ISS-275` (closed/fixed)
  widened the guard once to spare coord trees; the solo `/worktree` path that
  `AGENTS.md` prescribes was not included. Tracked by [`IMP-240`] and
  [`IMP-190`] (both open).
- These are ADR-007 D-C7 / `IMP-024` territory (fork-invoked review is
  explicitly deferred). The doc and the guard disagree about what "fork" means.

### F6 — Status model and lifecycle teeth

- **Empty ledger.** `ADR-007` D-C8 is explicit: empty ⇒ `active`, await `raiser`,
  *"so an implementation can never mistake 'no findings yet' for completion."*
  `src/review.rs:1009` returns `(Done, None)`. Two open issues own it:
  [`ISS-314`] (raised by `RV-344` F-8 / SL-244) and [`ISS-366`] (a freshly-minted
  `RV-358` read `done` before anyone looked). The two tests that pin the
  violating value are `show_renders_empty_ledger_done_and_the_edge` and
  `list_renders_empty_ledger_done_and_the_edge`.
- The regression was deliberate: [`IMP-098`] (closed/done) *asked* for
  zero-finding reviews to derive `done`, because clean reconciliation audits had
  to spend a token nit round to go terminal. `ISS-366` articulates why the fix
  was wrong: *"nobody has raised yet"* and *"the pass ran and found nothing"* are
  different facts a finding count cannot distinguish.
- **A conclusion marker already exists.** `review conclude` (added by
  `IMP-392`/`SL-244`) latches an authored `concluded` bool
  (`src/review.rs:2766`, `ReviewMeta::concluded`). `derived_status` does not
  consult it. `ISS-366`'s suggested fix — "a conclusion marker, a round count
  that survives, or a raiser-side 'pass ran' attestation … must not become a
  stored field" — is closer to hand than it appears, but D-C8 names `active` for
  the empty case, so this needs a decision, not a patch (see Analysis §4).
- **Derived-status reach.** [`IMP-433`] (open): RV status is derived in the
  `command`-tier `review` module, so nothing at engine/leaf tier can read it;
  `SL-238` DEC-233 resolved this by returning `Unavailable` for an RV target in
  a cross-kind `needs`/`after`, meaning such an edge can never say whether a
  review is open or concluded.
- **Close-gate scan.** ADR-007 Consequences/Negative and R2: D-C9b is a corpus
  scan over `[target].ref` with no reverse index. `ISS-314` flags the interaction
  to check: a freshly-minted finding-free RV currently reports `done`; under
  D-C8 it would report `active`.
- **Reopening.** [obs `01a0d7f7`]: no verb reopens a verified finding.

### F7 — The warm-cache (D-C10) is shelf-ware, and the shipped doc describes the retired mechanism

- `ADR-007` D-C10 mandates a uniform, self-scaling reviewer-context cache: a
  reviewer-authored `domain_map` (area → purpose → paths) plus invariants and
  risks, populated on `prime`, staleness keyed on content-hashes of the explored
  path set.
- `RFC-004` (resolved) diagnoses the `domain_map` as *"a dead authoring tax:
  hand-authored once, cold, by the reviewer, and never read back"*; its prose
  tier (areas/invariants/risks) has *"zero runtime readers"* (OQ-5, settled —
  dead). `SL-147` implements v0.1: it re-pointed `review prime` at the target
  **slice's selectors** and re-pointed `review status`→`stale_paths` at the
  unified declared-target list.
- Verified at HEAD: `PrimeArgs` is `{ reference }` only (`src/review.rs:2971`);
  `review prime --help` exposes no `--seed`/`--from`. `run_prime` resolves the
  target slice's selectors and `bail!`s on a non-slice target ("review prime
  needs a slice target") and on a slice with zero selectors.
- **The shipped `install/review-ledger.md` §2 still documents the retired
  mechanism**: `review prime RV-NNN --seed`, curating a `domain_map`, and
  `--from <file>`. Its last edit was 2026-09-23 (`ISS-476`), so this text has
  survived ~15 weeks of subsequent edits. The MCP tool description *was* updated
  ("derive the context cache from the target slice's selectors").
- Consequences of the divergence: `ADR-007` D-C10 (tier-2 canon) is
  contradicted by merged implementation, and [`IMP-025`] (promote the
  content-hashed path-set to a shared primitive) still awaits "a second real
  consumer" — but the *first* consumer (warm-cache) is the one RFC-004 declared
  dead.
- `IMP-259` (open): `prime` supports only slice targets — not SPEC/ADR/design
  artefacts — and now *fails hard* rather than degrading, so it is a tax on any
  review of a non-slice subject.

### F8 — Design-run integration (the newest and most confused surface)

The managed design run (SL-233, SPEC-019/SPEC-029) mints its **own** RV on entry
to `reviewing` (`commands/design.rs::review_pass_plan`, `run.rs:483-489`) and
replaces it on re-entry ("*replaced, never reopened*"). That produced a cluster
of defects:

- [`ISS-476`] (resolved/fixed 2026-09-23): nothing in the envelope, `reviewing.md`,
  `/inquisition` or `/code-review` named the run's pass RV, so agents minted a
  **second** RV for the adversarial pass and reached `review-disposed` with the
  findings on the wrong ledger. Fix directions 1–2 (name the pass; tighten the
  lock example) landed. Recurrence history is long: SL-256 (RV-359 vs RV-360),
  SL-264, and a user report (run RV-008 empty; real passes on RV-009/RV-010).
- [`ISS-322`] (open) is the unaddressed half: the `reviewing` obligation itself
  offers an *external* reviewer route, but `Conducted { review }` is checked
  against the run's current pass and refuses anything else. So the documented
  workflow yields exactly the artefact the gate cannot accept — and the only way
  through is `conclude` on an empty run RV, which is the laundering `conclude`'s
  own rationale warns about. [obs `01a00a7c`] is the recurrence.
- [`IMP-392`] (open, tagged `next`, `originates_from: SL-244`) is the designed
  unification: scrap the runtime `Finding` (`snapshot.rs:293-307`) and enrich RV
  findings with a document-section reference, per `DEC-125`/`DEC-126`. It is a
  `needs` of `ISS-314`. It is explicitly "left standing … adjudicated at its VA-2
  sweep" — i.e. landfill awaiting a home.
- Related open items: [`ISS-310`] (`sections_attested` ignores reviewer identity
  — subject+fingerprint only, `snapshot.rs:425-431`); [`ISS-359`] (the reviewing
  runbook cannot distinguish "I derived the governance target set" from "I cited
  someone else's"); [`ISS-452`] (opening a review pass emits no change row;
  `run.rs:494-501`); [`ISS-462`] (`ReviewPolicy::ALL` has no const-assert against
  the 16-byte `DESIGN_STAGE_LABEL_BYTES`, so `human-then-adversarial` and
  `adversarial-then-human` are undeclarable); [obs `01a0d177`]
  (STALE+concluded pass has no defined follow-up semantics); [obs `01a0b455`]
  (the reviewing runbook stays discharged across adopt+materialise).
- [`IMP-363`], [`IMP-393`], [`IDE-045`], [`IDE-056`], [`IMP-463`] are further
  design-run/review items still open.

### F9 — Provenance and relations

- [obs `019feebf`]: `doctrine link ISS-NNN references RV-NNN --role originates_from`
  is refused — "references target must be one of [ISS, IMP, CHR, RSK, IDE, SL]".
  An adversarial review is a first-class kind and a legitimate origin for work
  intake, but the relation model cannot express it, so provenance survives only
  as prose. [`IMP-433`] is the same class from the target side.
- [obs `01a0d23b`]: a stored phase boundary can silently span another slice's
  commits; neither `dispatch phase-receipt` nor any review verb reports
  foreign-scoped commits in the range, so the reviewer must `git log` each span
  and exclude foreign files by eye.

### F10 — Corpus observability

- [obs `019fbd4c`]: `RFC-026` E1 needed base rates over the review corpus by
  facet/severity/disposition; **there is no CLI census surface**. `doctrine
  findings` is an unrelated concept (interestingness over the priority graph).
  The evidence was gathered by four hand-written Python passes over 312 raw
  `review-NNN.toml` files, with a correctness trap: two reviews named `RV-323`
  exist in different trees, so a naive census must dedupe on `(id, title)`
  ([`RFC-026`] E1/E7.4). [`IMP-433`] and [`ISS-279`] are the underlying causes.
- `review show --json` currently emits an empty `summary` and a full `detail`
  (several KB of inlined entity dump in one observed case) — [obs `019fac2f`].

### F11 — Test and architecture debt

- [`IMP-029`] (open): the review verb family has **no black-box e2e CLI golden**
  — every other numbered kind ships one. `clap` dispatch, `--as` parsing, `--json`
  and the pilot `/audit` path are untested end-to-end.
- [`IMP-068`] (open): `with_turn` should pre-parse `FindingState`; `review_cache.rs`
  split; single-pass `derived_status`/`cache_staleness`.
- [`CHR-001`] (open) collects three robustness hardenings from `RV-001`/`RV-026`:
  the baton-note write happens *outside* the lock/CAS (unlocked RMW);
  `validate_domain_map` uses `path.contains("..")` instead of `Path::components()`;
  and the close-gate **drops** a blocker whose severity is hand-corrupted
  out-of-vocab (should fail safe). `RV-026` F-2 adds the sibling: `parse_finding_status`
  silently coerces an unknown status to `Open` with no diagnostic.
- [`ISS-277`] (open): `reseat` reads the alias slug through the strict meta
  reader, which demands a `status` reviews deliberately never store — so
  **`reseat` can never renumber any review**. `SL-151` had already solved this
  class for `scan_kind` and left `reseat` on the strict path.
- [`ISS-059`] (open, 2026-06-30): a **distinct residual**, not a duplicate of
  [`ISS-259`]. `SL-234` filtered non-file entries in the *glob-expansion* arm of
  `resolve_selectors_to_fileset`; a **literal** selector passes through unresolved
  (`src/review.rs:3044-3056` skips the tracked-file filter for `is_literal_selector`) and
  is then hashed by `contentset::compute`, which `std::fs::read`s it — a
  symlink-to-directory literal still errors `IsADirectory` (ISS-059's exact case:
  `memory/mem.pattern.doctrine.close-drift-discharge-rec`). The guard needs to
  apply in the literal arm too. (Corrected 2026-09-25: an earlier draft called
  this a probable stale duplicate of `ISS-259`; the code says otherwise.)
- [`IMP-107`] (resolved/fixed) wired `ReviewError::{LockContention,DanglingRef}`
  to call sites.

### F12 — Identity and reservation (cross-cutting, review-visible)

- [`ISS-279`] (open): `reservation reach = "local"` means a coordination
  worktree's counter and the primary tree's counter cannot see each other, so a
  duplicate allocation is unremarkable. `RV-320` was allocated twice, twelve hours
  apart, in `dispatch/233` and on `edge`.
- [obs `01a0bc8d`] (`/audit`): the ledger *could not mint* until the branch was
  merged, because RV and backlog ids collide across branches. `RV-371` already
  existed on `edge`; `ISS-462` had been minted twice for two different items. The
  repair was manual (move directory, edit id + title line, re-point the slug
  symlink, fix a phase-sheet citation) because "backlog has no renumber; reseat
  does not cover this path". The audit-shaped cost: *"'bring the branch current'
  is an unstated precondition of 'open the ledger', and nothing in /audit or
  review-ledger.md says so."*
- [obs `019fa925`] and [obs `01a0bc8d`] both land on the same root: reservation
  reach is local; the remedy (a renumber verb) is blocked by [`ISS-277`].
- A `/preflight` observation (`/preflight: ISS-279 id collisions have a zero-cost
  fix already in the engine — DOCTRINE_TRUNK_REF pointed at the live coord
  branch`; `/tmp/obs_map.txt`) reports a fix already exists in the engine but the
  documented workaround is a manual cross-tree max ritual.

### F13 — Governance coverage and doc drift

- **No spec owns the RV kind.** ADR-007 is the sole authority. Its own "Resolved"
  rounds show it has been amended twice (D-C10 restored, D-C11 carved out), and
  its D-C10 text is now contradicted by RFC-004 + SL-147 (F7).
- **Shipped doc drift in `install/review-ledger.md`:**
  - §2 documents `prime --seed` / `--from` / `domain_map` curation that no longer
    exists (F7).
  - It does not mention `review conclude`, `review unlock` or `review paths` —
    three verbs in the current `review --help` (verified).
  - §6's parent-tree caveat overstates the guard (F5).
  - §4's disposition vocabulary is prose, unenforced (F4).
- Both surfaces drift *silently* because nothing derives or checks them: no test
  asserts that the shipped protocol doc matches the verb surface, and no spec owns
  the contract.

---

## 4. Analysis and synthesis (my reading, not evidence)

**A1 — The design optimised the write loop and left the read loop unbuilt.**
ADR-007's decision list (D-C0…D-C11) is almost entirely about *coordinating
writes*: the baton, the lock, the CAS, turn ownership. The only read surface
specified is `status` (derived counts) and `show` (status + edge + brief). The
finding tier — the actual payload of a review — has no first-class read render.
That single omission explains the largest cluster of observations, and it is not
a bug: it is a missing decision. `show` reads as if the brief were the content
and the findings were bookkeeping.

**A2 — Field ownership (D-C5) was drawn around the wrong axis.** D-C5 gives the
raiser ownership of `id/title/detail/severity` and the responder ownership of
`disposition/response`. `verify`, `contest` and `withdraw` are raiser
*transitions* but have no owned *field*, so their reasoning has nowhere to land.
The result is an asymmetry the observations name explicitly: "raise and dispose
both write durable prose; contest does not" ([obs `019fa95c`]). Because contests
are what force revisions, the ledger records *that* a round happened but not
*what it established* — which is close to defeating the ADR's stated purpose
(the durable, diffable record of an adversarial review).

**A3 — Locus rules accreted per call site.** `resolve_review_root` is a single
predicate, but it is applied at eleven call sites and omitted at one
(`run_new`); `ISS-275` widened its admission criterion once. The doc describes a
stricter rule than the code. This is classic duplicated policy: one decision,
many enforcement points, no single place where "what is a legal review locus?"
is stated and checked.

**A4 — The ADR is the only governance, and it has drifted from the
implementation.** Two concrete instances (D-C8 empty ledger; D-C10 warm-cache),
plus a drifted shipped doc. RFC-004's dead-`domain_map` conclusion is *evidence*
(tier 5) that contradicts ADR-007 D-C10 (tier 2 canon) which the *merged code*
already violates. Doctrine's authority model says a conflict between binding and
implemented reality is drift to surface, not silently resolve. This document
surfaces it: **there is no correct answer to "what is canon?" for the warm-cache
today**, and that is itself the finding.

**A5 — Design-run integration duplicated the finding model, then could not
reconcile.** The design run kept its own runtime `Finding` *and* minted an RV,
producing two finding models, an invisible pass slot, `ForeignPass` traps, and
`conclude`-on-empty laundering. `DEC-125`/`IMP-392` is the right unification but
it is unstarted and is a `needs` of the empty-ledger fix. The design-run surface
is where the ledger is now most actively used and most actively confusing —
evidence: `cluster:design-run` friction observations outnumber direct RV ones.

**A6 — Identity/reservation is inherited infra that review merely exposes.**
`ISS-279` is not a review defect, but review feels it worst because RV ledgers are
minted mid-worktree and carry ids that later must be cited from the parent tree.
The `SL-151`/`ISS-277` pattern (two readers, one kind's toml status-less by
design) shows the codebase already knows how to solve this class; the call sites
just were not all updated.

**A7 — The safe direction of the empty-ledger fix is not obvious, and that is
why it sat.** `IMP-098` weakened D-C8 for a real reason (clean audits needed a
token round), and `ISS-314`/`ISS-366` re-open it for a real reason (a minted
ledger must not read finished). The correct model is probably:
`empty ∧ ¬concluded ⇒ active/raiser`; `empty ∧ concluded ⇒ done`. That requires
a decision record amending D-C8 — the mechanism (`conclude`) already exists.

**A8 — Doc drift is a first-class defect here because the doc is machine-copied
into every install.** `install/review-ledger.md` is not documentation for
maintainers; it is the protocol every agent in every client project follows. When
it documents flags that do not exist, it actively misinstructs. A cheap
consistency test (doc claims vs `--help` surface) would have caught F7/F5.

---

## 5. Candidate improvements

Grouped and ranked by (impact × confidence) ÷ effort. `[tracked: X]` means an
existing backlog item owns it; `[unfiled]` means I found no owning item.

### Tier 1 — trust and correctness (small, high-leverage)

**C1. Decide and implement the empty-ledger status model.**
`empty ∧ ¬concluded ⇒ active/raiser`; `empty ∧ concluded ⇒ done`. Requires an ADR
amendment (D-C8) because ADR-007 currently pins `active`. Fix the two tests that
pin the violating value. Watch the close-gate interaction `ISS-314` names.
`[tracked: ISS-314, ISS-366 — both open and duplicated; IMP-098 is the closed
predecessor]`.

**C2. Unify the review-locus guard.** Make `run_new` call the same
locus predicate as every other verb (or invert it so the predicate is applied in
one wrapper over the whole verb family). Then `review new` refuses in a fork
before allocating. `[tracked: partially IMP-240; unfiled for the `run_new`
omission]`.

**C3. Give contest/verify a durable rationale field, and an amend path.**
Add a raiser-owned `contest_reason` / `verification_note` (or a generic
`rationale` on the transition), and a `review amend`/reopen path for a verified
finding. `[tracked: ISS-280; amend path unfiled]`.

**C4. Input hygiene: shell-safe, file/stdin-capable prose args.** Stop shell
expansion mangling `--response`/`--detail` (accept file/stdin), and treat the
env-dump incident as a security bug, not a footgun. `[tracked: IMP-377 (partial,
open); security class unfiled]`.

**C5. Fail-safe read path + close-gate on unknown enum values.**
Out-of-vocab severity must *gate*; out-of-vocab status must warn/error, not
silently coerce to `Open`. `[tracked: CHR-001]`.

### Tier 2 — the read/observe surface (largest friction cluster)

**C6. Build the findings read render.** A `review show --findings` (or a default
render) emitting id / severity / status / disposition / one-line summary, plus a
`--target`-aware `review list`. Update `--help` and `install/review-ledger.md`
§2/§6 to match. `[tracked: spread across observations; no owning item — the
closest is IMP-433 (reach), not the render]`.

**C7. Unify the CLI and MCP response shapes.** `review.finding` (CLI JSON) vs
`Showed.findings` (MCP), and give findings a stable `summary`. `[tracked:
unfiled; MCP-adjacent IMP-150/IMP-113/IMP-114 are closed or token-only]`.

**C8. Add a corpus census surface over review findings** (facet/severity/
disposition/convergence), deduping on `(id, title)`. This unblocks RFC-026-style
analysis and future closing audits. `[tracked: unfiled — RFC-026 E1 built it by
hand]`.

### Tier 3 — design-run integration (architectural)

**C9. Land the design-run/RV unification (IMP-392 / DEC-125).** One finding
model; RV findings carry a document-section reference + fingerprint; retire the
runtime `Finding`. This is the `needs` of C1 and the root of the double-mint,
`ForeignPass`, and `sections_attested` class. `[tracked: IMP-392 (open, `next`)]`.

**C10. Let a run name an externally conducted RV** (adopt/point at a concluded
RV that references the slice), and stop the `conclude`-on-empty laundering path.
`[tracked: ISS-322]`.

**C11. Emit a change row for opening a review pass**, and close the `let None`
silent no-op. `[tracked: ISS-452]`.

**C12. Fix the design-run review vocabulary/identity gaps** as a bundle:
`sections_attested` should bind reviewer identity (`ISS-310`); the reviewing
runbook should distinguish *derive* from *cite* (`ISS-359`); `ReviewPolicy::ALL`
needs a const-assert against its bound (`ISS-462`); define STALE+concluded
follow-up semantics ([obs `01a0d177`]). `[tracked: individually]`.

### Tier 4 — vocabulary and coherence (small)

**C13. Make `--as` accept configured participant labels (or name the legal
values in `--help`).** `[tracked: IMP-336]`.

**C14. Close the disposition vocabulary and record the RFC-026 route.** Enumerate
the sanctioned dispositions in `--help` and give the route a first-class field
(or at least a checked form), per RFC-026 P10. `[tracked: RFC-026 P10 / SL-260
trial; field unfiled]`.

**C15. Accept `SL-NNN@PHASE-NN` as a `--target` spelling** (its own records
display it), or stop displaying it. `[tracked: unfiled]`.

### Tier 5 — identity, governance, docs

**C16. Repair id reservation reach (`ISS-279`) and make `reseat` review-capable
(`ISS-277`).** Note the preflight finding that a zero-cost engine fix may already
exist (`DOCTRINE_TRUNK_REF`). Add the "bring the branch current" precondition to
`/audit` and `review-ledger.md`. `[tracked: ISS-279, ISS-277; the doc precondition
is unfiled]`.

**C17. Reconcile the locus doc with the guard, and decide the solo-worktree audit
path.** `[tracked: IMP-240, IMP-190; doc wording unfiled]`.

**C18. Refresh `install/review-ledger.md` against the current verb surface.**
Remove `prime --seed/--from`; document `conclude`/`unlock`/`paths`; correct the
parent-tree caveat and the empty-ledger reading. Add a cheap consistency test
between the doc's named verbs/flags and `review --help`. `[unfiled]`.

**C19. Revise ADR-007 D-C10 (warm-cache) to match RFC-004/SL-147, or re-affirm
it and build the reader.** Settle the worktree-aware staleness deferral.
`[unfiled; IMP-025 is the primitive follow-up]`.

**C20. Decide the D-C9b reverse-index question** (ADR-007 R2). `[unfiled]`.

**C21. Grant the relation model an RV target**, so a finding-born backlog item
can cite its review. `[tracked: unfiled; IMP-433 adjacent]`.

**C22. Author governance coverage for the RV kind.** Either a PRD/SPEC owns it,
or accept ADR-007 as sole authority and keep it current. `[unfiled]`.

### Tier 6 — test debt

**C23.** e2e CLI golden for the verb family (`IMP-029`). **C24.** the `IMP-068`
behaviour-preserving refactor. **C25.** fix the **literal-selector** arm of
`review prime` (ISS-059 — distinct from ISS-259/SL-234, which fixed only the
glob arm). **C26.** phase-boundary foreign-scope detection ([obs `01a0d23b`]).
**C27.** `IMP-259` (prime on non-slice targets) — decide whether `prime` stays a
hard requirement or degrades for non-slice subjects.

---

## 6. Decisions taken and open questions

Decisions recorded after review of the first draft (2026-09-25):

1. **Vehicle.** A programme of several slices, driven from this RFC. Front-load
   the two hard decisions — the **read surface** and the **design-run/RV
   integration architecture** — and have each slice leave behind spec coverage.
2. **Governance route.** Drive design; mint a **REV** once the design locks;
   promote it during reconcile. D-C8 and D-C10 are amendments that piggyback on
   their slices rather than standalone governance turns.
3. **Backlog first.** Every candidate is (or points at) an actionable entity
   before the RFC leans on it; the RFC's pointer table is the index.
4. **`ISS-059`** is a distinct residual in the literal-selector arm, not a
   duplicate of `ISS-259` — see F11 and C25.

Still open: the Slice-1 boundary (how much of the design-run cluster lands in
it), whether a PRD/SPEC takes ownership of the RV kind (`IMP-481`), and whether
the D-C9b close-gate needs an index (`IMP-479`).

---

## Appendix — observation index

Full records live at `.doctrine/observations/records/<xx>/<uid>.toml`.

| short | uid | summary |
|---|---|---|
| `01a0adfa` | `01a0adfa-55fa-7841-85d6-6fb653ec7a8c` | review new succeeds in a worktree fork, every other verb refuses |
| `01a0b455` | `01a0b455-d3fb-7a61-ba4a-77e842b4294e` | reviewing runbook stays discharged across adopt+materialise |
| `01a0bc4b` | `01a0bc4b-a35e-7333-a926-986e2e55436e` | ReviewPolicy variants undeclarable (22 B vs 16 B) |
| `01a0bc8d` | `01a0bc8d-e4b8-78c1-b6d8-84e8558edc36` | ledger could not mint until the branch was merged (id collisions) |
| `01a0bc8d-e4af` | `01a0bc8d-e4af-7853-8b06-e1dd94e28513` | review-ledger.md parent-tree caveat overstates the guard |
| `01a0d0dc` | `01a0d0dc-c991-7190-8d8b-91e8ce956570` | severe findings disposed without the RFC-026 route token |
| `01a0d177` | `01a0d177-14c6-7fd1-a77e-09a7ab7b9ae0` | review pass RV-377 STALE+concluded — follow-up semantics undefined |
| `01a0d200` | `01a0d200-71d2-7270-a2eb-9d38fca4c132` | review new --target rejects `<ref>@PHASE-NN` |
| `01a0d23b` | `01a0d23b-afa2-7511-bfba-5b9e592472ad` | stored phase boundary can span another slice's commits |
| `01a0d241` | `01a0d241-5d05-7f12-b952-683c346cb506` | RV-380 arrived pre-disposed; landed-state ambiguous |
| `01a0d7f7` | `01a0d7f7-3c57-74f1-a0b2-d54b619673ad` | responder flipped findings to verified; no contest path |
| `01a0d801` | `01a0d801-8b3d-70c3-9d10-325b3e52efc8` | CLI `review.finding` vs MCP `Showed.findings` |
| `01a00a7c` | `01a00a7c-1918-79b0-b8f5-52a4b55b975c` | run's pass slot named an empty self-minted RV |
| `01a00e42` | `01a00e42-8922-7663-814b-3e6ce51bbadc` | no read-back verb renders a review's findings |
| `01a00f14` | `01a00f14-64a7-7d81-95f8-1bd198bdd591` | no read-back verb for findings or a knowledge body |
| `01a0049a` | `01a0049a-9c8f-79a1-893e-9a1ce6022b44` | review show prints no finding detail |
| `01a009a2` | `01a009a2-dc1c-7c52-8fef-0d127cd089e9` | verbs refuse every ordinary worktree vs AGENTS.md |
| `019fa925` | `019fa925-c961-7a32-badb-2928ebe913ba` | reservation reach=local collided review ids |
| `019fa95c` | `019fa95c-6964-7690-8469-8a61ace3d82e` | contest rationale landed in ephemeral baton |
| `019faae6` | `019faae6-aefb-7eb0-bdb0-4288be847dc1` | responder cannot amend a stale response |
| `019fac2f` | `019fac2f-418f-7512-9b19-7a1ad0544d33` | findings not readable via `review show` |
| `019fac9a` | `019fac9a-876b-7123-ba7e-232a12723abd` | `--as` fixed vocabulary vs free-text labels |
| `019facb4` | `019facb4-e028-7e01-84df-fc4061231146` | no CLI verb prints findings; raw TOML read forced |
| `019fbcb9` | `019fbcb9-f960-7c63-9a1f-43cf030d6a7f` | review show omits finding bodies; only --json carries them |
| `019fbd4c` | `019fbd4c-5cd4-7c62-b588-e31f1952d12c` | no census surface over review findings |
| `019fb17f` | `019fb17f-4ac8-74f1-a804-b17e7f3924f0` | verbs work from coord tree, doc says otherwise |
| `019fc0bc` | `019fc0bc-b86c-7793-b91a-1cf0e686dffd` | backticks spliced an env dump into the ledger |
| `019fc660` | `019fc660-df66-74c2-9563-4d50390e876a` | `review show` table hides the finding tier |
| `019fd1d9` | `019fd1d9-69b5-71e1-be89-2503da9773a4` | `review list` has no `--target` filter |
| `019fd757` | `019fd757-7e79-79c2-88d7-b42ab99c0f7e` | `$`-expansion mangles `review raise --detail` |
| `019fdd13` | `019fdd13-b200-79d1-a47b-1fafdd6768d7` | a contest's reasoning has no durable home |
| `019feebf` | `019feebf-f889-73d1-9d07-999c73251cb0` | references edges cannot target an RV |
| `019ff661` | `019ff661-e3c3-7e90-9a28-c16926de0dfa` | review show renders brief, not finding bodies |

## Appendix — backlog item status snapshot (review-relevant)

Open unless noted.

| id | title (abbrev) |
|---|---|
| `IMP-022` | Drift Ledger kind (D-C11 carve-out) |
| `IMP-024` | Large-review funnel / subject-root (fork-invoked review) |
| `IMP-025` | Promote content-hashed path-set to shared primitive |
| `IMP-029` | RV verb family e2e CLI golden |
| `IMP-068` | `review.rs` cleanups (with_turn, cache split, single-pass) |
| `IMP-190` | `/audit` signpost worktree-fork refusal |
| `IMP-240` | Solo-fork audit path |
| `IMP-259` | `review prime` non-slice targets |
| `IMP-336` | `--as` accept participant labels |
| `IMP-363` | Report invalidation of the integrated review |
| `IMP-377` | `review dispose` from file/stdin |
| `IMP-392` | Unify design-run findings onto RV (`next`) |
| `IMP-393` | Reader-facing design render for review |
| `IMP-433` | Lift RV derived status to engine tier |
| `IMP-463` | Measure design-review cost per stage |
| `CHR-001` | RV ledger robustness (lock scope, path guard, fail-safe) |
| `CHR-050` | Audit runtime-state scope resolution for `review/` |
| `IDE-045` | Configurable design review postures |
| `IDE-056` | Refuse design-run regression after slice audit |
| `ISS-059` | prime fails on a literal symlink-to-dir selector (glob arm fixed by `SL-234`; literal arm open) |
| `ISS-277` | `reseat` cannot renumber a review |
| `ISS-279` | Id reservation has local reach |
| `ISS-280` | contest records no durable rationale |
| `ISS-310` | `sections_attested` ignores reviewer identity |
| `ISS-314` | `derived_status` Done on empty ledger (D-C8) |
| `ISS-322` | run cannot name an externally conducted RV |
| `ISS-359` | reviewing runbook clears on a cited governance set |
| `ISS-366` | empty ledger derives done, not active |
| `ISS-452` | opening a review pass emits no change row |
| `ISS-462` | `ReviewPolicy` labels exceed 16 B bound |

Closed/done (for reference): `IMP-001`, `IMP-008`, `IMP-023`, `IMP-042`,
`IMP-059`, `IMP-098`, `IMP-107`, `IMP-113`, `IMP-114`, `IMP-128`, `IMP-467`,
`ISS-033`, `ISS-207`, `ISS-212`, `ISS-225`, `ISS-233`, `ISS-259`, `ISS-275`,
`ISS-285`, `ISS-287`, `ISS-293`, `ISS-294`, `ISS-315`, `ISS-329`, `ISS-476`.
Wont-do: `IDE-002`. Resolved/fixed: `IMP-066`, `IMP-272`.

Still open and adjacent (not tabulated above): `IMP-303` (bind admitted
`close_target` OID to its audit RV at the close gate).

---

[RFC-032 research]: `edge` `799b2f49b`
