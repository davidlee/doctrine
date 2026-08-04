Clippy lints whose span falls inside a `macro_rules!` **body** do not fire, even
for a macro defined in the same crate. Lints on tokens the **caller** wrote —
the macro's arguments — fire normally.

Measured on SL-244 PHASE-05 `T1`, in `src/design_run/gate.rs`, with
`clippy::all`/`pedantic` denied at workspace level:

- a generated `self.as_str().len() == 0` drew **no** `clippy::len_zero` under
  `cargo clippy --workspace`;
- the byte-identical construct hand-written in the same module **failed the
  build** (positive control — so the silence is real, not a stale build);
- the same run's macro *arguments* tripped `clippy::eq_op` (`1 + 1 == 2`) and
  `clippy::as_conversions`, reported at the invocation site.

**Why it bites.** Moving a hand-written table into a generator moves its bodies
out of the lint gate's reach without any diff, warning, or config change. A
clean `just gate` after such a refactor is evidence about the *invocation*, not
about the generated code.

**What still covers generated code:** rustc, not clippy — `dead_code`,
non-exhaustive matches, type errors, and `const _: () = assert!(…)` rows. Lean
on those plus tests; do not lean on the lint gate.

Two neighbouring diagnostics facts from the same probe:

- A failing per-row `const` assertion reports at the macro **definition** line
  and the whole invocation block, never the offending row. Put the row in the
  assertion *message* (`concat!(…, $token, …, stringify!($variant), …)`) if the
  build error has to name it.
- A repetition in an array-length position must reference a captured variable:
  `[T; 0 $($( + 1 )+)+]` is rejected ("attempted to repeat an expression
  containing no syntax variables matched as repeating at this depth"); use
  `[T; [$($( $token, )+)+].len()]`.

See [[mem.pattern.claude.docs-first-then-probe]] and
[[mem.fact.doctrine.negative-grep-needs-positive-control]] — the positive
control is what made this finding trustworthy.
