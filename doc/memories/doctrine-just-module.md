# Memory: ship an opt-in `just` module into `.doctrine`

A future-slice idea (not yet scoped), parked here so it isn't re-derived. Depends
on SL-010 ([../../.doctrine/slice/010/slice-010.md](../../.doctrine/slice/010/slice-010.md))
landing the symlink install model + the `.doctrine/skills` canonical tree.

## The idea

Materialise `.doctrine/doctrine.just` from the embed (derived → gitignored, same
tier as `.doctrine/skills`). Users adopt it with **one line** in their own
justfile — `mod doctrine '.doctrine/doctrine.just'` — so doctrine never owns or
clobbers their root justfile. Uninstall = delete the line.

## Why it's worth it

- **`vendor-skill <id>`** packages SL-010's override hatch as one command:
  `rm` the managed symlink → `cp -rL .doctrine/skills/<id> .claude/skills/<id>`
  (deref to a real copy, which `install` then refuses to clobber) → `git add -f`
  (`.claude` is gitignored). Makes "pin a skill locally" trivial.
- Read shortcuts: `slice <id>: glow $(doctrine slice show {{id}} --path) --pager`.
- fzf pickers over slices / memories / ADRs.

## Open dependencies / risks

- **`doctrine slice show [--path]` does not exist** — slice has new/design/plan/
  phases/notes/phase/list, no `show`. The read shortcuts need it first.
- Pushes a hard dep on `just` itself; mitigated by opt-in `mod` (only `just`
  users adopt). `glow`/`fzf` recipes are optional — degrade to "command not found".
- "Marginally invasive" worry resolved by the module form: a `mod` line, not a
  root-justfile takeover.
