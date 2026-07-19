# Spec requirement entities and their §-prose drift silently — sweep the narrative when a requirement changes

A spec is two tiers: the `REQ-NNN` requirement **entities** (structured
`description` + `acceptance_criteria`) and the spec's **§-prose** (Intent,
Principles, Behaviour, Verification) that narrates the same obligations. Nothing
in the toolchain checks that the two agree — `spec validate` proves FK integrity,
not tier consistency. So when you rewrite a requirement entity, the paragraphs in
§3/§5/§6/§7 that describe the *old* obligation stay put and silently contradict
the new one.

**Why it matters:** the drift is invisible from inside — you edited the entity and
moved on. It surfaces only when an external reviewer greps the prose and finds it
describing a model the requirements already abandoned. On PRD-017/SPEC-026 a first
codex pass fixed the requirement entities; a *second* pass was spent entirely on
the stale rule-derived-licence and resolver/manifest language still sitting in the
PRD's section prose. A whole review round to catch what one sweep would have.

**How to apply:**
- When you change a `REQ-NNN` entity, immediately re-read the spec's §-prose for
  every sentence that describes that obligation, and align it in the same edit.
  Treat entity + narrative as one change, never two.
- Grep the prose for the terms you just removed from the entity (old verbs, old
  licence/mechanism nouns) before considering the rework done — an empty grep is
  the cheap proof the sweep landed.
- **Mechanism nouns in a *product* spec are the smell.** `manifest`, `resolver`,
  `embed`, `runtime-loaded`, `backing source` leaking into a PRD requirement or
  its §-prose means the product tier absorbed tech detail that belongs in the tech
  spec. Product = observable what/why; the how lives one tier down. (ADR-019
  storage *vocabulary* named in a §4 constraint is the licit exception — that's a
  boundary the product deliberately draws, not a mechanism it prescribes.)

Related: the missing affordance behind this — no "sync §-prose to requirement
entities" check, and no `requirement edit` verb forcing full-field TOML
hand-edits (IMP-298) — is captured as backlog work. See [[mem.signpost.doctrine.specs]].
