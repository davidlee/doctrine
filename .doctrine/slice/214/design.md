# SL-214 — Knowledge authoring skill: design

Status: approved sections D1–D5 (session 2026-07-11). Origin IMP-182; RFC-007
ws3 ("populate — make gating bite").

## Current vs target behaviour

**Current:** the knowledge-record surface ships (SPEC-019; seven kinds since
SL-159: ASM/DEC/QUE/CON/EVD/HYP/CPT) but no skill routes capture intent to it.
Census 2026-06-26: zero records, zero `shapes` edges. RFC-008/SL-158's gating
(trinary actionability, ADR-017) has nothing to gate on.

**Target:** a `/knowledge` skill routes capture intent to the CLI at the moments
capture actually arises (design forks, consults, preflight), teaches the
association-vs-gating distinction, and the corpus gains its first real records
(dogfooded from this slice's own decisions).

## D1 — Placement

`plugins/doctrine/skills/knowledge/SKILL.md` — new skill dir inside the existing
`doctrine` plugin (sibling of `backlog`). No `.claude-plugin/marketplace.json`
change: registration is per-plugin, not per-skill; `discover()` collects skill
dirs. Freestanding-plugin and fold-into-`/record-memory` options rejected
(DEC: core-loop capture deserves a first-class route; memory is the
pointer/recipe layer, records are typed epistemic entities — folding muddies
both boundaries).

## D2 — SKILL.md content

Modeled on `plugins/doctrine/skills/backlog/SKILL.md` (thin router; ADR-005:
skills route, reference docs explain). POL-002 binds: no repo-local couplings.

- **Frontmatter `description` = trigger surface** (mem.pattern.skill.description-is-the-trigger):
  capture-intent phrases — "we're assuming/proceeding as if…", "decided that…",
  "open question…", "bound by…", "evidence for/against…", "hypothesis…" — plus
  settle/survey intent.
- **Four-homes one-liner** + citation of `using-doctrine.md` § Which home for
  which record (backlog = work intent; knowledge_record = epistemic/governance;
  ADR = architectural decision; memory = agent guidance).
- **Seven-kind picker table** — trigger phrase → kind:
  | phrase shape | kind |
  |---|---|
  | "proceeding as if X" (unvalidated premise) | ASM |
  | "we chose X over Y" (scoped/operational) | DEC |
  | "unresolved: X?" (needs an answer) | QUE |
  | "bound by X" (externally imposed limit) | CON |
  | "observed: X" (citable observation) | EVD |
  | "testable claim: X" (awaiting evidence) | HYP |
  | "stable mental model / term: X" | CPT |
- **DEC-vs-ADR altitude line:** project-global + architectural consequences →
  `doctrine adr new`; scoped/operational → DEC. (Deeper guidance is IDE-007,
  out of scope.)
- **Intent→verb table:** capture → `knowledge new <kind> [title]`; survey →
  `knowledge list`; read → `knowledge show <ID>`; settle → `knowledge status
  <ID> <STATE>`; relate → `link <REC-ID> shapes|spawns <TARGET>`; evidentiary →
  `link EVD-n supports|disputes <REC-ID>`; replace → `doctrine supersede`
  (SPEC-019 cross-kind supersession). CLI is source of truth for flags and
  per-kind status vocabularies — the skill does not enumerate them.
- **Wrong-home cross-pointers** (adversarial finding 1): work intent →
  `/backlog`; reusable agent guidance/recipe/gotcha → `/record-memory`.
  Discriminator: a record is a citable epistemic *entity* in the relation
  graph (relatable, gateable, supersedable); a memory is retrieval-layer
  guidance. CPT vs memory-`concept` rides this line — one discriminator
  sentence only; deeper guidance out of scope (IDE-007-adjacent).
- **Gating section (the one non-obvious teaching, ADR-017):** association ≠
  gating. `shapes` records influence, never blocks. To gate work on an
  unsettled record, the *dependent* authors the edge: `doctrine needs SL-x
  QUE-1`. Settling the record (`knowledge status` → terminal) unblocks — no
  unlink needed. Records never author dep/seq themselves.
- **Rules:** id prefix resolves kind on read/transition; capture seeds the
  kind's default state (held/proposed/open/active/…); don't hand-edit record
  TOML — use the verbs; prose body is hand-edited.

## D3 — Capture touchpoints

One-to-two-line pointers in three shipped skills — prompts at the moments
records are born; no template or harvest logic (that is SL-215's territory:
SL-214 owns during-work capture, SL-215 owns end-of-work harvest and will
consume this skill as its knowledge/decision sink).

- `plugins/doctrine/skills/design/SKILL.md` — in the clarifying-question loop:
  unresolved question worth outliving the session → QUE; locked design choice →
  DEC; assumption carried into the design → ASM. Point at `/knowledge`.
- `plugins/doctrine/skills/consult/SKILL.md` — consult outcome: tradeoff
  resolved → DEC; still open → QUE (+ `needs` gate on the blocked work).
- `plugins/doctrine/skills/preflight/SKILL.md` — before starting: check
  inbound gating records on the target surface; capture assumptions being made.

## D4 — Routing row + reference amendments

- Row in `install/routing-process.md`: epistemic capture / settle intent →
  `/knowledge`. **Sequenced after the skill is installable** (ADR-009 F14 —
  never route to shipped-not-reachable; install → row → `doctrine boot`).
- One sentence added to `using-doctrine.md`'s knowledge_record bullet naming
  the gating model (inbound `needs` on an unsettled record, ADR-017).
- `glossary.md` already current (seven kinds + lifecycle vocab) — no change.

## Distribution mechanics (implementation note)

RustEmbed no-rerun gotcha: `touch src/install.rs && cargo build` (embed root
`#[folder = "plugins/"]` lives in `src/install.rs` since IMP-226 removed
`src/skills.rs`), then install
from the rebuilt binary (`./target/debug/doctrine install -s knowledge -y` or
full install), then routing row, then `doctrine boot`. Source of truth is
`plugins/` only; never edit `.doctrine/skills/` or `.agents/skills/` copies.

## Verification alignment

- **VA — POL-002 reflex, concrete:** grep the new/edited skill files for
  `just `, repo-local paths, and branch names; `comm -23` any `[[mem.…]]` keys
  against the shipped `memory/` corpus keys. Zero hits.
- **VA — dogfood capture, cross-slice-reach only** (adversarial finding 2 —
  don't teach record-spam from record #1): one DEC (the SL-214/SL-215
  capture-vs-harvest boundary; `link DEC-n shapes SL-215`) + one ASM
  (touchpoint pointers drive population; validation plan = re-census after
  SL-215 lands). Census non-zero at close.
- **VH — sequencing:** routing row only after install; `doctrine boot --check`
  clean; skill visible in installed skill list.

## Code impact (design-target)

| path | change |
|---|---|
| `plugins/doctrine/skills/knowledge/SKILL.md` | new — the skill |
| `plugins/doctrine/skills/design/SKILL.md` | +2 lines, capture pointers |
| `plugins/doctrine/skills/consult/SKILL.md` | +2 lines, capture pointer |
| `plugins/doctrine/skills/preflight/SKILL.md` | +2 lines, gating check + capture pointer |
| `install/routing-process.md` | +1 routing row |
| `install/using-doctrine.md` | +1 sentence, gating model |

No Rust code changes. `src/install.rs` is touched (mtime only) to force
re-embed — not a content change, not a design target.

## Decisions

- D1: standalone skill in `doctrine` plugin (over freestanding / fold-in).
- D2: compact router content; CLI + reference docs carry depth (ADR-005).
- D3: three touchpoints (`/design`, `/consult`, `/preflight`); harvest-side
  integration deferred to SL-215 by explicit boundary.
- D4: no new reference doc; amendments only.
- D5: dogfood verification — this slice authors the corpus's first records.

## Open questions

- OQ-1: exact frontmatter description wording — tune at implementation against
  the description-is-the-trigger pattern.
- OQ-2: whether `/route`'s digest also needs a one-word mention or the routing
  row suffices — resolve when editing `routing-process.md` (lean: row suffices;
  boot snapshot inlines it).
