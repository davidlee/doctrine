# ISS-208: doctrine --help --boot-map / --commands undiscoverable — not listed in help, no flag registration

## Symptom

`doctrine --help --boot-map` emits the dense boot command map (identical to the
boot snapshot `## Commands` section); `doctrine --help --commands` emits the
grouped command table. Both work — but are **undiscoverable**:

- Not registered as clap args (`--boot-map` / `--commands` alone →
  `error: unexpected argument`).
- Not listed in `doctrine --help` output, so no user or agent can find them.
- No doc or skill points at them.

They ride the top-level help interception in `src/main.rs` (~L217–231): a
`DisplayHelp` / `MissingSubcommand` error kind triggers the intercept, which
then scans `args` for the literal strings `--boot-map` / `--commands`. Because
they are not real clap args, they only reach the render path when `--help` (or a
bare invocation) produces the trigger error — and nothing advertises the combo.

## Why it matters

SL-150 shipped `render_boot_map()` / `render_commands_table()` as the PUSH-tier
command projection. The user-facing entry points exist but are invisible, so the
feature is effectively dead for anyone who does not read `src/main.rs`. This also
undercuts the restate-line intent (ADR-005 R-OQ-4): docs/skills are meant to
*point at* the command surface instead of reproducing it, but the point-at
target cannot be discovered.

## Fix directions (not yet designed)

- Register `--boot-map` / `--commands` as real global bool flags on `Cli` so
  they self-document in `--help` and work standalone (not only riding `--help`).
- Or list the `--help --boot-map` / `--help --commands` combos explicitly in the
  top-level help footer.
- Ensure whatever lands is citable by a skill/doc as the restate-line point-at
  target.

## Relations

- Origin: surfaced during SL-144 (ADR-005 docs-IA) design — the command surface
  is derived + has a user entry point, but it is unreachable, which the
  reachability audit must not paper over.
- Concerns SL-150 (family-grouped help + boot-map projection) — the slice that
  shipped the half-wired entry points.
