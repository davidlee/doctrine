# Design SL-144: ADR-005 full compliance: reference-doc IA, user hooks, restate-line audit

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

The `install/` ship surface is the entire user-facing knowledge/customisation
surface doctrine copies into every client. It has never been audited as a
**system**: what each document is responsible for, how they cross-reference,
which access-pattern tier governs each, and how an agent discovers them. Since
ADR-005 was accepted (2026-06-08) and this slice was cut (2026-06-23) the surface
drifted — SL-186/187 added the prompt-cascade tier (`hymns/`, `model-band.md`)
and retired the boot-footer round-trip, while new entity kinds (REC/REV/POL/STD/
knowledge) and CLI verbs landed without the reference docs keeping pace.

The design produces **one coherent IA map** of the surface, then executes the
corrections that map implies: retire the dead hook, document the live ones,
bring the reference docs current, enforce the restate line, and make every
shipped doc reachable. It is a documentation/IA slice — no engine, entity, or
CLI architecture changes (ADR-011/SPEC-023 own the hymns mechanism; this slice
cross-references, never amends).

## 2. Current State

`install/` ship surface (live tree, 2026-07-03):

| Path | Role | Access tier | Drift |
|---|---|---|---|
| `routing-process.md` | boot digest: routing table, core process, guardrails, reference-forms | PUSH (boot) | verify R-OQ-5 block current |
| `governance.md` | user governance pointer | PUSH (`boot.rs GOVERNANCE_REL`) | live |
| `model-band.md` | universal model-band floor | PUSH (boot `Static`) | **new**, undocumented as hook |
| `boot-footer.md` | (was) `## Onboarding` source | **retired** (SL-187) | **orphan** — delete |
| `glossary.md` | vocabulary, ids, reference forms, verification taxonomy | PULL-ref | stale kinds/verbs |
| `using-doctrine.md` | which verb for intent, storage tiers, read via `show`, edit rules | PULL-ref | stale kinds/verbs |
| `review-ledger.md` | adversarial review protocol | PULL-ref | check currency |
| `hymns/` | context-keyed prompt cascade (framework site) | PULL cascade (ADR-011/SPEC-023) | map only; SL-191 owns internals |
| `agents/` | subagent defs (e.g. `dispatch-worker.md`, hymn-baked) | build asset | map only |
| `templates/` | seed bodies (`seed-onboarding.md` → orientation memory) | build asset | map only |
| `manifest.toml` | install manifest: dirs, memory seed, gitignore, hymns seal/expose, root markers | reachability oracle | not in IA today |
| `doctrine.just`, `doctrine.toml.example`, `LICENSE` | config/build scaffolding | build asset | out of knowledge-IA |

Onboarding hook (post-SL-187): `manifest.toml [memory]` seeds
`mem.signpost.project.orientation` from `templates/seed-onboarding.md`; the
onboarding-tagged memory body populates the boot `## Onboarding` section. No
`boot-footer.md` read path exists anywhere in `src/`.

Skills: 30 under `plugins/doctrine/skills/*/SKILL.md`; 8 carry `--flag <ARG>`
candidates for restate-line triage (backlog, close, worktree, dispatch,
dispatch-agent, dispatch-subprocess, reconcile, reviewing-memory).

## 3. Forces & Constraints

- **ADR-005** — the governing tiering: PUSH carries load-bearing rules only;
  PULL-reference docs must be *pointed-at* to be reachable; skills route, don't
  restate (R-OQ-4 restate line); no shipped doc duplicates `doctrine --help`.
- **ADR-011 + SPEC-023** — govern the hymns cascade. This slice maps, never
  amends; if the cascade needs folding into ADR-005's tiers, that is a review
  finding, not a mid-slice amendment.
- **POL-002** — platform independence: docs carry no host-project convention as
  correctness.
- **STD-001 / STD-002** — named constants (embed roots, marker strings), short
  entity titles, ids not slugs.
