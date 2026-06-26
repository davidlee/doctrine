# IMP-177: memory record CLI: add --trust and --severity flags (currently TOML-only)

Captured from [[mem_019ed0029e927a528b4f18b7f4a1d4c9]].

The `doctrine memory record` CLI has no `--trust` or `--severity` flags.
Risk axes are TOML-only — you must edit the TOML by hand after recording.
Producer follow-up from SL-008 remains open.
