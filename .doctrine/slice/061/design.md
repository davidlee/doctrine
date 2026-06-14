# Design SL-061: Rewire the review skills onto the RV ledger via a shared protocol doc

> Governed by ADR-007 (RV kind). Completes the fan IMP-001 piloted on `/audit`
> (IMP-023). Pure docs + plugin-restructure: one shared reference doc, three
> SKILL.md rewrites, one plugin retirement; `src/` touched in tests only.

## 1. Design Problem

`/audit` is on the RV ledger (the IMP-001 pilot); `/code-review` and
`/inquisition` are not — they emit free-form prose and a gitignored
`inquisition.md` respectively, gaining none of the ledger's append-only finding
identity, turn-graph dispositioning, severity/close-gate teeth, or warm
reviewer-context cache (ADR-007 D-C0). Rewiring them naively would produce a
third and fourth divergent copy of the RV-driving prose `/audit` already carries.

The design must (a) move both skills onto the ledger, (b) without three copy-pasted
variants of the protocol, and (c) resolve two structural gaps the rewiring exposes:
inquisition has no natural facet (it is a posture, not a lifecycle stage), and both
skills frequently run with no governing entity to target (the RV `--target` is a
validated canonical ref — `integrity::ensure_ref_resolves`).

## 2. Current State

- **`/audit`** (`plugins/doctrine/skills/audit/SKILL.md`) — drives the RV ledger
  inline: open `reconciliation` RV → `prime` → `raise` → `dispose` → `verify`/
  `contest`/`withdraw` → `## Synthesis`. The proven prior-art shape. Carries the
  disposition vocab (aligned/fix-now/design-wrong/follow-up/tolerated), the
  close-gate teeth, the `--as` self-audit guidance, the `--note` ephemerality caveat.
- **`/code-review`** (`plugins/review/skills/code-review/SKILL.md`) — a **standalone
  `review` plugin**. Embittered-staff-eng persona; emits prose findings with emoji
  severity labels (`👍🔴🟠🟡🔵`) and an `Overall/Synopsis/Findings/Haiku` framing.
  No ledger.
- **`/inquisition`** (`plugins/doctrine/skills/inquisition/SKILL.md`) — Inquisitor
  persona; writes `inquisition.md` (gitignored) with `Charges/Questions/Judgement/
  Sentencing`. No ledger.
- **Plugin layout** — single-skill standalone plugins (`review`, `handover`) sit
  beside doctrine core; `doctrine-memory`/`doctrine-partner` re-export core skills
  via **symlink** (`skills/pair -> ../../doctrine/skills/pair`). Embed seam is
  `#[folder = "plugins/"]` in `src/skills.rs` (location-transparent). A skill's
  `domain` derives from its plugin dir; `src/skills.rs` tests pin code-review's
  domain as `"review"`.
- **RV verb surface** (`doctrine review`, SL-040): `new/raise/dispose/verify/
  contest/withdraw/status/prime/unlock`. Single-tree, parent-locus; verbs refuse a
  fork-resolved root (IMP-024 not available). Facet enum closed 7-set
  (`src/review.rs:38`). Severity `blocker|major|minor|nit`; only `blocker` gates
  close (D-C9b).

## 3. Forces & Constraints

- **C1 — `--target` is a validated canonical ref.** `review new` refuses a
  non-resolving target. An RV exists only against an existing numbered entity. No
  raw diff / PR / sentinel target without a protocol change (out of scope).
- **C2 — backlog kinds are valid targets.** `ISSUE/IMPROVEMENT/CHORE/RISK/IDEA`
  are in `integrity::KINDS` (stem `backlog`) → `IMP-023`, `RSK-004` etc. resolve.
  A backlog item is a first-class proximate subject.
- **C3 — facet enum is closed + lifecycle-shaped.** Its 7 members are lifecycle
  *aspects*. Adding a posture member is a category error (and `src` churn).
- **C4 — non-goals (ADR-007 carve-outs).** No `drift` facet (IMP-022, the Drift
  Ledger kind). No reconciliation-seam work (IMP-008). No new verbs / coordination
  (IMP-024). No RV e2e goldens (IMP-029).
- **C5 — behaviour-preservation gate.** `/audit` is the working pilot; re-sourcing
  its mechanics onto a shared doc must not regress it.
- **C6 — SKILL.md edits re-embed.** `doctrine claude install` + touch the embed
  crate (`mem…skill-refresh-command`). Frontmatter is a YAML scalar (no
  colon-space, no double-quotes — `mem…skill-frontmatter-yaml`).
- **C7 — RV verbs refuse a fork-resolved root** (`mem…rv-verbs-refuse-on-worktree-fork`)
  — drive reviews from the parent tree.

