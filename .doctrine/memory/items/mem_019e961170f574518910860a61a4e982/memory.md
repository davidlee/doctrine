# gitignore: no inline comments — a trailing # is part of the pattern

`.gitignore` does NOT support inline comments. `#` only starts a comment at the
START of a line. A pattern like `!.doctrine/governance.md  # note` matches the
literal string including the trailing text — so the intended pattern never
matches and the rule silently no-ops (a negation re-include fails; the file stays
ignored). Put the comment on its own line above the pattern.

Diagnose with `git check-ignore -v <path>`: it names the file:line of the
pattern that actually wins, exposing the broken/overridden rule.
