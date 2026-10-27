# CHR-183: Retire interim quotient forms

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Remove IMP-522's interim quotient wiring once upstream lands. Waits on two
quotient backlog items (in `~/dev/quotient`):

- **IMP-003** (optional jail.nix form builder): replace flake.nix
  `quotientForms` (a copy of satan's `quotientFormFamily`) with the shared
  builder; drop the `just quotient-forms` recipe if the builder ships one.
- **ISS-001** (launch-dir profile layer blocks repo-root writes): once the
  policy names profile layers, drop the local-scope `quotient` MCP entry's
  `cd ~/.config/quotient/doctrine` launch wrapper, and the CLAUDE.md "always
  pass `cwd`" rule if the default cwd becomes usable again.

Keep the doctrine-scoped policy (`~/.config/quotient/doctrine.policy.json`)
unless quotient gains per-root form selection.

## Builder half done (2026-10-27)

The IMP-003 half is met another way: flake.nix `quotientForms` now calls
nix-config's `jailLib.quotientFormFamily` (quotient's `mkFormFamily` plus the
agents lib's posture and gateway wiring; nix-config IMP-052, CHR-023), with
`checks.quotient-forms` running quotient's form check. The interim copy is
gone. The `just quotient-forms` recipe stays: it installs the forms where
`doctrine.policy.json` names them. The form no longer binds the persisted
jail home, so that policy's `exposure` is `[]`.

Open: the ISS-001 half (the `quotient` MCP launch wrapper and the CLAUDE.md
"always pass `cwd`" rule).