## 4. Guiding Principles

- **Mechanics vs lens.** The RV-driving protocol is invariant across reviews; the
  persona, the review lens, and the output voice diverge. Factor on that seam.
- **One source, N consumers.** Lift the protocol once; all review skills point at it.
- **Posture is orthogonal to facet** (ADR-009 conduct axis; boot: "pairing/walkthrough
  are postures, orthogonal to the stage"). Carry posture in a role label, not the enum.
- **Steer toward governance, degrade only as last resort.** Prefer a proximate typed
  subject; create one when the work is durable; prose only for genuine throwaway.
- **No `if/else` modes in our skills.** A doctrine skill assumes doctrine.
- **Write less code.** Prefer prose + existing seams over new `src`.

## 5. Proposed Design

### 5.1 System Model

A new shared reference doc, **`review-ledger.md`** (install-wired, auto-shipped to
`.doctrine/`), owns the mechanical RV-driving protocol. The three review skills
(`/audit`, `/code-review`, `/inquisition`) each collapse to **persona + lens +
ledger-pointer + skill-specific tail**, referencing the doc the way nine skills
already reference `using-doctrine.md`.

```
                     review-ledger.md   (mechanics: target ladder, verb
                     ┌──────────────┐    sequence, vocab, close-gate, harvest)
                     └──────┬───────┘
          ┌─────────────────┼─────────────────┐
   /audit (lens:      /code-review (lens:   /inquisition (lens:
    conformance,       craft pathologies,    heresy hunt, Latin
    closure tail)      emoji→severity)       zeal, facet-by-target)
```

### 5.2 Interfaces & Contracts — the mechanics/lens seam

**`review-ledger.md` owns (invariant):**

| Concern | Detail |
|---|---|
| Target ladder | slice/phase → backlog item → **create** one (`backlog new`) → prose last-resort. Steer toward a proximate typed subject. |
| Facet selection | by what you interrogate (the subject's aspect). Posture via `--raiser <label>`. |
| Verb sequence | `review new --facet F --target REF [--phase P] [--raiser L]` → `prime --seed` → curate `domain_map` → `prime` → seed `## Brief` → `raise` → `dispose --as responder` → `verify`/`contest`/`withdraw --as raiser`. |
| Severity vocab | `blocker\|major\|minor\|nit`; raiser-owned, append-only; only `blocker` gates *target* close. |
| Disposition vocab | aligned / fix-now / design-wrong / follow-up / tolerated. |
| Caveats | `--note` is ephemeral baton chatter, not durable rationale; self-review drives both roles via `--as`; loose notes are insufficient — findings live in the ledger. |
| Synthesis + harvest | append `## Synthesis`; then a thin harvest pointer → `using-doctrine.md` work/knowledge/decision boundary (notes/memory/backlog). |
| Done + guards | review-done = every finding terminal (D-C9a); blocker gates target close (D-C9b); drive from the parent tree (C7). |

**Each skill owns (divergent):** persona/voice · review lens (what to hunt) ·
idiomatic severity labels mapped *onto* the shared axis · output framing (which
lands in `## Synthesis`, with findings `raise`d) · stage-specific tail.

**`review-ledger.md` skeleton:** Intro · §1 Pick the subject · §2 Open + prime ·
§3 Raise findings · §4 Dispose + resolve · §5 Synthesis + harvest · §6 Done +
close-gate.

**The ledger-vs-prose trigger (A1 — the doc's §1 must state this crisply).** Drive
the ledger when the review is **closure-grade**: it gates a lifecycle move, runs
adversarially across more than one round, hands off between agents, or its findings
must outlive the conversation. Use prose only for a genuinely throwaway one-shot
read with no durable subject. The cost asymmetry is the test — opening an RV +
`raise`/`dispose` per finding earns its keep exactly when finding identity,
turn-graph dispositioning, or the close-gate matter; for a quick eyeball it is pure
ceremony. Each skill restates this trigger in its own voice; the doc owns the
definition.

**Harvest is judgment-gated (A2).** §5 promotes durable findings to
notes/memory/backlog *when they exist* — not a mandatory step. A clean review
harvests nothing.

**The ladder is narrowed per skill (A3).** The doc presents the full ladder; each
skill's lens pins which rungs apply. `/audit` always targets its slice (rung 1) —
it never degrades to prose. `/code-review` and `/inquisition` walk the whole ladder.
This is what keeps INV-3 (audit-unchanged) true.

### 5.3 Data, State & Ownership

No new data shapes. RV `review-NNN.{toml,md}` is unchanged. The skills produce
ledger entries via existing verbs; no schema, no enum, no migration. Posture is
carried by the existing `--raiser` label field (e.g. `inquisitor`). `inquisition.md`
is retired for new inquisitions (existing files stay valid, like `audit.md`); the
manifest's gitignore of `inquisition.md` remains as harmless legacy.

### 5.4 Lifecycle, Operations & Dynamics — the three skills

- **`/audit` (refactor, behaviour-preserving).** Keep: reconciliation persona,
  audit modes (conformance/discovery), reconciliation-scope caveat, evidence
  gathering (run tests + `just check`), closure tail (`slice status reconcile` →
  `/close`), `--as` self-audit framing. Remove (now in doc): the inline verb
  mechanics, disposition vocab, prime flow, close-gate prose.
- **`/code-review` (relocate + rewrite).** Move `plugins/review/skills/code-review/`
  → `plugins/doctrine/skills/code-review/`; **retire the standalone `review`
  plugin** (drop from `marketplace.json`, delete dir) — the rewired skill
  hard-depends on doctrine, so a standalone install is incoherent. Keep: persona,
  focus axes, the `Context → High-Level → Line-by-Line → Summary` lens, the emoji
  labels. Redirect: facet `code-review` always; subject via the ladder; each
  finding → `raise` (`🔴→blocker · 🟠→major · 🟡→minor · 🔵→nit`; `👍` → synthesis);
  `Overall/Synopsis/Haiku → ## Synthesis`. Drop the non-doctrine bimodal branch.
- **`/inquisition` (rewrite).** Keep: Inquisitor persona, Latin/zealot voice
  mandate, the Procedure. Redirect: facet **by target-aspect**; `--raiser
  inquisitor`; each Charge → `raise` (sentencing gravity → severity);
  `Judgement + Sentencing → ## Synthesis`; `Questions` → Brief/synthesis. Retire
  `inquisition.md`.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1** — every finding is a `raise`; every narrative is `## Synthesis`. No
  skill writes findings to prose or a side file.
- **INV-2** — facet always names a lifecycle aspect, never a posture.
- **INV-3** — `/audit` observable behaviour is unchanged by the extraction.
- **ASM-1** — the existing `review` verb family suffices for all three skills'
  finding lifecycle (no new verb). Carried from scope.
- **Edge — no governing entity.** Walk the ladder: create a backlog item if the
  work is durable; prose only if genuinely ephemeral. Not a CLI-presence branch.
- **Edge — multi-aspect inquisition.** One RV = one facet = one aspect. Hitting
  design *and* impl is two RVs, or pick the dominant aspect (keeps D-C11's
  single-subject thesis true).
- **Edge — code-review in a non-doctrine repo.** Out of scope by D8: the skill is
  doctrine-native; generalist code-review skills exist elsewhere.
- **Edge — backlog item as a code-review/inquisition subject (A4).** Coherent but
  thin: a backlog item is intent, not code. The RV `[target]` is the *locus*; the
  actual code evidence lives in each finding's `detail`. The diff is reviewed; the
  item is what the review is filed against.
- **Edge — the ledger-vs-prose trigger is judgment, not a flag.** A skill never
  branches on CLI presence; it decides closure-grade vs throwaway per §5.2's trigger.

## 6. Open Questions & Unknowns

All three scope OQs are resolved:

- **OQ-1 (inquisition facet)** → facet-by-target + `--raiser` posture (D2). No enum.
- **OQ-2 (slice-less target)** → the target ladder, prose last-resort (D1).
- **OQ-3 (one slice vs split)** → one slice, zero production `src`; collapsed by D2/D5.

No open unknowns gate planning. Residual verify-time check: the installer copies
top-level `install/*.md` implicitly (no `[files]` manifest entry) — confirm at P1.

## 7. Decisions, Rationale & Alternatives

- **D1 — target ladder** (slice/phase → backlog → create → prose). *Alt:* always
  require a target (rejected: amputates the "review this diff" use); synthetic
  ambient target (rejected: protocol change, violates D-C11/non-goals).
- **D2 — facet-by-target, posture via `--raiser`.** *Alt:* new `inquisition` facet
  (rejected: posture≠aspect category error, enum/`src` churn for a query
  convenience — `mem…product-not-compromised`); fixed facet (rejected: wrong for
  design/scope targets).
- **D3 — shared `review-ledger.md`.** *Alt:* inline-consistent prose per skill
  (rejected: the three-divergent-copies the scope warns against).
- **D4 — migrate all three skills onto the doc** (`/audit` included; user relaxed
  the non-goal). Behaviour-preserving for `/audit`; the proven pilot is the
  extraction's fidelity test.
- **D5 — one slice, zero production `src`** (OQ-3 collapsed).
- **D6 — §5 harvest is a thin pointer**; the cross-corpus harvest DRY
  (`/notes`,`/handover`,`/next` + reviews all re-implement it) → backlog.
- **D7 — relocate code-review into doctrine core; retire the `review` plugin.**
  *Alt:* symlink re-export (rejected: advertises a doctrine-less install that
  breaks).
- **D8 — drop code-review's non-doctrine bimodal branch.** Doctrine-native skill.

## 8. Risks & Mitigations

- **R1 — `/audit` regression from re-sourcing.** *Mitigation:* the doc must absorb
  every mechanic `/audit` relies on; smoke audit end-to-end at P1; anything missing
  reveals an incomplete extraction.
- **R2 — over-extraction makes skills read as "go read the other doc".** *Mitigation:*
  skills keep enough lens/voice to stand as coherent instructions; the pointer
  pattern is well-worn (`using-doctrine.md` × 9). Each skill remains self-narrating
  for its lens; only the mechanics defer.
- **R3 — relocation breaks the embed/install/discover suites.** *Mitigation:* update
  the `src/skills.rs` tests that pin the old `review` domain; `just gate` green;
  marketplace integrity checked.
- **R4 — re-embed forgotten → stale installed skills.** *Mitigation:* `doctrine
  claude install` + touch `src/skills.rs` in the close phase; the relocation edits
  `skills.rs` anyway.

## 9. Quality Engineering & Validation

Docs + restructure slice — verification is behaviour-preservation + smoke, not new
unit tests (skills are prose; inventing goldens here is theatre — RV e2e goldens
are IMP-029).

- **VT (existing suite)** — `src/skills.rs` discover/embed/`claude_links` stay green;
  domain assertion updated `review→doctrine`; `just gate` clean, clippy zero.
- **VA (by agent) — per-skill smoke** — each skill opens its RV (code-review→
  `code-review` facet; inquisition→facet-by-target+`--raiser`; audit→
  `reconciliation`), raises + disposes + resolves a finding, renders `## Synthesis`.
- **VA — `/audit` parity** — the smoke audit exercises every migrated mechanic.
- **VA — marketplace integrity** — `review` removed; code-review discoverable under
  doctrine; no dangling plugin reference.
- **Closure** — IMP-023 updated (drift→IMP-022, reconcile→IMP-008 reassignment
  noted) and closed when the in-scope skills land; backlog minted for harvest-DRY
  and handover relocation.

### Provisional phases (firm at `/plan`)

| P | Work | Verify |
|---|---|---|
| P1 keystone | author `review-ledger.md` + migrate `/audit` | smoke audit end-to-end; doc absorbs every `/audit` mechanic |
| P2 | relocate + rewrite `/code-review`; drop bimodal; retire `review`; update `marketplace.json` + `skills.rs` tests | `just gate`; code-review smoke; marketplace integrity |
| P3 | rewrite `/inquisition` (facet-by-target, `--raiser`, retire `inquisition.md`) | inquisition smoke |
| close | re-embed; close IMP-023; mint backlog (harvest-DRY, handover relocation) | `just gate`; IMP-023 updated |

P1 is keystone (P2/P3 reference the doc). P2/P3 file-disjoint after P1 —
parallelizable; serial is acceptable for a slice this small (a `/plan` call).

## 10. Review Notes

### Internal adversarial pass (integrated)

- **A1 — ledger-vs-prose threshold was hand-waved.** Risk: every code-review
  becomes ceremony, or all degrade to prose (rewiring = theatre). *Integrated:*
  §5.2 now defines the **closure-grade trigger** (gates a lifecycle move / multi-round
  / agent-handoff / findings must outlive the conversation); the doc owns it, each
  skill restates it in voice.
- **A2 — harvest framed as always-on.** Risk: ceremony on quick reviews.
  *Integrated:* §5.2 — harvest is judgment-gated (promote durable findings *when
  present*).
- **A3 — target ladder not uniform across skills.** Risk: doc implies `/audit`
  could drop to prose, breaking INV-3. *Integrated:* §5.2 — the doc presents the
  full ladder; each skill's lens pins applicable rungs (`/audit` = rung 1 always).
- **A4 — backlog item as a thin code-review subject.** *Integrated:* §5.5 edge —
  the item is the locus, code evidence lives in finding `detail`.
- **Dismissed (verified):** installed agent symlink is by skill-id
  (`.doctrine/skills/code-review`), unchanged by source relocation — existing
  installs survive D7. The `review` plugin's "security review" skill never existed.
  `--raiser` posture is a recorded authored field though not filterable — D2
  consciously traded queryability away.

_(external adversarial pass / `/inquisition` — offered to the user; pending.)_
