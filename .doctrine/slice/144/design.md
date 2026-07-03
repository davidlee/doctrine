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

The IA is a **four-tier access model** over the ship surface. Each tier answers
"when does an agent encounter this content?" — the ADR-005 thesis extended with
the ADR-011 cascade as a first-class, separately-governed tier.

```
TIER 0 — PUSH (boot-resident, every session, no invoke)
  routing-process.md   workflow digest + guardrails + reference-forms (R-OQ-5)
  governance.md        user governance pointer      [boot GOVERNANCE_REL]
  model-band.md        universal model-band floor   [boot Static]
  (onboarding memory)  seeded from templates/seed-onboarding.md → boot Onboarding
        │  carries LOAD-BEARING rules only; points at Tier 1 for detail
        ▼
TIER 1 — PULL-reference (shipped docs; read when Tier 0 or a skill points)
  glossary.md          vocabulary · ids · reference forms · verification taxonomy
  using-doctrine.md    which verb for intent · storage tiers · read-via-show · edit rules
  review-ledger.md     adversarial review protocol
        │  each MUST be pointed-at (skill or boot) AND in the manifest ship set
        ▼
TIER 2 — Skills (thin routers, plugins/doctrine/skills/*/SKILL.md)
        │  name a verb, cite a rule by name; never reproduce flags/tables (R-OQ-4)
        ▼
TIER 3 — Hymns cascade (context-keyed PULL; ADR-011/SPEC-023) — MAPPED, not owned
  install/hymns/ (framework) + .doctrine/hymns/ (user overlay)
  resolved by `doctrine prompt resolve` on (harness, model, role, stage, project)

REACHABILITY ORACLE — manifest.toml
  [dirs] [memory] [gitignore] [hymns seal/expose] [root_markers]
  a doc ships iff install.rs embed-copies it; a hymn ships iff manifest exposes it
```

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
precedence · reset**. The contracts live in `using-doctrine.md` (the CLI/editing
reference — its rightful home), with a one-line pointer from the boot digest
where the hook is boot-resident.

| Hook | Controls | Mechanism | Precedence | Reset |
|---|---|---|---|---|
| `.doctrine/governance.md` | project governance pointer text in boot | boot reads `GOVERNANCE_REL`, injects as `## Governance (project)` | user text replaces the seeded default wholesale | delete → boot shows marker; re-install re-seeds |
| `install/model-band.md` → `.doctrine/…` | universal model-band floor directive | boot `Static` source → `## Model band` | shipped default; user edits the installed copy | re-install / restore from `install/` |
| `.doctrine/hymns/` overlay | per-context prompt bands (harness/model/role/stage/project) | `doctrine prompt resolve` merges framework + overlay | **overlay wins** over framework at equal key (SPEC-023) | remove overlay file → framework band resolves |
| onboarding seed | boot `## Onboarding` body | `manifest.toml [memory]` seeds `mem.signpost.project.orientation` from `templates/seed-onboarding.md`; onboarding-tagged memory bodies render | edit the seeded memory (or add onboarding-tagged memories, key-ordered) | `memory verify`/edit; re-install skips existing key |

**Retirement — `boot-footer.md`:** delete `install/boot-footer.md` and the
orphan `.doctrine/boot-footer.md`. Contract: none — the file is dead post-SL-187.
The audit verifies no `src/` read path, no skill pointer, no manifest reference
survives (all confirmed absent at design time). Hymns internals
(`traits:` frontmatter, band layout) are **not** documented here — cross-ref to
SPEC-023 / SL-191.

### 5.3 Data, State & Ownership — reachability contract (Objective 6)

**Reachability = shipped ∧ pointed-at.** A doc is reachable iff both hold:
1. **Shipped** — `install.rs` embed-copies it into `.doctrine/` (for `.md`
   docs) or `manifest.toml` exposes it (for hymns: `[hymns] expose`). A doc in
   `install/` that the embed set skips ships to nobody.
2. **Pointed-at** — a skill or the boot digest names it, so an agent has a path
   to open it (the AGENTS.md lesson: an unreferenced `.doctrine/*.md` is read by
   no one).

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

1. **IA audit pass** — walk the `install/*.md` set + manifest, produce the
   overlap/gap/contradiction ledger against the responsibility boundaries (a
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
   Evidence-bound; record the disposition per skill.
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
- **OQ-2 — does `install.rs` copy every `install/*.md`, or an allowlist?** To
  confirm in the reachability pass by reading the embed logic; shapes whether
  "ship" is automatic or needs a manifest entry. Low risk — either way the
  audit closes the gap.
- **OQ-3 — review-ledger.md tier.** Is it Tier-1 PULL-reference (a doc skills
  point at) or effectively skill-owned by review/inquisition? Resolve in the IA
  pass; affects which skill owns its pointer.

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
- **D5 — Restate-line: grep-assisted manual triage, evidence-bound.** 8/30
  candidates, manual disposition. Alt: build a permanent lint check — deferred
  as optional follow-up (not a deliverable; over-engineering for 8 candidates).

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
  `model-band.md` present in `.doctrine/`; `boot-footer.md` **absent**.
- **VA** — restate-line: the 8 candidate skills triaged; each disposition
  recorded (fixed offender / compliant pointer); no skill reproduces a
  command/flag table.
- **VA** — reachability: every Tier-0/1 doc is shipped ∧ pointed-at; zero
  orphans; the reachability contract is documented in using-doctrine.md.
- **VA** — currency: glossary + using-doctrine cover all current kinds/verbs
  (spot-check REC/REV/POL/STD/knowledge each named).

## 10. Review Notes
