# ISS-439: config get/set/unset --help panics in clap debug_asserts and exits 0

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

Three help renderings panic instead of rendering:

```
$ doctrine config get --help
thread 'main' panicked at clap_builder-4.6.0/src/builder/debug_asserts.rs:202:13:
Argument key: `required` conflicts with `required_unless*`
```

Same for `config set --help` and `config unset --help`.

**The exit code is 0.** A caller asking for help gets a panic trace on stderr,
nothing on stdout, and a success status — so anything scripting around it reads
the run as fine.

## Where it comes from

`debug_asserts.rs` is clap's *debug-build* self-check, so this is a malformed
argument definition rather than a clap defect: the `key` argument is declared
both `required` and with a `required_unless*` relation, which clap treats as
contradictory. Release builds skip the assert, which is why it has survived —
the panic is invisible wherever `debug_assertions` is off.

## Provenance

Pre-existing; **not** introduced by `SL-251`, which touches no `config` code.
Surfaced incidentally by `SL-251` PHASE-07, whose `EX-10` bound was measured by
rendering help for the entire CLI tree — 281 renderings, before and after. Three
of them differed only by a PID inside this panic, in both the before and the
after set. A whole-tree help sweep is not something the project normally does,
which is why three broken renderings were sitting in plain sight.

## Worth noting for whoever fixes it

The sweep is cheap and it found this in one pass. A test that renders every
subcommand's help and asserts none panics would be a few lines and would have
caught this at introduction — worth considering alongside the fix.
