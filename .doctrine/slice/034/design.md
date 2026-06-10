# Design SL-034: doctrine-partner skill subset and route comprehension/posture provision

<!-- Reference forms (.doctrine/glossary.md § reference forms): entity ids padded
     (SL-020, REQ-059, ADR-004); doc-local refs bare — OQ-1 (§6), D1 (§7),
     R1 (§10), Q1. -->

## 1. Design Problem

Two collaboration skills authored this session — `pair` (calibrated adversarial
pair programming) and `walkthrough` (expertise-reversal-aware comprehension +
critique) — need a permanent home, and `/route` needs to learn two things it
currently cannot express:

1. a **comprehension intent** with no change ("walk me through this PR") — today
   it has no row, so a route-obeying agent mis-files it; and
2. that pair/walkthrough are **conduct postures** orthogonal to the governance
   stage, not stages themselves.

Closing (1)/(2) means the embedded routing digest (`install/routing-process.md`)
references `/pair` and `/walkthrough`. Route today names only doctrine-plugin
skills; a hard reference to a *separate* marketplace plugin dangles when that
plugin is uninstalled. Resolution: make the skills part of the **doctrine
plugin** and ship a `doctrine-partner` standalone subset — mirroring the existing
`doctrine-memory` precedent exactly — so core may assume they are installed.

Out of scope: skill *content* changes (settled this session); a `--only-partner`
flag; any new ADR; the pre-existing boot drift.

## 2. Current State

- **Skills.** Authored as an interim freestanding plugin `plugins/partner/`
  (uncommitted: `plugins/pair` was `git mv`'d → `plugins/partner`, both
  `SKILL.md`s written, `marketplace.json` edited to a `partner` entry). This
  whole interim shape is folded/reset by this slice — net diff has no
  `plugins/partner/`.
- **doctrine-memory precedent (the template).**
  - Canonical skill source: `plugins/doctrine/skills/{record-memory,retrieve-memory}/`.
  - Subset plugin: `plugins/doctrine-memory/` = `.claude-plugin/plugin.json` +
    `README.md` + `skills/<id>` **symlinks** → `../../doctrine/skills/<id>`.
  - Marketplace entry describes it as "standalone subset … install one or the
    other, not both".
  - `src/skills.rs`: `MEMORY_SUBSET_DOMAIN = "doctrine-memory"` (`:31`) is the
    sole member of `MARKETPLACE_ONLY_DOMAINS` (`:36`). `discover()` drops those
    domains (`:129`) so the symlink-duplicated embed entries never collide on
    skill id. Test `discover_excludes_marketplace_only_domains` (`:883`).
- **Embed.** `#[derive(RustEmbed)] #[folder = "plugins/"]` (`skills.rs:24`).
  RustEmbed **follows symlinks**, so each subset skill is emitted twice — once
  under `doctrine/skills/…`, once under `doctrine-partner/skills/…`. The
  discovery exclusion is what makes that safe.
- **Routing digest.** `install/routing-process.md` is embedded and projected
  wholesale into the boot snapshot's "Routing & Process" section
  (`boot.rs:82`, `SourceKind::Static`). The full route table also lives, in
  prose form, in the `route` **skill** (`plugins/doctrine/skills/route/SKILL.md`)
  — a parallel surface (§5.2).
- **Boot golden.** *Presence checks only* — `digest.body.contains("Route before
  you act")` (`boot.rs:945`) and `snap.contains(…)` (`boot.rs:1291`). **No
  verbatim sentinel** over the routing table → adding rows is golden-safe
  (resolves prior OQ-1).

## 3. Forces & Constraints

- **ADR-005** (shipped-knowledge tiering: skills route, reference docs explain) —
  pair/walkthrough are routing skills; the doctrine domain is their correct home.
- **No drift across siblings** — doctrine-partner must structurally match
  doctrine-memory or the `MARKETPLACE_ONLY_DOMAINS` model fragments.
- **Re-embed footgun** (`mem.pattern.distribution.skill-refresh-command`) — a
  lone `plugins/` edit does not re-embed; `src/skills.rs` must recompile.
- **Behaviour-preservation gate** — the skills/boot suites must stay green; only
  the deliberately-extended discovery test changes.
- **Storage rule** — slice scope/design prose only; no queried data in MD.

## 4. Guiding Principles

- Mirror the precedent faithfully; do not invent a second mechanism.
- One coherent shippable change; resist scope creep into `--only-partner`.
- Make the route additions minimal and self-explaining.

## 5. Proposed Design

### 5.1 System Model

```
plugins/
  doctrine/skills/
    pair/SKILL.md          ← canonical source (moved here)
    walkthrough/SKILL.md   ← canonical source (moved here)
  doctrine-partner/        ← standalone subset (new)
    .claude-plugin/plugin.json
    README.md
    skills/
      pair        -> ../../doctrine/skills/pair        (symlink)
      walkthrough -> ../../doctrine/skills/walkthrough  (symlink)
  partner/                 ← REMOVED (interim)
```

