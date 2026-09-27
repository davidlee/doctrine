# RFC-034 supporting research — stewarding human understanding and agency

**Role.** Supporting research for `RFC-034` (*Steward Human Understanding and
Agency*). The RFC owns the deliberation; this file owns context, evidence, and
the map of surfaces a solution would touch. It asserts no canon, and **proposes
nothing** — proposals belong in `rfc-034.md`.
**Compiled:** 2026-09-27, on `edge` at HEAD `f02e62570`.
**Method.** Single-pass direct research by the orchestrating agent: read the
governing specs/ADRs/RFCs, the shipped skills and prompt assets, the CLI help,
and the `src/` module surfaces named below; swept the observation, memory,
backlog and RFC corpora by `grep` and `doctrine search`. No `raw/` research
threads were spawned (`./scripts/pi-scout`, `./scripts/pi-research`); the round
was run in-session instead, which the `/research` skill permits as graceful
degradation. Threads a refresh should run are listed in Appendix A.
**Verification legend.** ✓ = independently verified at authoring time by running
the command or reading the cited site. Unmarked = claim taken from the cited
artefact, not re-checked. A ✓ on a code claim means the *symbol* was read, never
that a line number was — per `IMP-344`, code is cited by function or type name,
not by line.
**Scale of the corpora at this baseline (✓):** 533 slices, 734 review ledgers
(`RV-`), 128 revisions (`REV-`), 429 knowledge records, 935 backlog items, 34
RFCs, 48 ADRs, 646 local memories, 31 shipped memories, 670 observations
(666 friction + 4 supersession), 35 tracked skill masters.

---

## 0. One-paragraph summary

RFC-034 asks whether doctrine should treat the human's *retained understanding*
as an outcome it stewards, alongside intent and execution. The context that a
solution would need spans five bodies of fact. **(1) The stimulus is external** —
a blog essay drawing the *accelerator / vibecoder* distinction, and a position
paper arguing that oversight affordances and oversight *capacity* degrade each
other. Neither is doctrine evidence; both are the tier-6 material the RFC's
Context is built from. **(2) Doctrine already claims comprehension as a product
priority** (`README.md` priority 2) but implements it as a *recording* property
(durable, queryable, cited knowledge) rather than a *capacity* property; no
requirement, spec or skill obliges anyone to confirm a human understood
anything. **(3) The machinery and the seams largely exist.** `walkthrough` and
`pair` are deliberate, portable, dialled conduct skills; `elicit` already
implements "ask the human before revealing the model"; the design run already
distinguishes *obligations* (gated acts) from *lenses* (continuous craft) and
already records user acts with a content-bound `AcceptanceAttestation`; the
hymns cascade already carries lifecycle-stage prompt text. **(4) The gaps are
specific and nameable.** No skill in the design → audit → reconcile → close path
mentions understanding at all; the RV ledger's read-back surface is mid-rebuild
(RFC-032); `design.md` and reconciliation documents are presented as artefact
deltas rather than read together with a human; the only *human*-verified mode
(`VH-`) has no derived gate, and the one negative product verdict this pass
found recorded is `SL-246`. **(5) The evidence base for a change does not exist yet.** Of 666
friction observations, **zero** mention understanding, comprehension,
walkthrough, explanation of the design, cognitive load, or abbreviation; the
theme arrived through reading and reflection, not captured tool friction, so a
programme will have to author its own measurement (VH/observation) rather than
mine the ledger.

---

## 1. Originating material (external and local)

### 1.1 `understanding.local.md` — the seed conversation

`.doctrine/rfc/034/` carries no seed copy; the RFC's originating artefact is
`understanding.local.md` at repo root (✓ present; gitignored by `.gitignore`
`*.local.*`; ~181 lines; **tier 6/tier 5 material** — a conversation, not an
entity).

Its content, as recorded:

- **The tension.** Delegation can be locally rational while eroding the capacity
  to evaluate whether further delegation is rational; accumulated sensible
  choices produce someone who no longer knows when to disagree.
- **Understanding can be selective; the selection cannot be delegated.** The
  capacities claimed to matter: deciding acceptable outcomes and trade-offs;
  recognising inadequate evidence; challenging the framing and what was omitted;
  redirecting or stopping with realistic consequences.
- **The framing hazard.** If the agent defines the problem, selects the
  alternatives, chooses the evidence and explains its own success, the human's
  apparent authority rests inside the agent's framing. A reviewing *agent* does
  not give the human an independent foothold.
- **The user's own diagnosis**, quoted in substance: current frontier agents are
  *convincing* at identifying critical questions, offering options,
  recommending, and critiquing; the risk is "authority without expertise" —
  managerial oversight of an agent with better detail awareness, code
  familiarity, and command of the governing documentation than the user.
  Compounded when the documents "aren't read directly, and referred to only via
  an abbreviation salad". Reported self-observation: *"nine times out of ten, I
  find myself hitting 'accept' without really understanding or reading the docs,
  just because of the way they're presented."*
- **The named candidate mechanisms** (not settled): an *intended-understanding
  dial* — a project default with local override, expressed as behavioural
  settings rather than a scale; forcing the human to volunteer an answer *before*
  seeing the agent's; separating **product inquiry** (intent, user impact,
  high-level behaviour) from **technical inquiry**, with independent
  understanding/authority calibrations; and treating **audit and reconciliation**
  as the strongest near-term opportunity, because understanding there attaches
  to concrete things (the code, the tests, the governing text).
- **A candidate setting ladder:** *Delegate · Orient · Understand · Practise*,
  with per-setting obligations stated as presentation and participation
  commitments, and never as a certification of a mental state.
- **A candidate success criterion:** after accepting a consequential change, the
  human can say what they agreed to, and why.

The RFC's `## Context` (empty at this baseline) is where this becomes an
argument; this research file treats it as source, not authority.

### 1.2 zanlib.dev — "Do You Still Read the Code?" (2026-09-14)

✓ fetched. URL: `https://zanlib.dev/blog/do-you-still-read-the-code/`.

The essay's load-bearing distinctions, as published:

- **Accelerator vs vibecoder.** An *accelerator* uses AI to translate their
  understanding into code and intends to retain enough understanding to explain
  the reasoning, anticipate consequences of change, and maintain the model and
  its implementation. A *vibecoder* delegates implementation and its continued
  revision, moving attention to specifying behaviour, supplying context, and
  establishing satisfaction checks. "The distinction concerns the developer's
  relationship to the output, rather than how much of it the model writes."
- **Cognitive debt vs intent debt.** Cognitive debt (attributed to Storey,
  *From Technical Debt to Cognitive and Intent Debt*, arXiv 2603.22106) piles up
  "whenever the team's reading falls behind the generating". Intent debt is the
  reader's version of the problem, and the passage most likely to be quoted in
  the RFC:

  > "…when a reader comes later and sees a value of `expiryTime = 6h`, he can
  > see what the software does, but the six hours could have come from several
  > places: an explicit business requirement, an existing convention, a
  > considered trade-off, or a guess that no one challenged. To decide whether
  > the value should change, the reader needs to know what justified it and
  > whether those circumstances still hold. […] This is intent debt, and reading
  > every line of the code doesn't pay it off."
- **The proxy claim.** "Code quality was a useful proxy for that understanding in
  the time before Claude Code, but a language model can now feign that
  understanding convincingly." Reading is therefore necessary but not sufficient.
- **The unbudgeted expectation.** The essay's closing concern is a *maintenance
  contract*: before merging, colleagues need to know how the change is meant to
  be maintained — through a developer's understanding, through specification and
  regeneration, or a combination, "as long as it's clear which parts are which".
  "Ceasing to read code is not, in itself, progress. But before asking whether
  your colleague *still* reads code, perhaps consider whether you're *still*
  expecting him to maintain yours."
- Tooling named inline: `crit.md` (implementation-vs-intent review), a diff
  review assistant. Horthy, *No Vibes Allowed* (YouTube 2025) is cited for the
  "dumb zone". Bainbridge, *Ironies of Automation* (*Automatica*, 1983) is cited
  for manual practice as a necessary skill-maintenance cost.

Directly relevant to doctrine: the essay's unit of concern is the **maintenance
contract between colleagues**, which is a governance-shaped question doctrine
already has kinds for (ADR, DEC, CON, REQ) — but no surface that states *which
parts of this codebase are meant to be understood by whom*.

### 1.3 Ghosh & Passi — *AI Agents Push Humans Out of the Loop* (arXiv 2608.23642v3)

✓ fetched. Position paper. Authors: Avijit Ghosh (Hugging Face), Samir Passi
(Data & Society). CC-style arXiv HTML.

Thesis and structure, as published:

- **Abstract claim:** current AI-agent development and deployment "do not support
  effective human oversight — they contribute to its degradation"; a top priority
  should be supporting the situated goals and cognitive requirements of
  oversight, "treating the human needs of overseers at the same level of
  importance as AI agent capability".
- **Presence is not oversight.** "the presence of an overseer does not entail
  reliable oversight"; the burden currently falls almost entirely on users,
  while "the mechanisms users would need to do this effectively are virtually
  absent".
