**When:** a design-run criterion asks for behaviour "on a real run" (e.g. `SL-264`'s
`VA`), and the only real runs are live or locked — mutating them would move another
slice's state.

**How (verified at `RV-390` `F-7`):**

1. `mkdir -p <scratch>/.doctrine/{slice,state/slice,review}` under a gitignored path
   (e.g. `.doctrine/state/audit/replay`), then `git init` it.
2. Copy `.doctrine/doctrine.toml` + `config.toml`, `.doctrine/slice/NNN`,
   `.doctrine/state/slice/NNN` (holds `design.toml` + the journal), and every
   review the run names (`design show` fails with *read the review pass RV-…*
   until they are present).
3. Drive the tree's fresh `./target/debug/doctrine` from inside the scratch root.
   `design apply` needs `run_uid`, `known_revision`, `submission_id` (read the
   revision from `design show --format status`). Declaration is legal at `locked`.
4. Read each act's currency from `design show --format json --full` → `acts[]`
   (`{act, current}`); the apply output's `act_invalidated` rows are the change-log half.

**Caveat:** the scratch root is disposable, so the evidence must be written into the
ledger response and `notes.md` — see [[mem_019fd1d862887d42b7a1f88c28fd28a7]] (a VA
over gitignored state leaves nothing to re-derive).