- **Evidence-bound (R-C1)** — restate-line fixes target named offenders, not a
  blanket rewrite.
- **Re-embed footgun** — `install/*` edits need `touch src/install.rs` +
  `cargo build` to ship; batch edits.
- **Scope shape C+** — audit `.md` + manifest as the system; map the new subdirs
  at stable altitude without restructuring SL-186/187/191 territory.

## 4. Guiding Principles

- **One responsibility per doc; one home per fact (DRY).** Every overlap
  resolves to a single owner + a pointer from the others.
- **Access pattern decides tier.** A rule needed *before* any skill invoke →
  PUSH; shared detail → PULL-reference; context-keyed → hymns cascade.
- **Reachable or retired.** A doc not in the ship set and not pointed-at is dead
  weight — delete it (boot-footer) rather than leave residue.
- **Map, don't rebuild.** The new subdirs are accounted-for in the IA with
  governance cross-refs; their internals stay with their owning slices.

## 5. Proposed Design

### 5.1 System Model

The IA is a **three-tier access model** over the ship surface, plus **one
adjacent mechanism** (the hymns cascade) the map cross-references but does not
own. Each tier answers "when does an agent encounter this content?" — the
ADR-005 thesis. The cascade is deliberately *not* a fourth tier: calling it one
would import it into ADR-005's taxonomy while disclaiming authority over its
semantics — the shared-responsibility fog D2 exists to avoid (external review
C). It is an interface-boundary cross-reference only.

```
TIER 0 — PUSH (boot-resident, every session, no invoke)
  routing-process.md   workflow digest + guardrails + reference-forms (R-OQ-5)
  governance.md        user governance pointer      [boot GOVERNANCE_REL]
  model-band.md        universal model-band floor   [boot Static]
  (onboarding memory)  seeded from templates/seed-onboarding.md → boot Onboarding
        │  ADMISSION: PUSH iff omission can cause incorrect action *before* any
        │  routed read. Everything else lives in Tier 1, reached by a pointer.
        ▼
TIER 1 — PULL-reference (shipped docs; read when Tier 0 or a skill points)
  glossary.md          vocabulary · ids · reference forms · verification taxonomy
  using-doctrine.md    which verb for intent · storage tiers · read-via-show · edit rules
  review-ledger.md     adversarial review protocol
        │  each MUST be pointed-at (skill or boot) AND in the manifest ship set
        ▼
TIER 2 — Skills (thin routers, plugins/doctrine/skills/*/SKILL.md)
        │  name a verb, cite a rule by name; never reproduce flags/tables (R-OQ-4)

ADJACENT — Hymns cascade (context-keyed; ADR-011/SPEC-023) — CROSS-REF, not a tier
  install/hymns/ (framework) + .doctrine/hymns/ (user overlay)
  resolved by `doctrine prompt resolve` on (harness, model, role, stage, project)
  the IA names it + its authority; it owns none of the cascade's semantics

SHIPPING ORACLE — manifest.toml
  [dirs] [memory] [gitignore] [hymns seal/expose] [root_markers]
  a doc ships iff install.rs embed-copies it; a hymn ships iff manifest exposes it.
  proves DISTRIBUTION only — not discoverability (that is the pointer graph, §5.3)
```

**PUSH admission criterion (the load-bearing test, tightened).** External
review flagged "load-bearing" as taste-based and thrash-prone. The auditable
rule: *a fact is PUSH iff its omission can cause an incorrect action before any
routed read* — i.e. it must be resident because the agent acts on it prior to
opening any Tier-1 doc or skill. The exhaustive pushed fact-classes:
routing/stage selection, reference-form syntax (so ids are cited correctly from
the first commit), storage-tier read/write rule (so the first `show`/edit is
correct), and "use the CLI, don't guess". Anything not on this closed list is
Tier 1, reached by a pointer. The IA audit tests each Tier-0 line against this
criterion; a line that fails demotes to Tier 1.

