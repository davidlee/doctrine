# IMP-327: Scope arrays are clearable via MCP but not via the CLI

Found incidentally while settling RV-314 F-2 (SL-232, DEC-081). Pre-existing;
**not** introduced by that slice, and deliberately left out of its scope.

## The asymmetry

`memory edit`'s three scope arms are declared as plain repeatable `Vec<String>`
and the CLI seam collapses an empty vector to `None` (`src/memory.rs:717-727`):

```rust
path_scope: if path_scope.is_empty() { None } else { Some(path_scope) },
glob:       if glob.is_empty() { None } else { Some(glob) },
command:    if command.is_empty() { None } else { Some(command) },
```

So from the CLI you can go from two entries to one, but never from one to none —
there is no argv that writes `globs = []`.

The MCP surface has no such collapse: `EditParams.glob` is
`Option<Vec<String>>`, so a caller sending `"glob": []` gets `Some(vec![])`,
`apply_edit` writes an empty array, and the field *is* cleared. Two surfaces over
one verb, with different capability.

## Why SL-232 did not fix it

SL-232 needed clearing for its **new** field only — `scope.unobservable`, whose
V2 rule ("an entry git *does* match is a stale declaration") has deleting the
entry as its sole remedy. DEC-081 gave that arm `num_args = 0..=1`, so a bare
`--unobservable` clears. The three inherited arms were left alone because
changing them is a behaviour change on a shipped verb, which is a slice's worth
of decision rather than a rider on someone else's.

## What to decide if this is taken up

- Whether uniformity is worth it at all — nothing has yet asked to clear
  `paths`/`globs`/`commands`, and the MCP route exists for anyone who must.
- If yes, `num_args = 0..=1` on all three matches what `unobservable` already
  does. Note the residual hazard documented for that flag: a **bare** flag
  placed immediately before the positional consumes `<REFERENCE>`. It always
  fails loudly (the sole positional is then unfilled, so clap errors and nothing
  is written), but the message names a missing argument without hinting that the
  flag ate it. Extending the shape to three more flags multiplies that surface.
- Clearing a **claim** field is not inert the way clearing `unobservable` is:
  `paths`/`globs`/`commands` are `ClaimSnapshot` members, so clearing one
  **clears the verification axis** (SL-230 D4/D8). That is correct behaviour, but
  it makes a mistyped bare flag more expensive than it looks, and it argues for
  doing this deliberately rather than for symmetry's sake.
