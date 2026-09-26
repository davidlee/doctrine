# ISS-491: prompt resolve accepts an unknown --model value silently

<!-- Backlog item body — context, detail, links. The structured, queried fields
     live in the sister `backlog-NNN.toml`; this prose is free-form and is never
     structurally parsed (the storage rule). -->

`doctrine prompt resolve` takes any `--model` value and never checks it against
the corpus's snippet labels. A value matching nothing yields **exit 0, zero bytes
on stdout, nothing on stderr** — indistinguishable from a band that is simply
empty.

## Evidence

    $ ./target/debug/doctrine prompt resolve --role orchestrator --band model --model nonexistent/x
    (silence; exit 0)

    --model deepseek/_default   → 146 bytes   (the real key form)
    --model deepseek            →   0 bytes, exit 0  (provider name, not a key)
    --model anthropic/claude-sonnet-4 → 153 bytes
    --model adherence/low       → 2522 bytes

`prompt model-keys` prints `deepseek/_default`. `prompt resolve --help` documents
the field as `Model key (e.g. "anthropic/claude-sonnet-4")`. Neither output tells
a caller what a *valid* value is for a default-bucket key, and a wrong value
produces the same output as no match. `prompt check` validates the `replaces`
graph and the stage vocabulary — not the keys a caller supplies.

## Cost

The boot snapshot carries a floor directive: self-identify the model, run
`prompt resolve`, and re-resolve when the model changes. A session that guesses
the key form gets silence, reads it as "the corpus has nothing for me", and
proceeds without its model band. Observed live during this session
(observation `01a0dbef-5b44-78f0-9e41-beb7de868f60`); cost was two extra
round-trips and an unverifiable guess on a directive the boot says to act on
immediately.

## Direction

- Report a `--model` value that matches no snippet label — a line on stderr
  naming the value and the known labels, or a non-zero exit. The silence is the
  defect; either remedy closes it.
- Consider cross-referencing the two verbs' help text, so the shape of a valid
  key is derivable from either.

## Related

- ISS-308 — `prompt resolve` has no role for non-dispatch agents (the same verb's
  other silent gap)
- ISS-447 — hymn stage band is unreachable; nothing passes `--stage`
- IMP-489 — the review that should own the wider surface