**Responsibility boundaries** (resolves the overlaps Objective 1 hunts):

- `routing-process.md` = *WHEN/route* + the handful of load-bearing rules
  (reference forms, storage tiers, "use the CLI"). It **points at** Tier 1 for
  the *how*; it never carries verb-by-intent tables or edit mechanics.
- `using-doctrine.md` = *which verb for which intent* + storage-tier read/write
  + edit-preserving rules. The single home for CLI/editing mechanics prose.
- `glossary.md` = *vocabulary*: kinds, id/reference forms, verification taxonomy.
  Definitional only — no workflow, no verb-routing.
- `review-ledger.md` = the review protocol; owned by the review/inquisition path.

Boundary test: if two docs state the same fact, one becomes the owner and the
others cite it by name. If a fact has no home, it lands in the tier its access
pattern dictates.

**Ownership matrix — decided up front, not deferred (external review A/F2).** The
internal pass parked "does using-doctrine.md over-concentrate?" as a watch-item
for the audit's own coherence test — self-grading, and a known ambiguity
execution would optimise around. Replaced with a **pre-execution deliverable**:
the plan's first phase produces an **ownership matrix** — one row per fact-class,
columns *owning surface · pointer surfaces · non-owner prohibition*. It is
authored and reviewed **before** any doc is edited. The split then falls out of
the matrix, not later taste:

- `using-doctrine.md` provisionally carries verb-for-intent + storage/edit
  mechanics + the hook-contract table + the reachability contract — four
  fact-classes. **Hard pre-commit:** if the matrix shows one doc owning
  unrelated fact-classes that fail a single-invariant test (do they all serve
  one "how do I operate doctrine" responsibility?), the surplus splits to a
  dedicated Tier-1 doc (candidate: `hooks.md` for the hook contracts +
  reachability contract, which are *install/integration* facts, not
  operator-workflow). No new doc is minted pre-emptively (YAGNI); the matrix,
  not intuition, triggers the split.
- The matrix is the externalised artefact the coherence VA reconciles against
  (§9) — it is not the audit grading its own prose.

**Cross-reference edges** (the reachability graph the audit must close):
- Tier 0 → Tier 1: routing-process points at glossary + using-doctrine.
- Skills → Tier 1: skills that need the *how* cite using-doctrine / glossary
  by name (not by reproducing it).
- manifest → every shipped doc: the ship set must list each Tier 0/1 doc;
  an unshipped doc is unreachable regardless of pointers.

**Governance partition:** Tiers 0–2 are ADR-005's domain (this slice's audit
target). Tier 3 is ADR-011/SPEC-023's domain (mapped, cross-referenced, not
restructured). The IA map names both so the surface is fully *accounted for*
without the slice reaching into cascade internals.

**Derived vs authored (the currency line).** The boot `## Commands` section is
**derived** — `render_boot_map()` projects it from the compiled clap tree
(SL-150), exposed to users as `doctrine --help --boot-map` (dense map) and
`doctrine --help --commands` (grouped table). Two consequences bind the audit:
- **Currency is automatic** for anything projected from the CLI (commands,
  `## Invoking doctrine`, gov-row sections) — it cannot drift, so the currency
  work (Objective 4) targets only the *authored* Tier-1 docs.
- **Restate-line has a concrete point-at target.** A skill or doc that needs to
  reference the command surface cites `doctrine --help --boot-map` /
  `--commands` — it never reproduces a command list. This sharpens the R-OQ-4
  test: any command/flag table in a skill is a violation *because a live
  generator exists*.
  - **Finding (ISS-208):** that point-at target is currently **undiscoverable**
    — `--boot-map`/`--commands` are not registered clap args and are absent from
    `doctrine --help`; they only work riding `--help`. CLI wiring, out of this
    slice's scope, but the restate-line fixes must cite an invocation that
    actually resolves. Until ISS-208 lands, docs cite the full `doctrine --help
    --boot-map` combo (which works today), not a bare flag.

