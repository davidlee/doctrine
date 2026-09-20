# ISS-474: Plan scaffold ships an invalid-TOML VT mandate example

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Two shipped surfaces show the structured `VT` mandate as a **multi-line inline
table**:

- the template `doctrine slice plan <ID>` writes into a fresh `plan.toml`
  (the comment block above the first `[[phase]]`), and
- `plugins/doctrine/skills/plan/SKILL.md` step 4, in the fenced example.

TOML 1.0 forbids a newline inside an inline table, so an author who copies the
shape verbatim gets a parse error at the first `VT` row. Every real `plan.toml`
in the corpus uses the single-line form, so the templates are the only place the
invalid shape appears — which means the error is reached by following the
instructions rather than by deviating from them.

Cost when hit: one full authoring round-trip of `plan.toml` (rewrite every
verification row onto one line). Observed authoring `SL-260`'s plan.

Fix is to render both examples single-line. Note the skill file is rewritten on
every install (`copy_skill`, `src/install.rs`), so there is no user copy to
migrate; the scaffold template lives in the `slice plan` writer.

Likely arrived with `IMP-209` (*Plan skill should author structured VT mandates
so verify-vt has signal*), which introduced the mandate and its example.
