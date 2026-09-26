# IMP-475: review: render the finding tier (show --findings, list --target)

`review show` renders derived status and the brief; the table prints `findings: N`
as a count. The finding tier is reachable only via `--json` (nested at
`review.finding`). `review list` has no `--target`.

Requested: a first-class read render — `review show --findings`
(id / severity / status / disposition / one-line summary) and `review list
--target`. The `--findings` projection should be ~200 tokens where the workaround
(a raw TOML read, or a jq projection over a multi-KB `--json` dump) is thousands.

Observed 9+ times across 4 slices: obs `019fac2f`, `019facb4`, `01a00e42`,
`01a00f14`, `019fc660`, `01a0049a`, `019ff661`, `019fbcb9`, `019fd1d9`.
See RFC-032 `research.md` F1.

---

**Split (RFC-032 step 0c).** The finding index (now the *default* on
`review show`) and `review list --target` are delivered by `IMP-490`. The
`--findings` opt-in flag this item proposed is rejected by RFC-032 `D5`: the
finding tier should be the default, not a flag. What remains here is the `D5`
read-surface remainder — the CLI/MCP JSON projection unification (`IMP-476`) and
the shared filters — which RFC-032 sequences into its slice 2.