### 5.2 Interfaces & Contracts — user-serviceable hooks (Objective 2)

Each live hook gets a documented contract: **what it controls · mechanism ·
precedence · reset**. The contracts live in the surface the §5.1 ownership
matrix assigns — provisionally `using-doctrine.md`, splitting to a dedicated
`hooks.md` if the matrix shows fact-class overconcentration — with a one-line
pointer from the boot digest where the hook is boot-resident.

**Type split (external review).** The table below spans two *kinds* of surface,
which the contract doc must keep visually separate so customization semantics
never bleed into everyday usage: **user-editable hooks** (`governance.md`, hymns
overlay, onboarding seed — the user is meant to change them) vs **shipped inputs
consumed by boot** (`model-band.md` — a constant the user normally leaves). They
share a table only for the reader's convenience; the precedence column carries
the distinction.

| Hook | Controls | Mechanism | Precedence | Reset |
|---|---|---|---|---|
| `.doctrine/governance.md` | project governance pointer text in boot | boot reads `GOVERNANCE_REL`, injects as `## Governance (project)` | user text replaces the seeded default wholesale | delete → boot shows marker; re-install re-seeds |
| `install/model-band.md` → `.doctrine/…` | universal model-band floor directive | boot `Static` source → `## Model band` | **shipped constant**, not a user-authored hook: the "universal, model-agnostic" invariant means projects normally leave it; per-model content belongs in the hymns cascade, not here | restore from `install/` |
| `.doctrine/hymns/` overlay | per-context prompt bands (harness/model/role/stage/project) | `doctrine prompt resolve` merges framework + overlay | **overlay wins** over framework at equal key (SPEC-023) | remove overlay file → framework band resolves |
| onboarding seed | boot `## Onboarding` body | `manifest.toml [memory]` seeds `mem.signpost.project.orientation` from `templates/seed-onboarding.md`; onboarding-tagged memory bodies render | edit the seeded memory (or add onboarding-tagged memories, key-ordered) | `memory verify`/edit; re-install skips existing key |

**Retirement — `boot-footer.md`:** delete `install/boot-footer.md` and the
orphan `.doctrine/boot-footer.md`. Contract: none — the file is dead post-SL-187.
The audit verifies no `src/` read path, no skill pointer, no manifest reference
survives (all confirmed absent at design time). Hymns internals
(`traits:` frontmatter, band layout) are **not** documented here — cross-ref to
SPEC-023 / SL-191.

### 5.3 Data, State & Ownership — reachability contract (Objective 6)

**Two notions, kept distinct (external review D).** The internal model conflated
"the doc arrives" with "the doc is current" — write-if-absent makes those
different:

- **Distribution reachability = shipped ∧ pointed-at.** What this slice owns and
  audits, for a **fresh install**.
- **Semantic currency = the shipped copy is not stale.** A client that installed
  an older doctrine keeps its `.doctrine/*.md` copy: `build_plan` is
  write-if-**absent**, so re-install/upgrade does **not** overwrite a diverged or
  outdated local copy. This slice's reachability claim is therefore scoped to
  *what ships from this repo*, not *what every installed client currently holds*.
  The stale-client hazard is **named, not solved here** — install-currency
  machinery (cf. `doctrine reseat`) is out of a docs-IA slice's remit; filed as a
  backlog idea (IDE-030). Verification claims (§9) assert shipped==regenerated in
  *this* repo, never client-repo state.

**Distribution reachability = shipped ∧ pointed-at.** A doc is distribution-reachable
iff both hold:
1. **Shipped** — for `.md` docs this is **automatic**: `build_plan` step 2
   copies every embedded `install/*` file (except `manifest.toml`) into
   `.doctrine/`, write-if-absent — no per-file allowlist (confirmed, OQ-2). So
   "shipped" ≡ "exists under `install/`" (RustEmbed `#[folder="install/"]`).
   For hymns, `manifest.toml [hymns] expose` is the ship gate. **Consequence:**
   the reachability *risk* for `.md` docs is not shipping — it is being
   *unreferenced* (pointer side); and retiring a doc means deleting the file
   from `install/` (removing it from the automatic ship set), which is why
   boot-footer.md must be file-deleted, not just de-referenced.
