Measured on clap 4.6.1 (SL-236 design §10 F-8), in an isolated spike.

Declaring `#[arg(short = 'p', long, global = true)]` on the top-level `Cli`
while a subcommand still declares its own `#[arg(short = 'p', long)] path`:

- `Cli::command().debug_assert()` **passes**. clap does not reject the duplicate.
- `spike collide -p LOCAL` parses, and the value lands in **both** the global
  field and the local field — they share an arg id, so clap treats them as one
  argument.

## Why this bites

The intuitive assumption is that a global short flag collides with any surviving
local, forcing a flag migration to land atomically. It does not. Consequences:

- A flag migration **can** be staged incrementally; intermediate states parse and
  behave correctly.
- But the compiler exerts **no pressure toward completeness**. Deleting a field
  makes its *readers* a compile error (so nothing ships broken), while *leaving*
  a declaration breaks nothing at all. A half-migrated tree compiles and passes
  behavioural tests.
- Therefore a source-scanning test is the only guarantee such a migration
  finished. Do not rely on "it compiles" as evidence of completeness.

## Help rendering

A global arg renders inside each subcommand's `Options:` block, in roughly the
position a local occupied. The line does not move; the **description text**
changes, because help prints the global's doc comment. For a codebase with
byte-exact help goldens, choose the global's doc string to match the most common
existing wording to minimise golden churn.

## Related

Globals propagate to nested subcommands at least 3 levels deep (verified against
doctrine's own `--color`: `doctrine slice selector list <id> --color never`).
