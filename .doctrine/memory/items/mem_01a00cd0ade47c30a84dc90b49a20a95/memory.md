Deleting an underscore-prefixed dead parameter is normally safe — that is the
point of the prefix. The trap is one level **up**, at the call site that produced
the argument.

SL-238 PHASE-05 T0 culled `_estimation_unit`, `_lower_pct`, `_upper_pct` from
`backlog::format_metadata` (to get under `clippy::too_many_arguments`, which
fires at >7 and is denied here). Two of them came from

    let (lower_pct, upper_pct) = crate::estimate::resolve_confidence(&cfg.estimation)?;

`resolve_confidence` is **not** a getter. It is a five-arm validator: finite,
in `[0.0, 1.0]`, and `lower < upper`, each an `anyhow::bail!`. Deleting the
binding along with the parameters would have compiled clean, passed every test,
and silently removed an error path from `backlog show` / `backlog inspect` — a
malformed `[estimation]` config would have started succeeding.

**The rule.** A parameter's value dying does not mean its producer should die.
Follow each culled argument up to the expression that computes it and ask what
else that expression does. If it validates, keep the call and discard the value:

    // Called for its VALIDATION, not its value.
    crate::estimate::resolve_confidence(&cfg.estimation)?;

`?` on a discarded tuple is not a lint here (a plain tuple is not `#[must_use]`),
so this costs one line and a comment.

**Why the compiler cannot help.** The cull is behaviour-preserving *as far as
rustc and the suites can see* — the deleted code path only fires on a malformed
config no fixture carries. A "no behaviour change" refactor claim rests on the
existing suites staying green, and that claim is exactly as strong as the suites'
coverage of the error paths. Check producers by reading, not by running.

Related: [[mem.pattern.lint.dead-code-derives-count-as-reads]] and
[[mem_019e985028947ef2ad86d43997214aca]] (the arg-ceiling pressure that motivates
these culls in the first place — the house pattern is an args struct, but
deleting genuinely dead code beats both a struct and an `#[expect]`).