2. **Pointed-at** — checked as a **machine-checkable pointer class**, not a prose
   judgment (external review D): an explicit filename reference from the boot
   digest, a skill body, or the manifest, giving at least one path from
   session-start (boot) or a routed stage (skill) to the doc. A passing mention
   in unrelated prose does not count; the audit records the specific pointer edge
   per doc (the AGENTS.md lesson: an unreferenced `.doctrine/*.md` is read by no
   one).

The audit builds the reachability graph and closes every gap:
- **Tier 0** docs are pointed-at by construction (boot renders them).
- **Tier 1** docs (`glossary.md`, `using-doctrine.md`, `review-ledger.md`) each
  need a skill/boot pointer — the audit records which skill owns each pointer.
- **Orphans** (shipped-but-unreferenced, or referenced-but-unshipped) are
  resolved: delete the dead (`boot-footer.md`), or add the missing pointer/ship
  entry.

Ownership: this slice owns the *authored* `install/*.md` docs + the manifest's
knowledge-relevant entries. The clap-derived sections (commands, invoking
doctrine, gov rows) are owned by the CLI and out of the audit (they regenerate).

### 5.4 Lifecycle, Operations & Dynamics — execution shape

The corrective work executes against the §5.1 map in batches (the re-embed
footgun forces batching):

0. **Ownership matrix** (plan phase 1, before any edit) — author the fact-class →
   owner · pointers · prohibition matrix (§5.1). Reviewed before doc edits begin;
   triggers the `hooks.md` split if fact-classes overconcentrate. This is the
   externalised artefact §9's coherence VA reconciles against.
1. **IA audit pass** — walk the `install/*.md` set + manifest, produce the
   externalised check artefacts (doc→fact inventory, fact→owner map, pointer
   graph) and the overlap/gap/contradiction ledger against the ownership matrix (a
   working artefact, may live in slice notes).
2. **Currency pass** (Objective 4) — bring `glossary.md` + `using-doctrine.md`
   current for REC/REV/POL/STD/knowledge kinds and the revision/policy/standard/
   review/knowledge verbs. Definitional content → glossary; verb-for-intent →
   using-doctrine.
3. **Hook-contract pass** (Objective 2) — write the §5.2 table into
   using-doctrine.md; retire boot-footer.md.
4. **Restate-line pass** (Objective 3) — triage the 8 `--flag <ARG>` candidate
   skills; for each, decide *reproduces-a-table* (fix → cite `doctrine --help
   --boot-map` / the verb + rule by name) vs *names-a-verb* (already compliant).
   Evidence-bound; record the disposition per skill. **Then land a cheap
   structural regression gate** (external review E, D5-amended): a
   grep/regex denylist over `plugins/doctrine/skills/**` for flag-table /
   storage-tier-prose shapes. **Delivered as a shell/`just` recipe** (candidate:
   `doctrine.just` + a script under `install/scripts/`), *not* wired into the
   `doctrine check` Rust path — that would breach this slice's no-CLI-code
   boundary (§1). A `doctrine check` integration is a follow-up if the tripwire
   proves its worth. Imperfect by design — a structural tripwire, not a full
   linter — so a re-introduced table fails a gate rather than silently regressing
   the day after close.
5. **PUSH-tier pass** (Objective 5) — verify the reference-forms block in
   routing-process.md is present + correct (R-OQ-5).
6. **Re-embed + verify** — `touch src/install.rs` + `cargo build`; regenerate
   boot (`doctrine boot`); run `doctrine boot --check`; confirm reachability.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV — no ADR amendment.** Hymns stay under ADR-011/SPEC-023; any pressure to
  fold them into ADR-005's tiers becomes a review finding, not a slice edit.
