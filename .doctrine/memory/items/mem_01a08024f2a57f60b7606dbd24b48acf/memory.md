`verify-vt` matches each `keywords` entry as a **raw-byte substring** of the
mandated `test_file` (`src/vtgate.rs`, the keyword loop). It is a presence check
over source text, not an assertion about behaviour — a keyword in a comment
satisfies it, and a keyword the file will never spell can never pass.

So a mandate must name a literal the *finished file* contains. The trap is
naming a **derived value** instead — a rendered id, a subject, a wire token —
because a well-written suite composes those from their source rather than
re-typing them (STD-001):

```rust
// the test composes; no subject literal exists in the file
let subject = act_subject(IdKind::CheckpointAct, ActKind::DesignAccepted);
```

Against that idiom, `keywords = ["cpa-design-accepted"]` fails whatever the test
does, and the only way to make it pass is to hardcode the string the standard
told you not to hardcode. Worse, the *spelling* can be wrong too and nobody
notices until execution: `SL-256`'s `PHASE-02` `VT-1` mandated
`cpa-design_accepted` while act tokens are kebab (`design-accepted`), so the
mandate named a row that does not exist.

## How to apply

- **Author keywords over things the file literally spells**: the test's own `fn`
  name, a type or variant path (`ActKind::DesignAccepted`), an `enum`/`const`
  declaration (`const EMITTABLE`), a call shape you want pinned
  (`is_subset(&ChangeEvent::EMITTABLE`).
- **Never a value the code derives.** If the mandate wants a derived subject or
  token, name its **composed source** — the enum path the test builds it from.
- **Check satisfiability at `/phase-plan`, not at execution.** Read the
  criterion, then grep the target file (or the idiom of its siblings) for the
  literal. Fixing it is legal: criterion **ids** are immutable, mandate **text**
  is not — correct it in place with the reasoning inline, and if a locked design
  carries the same spelling, log that half for `/reconcile` rather than editing
  locked prose.
- A mandate whose keyword is unmatchable reads as `FAIL` for the phase's whole
  life and then, if someone "fixes" it with a literal, as a green light over an
  assertion nobody checked — the same inert-gate failure mode as
  [[mem.pattern.doctrine.assign-vt-by-code-owner-not-provenance-block]].
- Mid-phase `verify-vt` output is separately untrustworthy (`ISS-449`): it
  prints "keyword present" for keywords it never checked on `UNATTRIBUTABLE`
  rows, so read it only after the phase flips `completed`.