- **The oversight load is a working-memory problem.** As the user is relegated to
  "approver", oversight requires "assessing each decision point as a task expert,
  anticipating and guarding against unintended outcomes, tracking the agent's
  multi-step plan, and maintaining a mental model of its goals, current execution
  state, and prior actions — a working memory and situational awareness load that
  current interfaces are not designed to support".
- **Governance assumes the capacity it erodes.** EU AI Act oversight
  recommendations are "applicable only if the human overseer maintains reliable
  cognition, attention, and skill; missing that the system itself might be eroding
  those very capacities."
- **§4 *The irony of automation*.** Documents: deskilling / skill atrophy,
  "intuition rust", decreased critical and analytical thinking, reduced vigilance
  and pattern recognition, overreliance, cognitive dependence; **automation
  bias** (accepting suggestions when wrong), **anchoring bias** (agreeing more
  when an AI position precedes one's own), **complacency bias**; expert
  overreliance, "particularly pronounced in contexts slightly outside of their
  expertise". Also §4.3: users are beginning to *avoid* engaging, and §4.4: the
  feedback loop is alignment-relevant because models train on the resulting data.
- **Explanations are not the fix.** Against the "better explanations / reasoning
  traces / richer audit logs" objection: explanations "operate on cognitive
  capacities that AI agent use itself degrades and can increase inappropriate
  trust… explanations can themselves be incorrect or act as cognitive anchors."
- **§5 Solutions — two prongs, one inventory.** Design-level affordances
  (development) and organizational protocols (deployment). Appendix A's inventory,
  verbatim grouping:

  | group | interventions |
  |---|---|
  | Strategic friction | pre-commitment mechanisms; delay-and-choice mechanisms; reasoning probes; action gating |
  | Decision design | bounded autonomy; batch review; automated pre-checks; supportive interfaces |
  | Behavioral monitoring | time-based signatures; override signatures; evidence-seeking signatures; guardian agents; canaries; style-fatigue covariance; audits |
  | Workload & scheduling | enforced breaks; rotations |
  | Trainings & exercises | domain skill maintenance; critical evaluation training; self-monitoring training |
  | Role design | assigning roles; separating roles; aligning incentives |

  Definitions given for the four strategic-friction items: **pre-commitment** =
  "the user record[s] their own view or decision before seeing the agent's
  recommendation… prevents anchoring bias and creates an audit trail";
  **delay-and-choice** = the user decides whether/when to see output at all;
  **reasoning probes** = "inline prompts at high-stakes moments, such as 'what
  evidence would change your mind?' or 'what assumption does this approval rest
  on?'"; **action gating** = "explicit verification before the agent proceeds
  down a consequential path, and may surface alternative options for the user to
  choose between rather than presenting a single recommendation to accept or
  reject". **Batch review** = "the agent completes a logical unit of work and then
  surfaces the whole batch as a diff".

The paper is a *position* paper (its own framing), not a controlled study; its
empirical anchors are cited second-hand. The two interventions doctrine most
resembles already are **bounded autonomy** (ADR-020 capsules, the authority
envelope) and **batch review** (the phase/audit boundary). The two it most
clearly lacks are **pre-commitment** and **reasoning probes**.

### 1.4 Secondary sources cited by the above (unread; for citation completeness)

Storey, *From Technical Debt to Cognitive and Intent Debt* (arXiv 2603.22106);
Horthy, *No Vibes Allowed* (YouTube 2025); Bainbridge, *Ironies of Automation*
(*Automatica* 1983); Willison, *Not all AI-assisted programming is vibe coding*;
EU AI Act human-oversight provisions. Note that the seed conversation's framing
("the smarter agents become…") and the paper's are not identical — the seed is
about *authority without expertise*; the paper is about *capacity degradation*.
A solution that serves one need not serve the other, which is itself a fact the
RFC's Discussion must confront.

---

## 2. What doctrine already claims about the human's understanding

This is the narrow set of places where doctrine states a position on human
comprehension. It matters because it fixes what a change would be *amending*,
not merely adding to.

| artefact | what it says | tier |
|---|---|---|
| `README.md` priorities | **Priority 2 of 5 is "Comprehension"**: "You should be able to understand what your agents built and why, months later. Intent, decisions, and evidence are recorded as they happen, cited by durable ids, and linked into one graph you can query." | tier 2 prose (product positioning) |
| `PRD-004` (memory) | The problem statement is rediscovery cost and stale understanding; the product answer is durable memory whose value is that "understanding compounds instead of evaporating", and that a "future reader can judge whether it still holds rather than taking it on faith". | tier 2 |
| `PRD-003` (Skills) | Skills exist so the agent "actually carries doctrine's conventions instead of improvising". The marketplace channel "must remain consumable with no doctrine binary present". | tier 2 |
| `ADR-005` | Shipped knowledge is tiered by access pattern — "skills route, reference docs explain". Deliberately nominates no single source. | tier 2, accepted |
| `ADR-023` + `reference/authority-model.md` | The binding rules for *acting for the user*: lay out the proposition, ask for a plain reply, record the act yourself; "Never ask the user to run the CLI or author payload fields"; "Never record an act they did not give"; "Doctrine does not authenticate humans in this model". Explicitly **out of scope: verifying the human**. | tier 2, accepted |
| `ADR-024` | Shipped-corpus grounding: cite any address a client can resolve, never a repo-private id or path. (Constrains how a shipped understanding policy may cite.) | tier 2, accepted |
| `ADR-009 §2` | The **conduct axis** (axis B): `actor × autonomy` declared per lifecycle state, configured in `doctrine.toml [conduct]`, resolved by `src/conduct.rs::resolve`. Its own module docs state it is **"Advisory, never enforced (F15)"** and that "autonomy is exit semantics (F19)". | tier 2, accepted |

Two things this table shows. First, comprehension is already a *stated product
priority*, but every mechanism it names is about **recording** (durable,
queryable, cited, linked) — none is about the human's **capacity**. Second,
doctrine already has (a) a declarative, project-level, per-stage posture
mechanism (`[conduct]`) that is deliberately advisory, and (b) a written
authority model for user acts. Both are candidates for extension; both carry
explicit design constraints (advisory-only; invoker-blind; no human
authentication).

### 2.1 Adjacent doctrine positions that bear on the theme

| artefact | relevance |
|---|---|
| `RFC-017` (*Human onboarding docs*, open) | Frames doctrine's UX as **agent-mediated**: humans need only *why*, *the mental model*, and *a path to first success*. RFC-033 records that RFC-017 is "the human half of this RFC and is absorbed by it". |
| `RFC-033` (*Learning surface*, open) | One normative home per rule; two audiences honestly served; the human set is "small and finite"; the boot snapshot should carry pointers not restatements. Its Outcome records the frame settled on 2026-09-26. |
| `RFC-009` (*Epistemic records as the human-facing relational substrate*, open) | The sharpest existing statement of the thesis: "epistemic records are the human-facing, lifecycle-bearing, deeply relational data model for design ambiguity which dominates human work once agents write and review code." The memory↔record boundary test ("real graph edge, human eyes, a lifecycle, or something to argue with → record"). |
| `RFC-026` (*Design review response effectiveness*, open) | Carries the human-comprehension problem statements already: **G2** "acronym obsession leads to human-incomprehensible coordinate-speak between agents"; **G3** design artefacts "absorb so much history that the underlying design is no longer apparent"; **G12** `design.md` performs four roles under one review scope; **G15** "intent and implementation approach are one undifferentiated artefact"; **S3** epistemic warrant missing from load-bearing claims; **S8** three decision-authority tiers. Its **P10** (route each severe finding to the instrument that settles it) shipped via `SL-260`. |
| `RFC-022` (*Agent trust model without human attestation*, open) | The threat model and the independence axes (different model > no shared context > different session). Bounds what an agent reviewer can supply. |
| `RFC-027` (*Progressive discovery and proof-bearing plans*, open) | **H12**: three decision-authority tiers — "worker discretion → delegated LLM semantic arbitration → reserved human authority" — and the claim that the two-tier model is the wrong premise. Directly relevant to "who is the human's independent foothold". |
| `RFC-028` (*Verifiable Human Authorization*, draft) | The one artefact that takes "how does doctrine believe a human authorised this" seriously: typed authorizations with claim/basis/validity/attestation; privileged-filesystem attestation first (a file immutable in the jail must have been written from outside it). Its Outcome section is still empty. |
| `RFC-030` (*Inquiry map*, open) | **T4** deliberately parks the authority model, and records the key analysis: `AcceptanceAttestation` has teeth against *content* laundering (its digest binds the checkpoint payload fingerprint, the disposition, and the run revision) but **nothing binds presence** — "The agent authors `basis`, `provenance: user-directed`, and `authority: user-pinned`." It also names the codebase's own pattern for the fix (`DischargeClaim` omitting `verified` from the wire so the category error is unspellable). |
| `RFC-031` (*Design run fitness*, open) | The programme for the design run's warts (T1–T6), including T3 contract legibility and T4 "map surfaced, then made live". Relevant because any design-time understanding obligation adds ceremony to a run already judged "fit for use, with warts". |
| `RFC-032` (*Review ledger effectiveness*, open; slice 1 landed) | The programme for the RV read surface and durable journal; see §5.7. |
| `RFC-021` (*Dynamic behaviours and minimal projection*, **resolved**) | Settled (its design interview) that lifecycle skills may require the binary — "the installed skill should be a small, MIT-licensed activation contract" that invokes it — while **"genuinely generic skills such as pair or walkthrough remain self-contained"**. This is the governance constraint on where an understanding policy may be delivered. |
| `RFC-010` (*Skill improvement sweep*, **resolved**) | Judged all 30 skills then shipped against four lenses in priority order: **adherence**, token efficiency, outcome quality, coherence. Its conclusion — "the set is healthy; no skill required a redesign" — is the baseline against which a new obligation on the skill set should be argued. |

---

## 3. Skill surface area

### 3.1 Source of truth and the projection drift

- The **canonical skill masters are tracked in `plugins/doctrine/skills/<name>/SKILL.md`** ✓ (35 dirs at this baseline). The embed root is `plugins/` (`src/install.rs`, `#[folder = "plugins/"]`).
- `.doctrine/skills/` and `.pi/skills/` are **gitignored install projections** ✓ (`.gitignore` `/.pi/*`, `.doctrine/skills/*`).
- The projection does **not** prune: `.pi/skills` holds 39 dirs and `.doctrine/skills` 37, against 35 tracked masters ✓. The extra names are `dispatch-agent`, `dispatch-subprocess`, `next`, `notes` — `next` and `notes` were **consolidated into `/handover` and `/harvest`** by commit `9cf3e2bea` ("consolidate capture skills — /notes → /harvest, /next merged into /handover") and live on as orphans in the projections. This is `ISS-353` ("doctrine install never prunes orphaned skills") and the cause of the recurring "skill mirror lag read as a failed install" friction reported in the observation ledger.
- `walkthrough` and `pair` are **also mounted by the `doctrine-partner` plugin via tracked symlinks** into the canonical tree ✓ (`plugins/doctrine-partner/skills/{pair,walkthrough} -> ../../doctrine/skills/…`);
  `record-memory` and `retrieve-memory` likewise via `doctrine-memory`.
- Non-Claude harnesses are served by the **published** source, not the working tree (reported in the observation ledger: "doctrine install projects skills from the GitHub source, not the local tree"; "Local plugins/ skill edits cannot reach .agents/ … npx-fetched from the published remote") — so a locally edited master may not reach `.pi/skills`. **Caveat:** observation summaries in this corpus carry backlog-id labels that do not always match the current title of that id (three checked, all mismatched), so an id lifted from an observation summary must be re-resolved through `doctrine show` before it is trusted.

Practical consequence for RFC-034: **a skill change means editing `plugins/doctrine/skills/…`; the `.pi/skills` and `.doctrine/skills` copies are regenerated, and this session's own skill list is the projection, not the master.**

### 3.2 The 35 masters, grouped by role in the loop

Rows in **bold** are the surfaces the seed conversation names as candidates for an
understanding-aware posture.

| group | skills |
|---|---|
| Entry / routing | `route`, `preflight`, `canon`, `retrieve-memory`, `backlog`, `knowledge` |
| Work capture & governance authoring | `spec-product`, `spec-tech`, `spec-coverage-assessment`, `slice`, `research` |
| Lifecycle stages | **`design`**, `plan`, `phase-plan`, `execute`, **`audit`**, **`reconcile`**, `close` |
| Adversarial review | `code-review`, `inquisition`, `feedback`, `reviewing-memory` |
| **Conduct postures** | **`walkthrough`, `pair`, `rigour`**, `consult` |
| Instrumented elicitation | `elicit` |
| Orchestration | `dispatch`, `dispatch-spawn`, `capsule-driver`, `worktree` |
| Durable memory & handoff | `record-memory`, `harvest`, `handover`, `dreaming` |

### 3.3 `walkthrough` — the guided-comprehension skill

Read in full ✓. Frontmatter: "build the reader's mental model, explain the
choices made and their tradeoffs, and critically evaluate the artifact… Adapts
to expert vs learner; companion to the pair skill."

- **Dual identity.** When understanding *is* the task it is the governing
  activity; layered onto change work it is a **conduct posture** over whatever
  stage governs. "Layered use never replaces the governing stage or its process."
- **Dials.** `audience = expert | mixed (default) | learner`; `depth = skim |
  guided (default) | deep`.
- **The expertise-reversal warning**, verbatim in emphasis: "Detailed remedial
  explanation *helps* a novice and *hinders* an expert (the expertise-reversal
  effect)… Over-explaining to an expert is a failure, not thoroughness."
- **Loop:** orient (agree a budget and a stopping condition) → map → trace →
  explain the choices → challenge → **check understanding** → record/harvest.
- **Check understanding ("learner / mixed only"):** "use comprehension checks
  instead of just telling — they're how the model transfers, not a quiz:
  predict before reveal…; compare-and-contrast…; 'what breaks if this
  changes?'; ask them to name an invariant, or why an alternative was rejected.
  Never gratuitous… For an `expert` audience, skip checks entirely."
- **Always-on challenge**, calibrated; blocker vs preference.
- **Handoff:** to `pair` when a concrete change emerges; in a doctrine repo the
  handoff target for a discovered change is `/route`, "not free pair edits".
- **Doctrine-repo note:** read entities via `doctrine <kind> show <ID>`; treat
  `/canon` + memory as the authority on *why*; closure-grade findings → RV.

### 3.4 `pair` — the in-the-loop skill

Read in full ✓. Frontmatter: "calibrated friction… you stay a challenging
partner, never a passive code generator, at every autonomy level."

- **Dials.** `role = code-author (default) | navigator | switching-pair`;
  `detail = sketch | balanced (default) | deep`; `autonomy = ask-first | bounded
  (default) | weapons-free`.
- **The autonomy/detail separation**, verbatim: "Autonomy controls *execution
  authority*, not *intellectual deference*."
- **Setup:** state the frame once; **echo the active dials in one line** so the
  contract is visible, and re-echo whenever a dial changes. (The closest existing
  precedent to an "understanding dial" being made *visible*.)
- **Drift checks** include "Lost the partner — moving without shared
  understanding; re-sync the frame" and "Gone passive — accepting without
  challenge", with a **mechanical tripwire**: "three consecutive increments with
  nothing challenged and no assumption surfaced → say so".
- **Weapons-free stop-list** exempts destructive ops, credentials, production,
  billing, legal/compliance/security, scope expansion.
- **Handoff** to `walkthrough` when the need shifts from change to
  understanding; dials are preserved both ways.

### 3.5 `elicit` — the existing "human answers first" workflow

Read in full ✓. The nearest incumbent to the seed's "force the human to
volunteer before seeing the agent's input".

- A **queue/curator split**: `doctrine compare elicit` picks mathematically
  productive pairwise questions; the agent curates (filter, reframe, sequence,
  translate) and "never re-rank[s] by your own opinion of the items' value".
- Batch discipline: "Session size 5–10 questions; stop before fatigue degrades
  answers."
- **`incomparable` is a first-class answer**, always available, always yields
  zero, "a legitimate answer, not a failure"; "repeated incomparables are a
  signal the pairing or audience is wrong".
- Provenance is honest: a human's answer is `--rater human`, always; the agent's
  own is `--rater agent`.
- Footer semantics (`candidates` / `stalled` / `stable`) and the warning never
  to present `stable` as "priority settled".

### 3.6 Where "understanding" is *not* mentioned

✓ Verified by `grep -c -iE 'understand|comprehens|cognitive'` over
`plugins/doctrine/skills/*/SKILL.md`:

| skill | hits |
|---|---|
| `walkthrough` | 9 |
| `code-review` | 3 |
| `preflight` | 2 |
| `pair` | 2 |
| `canon` | 1 |
| **`design`, `audit`, `reconcile`, `close`, `execute`, `plan`, `phase-plan`, `slice`** | **0** |

So the two conduct skills own the concept, and **no skill on the
design → audit → reconcile → close path names it at all**. The seed's
observation that the strongest opportunity sits in that path is, at the level of
shipped prose, a description of an absence.

---

## 4. Design-run touchpoints

Governance: **`PRD-019`** (*Managed design workflow*, active, product level
capability) → **`SPEC-029`** (*Design run engine*, active, container). The
`/design` skill is explicitly **an adapter, not the workflow**: "The stages,
their order, and the obligations on each live in the design run… Every
`doctrine design` command prints the guidance for the turn you are on, and that
output is your instruction." A solution that changes design-time interaction
must therefore land in the run's **assets or gate tables**, not only in the
skill — the skill will not carry it.

### 4.1 The machine: stages, edges, conditions

From `reference/design-run-stages.md` ✓ (**generated** from the gate tables and
pinned by test — "what you read here is what refuses you"):

```
[*] → exploring → inquiring → drafting → reviewing → locked
       2 cond.     4 cond.     6 cond.      8 cond.      (own + inherited)
```

| condition | kind | binding (what invalidates it) | reach |
|---|---|---|---|
| `governing-context-recorded` | attested | artefact observes(governance-edges) | cumulative |
| `initial-concerns-recorded` | attested | reviewed-graph | cumulative |
| `blocking-inquiries-dispositioned` | derived | engine(dispositions) | cumulative |
| `user-accepts-sufficiency` | attested | inquiry-map | cumulative |
| `drafting-readiness-attested` | attested | artefact | edge-local |
| `materialisation-current` | derived | engine(materialisation) | cumulative |
| `section-attestations-current` | attested | per-section | cumulative |
| `review-disposition-attested` | attested | artefact | cumulative |
| `user-acceptance-attested` | attested | every-section | cumulative |

Every edge re-derives **all inherited conditions against current content**.
Going back is a distinct verb needing a *reason*, and "nothing is replayed on
the way back".

Design-run code homes (cite by symbol, per `IMP-344`): `src/design_run/gate.rs`
(`requirement_for`, `cumulative_conditions`, `advance`, `regress`,
`forward_unmet`, `unmet_line`); `src/design_run/runbook.rs` (`steps`,
`live_discharge`, `standing`, `attested`, `verify`);
`src/design_run/inquiry.rs` (`needs_in_degree`, `blocked`, `is_blocked`,
`effective_blocking`, `resolve`, `transition`);
`src/design_run/submission.rs` (`apply`, `inert_key`, `inert_at_state`,
`nulled_keys`); `src/design_run/traversal.rs` (`propose`).

### 4.2 The acts, and how a user act is recorded

The vocabulary is **closed at eight** (`src/design_run/attestation.rs::ActKind`,
whose own doc says "Closed at eight, in the order that table lists them"), in
order: `GovernanceConfirmed`, `GraphReviewed`, `BlockingSetDeclared`,
`SufficiencyAccepted`, `DraftingReady`, `SectionReviewed`, `ReviewDisposed`,
`DesignAccepted`.

- **`BlockingSetDeclared` is legacy, read-only.** ✓ `ActKind::is_legacy` names
  it; a submitted one is refused (`Refusal::RetiredAct`); it is absent from the
  wire contract (`reference/design-payload-contract.md`'s `enum ActKind` lists
  seven). So the **live** act set is seven; the legacy variant is readable on
  old snapshots and unrepresentable on the wire.
- **Agent-authored acts are a narrower, separate type** — `AgentActKind`
  (`BlockingSetDeclared`, `DraftingReady`), with a one-way widening into
  `ActKind`. The design notes the narrowing exists so "a requirement may name an
  agent declaration to be confirmed, and naming a *user* act there is a
  contradiction that should not be spellable."
- **A user act is carried by `AcceptanceDeclaration`** — `basis` (required) +
  `turn` (optional) — inside a `checkpoint_act` (`CheckpointActDeclaration {
  act, acceptance, disposition? }`) or beside a `declare` batch. Per the
  authority model, the **agent** authors `basis`, quoting the user's reply.
- **`AcceptanceAttestation`** (`src/design_run/attestation.rs`) binds the act to
  content: `is_current`, `bind`, `moved`, `moved_among_carried`, `with_payload`.
  RFC-030 T4 names exactly what it binds and what it cannot: the digest covers
  "the checkpoint payload fingerprint, the inquiry disposition, and the run
  revision", so an acceptance cannot be lifted onto different content, a later
  revision, or a different disposition — "What no digest over content can bind
  is **presence**: whether a human was there at all."
- **The run's review disposition** is `ReviewDisposition::{conducted{review},
  waived{reason}}`; `conducted` names the run's **own** pass RV
  (`review_pass RV-NNN`) and blockers still open or contested hold the edge.
- The `Authority` enum on a traversal/edit is `agent-proposed | user-pinned |
  user-locked` — a vocabulary that already distinguishes *agent proposed* from
  *user pinned*.

### 4.3 The runbook seam: obligations vs lenses

This is the single most load-bearing fact for RFC-034, and the cleanest place a
new design-time duty could attach. From `reference/design-run-obligations.md` ✓:

- A **lens** is guidance with no truthful completion point — "It improves every
  turn and can never be declared finished." No refusal; a run that ignores it
  merely reads worse.
- An **obligation** is "a discrete act whose completion *completes* it. It
  states its own completion condition, in its text, at authoring time."
- **The discriminator, decidable from text alone:** "If a truthful completion
  condition can be stated for the act, it is an obligation. If no such condition
  can be stated — only a direction of travel — it is a lens."
- **Why blocking is the point:** "That refusal is the entire difference between
  the two, and it is the reason a small set of acts is worth imposing on every
  run. […] The refusal — not the receipt — is what makes the obligation bind."
- Two named failure modes to avoid when classifying: "Counting files is not a
  criterion"; and "**A direction of travel in a step's costume.** *'Capture
  everything important'* reads like an obligation and behaves like a lens: it can
  be asserted without ever being true."

The **authoring rule** for a runbook step, from
`install/design-prompts/exploring.toml` ✓: "Could a project legitimately do this
differently? Yes → a runbook step. Overridable, verifier substitutable. No → an
engine invariant, enforced by `apply`/`advance`. Never a step." Also stated once
there and inherited by the other files: **step ids are API** (explicit,
author-assigned, "never positional"), and a discharge binds the **digest of the
whole step definition** (`id`, `text`, `required`, `verify`), so editing the
prose deliberately stales every discharge of it.

The step inventory at this baseline ✓:

| file | mode | steps |
|---|---|---|
| `exploring.toml` | sequence | `explore.scope`, `explore.research` (verified by `doctrine verify research-current`), `explore.canon`, `explore.memory`, `explore.triage` |
| `inquiring.toml` | sequence | `inquire.knowledge` ("Record, via /knowledge, what this inquiry settled that outlives the session…"), `inquire.scope` ("Update or explicitly confirm the slice scope against the decisions this inquiry accepted…") |
| `drafting.toml` | sequence | `draft.selectors` (one step, deliberately) |
| `reviewing.toml` | sequence | `review.scope`, `review.selectors`, `review.passes` |

Sibling `.md` fragments are the **lenses**, delivered every turn of their stage:
`inquiry.md` (the questioning loop and craft), `drafting.md` (eighteen heuristics
as framing, not a checklist), `reviewing.md` (the attack surfaces),
`delegation.md` (proposal-only delegation). `reference/design-run-obligations.md`
notes neither inquiring obligation "ships a verifier, and neither could: both are
semantic" — and treats that as a fact about *evidence*, not about classification:
"An obligation with no reachable verifier is still an obligation: it is
discharged by an agent asserting the act is done, and that assertion is the
honest evidence."

**Consequence.** A duty phrased as *"ensure the user understands the design"* is
a **lens** by the condition test ("can be asserted without ever being true").
A duty phrased with a completion condition the user performs is an **obligation**.
The existing obligations are already user-assent-shaped (`sufficiency-accepted`,
`design-accepted`, `section-reviewed`), and the `basis` field already captures a
one-line quotation of the reply — *but nothing in the model distinguishes a
reply that demonstrates understanding from one that does not*, and nothing
records a proposition the user volunteered before the agent's answer.

### 4.4 Delivery surfaces for the human

| surface | what it gives | notes |
|---|---|---|
| `doctrine design show <SLICE>` | the design document + knowledge block (`--knowledge skip|facets|full`, default `skip`) ✓ | the read surface for the design *text* |
| `doctrine design show <SLICE> --format prompt\|json\|status\|tree` | the turn envelope: `forward` rows (runbook steps first, then unmet conditions), the `ready` payload to cross, `--full` to widen ✓ | the agent's instruction channel |
| `doctrine design tree` | the whole inquiry map as a tree, **for a human** ✓ | the only explicitly human-facing design-run surface; `inquiry.md` insists the agent paste this output, never a listing of its own |
| `doctrine design resume` | the compact projection a fresh context needs ✓ | re-entry seam; "Plain resume never infers missing procedural history" |
| `doctrine design materialise` | renders runtime sections into authored prose ✓ | edit loop is RFC-031's T2 (most-reported friction) |
| `doctrine design contract --format prompt` | the payload contract ✓ | |

Related frictions on these reads, **with current status**, because a design that
cites RFC-031's track history needs to know which have since closed:
`ISS-299` (map never reaches the user) — **resolved/fixed**;
`IMP-390` (three contract-legibility candidates left by `SL-251`) —
**resolved/fixed**; `IMP-470` (runbook names no growth obligation for the map) —
**resolved/done**; still **open**: `ISS-300` (a deferral records no reason),
`IMP-412` (locked run has no handover exit), `IMP-471` (dynamic mode ungoverned
at requirement tier, gated on `QUE-218`), `ISS-488` (a re-word emits no change
row), `CHR-067` (design skill's Recovery block omits the positional SLICE),
`IMP-375` (no project extension interface for design-prompt assets), `IMP-373`
(`set` mode's render unsketched).

### 4.5 The inquiry map specifically (RFC-030 territory)

`src/design_run/inquiry.rs` carries `InquiryNode { id, question, provenance,
lifecycle, disposition, parent, needs, seq }` — with **no field for reasoning**
(RFC-030 T1). `body` is refused at `inq-` (homed at `IdKind::Section`). The cheap
forms (`unresolved { note }`, `non-durable { note }`) mark the node `Resolved`,
which drops it off the frontier. Reversal was never designed: `transition` clears
`disposition` but withdraws nothing. **The map is runtime-tier and disposable**
(`.doctrine/state/design/` is gitignored; `DEC-059`).

For RFC-034 this matters because the map is the one design artifact whose
*purpose* is to expose the question space to a human, and RFC-030's own reading
is that the exit from the map is a single governance gear (`DEC-086` minting)
with no cheap provisional tier. It also carries the sharpest statement of the
presence problem (§2.1, T4) and the `DischargeClaim` analogy for making a
category error unspellable.

### 4.6 What the design run does not have

Facts, from the reads above:

- No act, condition, obligation or lens names **understanding**, **comprehension**,
  or the human's **model**.
- `user-acceptance-attested` binds the payload fingerprint, the disposition, the
  node and the revision — **not presence**, and not any human action beyond assent.
- No design-run surface presents the **governing text in place** at a decision
  point; the user meets the design as `design.md` prose or as identifiers, and
  the *governing* text (ADR/REQ/DEC) is reachable only by asking for it.
- The run's own pass RV is invisible on the CLI by default (`review show` hides
  the finding tier; `IMP-475`), so a human asked to accept a design review sees a
  count, not findings — the exact presentation the seed conversation complains
  about.

---

## 5. Audit → reconcile → close touchpoints

The seam is `audit (discovery) → reconcile (write) → close (confirm)`. Governing
skills: `audit`, `reconcile`, `close`. Governing kinds: the RV ledger (ADR-007;
now SPEC-032), the REV change axis (ADR-013), the REC reconciliation act
(SPEC-002). Reference protocol: `reference/review-ledger.md`.

### 5.1 The RV ledger mechanics (what a solution may ride, and may not duplicate)

From `reference/review-ledger.md` ✓ and `doctrine review --help` ✓:

| act | role | finding moves | required prose |
|---|---|---|---|
| `raise` | raiser | new → `open` | `--title`, `--detail` |
| `dispose` | responder | `open`/`contested` → `answered` | `--response` |
| `amend` | responder | `answered` → `answered` | `--response`, `--note` |
| `verify` | raiser | `answered` → `verified` (terminal) | — |
| `contest` | raiser | `answered` → `contested` | `--note` |
| `reopen` | raiser | `verified` → `contested` | `--note` |
| `withdraw` | raiser | `open`/`answered` → `withdrawn` (terminal) | — |
| `conclude` | raiser | the pass, not a finding | `--basis` |

Key invariants: **roles belong to acts, not agents** (`--as` is "cooperative role
assertion, not a security boundary"); every `--note`/`--response`/`--basis` is
durable; the **target ladder** prefers a proximate typed subject (slice/phase >
backlog item > mint one > prose last resort), and each consuming skill pins
which rungs apply; `prime` degrades ("primed nothing: <reason>") rather than
failing, and its staleness signal is "an optimization signal, never a gate".
At this baseline `amend` and `reopen` **exist** (ledger v2, §5.7): the two items
that described their absence — `ISS-485` (*No review amend or reopen*) and
`ISS-280` (*review contest records no durable rationale*) — each carry
`fulfilled by: SL-268` and remain **open** (closure hygiene, not an
implementation gap that still bites).

The **disposition vocabulary** and the separate **`--route`** field are the
ledger's mechanism for "what settles this"; `/audit`'s permitted dispositions are
`aligned`, `fix-now`, `tolerated`, and `verified` + a reconciliation-brief link,
and the skill forbids `design-wrong`/`follow-up` for governance items (they
belong to reconcile). RFC-032's C14 and `SL-260`/RFC-026 `P10` govern the route
vocab.

### 5.2 `/audit` — the audit lens

Read in full ✓. Subject is always the slice (target-ladder rung 1); facet is
`reconciliation`; modes are **conformance** (usual) and **discovery**;
self-audit drives both roles with `--as`.

- Evidence step includes `doctrine slice conformance <id>`, which reports the
  mechanical delta between `design-target` selectors and recorded source-deltas
  in three cells — **undeclared** (highest signal), **undelivered**, **conformant**
  — and is "**necessary, not sufficient**: it says *where to look*, never
  *whether it passes*".
- Findings are dispositioned on the ledger, then the closure story is written as
  the review's `## Synthesis`, and the handoff as a separate
  `## Reconciliation Brief` split into **Per-slice (direct edit)** and
  **Governance/spec (REV)**. Brief guardrails: plan criteria are off-surface
  (immutable-append); conformance findings name the **selector registry** verb,
  not prose alone.