- **INV — derived content untouched.** The audit never hand-edits a clap-derived
  boot section; currency there is automatic.
- **INV — PUSH stays compact.** Load-bearing rules only in Tier 0; detail is
  pushed to Tier 1 pointers (ADR-005).
- **Assumption** — `install.rs` embed-copies the full `install/*.md` set to
  `.doctrine/`; the manifest has no per-`.md` allowlist (confirmed: only `[dirs]`
  `[memory]` `[gitignore]` `[hymns]` `[root_markers]`). *Edge:* if a `.md` is in
  fact NOT copied, that is an orphan to fix in the reachability pass.
- **Edge — boot-footer residue.** The orphan `.doctrine/boot-footer.md` may be
  tracked in some client installs; deletion is safe because no read path
  consumes it.
- **Edge — restate-line false positives.** A `--flag <ARG>` inside a fenced
  *example command* a skill tells the agent to run may be legitimate; the test
  is reproduction of a *table/enumeration*, not a single cited invocation.

## 6. Open Questions & Unknowns

- **OQ-1 — model-band.md contract home.** Confirmed: documented in
  using-doctrine.md as a PUSH hook. No residual question.
- ~~**OQ-2** — does `install.rs` copy every `install/*.md`, or an allowlist?~~
  **Resolved (design):** every embedded `install/*` file except `manifest.toml`
  is copied (`build_plan` step 2), no allowlist. "Ship" is automatic; reachability
  risk is the pointer side. Folded into §5.3.
- ~~**OQ-3** — review-ledger.md tier: Tier-1 PULL-reference or skill-owned?~~
  **Resolved (external review, decide-now criterion):** a doc that governs a
  protocol spanning **more than one skill path** is a Tier-1 reference; one that
  governs a single routed protocol is skill-owned with a thin reference surface.
  review-ledger.md governs the review protocol used by `review`,
  `inquisition`, `contest`, and `dispose` — multi-skill → **Tier-1 PULL-reference**,
  pointed-at by each. The IA pass records the specific pointer edges; it no longer
  *decides* the tier.

## 7. Decisions, Rationale & Alternatives

- **D1 — Scope shape C+ (audit `.md`+manifest; map new subdirs).** Alt A (tight,
  6 files) leaves Objective 1 knowingly partial; Alt B (full IA incl. hymns
  internals) drags in SL-186/187/191 + an ADR amendment. C+ makes the surface
  *accounted for* without reaching into cascade internals. **Chosen.**
- **D2 — Hymns governed by ADR-011/SPEC-023, no ADR-005 revision.** The cascade
  is a resolver mechanism, not a static-doc tier. IA map cross-references.
  `SL-144 references(concerns) SPEC-023`; no `after SL-191` edge (altitude
  de-conflicts). Alt: amend ADR-005 to a 4-tier model — rejected as mid-slice
  governance churn; filed as a conditional review finding instead.
- **D3 — reconcile-rules.md dropped → IDE-029.** A per-skill hook file would
  parallel the hymns `stage`/`project` bands (DRY violation). General form
  captured as IDE-029, sequenced after SL-191. Alt: build it now — rejected
  (YAGNI, no consumer, parallel implementation).
- **D4 — boot-footer.md retired, not hardened.** Dead post-SL-187; header
  comment is a lie. Delete both copies. Alt: repurpose as a second onboarding
  hook — rejected (onboarding already has a live seed mechanism; two hooks for
  one slot re-creates the drift).
- **D5 — Restate-line: grep-assisted manual triage + a cheap structural gate.**
  8/30 candidates, manual disposition, *plus* a grep/regex tripwire recipe
  (§5.4 step 4) so compliance holds past close. Amended from "no permanent lint"
  after external review E: a *compliance* slice that ships zero regression
  control invites day-after regression. The gate is a shell/`just` tripwire, not
  a Rust linter (keeps the no-CLI-code boundary); a full `doctrine check`
  integration remains an optional follow-up.
