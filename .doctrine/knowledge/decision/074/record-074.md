# DEC-074: Human review is the v1 default section posture

<!-- Knowledge record body — context, detail, links. The structured, queried
     fields live in the sister `record-NNN.toml`; this prose is free-form and is
     never structurally parsed (the storage rule). -->

New SL-233 managed design runs require human section review by default.
Adversarial section review remains a first-class, per-run opt-in capability;
the user may change the run policy to adversarial-only or require both lanes in
either order.

This preserves the current interactive design contract and avoids silently
adding reviewer cost or latency in v1. The default is not a permanent product
constant: IDE-045 captures future project-level configuration of both the
posture and the reviewer invocation.
