# ISS-515: slice record-delta usage marks exclusive modes all required

## What

`doctrine slice record-delta --help` prints

    Usage: doctrine slice record-delta [OPTIONS] --commit <COMMIT> --end <END> --start <START> <ID> <PHASE>

listing all three as required, while the description (and the runtime) say `--commit`
and `--start/--end` are mutually exclusive modes. Agents guess the shape wrong.

## Fix sketch

Express the modes as a clap `ArgGroup` (required, multiple = false) so the usage line
renders `<--commit <COMMIT>|--start <START> --end <END>>`.

## Evidence

`01a0bac1` (2026-09-19), `01a0ded3` (09-26), `01a0e4d6` (09-27). Confirmed against the
v0.46.5 binary on 2026-10-01.
