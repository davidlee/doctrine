Found during the SL-284 audit (RV-424 F-2).

The pi boot arm built its report as one struct literal:

    Ok(RefreshReport {
        extension: install_pi_extension(..)?,
        mcp_extension: install_mcp_extension(..)?,
        surface_extension: install_surface_extension(..)?,
        pi_settings: install_pi_settings(..)?,
        ..
    })

Each field is an independent side effect, and the settings leg has to run last —
the registration names the directory the extensions live in. So when the settings
write failed, `?` propagated out of the arm and the three extension outcomes, whose
writes had already landed, were destroyed with the struct. The caller printed only
"refresh failed", over three files on disk that were now unregistered and absent
from the commit hint. Nothing lied; it was just silent about the part that worked
(STD-003).

## The general shape

`?` in a struct literal is a **sequence** claim wearing a **parallel** hat. It
reads like "gather these outcomes", but it means "abort everything gathered so far
on the first error". That is correct when the fields are pure reads over one
input. It is wrong when they are ordered side effects, because then a later
failure has earlier work to disclose.

Two fixes, pick by intent:

- **Disclose the partial state** — compute into locals, then attach context to the
  last fallible call naming what already landed:
  `install_pi_settings(..).with_context(|| format!("... failed; {dir} was written but is unregistered ({a:?}, {b:?})"))`.
- **Make failure an outcome, not an `Err`** — give the leg a `NotApplicable` /
  `Failed` variant in the report type (the `McpOutcome` idiom next to it) so the
  report is always constructible and the caller decides what blocks a stamp.

## Where to look for siblings

Any `Ok(SomeReport { .. ?})` over fields that write to disk: harness/boot arms,
install legs, `LegOutcome` sequences. Ask of each: if this field fails, is there
earlier work the operator now cannot see? The tell is a `?` on the *last* field of
a struct literal whose earlier fields have side effects.
