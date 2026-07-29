Editing a skill master (`plugins/doctrine/skills/<id>/SKILL.md`) and running
`doctrine install` does **not** refresh the installed copy at
`.agents/skills/<id>/SKILL.md`. This is correct behaviour, not a failed install.

`install.rs::delegate_argv` assembles `npx skills add <repo>` for every
non-Claude agent (codex / pi / universal), where `repo` is the **github slug**
(`skills-lock.json` records `"sourceType": "github"`, `"source":
"davidlee/doctrine"`). The mirror is therefore sourced from the **published**
repository. A working-tree edit is invisible to it by construction until it
lands and is published.

Two traps this sets:

1. **It looks like a stale RustEmbed.** The install plan line says only
   "delegates to npx", so the natural hypothesis is the `#[folder = "plugins/"]`
   embed in `src/install.rs` going stale. Chasing it (`touch src/install.rs &&
   cargo build`) changes nothing, because the embed was never the source for
   this path.
2. **`skills-lock.json` moves anyway.** The re-fetch bumps `computedHash` for
   the edited skill — to the hash of the *published* (pre-edit) content. That
   dirty tracked file describes remote state, not your change; don't commit it
   as part of the edit.

What to do: verify the **master** under `plugins/`, and treat `.agents/` as
gitignored derived state that catches up on publish. `--dev` swaps the *claude*
marketplace to the local root; it does not redirect the npx path.

Related: [[mem.signpost.doctrine.install]],
[[mem_019ed423e5c07040837d4d0d5a677594]] (skill masters live in
`plugins/doctrine/skills/`),
[[mem_019e98a783ea7471ac4bfcefdc04ae5e]] (the genuine RustEmbed re-embed
footgun — distinct from this, and the reason this one misdiagnoses so easily).
