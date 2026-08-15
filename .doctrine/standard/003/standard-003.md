# STD-003: No silent skip — a degraded read is disclosed

## Statement

When a read of authored corpus data fails or degrades — a file that will not
open, a TOML that will not parse, a field that is missing where the schema
requires it — the reader **tolerates it and discloses it**. Both halves are
required, and neither substitutes for the other:

- **Tolerate.** Do not abort the surrounding walk. One unreadable entity must
  not deny the caller every other entity. This half is already settled practice
  (`IMP-036`).
- **Disclose.** Name what was skipped and why, on a channel the caller of *that
  command* will see. A degraded read is never absorbed into a default value, an
  empty collection, an empty string, or a `continue`.

Three specific prohibitions, each of which is the same error wearing different
clothes:

1. **No laundering into a legitimate value.** `read_to_string(p).unwrap_or_default()`
   and `toml::from_str(&s).unwrap_or_else(|_| Table::new())` turn a corpus defect
   into a well-formed answer the caller cannot tell apart from a real one. If a
   type carries a "cannot read" case, a read failure is *not* it — a tooling
   limit and a broken file must not share a signal.
2. **No empty success.** A check or report that returns "nothing found" because
   it could not read the corpus is asserting health it did not observe. This is
   worst on diagnostic surfaces, where the empty result *is* the claim.
3. **No unnamed skip.** "Some entities were skipped" is not disclosure. Name the
   entity and the reason, so the reader can repair it.

A repair path should exist for anything disclosed. A report that names a defect
no verb can clear converts a silent problem into a loud one without closing it.

## Rationale

Silent degradation is the failure mode that reads as success. Every other class
of bug announces itself eventually; this one produces output that is
well-formed, plausible, and wrong, and nothing downstream is looking for it. The
cost is paid by whoever later trusts the answer — typically an agent, which has
no independent picture to check it against.

The specific harm in a governed corpus is that the tooling's whole value is its
claim to see the authored state. `doctor: corpus clean` over a corpus that could
not be read is not a degraded answer, it is a false one, and it is the same
species of error as reporting an edge as `dangling: absent` without checking
whether the target exists (`SL-238`). A tool that lies quietly about the corpus
is worse than no tool, because it displaces the reader's own inspection.

Tolerance without disclosure is how the pattern spreads. Each individual
instance looks like robustness — the walk survives, the command succeeds — so it
propagates by copy-paste into every sibling reader, and the accumulated effect
is a corpus that can rot without any surface reporting it. Broken windows: the
second silent skip is written because the first one was there to copy.

Loud is the right default because a corpus parse error is almost always
immediately actionable and almost never something the reader wanted to defer.
The authored corpus is small, hand-edited, and under version control; a file
that will not parse is a mistake made minutes ago, not a condition to design
around.

## Scope

**Applies to:** every reader of authored `.doctrine/` state under `src/` —
entity TOML/MD reads, corpus walks, cross-kind scans, `doctor`/`validate`
checks, and any probe that resolves a reference to another entity's data.

**Also applies to** derived and runtime state where the consumer cannot
distinguish "absent" from "unreadable" on its own.

**Does not apply to:**

- Genuinely optional data whose absence is a modelled, expected state. `Absent`
  is a legitimate answer when the schema says the field is optional; it is not a
  legitimate answer when the file failed to open.
- Reads outside the authored corpus (network, user input, third-party output),
  which have their own error contracts.
- Cases where aborting is correct because continuing would write. A degraded
  read on a **write** path refuses; tolerate-and-disclose is a rule for readers.

**Deliberately not mandated:** the channel. A listing surface may disclose in a
footer, a repair verb on stderr, a diagnostic command as a finding. What the
standard fixes is that a channel is chosen, not which one.

## Verification

- **VA — no laundering.** Review and `grep` for `unwrap_or_default()` on a read
  or parse of authored state, and for `Err(_) =>` arms yielding a default value
  rather than a distinct case. Each surviving instance is justified against the
  Scope exclusions or repaired.
- **VA — diagnostic surfaces cannot report empty on a failed read.** Every
  `doctor`/`validate` check whose corpus read fails emits a finding naming the
  failure, rather than returning an empty finding set.
- **VT — per adopting slice.** A slice touching a reader in scope pins the
  degraded case with a test: a fixture whose target is present but unparseable
  is disclosed, and is never classified as a healthy or terminal value.

Enforcement is by review and by the adopting slice's own tests; there is no
lint that can distinguish a laundered read from a legitimate default, because
the difference is in what the value means, not in its shape.

## References

- `IMP-036` (closed) — established the **tolerate** half: a full-corpus scan
  must survive a malformed sibling entity rather than abort corpus-wide. This
  standard adds the disclose half; it does not reverse that outcome.
- `RSK-013` (open) — `scan_coverage` silently skips malformed/unreadable
  `coverage.toml`. A live instance of the class, outside `SL-238`'s scope.
- `SL-238` — the slice this standard was extracted from. Its `--prune`
  terminality probe carried four copies of the laundering pattern, and its
  `doctor` check was drafted to return no findings on a corpus read failure.
- `STD-001` — no magic strings. Same shape: a cross-cutting rule about how code
  represents facts, enforced by review rather than by a lint.
- `ADR-009` — kind lifecycle vocabulary, for what counts as a legitimate
  terminal value as against an unreadable one.
