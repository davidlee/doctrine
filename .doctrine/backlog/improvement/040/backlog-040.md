# IMP-040: Add --color=auto|always|never flag for listing colour control

Follow-up deferred by **SL-053** (terminal output polish), governing decision
**D5**: the `--color=auto|always|never` flag was held out of scope; SL-053 ships
**auto-detection only** (`NO_COLOR` + isatty, resolved in `tty::stdout_color_enabled`).

Scope: add an explicit `--color` flag (`auto` default — current behaviour;
`always` forces colour even when piped; `never` forces plain even on a TTY). The
flag overrides the auto-detected capability bool before it is injected into
`render_columns`. The injection seam already exists (the `color: bool` on
`ListArgs`) — this item only adds the shell-side override that produces that bool,
so the pure render layer is unchanged.

Precedence to settle at design: `--color=never` vs `NO_COLOR`, and `--color=always`
vs a non-TTY pipe (the flag is the explicit user override, so it should win over
auto-detection; reconcile against the `no-color.org` convention).