- **D6 — Ownership matrix decided pre-execution, not deferred.** After external
  review A/F2: the using-doctrine.md overconcentration question is settled by a
  reviewed fact-class matrix in the plan's first phase, which mechanically
  triggers a `hooks.md` split if needed — rather than a self-graded coherence
  test during the audit. Alt: keep it a watch-item — rejected as marking-own-homework.

## 8. Risks & Mitigations

- **R1 — restate-line scope creep.** *Mitigation:* evidence-bound (R-C1); a
  cited invocation ≠ a reproduced table. Disposition recorded per skill; genuine
  architectural cases → ADR-005 review finding, not papered over.
- **R2 — re-embed footgun strands edits.** *Mitigation:* batch per §5.4;
  `doctrine boot --check` gates that shipped == regenerated.
- **R3 — ISS-208 makes the restate point-at target undiscoverable.** *Mitigation:*
  docs cite the working `doctrine --help --boot-map` combo; ISS-208 tracks the
  CLI fix separately (out of scope).
- **R4 — hymns section drifts as SL-191 lands.** *Mitigation:* document hymns at
  altitude only (existence, two-site, resolver, authority cross-ref) — nothing
  SL-191 can invalidate.

## 9. Quality Engineering & Validation

Verification alignment (maps to ADR-005 VT/VA + this slice's closure intent):
- **VT** — `doctrine boot --check` passes after edits (shipped snapshot ==
  regenerated); reference-forms block present in the regenerated boot (R-OQ-5).
- **VT** — fresh-install / embed test: `glossary.md`, `using-doctrine.md`,
  `model-band.md` present in `.doctrine/`; `boot-footer.md` **absent** from the
  install set. Plus a **grep gate**: no `boot-footer` reference survives in
  `src/` (incl. tests), `install/manifest.toml`, or any skill — so the deletion
  cannot strand a dangling reference (existing boot.rs tests assert the retired
  round-trip; they reference the *concept*, not the file, and stay green).
- **VT** — restate-line regression tripwire: the grep/`just` gate (§5.4 step 4)
  fails on a seeded flag-table fixture and passes on the cleaned skill corpus —
  so the control demonstrably bites.
- **VA** — IA coherence is **falsifiable against externalised artefacts**, not
  self-graded (external review B): closure reconciles three concrete artefacts —
  the **ownership matrix** (fact-class → owner), the **doc→fact inventory**, and
  the **pointer graph** — and the overlap/gap/contradiction **ledger** is the
  *reconciliation record* of those, not a substitute. A second agent (or human)
  can re-derive coherence from the artefacts; "coherent" is never asserted from
  prose alone. Comparison basis is fixed: overlap at the *fact* unit; gap
  relative to the closed corpus {ADR-005 duties, live `install/*.md`, skills,
  boot, manifest}.
- **VA** — restate-line: the 8 candidate skills triaged; each disposition
  recorded (fixed offender / compliant pointer); no skill reproduces a
  command/flag table.
- **VA** — reachability: every Tier-0/1 doc is **distribution-reachable**
  (shipped ∧ machine-checkable pointer edge); zero orphans; the reachability
  contract is documented in its matrix-assigned home. Scoped to fresh-install;
  semantic currency of stale client copies is out of scope (IDE-030).
- **VA** — currency: glossary + using-doctrine cover all current kinds/verbs, and
  each kind's facts have a **unique owner** across the two docs (external review:
  mention ≠ ownership) — spot-check REC/REV/POL/STD/knowledge each named *and*
  singly-owned.

## 10. Review Notes

### Internal adversarial pass (2026-07-03)

- **F1 — reachability rested on an unconfirmed embed assumption.** *Resolved
  during design:* `build_plan` step 2 copies every `install/*` file except
  `manifest.toml`, no allowlist (§5.3, OQ-2). "Ship" is automatic.