- Audit tail: harvest disposable phase-sheet findings into `notes.md`, then
  `doctrine slice status <id> reconcile`.

**What an audit currently presents to the human:** a ledger of findings with
severities and dispositions, a synthesis, and a brief. What it does **not**
present: a reading route through the implementation, a pairing review of the test
strategy, or the consequential *non-defects* ("this works, but differently from
how you probably imagine"). Nothing in the skill requires the human to look at
the code at all.

### 5.3 `/reconcile` — the sole explicit writer

Read in full ✓.

- Two write surfaces: **per-slice artefacts** (`design.md`, `slice-NNN.md`) by
  direct edit with user agreement; **governance/spec truth** by **REV**.
- **No CLI verb surface**: `doctrine slice reconcile` "is not built yet
  (deferred)"; the pass is manual discipline over `doctrine revision *` and file
  edits.
- **Inspect, don't re-audit**: new gaps go back to `/audit` or `/consult`.
- Per-slice edits are **presented to the user before writing** ("Show the exact
  location, the old text, and the new text. Get confirmation before writing.").
  This is the one place in the loop with an explicit show-then-confirm shape.
- REV authoring detail: `revision new` → `revision status <N> started` →
  `revision change add …` per item → narrative into `revision-NNN.md` →
  `revision approve` → `revision apply` → manual landing of surfaced rows →
  `revision status done` → append `## Reconciliation Outcome` to the RV.
- **Split rule**: a row needing separate debate gets its own REV, so a stuck row
  cannot block close via an omnibus.
- **Escalation gate**: if the model itself is inadequate, `slice status <id>
  design` (the `reconcile → design` back-edge).
- Collision guard, no-op gate, and the `## Reconciliation Outcome` template are
  all specified.

The presentation the seed conversation complains about is:
`> Updated ADR-042 and SPEC-017 to reflect retry semantics. Accept?` — a patch
list plus an acceptance request. The skill's own shape (surface name → target →
intent) is the material a richer presentation would build on.

### 5.4 `/close` — confirmation, and the only human-visible *gate* text

Read in full ✓. Steps: pre-check (`doctrine slice list` rollup: `X/X complete`,
no `!N` blocked, no `?N` anomalous, no `—` untracked); **spec-coherence gate**
(every brief item resolved via *REV done | withdrawn | tolerated | escalated to
design*, plus the per-slice edits recorded and the RV resolved); commit cleanly
(stage the slug symlink); dispatched-slice integrate with the two tree-true
checks (`git diff --quiet HEAD` phantom detector; journal trunk row vs
`doctrine dispatch deliver-to`); the split-lineage `--record-integration` path;
`doctrine slice status <id> done`; close the originating backlog item.

The **drift discharge recipe** is the closest existing *user-act* form outside
the design run: `doctrine slice status done` refuses with *undischarged residual
drift*, and each flagged requirement needs an `accept` **REC** owned by the
closing slice satisfying three clauses (a: `move = accept`; b: a
`[[status_delta]]` naming the requirement at its current authored status; c: the
REC's `[[evidence_ref]]` ⊇ every distinct coverage key feeding the requirement's
composite). Full predicate:
`mem.pattern.doctrine.close-drift-discharge-rec`.

Note `close`'s own words on the empty ledger: "the absence of unresolved blockers
is what gates the transition, never the status string (a display summary, never a
gate)."

### 5.5 The human-verification mode (`VH-`) and the one recorded negative product verdict

`reference/glossary.md` ✓ defines the three criterion modes: `VT-` verification
by **test** (automated), `VA-` by **agent** check, `VH-` by **human**
acceptance — "pick by *who/what* confirms the criterion."

Facts about the implementation: `doctrine coverage verify` re-derives **VT**
coverage only (help text: "Re-derive `VT` coverage status by re-running each
entry's check"); `src/vtgate.rs` judges every **VT-mode** criterion against its
structured mandate (`test_file`/`keywords`/`patterns`) and its threat model is
explicitly "worker **omission**, not an adversary… Semantic correctness of the
assertion is a non-goal." So **`VA` and `VH` have no derived gate** — they are
authored and attested, not checked.

The one product-level `VH` outcome this pass found recorded at length in the
corpus is from `SL-246`, the composed-knowledge read, recorded verbatim at
`RV-372` `F-5` and carried into two open backlog items. (`VH-` criteria are
common — some 276 rows across the corpus's `plan.toml` files — but no derived
gate reads them, so an outcome lives in a phase sheet or an RV rather than a
queryable verdict.)

- `IMP-465` (*Composed knowledge read: the product pass VH-1 asked for*) quotes
  the verification: *"it seems like a scrappy prototype of what I actually want
  to see. Styling is incongruent with the document; the omitted facets defy
  understanding … would I reach for it again? the CLI flags are cumbersome and
  TBH I expected knowledge would be visible by default… The verdict is:
  **disappointing as a feature**, but not obviously [broken]."*
- `ISS-467` (*Facets withholds the Argument tier with no disclosure*) traces the
  mechanism: `DEC-150` splits a knowledge record's facet into a **Deciding** tier
  and an **Argument** tier; `--knowledge facets` renders the first and **silently**
  drops the second.

Two facts follow for RFC-034. First, doctrine has a *mode* for "a human judges
the product, not the correctness" and has used it once on exactly the surface
this theme cares about. Second, that usage cost nothing structurally: the
negative verdict was recorded and filed, and the slice's own gate had already
passed.

### 5.6 Attestation roster and review policy

`ReviewPolicy` (wire) is `human-only | adversarial-only | human-then-adversarial |
adversarial-then-human`; `Reviewer` (stored) is `Human | Adversarial`, "the v1
default" being human. `section-attestations-current` requires "every lane the
run's review policy requires" to perform `section-reviewed` — the human lane's is
recorded on the user's assent. A friction observation records that the two
chained policies are currently **undeclarable** (a 22 B label against a 16 B
admission bound). `drafting.md`/`reviewing.md` also record that configurable
reviewer postures are **deferred**, with the instruction not to invent one.

### 5.7 RFC-032 and the ledger's current state

RFC-032 is the programme for the RV ledger, and it is **mid-flight**:

- **Landed:** slice 1 (ledger v2) via `SL-268`, governed by `REV-064` (done,
  approved), which amended ADR-007 and activated the new tech spec **SPEC-032**
  (*Review ledger*, active). ✓ The v2 write side adds the per-finding turn
  journal, `amend`, `reopen`, closed dispositions with a separate `route`, a
  `done` needing a concluded pass, fail-safe reads, and a `prime` that degrades.
  `ISS-280` and `ISS-485` are marked `fulfilled by: SL-268` and remain open
  pending closure.
- **Pending (open):** the read surface (`IMP-475` show `--findings`, `list
  --target`), `IMP-029` verb-family e2e golden, `IMP-392` design-run↔RV
  unification, and `CHR-079` refresh of `review-ledger.md` to the v2 verb surface
  (the shipped doc already claims note durability, which is the v2 turn journal's
  behaviour; the refresh trues the doc up to the code).
- RFC-032's own findings that bear on RFC-034: the read-back surface "was never
  designed" (reported nine times across four slices); the run's own pass RV is
  invisible; an externally conducted RV cannot be named; ADR-007 is the *only*
  artefact owning the kind.

---

## 6. Entity and kind facts a designer will need

Prefixes and reference forms: `reference/glossary.md` (entity ids prefixed and
zero-padded; doc-local ids bare and meaningless outside their artefact — `OQ-1`,
`D1`, `R1`; phase ids `PHASE-NN` immutable-append; criteria `EN-/EX-/VT-`
immutable-append, with `VA-`/`VH-` as the other two modes).

| kind | prefix | what it is | lifecycle / notes |
|---|---|---|---|
| slice | `SL-NNN` | unit of intentional change | `new → design → plan → ready → … → audit → reconcile → done` (or `abandoned`); `slice status` |
| RFC | `RFC-NNN` | **governance-neutral deliberation** (ADR-014); asserts no canon | `open → resolved` (outcome-blind status) |
| revision | `REV-NNN` | the **change axis** for governance/spec truth (ADR-013) | `proposed → started → done`; `abandoned` from any non-terminal; `approval` orthogonal |
| reconciliation record | `REC-NNN` | the immutable record of **one reconciliation act** (SPEC-002) | **no status** — "the act is the commit" |
| review | `RV-NNN` | adversarial-review ledger (ADR-007/SPEC-032) | findings: `open → answered → verified`/`withdrawn`; derived review status; baton |
| knowledge record | `DEC-/QUE-/ASM-/CON-/EVD-/HYP-/CPT-` | epistemic records (RFC-009; SL-159, SL-197) | per-kind lifecycles; `doctrine knowledge settle` moves to a resolving state **and** captures the disposition in one write |
| spec | `PRD-` (product) / `SPEC-` (tech) | durable product/technical intent | `PRD` owns *what/why*; `SPEC` owns *how*; `SPEC-NNN` `descends from` / `parent` a PRD or SPEC |
| requirement | `REQ-NNN` | a spec's normative requirement | carries a **mobile membership label** (`FR-`/`NF-`) — cite the `REQ-NNN`, never the label; status moves via REV `status` rows |
| ADR / policy / standard | `ADR-` / `POL-` / `STD-` | constitutional decisions, required rules | `accepted`/`required` etc. |
| backlog | `ISS-/IMP-/CHR-/RSK-/IDE-` | latent **work** intake | statuses incl. `open`, `triaged`, `started`, `resolved`, `mitigated`, `abandoned` |
| memory | `mem_…` uid + optional `mem.*` key | agent-owned private cache (RFC-009 D1) | trust/severity/staleness; shipped corpus vs local |
| observation | uuid | authored friction/measurement record (SPEC-028) | `friction`, `measurement`, `supersession`, `retraction` |
| concept map | `CM-NNN` | DSL wrapping **virtual** nodes orphaned from the graph (RFC-009 D4) | CPT is the reified half, landed |
| definition | — | **no kind exists**; `IDE-050` proposes one for ubiquitous-language terms | nearest is `CPT`, which is "a different job" |
| REC vs RV vs REV | — | *three different things* — the single most confusable trio in this area; see §6.1 | |

### 6.1 The REV kind in detail (ADR-013; `doctrine revision --help` ✓)

- The **only writer of a `revises` edge** is `revision change add`;
  `doctrine link … revises …` is refused (`TypedVerbOnly`).
- Rows are typed: `action` ∈ `modify | retire | move | status | introduce |
  create` (+ a `prose` action surfaced in prose rows), with `target` (durable
  peer id), `primary` (display hint, at most one), and `new_label`/`member_of`/
  `new_statement` for introducing requirements.
- `approve` records an orthogonal approval; **`apply` refuses without it**, and
  is **invoker-blind** (a solo dev self-approves; ADR-009).
- `apply` **auto-lands `status` rows only**; `modify`/`retire`/`create`/`move`/
  `prose` rows are **surfaced for manual landing** under the authored-truth
  honour model.
- A **pre-flight all-or-nothing from-guard** aborts the whole apply if any target
  moved since the row was drafted.
- A REV stays `started` until **all** rows land; the reconcile split rule exists
  precisely so a stuck row cannot block close.
- A fresh REV is a skeleton: `proposed`, `approval = none`, no rows.

**Why it matters here:** reconcile *is* the surface where "the code does this"
could be laundered into "the requirement says this". The REV's typed rows and
approval checkpoint are the only structural defence; the human's role at that
checkpoint today is to accept a row list.

### 6.2 REC in detail

Immutable, no lifecycle; `[rec] move = accept | revise | redesign`;
`owning_slice`; `[[status_delta]] {requirement, from, to}` (may be empty for
`redesign`); `[[evidence_ref]] {slice, requirement, contributing_change, mode}`
— cited by the stable 4-tuple key, "never a `file#line`". Full predicate:
`mem.pattern.doctrine.close-drift-discharge-rec`.

### 6.3 Knowledge records in detail

Seven kinds; the facets the design run can mint inline are enumerated in
`reference/design-payload-contract.md` (`assumption` claim/confidence/basis/
validation_plan/…; `decision` context/choice/alternatives/rationale/consequences/
decided_by/decided_on; `question` question/why_matters/answer/…; `constraint`
statement/source/applies_to/waiver_…; `evidence` datum/provenance/confidence;
`hypothesis` proposition/predicts; `concept` opens no keys). `DEC-150` splits
each facet into **Deciding** and **Argument** tiers. `IMP-415` records that
skills do not instruct an agent to fill a knowledge facet.

---

## 7. Configuration, prompt, and install seams

Where an interaction/understanding policy could physically live, with the
constraint each seam carries. **Facts only** — no recommendation is implied by
ordering.

| seam | mechanism | constraints |
|---|---|---|
| `doctrine.toml [conduct]` | axis-B `actor × autonomy` per lifecycle state; parsed by `src/conduct.rs` (`ConductConfig`, `resolve`) | **advisory only** ("what the project intends… nothing here gates a write"); autonomy is exit semantics |
| `doctrine.toml` other tables | `[verification]`, `[dispatch]`, `[reservation]`, `[capsule]`, `[priority]`, `[install]` — project-local config parsed by `src/dtoml.rs` | POL-002 forbids host-project conventions leaking into platform defaults |
| **hymns cascade** (`SPEC-023`) | bands `preamble → harness → model → role → stage → project`; trait-keyed model band; `seal`/`expose` sidecars; precedence **band → specificity → provenance → alpha**; `replaces` is the only suppression | `install/manifest.toml` declares `seal = ["preamble/core","stage/design"]`, `expose = ["harness/claude","harness/cursor","model/anthropic/claude-sonnet-4","model/deepseek/_default","role/worker"]`; on-disk `.doctrine/hymns/` currently ships `harness/claude`, `harness/cursor`, `model/adherence/low`, `model/anthropic/claude-sonnet-4`, `model/deepseek/_default`, `preamble/core`, `role/orchestrator`, `role/worker` — **no `stage/` dir on disk** (the sealed `stage/design` lives only in the embed) |
| `install/hymns/stage/design.md` | **"Design stage invariants… These hold for every turn of a managed design run, whatever the next obligation is."** | sealed — a user twin at that slot is dropped before matching ✓ |
| design-prompt assets (`install/design-prompts/`) | `.toml` runbook steps + `.md` stage lenses, embedded and rendered through `src/design_run/prompt.rs` (`asset_key`, `contract_store`) | step ids are API and digest-bound; authoring rule (step iff a project could legitimately differ); no project extension interface yet (`IMP-375`) |
| boot snapshot + `boot-footer.md` | `doctrine boot` writes the resident prefix; `.doctrine/boot-footer.md` is injected as `## Onboarding` | RFC-033's frame makes the boot carry **pointers, not restatements**; `boot --check` |
| harness hooks | `plugins/doctrine/hooks/hooks.json` — `SessionStart → doctrine boot --emit`, `PreToolUse → memory surface`, `WorktreeCreate → worktree create-fork`; Claude writes hooks directly into `.claude/settings.json` (`[install] claude-settings-scope`) | POL-003 harness independence; command form `${DOCTRINE_BIN:-doctrine}` |
| `doctrine prompt resolve --role worker|orchestrator [--stage …]` | the composed cascade | `--role` is required and accepts only those two (`ISS-308`); an unknown `--model` is accepted silently (`ISS-491`); `IMP-489` proposes a cascade review; `IDE-042` proposes model-tiered worker prompts |

**The two seams that are shaped exactly like this problem:** `[conduct]` is a
declarative, per-state, project-level posture (advisory), and the hymns cascade
already has a **`stage` band** with a sealed design-stage fragment delivered
every design turn. `IDE-029` (*Lifecycle-stage hymn seams for project
customisation*) is the open item that would generalise the second.

### 7.1 Where generic vs lifecycle-doctrine content may live

RFC-021 (resolved) settled the boundary: lifecycle skills may require the binary
and act as activation stubs; **`pair` and `walkthrough` are deliberately
self-contained** and must remain so. Any understanding policy that must reach a
non-doctrine harness without the binary cannot live in a lifecycle skill.

---

## 8. Evidence inventory

### 8.1 The observation ledger — shape, and a verified negative

- ✓ 670 records under `.doctrine/observations/records/NN/<uuid>.toml`, sharded by
  the first uuid byte; 666 `friction`, 4 `supersession`.
- Record shape (read a sample ✓): `schema`, `uid`, `recorded_at`, `kind`,
  `summary`, `detail`, `[facets.execution]` with `interface`,
  `product_surface`, `command`, `repository_context` (all auto-enriched,
  `*_origin = "automatic"`), plus optional `provenance`/`work_context`/
  `correlation`/`usage` groups. Records are **authored** (committed, diffable);
  only publication temps are gitignored.
- Query verbs: `doctrine observation {record,show,list,search,supersede,retract}`
  ✓. The MCP adapter `observation_record` refuses measurement authority and
  supersession controls, and **refuses capture from a worker fork with no
  broker** (a fork-local record would die with the tree). The boot snapshot
  carries RFC-011's instrumentation instructions.
- **Verified negative (✓, by `grep -ril` over all 670 records):**

  | term | records |
  |---|---|
  | `understanding` | **0** |
  | `understand` | **0** |
  | `comprehens` | **0** |
  | `walkthrough` | **0** |
  | `cognitive` | **0** |
  | `abbreviat` | **0** |
  | `acronym` | **0** |
  | `coordinate-speak` | **0** |
  | `read the code` | **0** |
  | `explain` | 12 (all incidental to command/verb legibility) |
  | `human` | 9 |

  The 12 `explain` hits are surveyed: `/design: no read verb for a knowledge
  record's prose body`; `design resume/show` snapshot-only governance
  confirmation; boot grammar omitting `knowledge show`; apply payload
  reverse-engineering; no read-back verb for phase criteria; a multi-file grep
  bug; a dispatch commit-path failure; an `execute` snapshot/mutate/restore
  driver; a wrong memory claim; `/reconcile` brief naming a verb without
  `--intent`; `CreateRecord` swallowing an unknown facet key; `phase-plan`
  inventing a compat requirement. **None is about human comprehension.**

  **Reading:** this theme has no captured friction base. A programme must author
  its measurement (VH criteria, a measurement observation, an experiment) rather
  than mine the ledger — and RFC-011's instrumentation guidance means future
  friction *can* be captured as it is felt, which is the cheapest way to build
  the base.

### 8.2 Memories worth retrieving

Shipped corpus (31, embedded; cite by key or uid): `mem.signpost.doctrine.*`
(overview, reference-docs, skill-map, lifecycle-start, conventions,
reading-entities, review, audit, design-run.cluster, requirements,
relating-entities, knowledge, recording-memories, specs, revisions, backlog,
policies-standards, adrs, boot-snapshot, storage-model/tiers, cli-source-of-truth,
tdd-loop, routing-gate, core-loop, hymn cascade), `mem.signpost.project.orientation`,
`mem.signpost.spec-002.slice-roadmap`, plus `mem.pattern.doctrine.*` and
`mem.fact.doctrine.*`.

Directly relevant local memories found by targeted search (retrieve before
relying):

- `mem.pattern.reconcile.edit-design-out-of-band` — reconcile edits `design.md`
  after lock; the review pass legitimately reads `STALE` (RFC-031 T5 records this
  as expected, not a defect).
- `mem.pattern.design-run.*` — read run state via `show` not raw TOML; measure a
  VA on a **scratch copy** of a real run; e2e submission-id minting; probe the
  parser through `adopt --dry-run`.
- `mem.fact.design-run.snapshot-outlives-the-binary` — runtime state is not
  freely serde-breakable.
- `mem.fact.design-run.gate-invalidation` — "shape voids acceptance, progress
  does not" (the `DEC-300`/`DEC-301` outcome).
- `mem.pattern.doctrine.close-drift-discharge-rec` — the REC predicate.
- `mem.pattern.feedback.rule-findings-in-dependency-order` (cited by RFC-026's
  Discussion).
- `mem.pattern.review.sweep-defect-class-not-instance`,
  `mem.pattern.doctrine.amend-knowledge-both-tiers` — RFC-026 S4's P3-shaped
  compensations.
- `mem.pattern.build.just-check-workspace-gates-members` — a new workspace member
  is not auto-gated.

### 8.3 Theme-relevant backlog items (status at baseline ✓)

| id | status | relevance |
|---|---|---|
| `IMP-344` | open | **Design line-number citations rot fast — cite symbol names instead.** Directly governs how this file and any slice design cites code. |
| `IMP-351` | open | Skill content is ungoverned under taxonomy change (published skills reach `.pi`/`.agents` via the *published* repo, not the tree). |
| `IMP-307` | open | Lint SKILL.md frontmatter description hazards. |
| `IMP-395` | open | Skills hand collaborators their exact invocation (guessed flags / wrong binary). |
| `IMP-394` | open | Bootstrap context for non-indoctrinated collaborator agents (what a reviewer needs to read the prose). |
| `IDE-055` | open | Fold `/consult` into the authority model — an existing "retire a skill into ADR-023" proposal. |
| `IDE-016` | open | Agent UX research: which MCP read surfaces beat the CLI; "review-adjacent reads proven valuable". |
| `IDE-029` | open | **Lifecycle-stage hymn seams for project customisation** via the existing cascade (not per-skill hook files). |
| `IDE-050` | open | Definition records for ubiquitous-language terms (`admission`/`verify`/`conform` collisions cost a retention decision). |
| `ISS-308` | open | `prompt resolve` has no role for non-dispatch agents (`--role` is `worker|orchestrator` only). |
| `ISS-491` | open | `prompt resolve` accepts an unknown `--model` silently. |
| `IMP-489` | open | Prompt cascade review: model coverage, currency, role fitness. |
| `IMP-415` | open | Skills do not instruct an agent to fill a knowledge facet. |
| `IMP-373` | open | Runbook `set` mode: coverage-set admission and its render bound. |
| `IMP-375` | open | Project extension interface for design-prompt assets. |
| `IMP-412` | open | Locked design run has no handover exit. |
| `IMP-392` | open | Unify design-run findings onto the RV ledger (RFC-031 T5; RFC-032 slice 3). |
| `IMP-475` | open | Review read surface: `show --findings`, `list --target` (RFC-032 slice 0c/2). |
| `IMP-029` | open | RV verb-family black-box e2e golden (RFC-032 C23). |
| `ISS-280` | open | `review contest` records no durable rationale — `fulfilled by: SL-268`, open pending closure. |
| `ISS-485` | open | No review `amend`/`reopen` — `fulfilled by: SL-268`, open pending closure. |
| `CHR-079` | open | Refresh `install/review-ledger.md`; add a doc-to-help consistency check. |
| `IMP-465` | open | Composed knowledge read: the product pass `VH-1` asked for (`SL-246`). |
| `ISS-467` | open | Facets withholds the Argument tier with no disclosure. |
| `IMP-314` | open | Research artefact has no harvest pointer at close. |
| `CHR-049` | open | Run post-`SL-233` managed-design measurement exercise. |
| `CHR-024` | **started** | Entity relationship design review → redesign RFC (overlaps RFC-009 D3). |
| `CHR-023` | open | ADR-005 compliance: reference-doc IA, user hooks, restate-line audit (RFC-033 owns it now). |
| `IMP-130` | open | Land the `RV-116` operator guard in SPEC-021 / close+audit skills. |
| `IMP-096` | open | Requirements capture and refinement skills for the reconcile loop. |
| `IMP-205` | open | Spec consistency gate: detect contradictory `REQ` pairs. |
| `ISS-353` | open | `doctrine install` never prunes orphaned skills/agent defs — the mechanism behind §3.1's projection drift. |
| `CHR-045` | open | Bump `plugin.json` version when the skill set changes (skill-delivery hygiene). |

### 8.4 RFC index (all `open` unless noted)

030 (inquiry map) · 031 (design-run fitness) · 032 (review ledger) · 033
(learning surface) · 009 (epistemic records) · 017 (human onboarding docs) ·
022 (agent trust model) · 026 (design review response effectiveness) · 027
(progressive discovery) · 028 (verifiable human authorization, draft) · 029
(proof binding). **Resolved:** 010 (skill improvement sweep), 021 (dynamic
behaviours and minimal projection), 016 (zero-rescue dispatch). RFC-034's own
`status` is `open` with no relations authored ✓.

---

## 9. Adjacent programmes and overlaps

| programme | relationship to RFC-034 |
|---|---|
| RFC-032 (review ledger effectiveness) | Owns the RV read surface a human would need to *see* findings; slice 1 landed, read surface pending. Any "walk the human through the findings" work should ride its projection, not build a parallel one. |
| RFC-031 (design run fitness) | Owns the design run's T1–T6. Any design-time obligation adds to a machine already judged to have an untruthful exit signal (T1) and an expensive prose loop (T2). |
| RFC-030 (inquiry map) | Owns the map's purpose and the deferred authority model (T4). The `DischargeClaim` precedent and the presence analysis live there. |
| RFC-026 (design review response) | Owns G2/G3/G12/G15 (comprehension-shaped problem statements), S1–S8, P1–P10. `P2` (records referenced DRY, never retyped) and `P3` ("a skill change is a compensation, not a structural fix") are direct constraints on cheap-prompting answers. |
| RFC-009 (epistemic records) | The "human-facing substrate" thesis and the memory↔record boundary; D3 (record/entity edges) still open and overlapping `CHR-024`. |
| RFC-028 + ADR-023 + RFC-030 T4 | The three pieces of "can doctrine believe a human was there". ADR-023 explicitly excludes verifying the human; RFC-028 is the unlanded mechanism; RFC-030 T4 the deferred design. |
| RFC-027 (`H12`) | The three-tier authority model; the "independent foothold" question belongs here. |
| RFC-033 / RFC-017 | Where the *documentation* half of human comprehension lives (one normative home, human-voiced on-ramp). |
| ADR-020 + capsules, `RFC-027` | Bounded autonomy is the one intervention from the paper's inventory doctrine has already implemented; it bounds consequences, not understanding. |
| `CHR-024` | Entity-relationship redesign; a kind-based solution (e.g. IDE-050 definitions, an "understanding" record) must reconcile with it first. |

---

## 10. Unclaimed ground (facts, no proposals)

Each row states a verified absence. It is deliberately not a proposal list.

1. **No requirement, spec or skill obliges anyone to confirm the human
   understood anything.** `README.md` priority 2 states comprehension as a
   product goal and implements it as recording. The design-run gate set contains
   no understanding condition; `/design`, `/audit`, `/reconcile`, `/close` never
   mention it (§3.6).
2. **No stored field distinguishes demonstrated understanding from assent.**
   `AcceptanceDeclaration { basis, turn }` is a one-line quotation by the agent;
   `AcceptanceAttestation` binds content, not presence (RFC-030 T4).
3. **No surface records a human-originated proposition before the agent's
   answer.** The `elicit` skill is the only workflow shaped that way, and it is
   about value, not design or reconciliation.
4. **No product/tech spec governs human-comprehension surfaces.** `walkthrough`
   and `pair` have no PRD/SPEC; the RV kind had none until SPEC-032 (RFC-032
   C22/`IMP-481`).
5. **`VH` has no gate.** VT is re-derived (`doctrine coverage verify`, `vtgate`);
   VA/VH are authored. The one negative product-level `VH` verdict this pass
   found was filed as backlog after its slice closed (`SL-246`/`RV-372`).
6. **Reconciliation presents patches, not text.** `/reconcile` asks for
   confirmation of edits and REV rows; nothing requires the governing passage to
   be shown in place, or the distinction between "correcting a description",
   "recording a discovered constraint", "changing a requirement", and
   "recording an unresolved mismatch" to be made explicit.
7. **No observation base exists for the theme** (§8.1).
8. **`design.md` is presented as one document for four roles** (RFC-026 G12),
   with no separation of history from current meaning (S1/P1) and no structural
   intent/realization boundary (G15/P9) — so a human reading it meets review
   residue and implementation approach undivided.
9. **The design run's own review pass is invisible on the CLI by default**
   (RFC-032; `IMP-475`), and `IMP-392` (unify findings onto the RV) is open.
10. **Generic skills are contractually self-contained** (RFC-021), while
    lifecycle skills may require the binary — so the delivery channel for any
    policy differs by target audience.
11. **The `stage` hymn band exists and is sealed only for `stage/design`**; there
    is no project-facing per-stage customisation seam yet (`IDE-029`), and no
    `stage/` directory on disk.
12. **`[conduct]` exists as a declarative per-state posture and is explicitly
    advisory** — the nearest incumbent to a dial, with a stated reason it cannot
    gate.

---

## Appendix A — re-verification, and threads a refresh should run

Commands that re-derive the load-bearing claims (run from the coord tree's
`./target/debug/doctrine`):

```bash
./target/debug/doctrine design --help
./target/debug/doctrine design contract --format prompt     # the payload contract
./target/debug/doctrine design show <SLICE> --format status # the run's envelope
./target/debug/doctrine review --help                       # ledger verbs
./target/debug/doctrine revision --help ; doctrine rec --help
./target/debug/doctrine knowledge --help ; doctrine memory --help
./target/debug/doctrine observation search <term> --history
./target/debug/doctrine prompt model-keys ; prompt resolve --help
./target/debug/doctrine backlog list -t cluster:design-run
./target/debug/doctrine search "<keywords>" ; doctrine search -k all "<keywords>"
```

Greps that re-derive §3.1, §3.6 and §8.1:

```bash
comm -3 <(ls .pi/skills | sort) <(git ls-files plugins/doctrine/skills | cut -d/ -f4 | sort -u)
grep -rc -iE 'understand|comprehens|cognitive' plugins/doctrine/skills/*/SKILL.md
for p in understanding comprehens walkthrough cognitive abbreviat acronym; do
  echo "$p: $(grep -ril "$p" .doctrine/observations/records/ | wc -l)"; done
```

**Threads a refresh should run** (not run this pass; each is read-only and maps
to a section above):

1. **Governance applicability** — via `./scripts/pi-research`: does any PRD/SPEC
   own the human-comprehension surface, the skills' *content* (vs distribution),
   or `VH`? (Feeds §2, §5.5, §10.4.)
2. **CLI read surface census** — via `./scripts/pi-scout`: enumerate every
   surface that renders an entity for a human (`show`, `inspect --knowledge`,
   `design show`, `review show`, `library show`, `map`, `onboard`), with the
   default projection and what it withholds. (Feeds §4.4, §5.7, §10.9.)
3. **Harness/activation census** — how each harness injects boot, skills and
   hooks, and what a project can override without a binary. (Feeds §7, §7.1.)
4. **External prior art** — read the secondary sources in §1.4 (Storey, Horthy,
   Bainbridge, the EU AI Act oversight text) to give the RFC's Context primary
   citation rather than second-hand.

---

## Appendix B — corpus counts at baseline (for staleness judgement)

```
slices          533      reviews (RV)     734
revisions       128      knowledge recs   429
backlog items   935      RFCs              34
ADRs             48      local memories   646
shipped memories 31      observations     670  (666 friction, 4 supersession)
skills (tracked) 35      skill projections 39 (.pi) / 37 (.doctrine)
```

Corpus figures go stale by growth — re-run before load-bearing use. The
`/research` staleness verb is slice-scoped (`doctrine slice research <id>`);
RFCs carry no `baseline.toml`, so a consumer must re-check by hand or refresh
this artefact (`IMP-314` records the missing close-time harvest pointer for
slice research; RFCs have no equivalent mechanism at all).
