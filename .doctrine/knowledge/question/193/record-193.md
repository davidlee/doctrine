# QUE-193: Human-in-the-loop evaluation commitment for managed design

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

How much explicit agent evaluation must SL-233 deliver before closure, given
that design is unbounded, stochastic, and necessarily human-mediated but is
also a core Doctrine user experience?

Options:

1. Author a bounded evaluation protocol and run a small paired
   human-in-the-loop pilot before closure. Compare the current static skill with
   the managed design path using the same model family, harness, slice fixture,
   and moderator scenario. Preserve raw transcripts/command evidence and report
   limitations rather than claiming statistical significance.
2. Author the protocol and deterministic fixtures in SL-233, but defer all live
   paired runs until later.
3. Rely on the representative implementation E2E and ordinary dogfooding,
   without a named comparative protocol.

Option 1 is recommended. The protocol should measure process and outcome
separately. Primary process measures include:

- whether accepted decisions, retained questions, and carried assumptions
  produce linked `DEC`, `QUE`, and `ASM` records without human reminders;
- checkpoint latency measured in turns;
- correct refresh after state changes and recovery after context replacement;
- visible map coherence and successful user traversal direction; and
- prompt/tool/token overhead.

Outcome measures include human-rated interview usefulness and a blinded
adversarial assessment of the resulting design. A prepared moderator script can
fix decision opportunities, a correction/regression, a traversal redirect, and
a context break while still permitting natural follow-up questions.

If managed runs activate but adoption remains weak, an additional small arm may
add the user-owned `.doctrine/governance.md` authority primer described by
RSK-229. That isolates a plausible authority confound without treating the
local mitigation as a shipped product contract.

## Answer

DEC-079 chooses option 2 for the slice boundary, paired with an immediate,
explicit post-close measurement exercise in CHR-049. SL-233 delivers the
protocol and fixtures; CHR-049 runs the live evaluation after the changed skill
is actually installed.