- **F2 — using-doctrine.md as contract home risks kitchen-sink bloat**, the very
  IA smell this slice cures. *Disposition:* contracts land in using-doctrine.md
  (it *is* the CLI/editing/mechanics reference), but the IA pass carries a
  **watch-item**: if using-doctrine.md loses coherence under the added hook +
  reachability contracts, split a dedicated `hooks.md` (Tier-1, pointed-at).
  Not pre-emptively created (YAGNI); decided by the audit's own coherence test.
- **F3 — Objective 1 "no overlaps/gaps" was unfalsifiable.** *Resolved:* the
  audit retains a resolved-entry ledger; VA checks it (§9). Coherence is now
  evidenced, not asserted.
- **F4 — restate-line disposition had no named home.** *Resolved:* per-skill
  dispositions (fixed / compliant) live in slice notes; only genuine
  *architectural* exceptions (a skill whose design needs inline flag reference)
  escalate to an **ADR-005 review finding** (RV), per closure intent. Slice
  notes for the pass; RV for the exceptions.
- **F5 — boot-footer deletion could strand a reference.** *Resolved:* added a
  grep gate to §9 (no `boot-footer` survives in src/tests/manifest/skills).
- **F6 — model-band.md conflated with a user hook.** *Resolved:* reframed in
  §5.2 as a **shipped constant** (universal floor), distinct from the genuinely
  user-authored `governance.md`; per-model content belongs in the cascade.

Residual open items: OQ-3 (review-ledger.md tier) — resolve in the IA pass.

### External review (codex / GPT-5.5, 2026-07-03)

Hostile pass focused on the IA responsibility boundaries. 6 HIGH + 7 MED/LOW,
convergent and strengthening — none fatal. All integrated:

- **A — using-doctrine.md concentrates 4 fact-classes (3 HIGH converge).**
  *Integrated:* §5.1 ownership matrix as a **pre-execution** deliverable (D6),
  with a hard pre-commit to split `hooks.md` if fact-classes fail a
  single-invariant test. Not deferred to a self-graded audit.
- **B — agent-judged coherence ledger = marking own homework.** *Integrated:*
  §9 VA now reconciles three **externalised artefacts** (ownership matrix,
  doc→fact inventory, pointer graph); the ledger is their reconciliation record,
  re-derivable by a second reviewer. Comparison basis fixed (fact-unit; closed
  corpus).
- **C — "Tier 3 hymns, mapped not owned" is incoherent.** *Integrated:* §5.1
  demotes hymns from a tier to an **adjacent cross-referenced mechanism** —
  removes the shared-responsibility fog D2 exists to avoid.
- **D — reachability ignores freshness; "pointed-at" is fuzzy.** *Integrated:*
  §5.3 splits **distribution reachability** (this slice, fresh-install) from
  **semantic currency** (stale client copies under write-if-absent — named,
  scoped out, → IDE-030); "pointed-at" tightened to a **machine-checkable
  pointer class** (explicit edge from boot/skill/manifest).
- **E — no restate-line regression control.** *Integrated:* §5.4 step 4 + D5 add
  a cheap grep/`just` **structural tripwire** (not a Rust linter — respects
  no-CLI-code).
- **manifest "reachability oracle" overclaim; OQ-3 avoided; currency=mention.**
  *Integrated:* renamed **shipping oracle** (§5.1); OQ-3 **resolved** by the
  decide-now multi-skill criterion (review-ledger = Tier-1); currency VA now
  checks **unique ownership**, not mere presence (§9).
- **model-band in the hook cluster = category error.** *Integrated:* §5.2 type
  split — user-editable hooks vs shipped-inputs-consumed-by-boot.

Not adopted as slice work: the reviewer's push for client-repo update semantics
(D/MED) — correctly out of a docs-IA slice; captured as IDE-030, claim scoped to
fresh-install.

Residual open items: none — OQ-3 closed; OQ-1/OQ-2 already resolved.