`discover()` walks `<domain>/skills/<skill>/`; the `doctrine` domain is not
excluded, so `pair` + `walkthrough` automatically enter the CLI catalog under
domain `doctrine`. The duplicate `doctrine-partner/skills/…` embed entries are
dropped because `doctrine-partner ∈ MARKETPLACE_ONLY_DOMAINS`. Identical shape to
doctrine-memory.

### 5.2 Interfaces & Contracts

**(a) `src/skills.rs`** — extend the exclusion set:

```rust
/// The subset domain whose enumerated skills `--only-memory` resolves to.
const MEMORY_SUBSET_DOMAIN: &str = "doctrine-memory";

/// The partner subset domain (pair + walkthrough), symlinked into `doctrine`.
const PARTNER_SUBSET_DOMAIN: &str = "doctrine-partner";

/// Marketplace-only domains the CLI does not install: their skills are symlinks
/// to a canonical domain, so the embed carries duplicates that would collide on
/// skill id. Excluded at discovery.
const MARKETPLACE_ONLY_DOMAINS: &[&str] = &[MEMORY_SUBSET_DOMAIN, PARTNER_SUBSET_DOMAIN];
```

The "(sole) marketplace-only domain" wording on `MEMORY_SUBSET_DOMAIN`'s doc
comment is corrected (no longer sole). `--only-memory` is unchanged — it still
resolves to `MEMORY_SUBSET_DOMAIN` only; no `--only-partner` analog (§Non-Goals).

**(b) `install/routing-process.md`** — the routing gate. Add one table row and
one posture line. Proposed (final wording is OQ-2):

- new row, after the `/preflight` row:
  ```
  | Understand / audit an existing artifact, no change intended | `/walkthrough` (no slice) |
  ```
- posture line, in the mid-flight block:
  > Pairing / walkthrough are **conduct postures**, orthogonal to the stage —
  > layer them on the routed stage, don't route to them as an alternative. A
  > walkthrough that surfaces a concrete change re-enters `/route`.

**(c) `plugins/doctrine/skills/route/SKILL.md`** — the parallel prose surface.
Mirror the same comprehension exit + posture note in its "Choose the governing
skill" list, so the digest and the full skill don't diverge (§6 OQ-3 confirms
this is wanted, not just the digest).

**(d) `.claude-plugin/marketplace.json`** — replace the interim `partner` entry
with:
```json
{ "name": "doctrine-partner", "source": "./plugins/doctrine-partner",
  "description": "Doctrine's collaboration skills alone — pair programming and guided walkthroughs. A standalone subset of the doctrine plugin; install one or the other, not both." }
```

**(e) `plugins/doctrine-partner/README.md`** — mirror doctrine-memory's README
*shape* (title, what-it-is, dependency note, "install one, not both") but with
**accurate** source-vs-distribution wording (OQ-1 resolved (b)+): in-repo the
skills are **symlinks** into `../../doctrine/skills/<id>` (single source of truth,
no drift); on distribution Claude Code plugins are self-contained, so each symlink
resolves to a real copy in the published artifact. Drop the false "duplicated /
byte-identical copies" framing and the "update both copies" instruction (there is
one source).

**(f) `plugins/doctrine-memory/README.md`** — correct the same inaccuracy in the
precedent (OQ-1 (b)+, folded in, no longer a follow-up): `:12-14` "duplicated …
byte-identical copies" and `:19` "update both copies" describe a copy model the
source does not use (symlinks). Rewrite to the same accurate source-vs-distribution
framing so the siblings agree on the truth, not on a shared falsehood.

### 5.3 Data, State & Ownership

No runtime/authored state. All changes are embedded source (`plugins/`,
`install/`) + one `src/` const + tests. The boot snapshot (`boot.md`) is derived
— regenerated, not hand-edited.

### 5.4 Lifecycle, Operations & Dynamics

Build/refresh sequence (mandatory order — the re-embed footgun):

1. Move skills, create `doctrine-partner/` (symlinks), delete `partner/`, edit
   `marketplace.json`, `skills.rs`, `routing-process.md`, route `SKILL.md`.
2. `touch src/skills.rs` (force RustEmbed recompile) → `cargo build`.
3. `cargo test` / `just check` — discovery + boot suites green.
4. `doctrine boot` — regenerate the snapshot; confirm the new row appears.
5. (materialisation, verify-only) `doctrine skills install -y -d doctrine` lists
   `pair` + `walkthrough`; `doctrine-partner` is not independently installable.

### 5.5 Invariants, Assumptions & Edge Cases

- **INV-1.** Exactly one catalog entry per skill id — the `doctrine` copy. The
  `doctrine-partner` duplicates are excluded. (Asserted by the extended test.)
- **INV-2.** `pair` + `walkthrough` resolve under domain `doctrine`.
- **EDGE.** A consumer installing *both* `doctrine` and `doctrine-partner` gets
  name collisions resolvable only by namespaced form — same as doctrine-memory;
  the README's "install one, not both" is the mitigation, not code.
- **ASSUMPTION.** Symlink relative target `../../doctrine/skills/<id>` resolves
  from `plugins/doctrine-partner/skills/` — identical to the working
  doctrine-memory links.

## 6. Open Questions & Unknowns

