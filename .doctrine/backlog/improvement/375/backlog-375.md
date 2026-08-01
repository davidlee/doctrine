# IMP-375: Project extension interface for design-prompts assets

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Nothing under `install/design-prompts/` is project-overridable, and two shipped
artefacts say it is.

## The gap

The authoring rule every runbook is written against
(`install/design-prompts/exploring.toml:8-13`) promises:

```
Yes -> a runbook step. Overridable, verifier substitutable.
```

DEC-102 makes the same promise on the asset-policy axis — craft is overridable,
invariants stay sealed. Neither holds for this store:

- Runbooks resolve to `{STORE}/<name>.toml` and stage fragments to
  `{STORE}/<name>.md` (`runbook.rs:185-189`, `prompt.rs:68-70`) — **embedded
  assets, both**. There is no disk tier.
- `doctrine prompt check` *"loads `.doctrine/hymns` plus the embedded hymns …
  it knows nothing about this store's `*.toml` siblings"* (`runbook.rs:318-322`).
  The hymn cascade's `.doctrine/hymns` overlay, its `replaces` suppression and
  its seal/expose projection **do not reach `design-prompts/` at all**.
- *"Verifier substitutable"* means something narrower than it reads: the closed
  thing is the **placeholder vocabulary** a `verify` argv may interpolate
  (`runbook.rs:59-66`), substitutable *within* an authored asset. It is not a
  project override.

The design anticipates project authorship that the load path does not admit —
`RUNBOOK_STEP_ID_BYTES` is sized at 64 rather than 32 because the latter *"would
fit the shipped five while leaving **a project** no room"* (`runbook.rs:55-57`).

## Why it matters more after DEC-104

DEC-104 splits tier 2 into runbook steps (2a) and stage fragments (2b), and
routes the **craft** — question-asking preferences, design-quality lenses,
adversarial attack surfaces — into 2b. That is exactly the content a project
would legitimately hold its own opinions about, and it now lands in the store
with no override path. The gap was latent before; DEC-104 makes it load-bearing.

## `design.md §7`'s rejection likely needs reopening

`SL-233`'s design rejected carrying the four process fragments as `stage/*`
hymns, and the reasoning is **sound on its own terms**:
`src/install.rs::KNOWN_STAGE_LABELS` is an *enforced* lifecycle vocabulary
(`route, canon, preflight, slice, design, inquisition, plan, phase-plan,
execute, audit, reconcile, close`) and `drafting` is not a lifecycle stage, so
admitting it would pollute what a `stage` label means.

But that rejection **bundled two separable questions and answered only one**:

1. *Should intra-design obligations be `stage`-band labels?* — No, correctly.
2. *Should intra-design obligation prose be project-overridable?* — **Never
   asked.** It was inherited as a side effect of the mechanism chosen to settle
   the first.

Reopening `§7` should address (2) on its own reasoning rather than leaving it
settled by consequence.

## Options not yet weighed

- **`project`-band hymn.** The band registry already reserves `project` for
  "Project-specific / user-authored" with no enforced label vocabulary, so a
  project could express design priorities without touching
  `KNOWN_STAGE_LABELS`. Cost: hymns are composed at context load, so it is
  always-on rather than stage-scoped, and a project could *add* but not
  *replace* a shipped lens.
- **Seal/expose on the fragment store.** Genuine replacement, at the cost of
  contradicting DEC-077's explicit *"this store does not want user overrides"*.
- **Do nothing and amend the promises instead** — strike "overridable" from the
  authoring rule and from DEC-102. Cheapest, and honest, but abandons the
  extensibility goal rather than serving it.

## Why it is not in SL-233

Out of PHASE-08's scope. The phase converts a skill body into runbooks and
fragments; it does not build a new override tier for the store they live in, and
widening it to do so would put unreviewed cascade machinery inside a phase whose
design gate (`EN-2`) is already under adversarial review.
