# REV REV-069 — ADR-005 names essentials.md as the boot onboarding summary

Revision — a pending revise-intent against authored governance/spec
truth. The structured `[[change]]` payload lives in the sister `revision-NNN.toml`;
this prose companion carries the rationale and the free-text before/after excerpts
for prose-body section edits.

## Rationale

`install/routing-process.md` is already the boot snapshot's static digest:
`src/boot.rs` embeds it whole as the "Routing & Process" section. It carries
the routing table, postures, core process and guardrails, and it also carries
per-message copies of rules that `glossary.md` and `using-doctrine.md` own
(reference forms, storage tiers). ADR-005 wants those rules resident (the PUSH
tier) and allows exactly one workflow doc, with no parallel summary. So the
file's defect is its framing and its name, not the duplication.

DEC-344 (decided by the user, 2026-09-27) recasts the file as the boot's
compact **onboarding summary** and renames it. It owns what only it holds
(routing, postures, core process, guardrails, and the `lib:` resolution rule
with the register of reference docs). For rules another doc owns, it carries
only a per-message summary that ends with a `lib:` cue to the owner. SL-273's
design (section 5.1) settles the new name as `essentials.md`, published as
`reference/essentials.md`, with the boot section heading "Essentials". The name
is not `onboarding.md`, because boot already has an `## Onboarding` section fed
by the project-orientation memory.

SL-273's design (section 5.4) routes the rename and the digest's stated role
through this Revision (ADR-013), before the rename lands. Three governing
records name the file or describe its role:

- **ADR-005.** The PUSH tier, the one-workflow-doc invariant, the Neutral
  consequence that anticipated a rename, and the References line.
- **ADR-024.** Its Verification section says the authoring rule is pointed at
  from `install/routing-process.md`.
- **SPEC-011.** Its section on the governance pointer layer contrasts that
  layer with the embedded `routing-process.md` digest.

**This reverses a resolved open question.** ADR-005's R-OQ-3 (resolved at
review, 2026-06-08) chose to "leave `routing-process.md` as-is", on the grounds
that a rename would churn every `@`-import and hook reference for cosmetic gain.
Two things have changed since then. The file is no longer `@`-imported by name:
boot embeds it, and clients reach it through the boot snapshot. And SL-273's
citation sweep already rewrites every reference, so the rename's marginal churn
is small. The gain is no longer cosmetic either: a name that states the role
tells future authors what belongs in the file. R-OQ-3 stays in the ADR verbatim
as the record of the original call, and the Neutral line records the reversal
(Change 1c).

Governing text names the file by its repo path (`install/essentials.md`), as
the existing text does. It does not use a `lib:` address.

No REQ rows: ADR-005, ADR-024 and SPEC-011 are the only governed truth that
changes. `modify` rows are hand-landed at apply, so each **After** block below
is the exact replacement text.

Deliberately **left unchanged**:

- ADR-005's Context section, its resolved open questions R-OQ-1 to R-OQ-5, and
  the review charges. They are the record of the original decision. R-OQ-3 and
  R-OQ-5 name `routing-process.md`, and that is history.
- ADR-005's `inquisition.md`, which is the review record.
- ADR-005's ownership-by-material-type table (from REV-067). It names homes by
  tier, not by file, and the rename does not change who owns what.
- ADR-005's generic uses of "the boot digest" (tier 2 and its `using-doctrine.md`
  bullet). They name the boot snapshot as a pointing surface, not the file.
- SPEC-011's section source-kind taxonomy, where `Static(name)` reads "a
  canonical embedded digest" by filename. That is the mechanism, and it does
  not name the file.

## Change 1 — ADR-005 (modify, primary)

### (a) Tier 1 PUSH — name the onboarding summary and its cue rule

**Before:**

````markdown
1. **PUSH — the boot snapshot (resident, compact).** Carries the workflow digest
   (`install/routing-process.md`, which already rides boot) **and** the load-bearing
   usage rules an agent must never get wrong without a lookup: reference forms,
   storage tiers (structured→TOML / prose→MD, read via `show`), and "use the CLI,
   don't guess." These fire without an invoke. The snapshot stays compact —
   load-bearing only, never reference detail.
````

**After:**

````markdown
1. **PUSH — the boot snapshot (resident, compact).** Carries the boot onboarding
   summary (`install/essentials.md`, which rides boot as its "Essentials" section)
   **and** the load-bearing usage rules an agent must never get wrong without a
   lookup: reference forms, storage tiers (structured→TOML / prose→MD, read via
   `show`), and "use the CLI, don't guess." These fire without an invoke. A rule
   the summary does not own rides as a per-message summary that ends with a `lib:`
   cue to its library owner. The summary itself carries the `lib:` resolution rule
   (a citation is read with `doctrine library show`) and the mandatory-retrieval
   line (a retrieval a skill or reference doc specifies is not optional reading).
   The snapshot stays compact — load-bearing only, never reference detail.
````

### (b) Invariants — the one-summary invariant

**Before:**

````markdown
- There is exactly **one** workflow doc (`routing-process.md`) — no parallel summary.
````

**After:**

````markdown
- There is exactly **one** boot onboarding summary (`essentials.md`) — no parallel
  summary. It owns routing, the core process and the guardrails, and cues the
  owner of everything else.
````

### (c) Consequences / Neutral — record the rename as done

**Before:**

````markdown
- `routing-process.md` may be renamed/promoted to a clearer "workflow" identity.
````

**After:**

````markdown
- `routing-process.md` was renamed to `essentials.md`, the boot onboarding summary
  (SL-273, DEC-344). This reverses R-OQ-3, which stays below as the record of the
  original call.
````

### (d) References — the affected-surface line

**Before:**

````markdown
- `install/routing-process.md` (the push digest); `src/boot.rs` `boot_sequence`
  (the snapshot sections); `src/install.rs` / `src/skills.rs` (the RustEmbed sets).
````

**After:**

````markdown
- `install/essentials.md` (the boot onboarding summary); `src/boot.rs` `boot_sequence`
  (the snapshot sections); `src/install.rs` / `src/skills.rs` (the RustEmbed sets).
````

## Change 2 — ADR-024 (modify)

### Verification — the pointing surface

**Before:**

````markdown
- The rule and its disposition procedure are delivered by
  `reference/shipped-corpus-authoring.md`, reachable through
  `doctrine library show` and pointed at from `install/routing-process.md`.
````

**After:**

````markdown
- The rule and its disposition procedure are delivered by
  `reference/shipped-corpus-authoring.md`, reachable through
  `doctrine library show` and pointed at from the boot onboarding summary
  (`install/essentials.md`).
````

## Change 3 — SPEC-011 (modify)

### The governance pointer layer — the embedded summary it is distinct from

**Before:**

````markdown
`.doctrine/governance.md` body is the editable user-owned layer projected as the
`Governance` section — distinct from the embedded `routing-process.md` digest.
````

**After:**

````markdown
`.doctrine/governance.md` body is the editable user-owned layer projected as the
`Governance` section — distinct from the embedded `essentials.md` onboarding
summary (the "Essentials" section).
````