- **OQ-1 (README wording) — RESOLVED (b)+.** doctrine-memory's README says skills
  are "duplicated, byte-identical copies" and to "update both copies", but the repo
  uses **symlinks** (one source). The README conflates the source model (symlinks,
  drift-proof) with the distributed artifact (self-contained copies). Resolution:
  write doctrine-partner's README *accurately* (source = symlink, distribution =
  resolved copy) **and** correct doctrine-memory's README the same way in this
  slice (§5.2e/f) — siblings agree on the truth, not a shared falsehood. The "lone
  accurate sibling is drift" argument was rejected: an accurate doc is not drift;
  misleading content is. Kills the former reconcile-the-READMEs follow-up.
- **OQ-2 (route wording/placement).** Exact row text and where the posture line
  sits. Proposed text in §5.2(b); reviewer to finalise.
- **OQ-3 (two surfaces).** Confirmed both `install/routing-process.md` *and*
  `plugins/doctrine/skills/route/SKILL.md` get the change (§5.2c) — else the
  inlined digest and the full skill diverge.

## 7. Decisions, Rationale & Alternatives

- **D1 — doctrine-partner subset, not a freestanding plugin.** Lets core
  hard-reference `/pair`/`/walkthrough` without a dangling-link risk. Alternative
  (soft generic reference from route) rejected earlier as fragile; freestanding
  plugin rejected because route can't safely assume it.
- **D2 — symlink subset (mirror on-disk precedent), not copies.** Prevents
  drift; the `MARKETPLACE_ONLY_DOMAINS` machinery already exists for exactly this.
- **D3 — no `--only-partner` flag.** YAGNI; exclusion alone is sufficient.
- **D4 — update both route surfaces.** Avoid digest/skill divergence.

## 8. Risks & Mitigations

- **R1 — re-embed footgun** (stale bytes ship). Mitigation: `touch src/skills.rs`
  before build; §5.4 step 2 is mandatory.
- **R2 — symlink double-emit collision.** Mitigation: `MARKETPLACE_ONLY_DOMAINS`
  exclusion + INV-1 test.
- **R3 — interim-state residue** (`plugins/partner/` left behind). Mitigation:
  explicit deletion + a net-diff check that `plugins/partner/` is absent.
- **R4 — boot drift masking.** `boot --check` already reports `stale` +
  unpopulated `Active Policies` *before* this work. Regenerating here may change
  unrelated sections; keep the boot regeneration commit reviewable, and do not
  fold the pre-existing drift fix into this slice (follow-up).

## 9. Quality Engineering & Validation

- **VT** — extend `discover_excludes_marketplace_only_domains` (`skills.rs:883`):
  assert no `doctrine-partner` domain in the catalog **and** `pair`/`walkthrough`
  present under domain `doctrine`. Mirrors the doctrine-memory assertions.
- **VT** — existing skills + boot suites stay green unchanged (behaviour gate).
- **VA/VH** — `doctrine boot` output shows the comprehension-exit row; the
  routing digest and route `SKILL.md` agree.
- **Gate** — `cargo clippy` zero warnings (plain, no `--all-targets`); `just
  check` green.

## 10. Review Notes

Internal adversarial pass (pre-handover):
- *Is the boot side really golden-safe?* Verified: only `contains()` checks at
  `boot.rs:945,1291`; no verbatim routing snapshot in `src/` or `tests/`. ✔
- *Does moving skills into `doctrine` auto-register them?* Yes — `discover()`
  walks all non-excluded domains; no manifest wiring needed (skills are embedded
  via `plugins/`, unlike authored `.doctrine/` entities). ✔
- *Weakest point:* OQ-1 — inheriting the precedent's README inconsistency. Called
  out explicitly rather than papered over; recommendation + follow-up given.
- *Scope honesty:* `--only-partner` and the boot-drift fix are fenced as
  non-goals/follow-ups, not silently dropped. ✔

A fresh reviewer should attack: OQ-1's recommendation, the exact route wording
(OQ-2), whether OQ-3's two-surface edit is complete (is there a third surface
reciting the route table? — confirm `CLAUDE.md`'s inlined copy is generated, not
hand-maintained), and whether the symlink relative paths resolve.

Focused review pass (SL-034 resume, pre-plan) — all four attacked:
- **OQ-1:** resolved **(b)+** — accurate wording in *both* READMEs, no propagated
  falsehood; follow-up folded into scope (§5.2e/f, §6).
- **OQ-2:** row grammar fits the boot table; `walkthrough` = no-change exit row,
  `pair` = posture-only (no standalone row). Locked.
- **OQ-3:** verified **exhaustive** — exactly two table-reciting surfaces. The
  suspected third surfaces are all derived/pointers: `CLAUDE.md` **and** `AGENTS.md`
  both `@import` `boot.md` and state the table "is not recited here"; `boot.md` is
  projected from `routing-process.md` (`boot.rs:84`); `preflight/SKILL.md` only
  defers to `/route`. No third hand-maintained surface. ✔
- **Symlink form:** `../../doctrine/skills/<id>` is byte-identical to the working
  doctrine-memory links (`ls -la plugins/doctrine-memory/skills/`). ✔
