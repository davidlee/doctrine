# QUE-198: How managed checkpoints prove user acceptance

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

DEC-062 says the user owns accepted design truth, but the current checkpoint
example lets the agent place `"status": "accepted"` inside a new decision
payload. The mutation protocol therefore needs to distinguish semantic content
from the evidence that authorises an accepted/held status.

Doctrine receives the submission through the agent and cannot independently
authenticate a conversational user turn in v1. It can nevertheless require an
explicit, content-bound claim rather than treating agent submission as implicit
acceptance.

## Options

1. **Content-bound acceptance attestation (recommended).** A checkpoint may
   create/adopt semantic content and separately declare a user-acceptance
   attestation containing a concise basis and optional harness turn reference.
   Doctrine binds the attestation to the checkpoint payload fingerprint and
   current run revision. Only a current attestation permits the managed
   operation to transition a DEC to `accepted` or an ASM to its accepted
   equivalent. Without it, creation uses the knowledge kind's default state.
   The attestation records that the agent claims user acceptance; it does not
   pretend the CLI authenticated the human.
2. **Always create proposed, settle separately.** Checkpoint creation always
   uses the default knowledge status. A later design apply or
   `knowledge status` operation settles it after user review. This is simple and
   honest but restores a multi-command checkpoint ritual and complicates
   atomic inquiry disposition.
3. **Submission implies acceptance.** Treat a checkpoint emitted by the active
   design agent as sufficient to author accepted truth. This is frictionless
   but contradicts DEC-062 and can turn conversational momentum into durable
   acceptance.

The recommendation is option 1. It makes the authority claim explicit and
fingerprinted without requiring a harness integration or separating semantic
checkpointing back into several adherence-sensitive commands.

## Answer

Option 1 was accepted and is recorded by DEC-088. The required `basis` remains
subject to measurement in CHR-049 rather than being presumed permanently useful.
